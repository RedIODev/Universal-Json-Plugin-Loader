use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{
    BareFnArg, BareVariadic, FnArg, GenericParam, Generics, Ident, ItemFn, ItemImpl, ItemTrait, ItemType, PatType, Signature, Token, TraitItem, TraitItemFn, TypeBareFn, Variadic, Visibility, parse::Parse, parse_macro_input, parse_quote, punctuated::Punctuated, token::Comma
};

struct AttrArgs {
    trait_ident: Ident,
    struct_rename: Option<Ident>,
}

impl Parse for AttrArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let trait_ident = input.parse()?;
        let trait_fn = if input.is_empty() {
            None
        } else {
            let _: Token![,] = input.parse()?;
            Some(input.parse()?)
        };
        Ok(AttrArgs {
            trait_ident,
            struct_rename: trait_fn,
        })
    }
}

#[proc_macro_attribute]
pub fn trait_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as AttrArgs);
    let func = parse_macro_input!(item as ItemFn);
    let mut impl_func = func.clone();
    impl_func.sig.ident = Ident::new("safe", Span::call_site().into());
    let mut struct_name = func.sig.ident.clone();
    if let Some(name) = attr.struct_rename {
        struct_name = name;
    }
    impl_func.vis = syn::Visibility::Inherited;
    let trait_ident = attr.trait_ident;
    let struct_vis = func.vis;
    let implementation = quote! {
        #struct_vis struct #struct_name;
        impl #trait_ident for #struct_name {
            #impl_func
        }
    };
    implementation.into()
}

#[proc_macro_attribute]
pub fn fn_trait(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let trait_item = parse_macro_input!(item as ItemTrait);
    let x =match fn_trait_result(trait_item) {
        Ok(ok) => ok,
        Err(err) => err,
    };
    println!("{}", x);
    x
}


fn fn_trait_result(mut item: ItemTrait) -> Result<TokenStream, TokenStream> {
    let adapter = find_func(&item, "adapter")?;
    let safe = find_func(&item, "safe")?;
    let safe_generics = safe.sig.generics.clone();
    let adapter_generics = adapter.sig.generics.clone();
    let adapter_fp_type = sig_to_fp(adapter.sig)?;
    let safe_fp_type = sig_to_fp(safe.sig)?;
    let mut call_site_adapter_generics = adapter_generics.clone();
    call_site_adapter_generics.params.iter_mut().for_each(|param| if let GenericParam::Type(t) = param {
        t.colon_token = None;
        t.bounds = Punctuated::default();
        t.default = None;
    });
    let params = call_site_adapter_generics.params;
    let adapter_func = parse_quote! {
        fn unsafe_fp #adapter_generics () -> #adapter_fp_type {
            Self::adapter::<#params>
        }
    };
    let safe_func = parse_quote! {
        fn safe_fp #safe_generics () -> #safe_fp_type {
            Self::safe
        }
    };

    item.items.append(&mut vec![adapter_func, safe_func]);
    let trait_vis = item.vis.clone();
    let trait_name = item.ident.clone();
    let safe_fp_type = create_type(&trait_vis, &trait_name, "SafeFP", &safe_generics, &safe_fp_type);
    let adapter_fp_type = create_type(&trait_vis, &trait_name, "UnsafeFP", &Generics::default(), &adapter_fp_type);


    let from_fp = find_func(&item, "from_fp")?;
    item.items.retain(|item| *item != TraitItem::Fn(from_fp.clone()));
    let to_safe_trait_name = Ident::new(&format!("{}ToSafe", trait_name), Span::call_site().into());

    let from_fp_generics = from_fp.sig.generics;
    let from_fp_return_type = from_fp.sig.output;
    let from_fp_body = from_fp.default.ok_or(new_error("from_fp needs to have a block."))?;

    let to_safe_trait:ItemTrait = parse_quote! {
        #trait_vis trait #to_safe_trait_name {
            fn to_safe #from_fp_generics (self) #from_fp_return_type;
        } 
    };

    let from_fp_receiver = from_fp.sig.inputs.first().ok_or(new_error("from_fp needs to take a parameter."))?;
    let FnArg::Receiver(from_fp_receiver) = from_fp_receiver else {
        return Err(new_error("from_fp needs to take a self parameter."));
    };
    let from_fp_receiver_type_name = from_fp_receiver.ty.clone();
    //let adapter_fp_type_name = adapter_fp_type.ident.clone();

    let to_safe_impl: ItemImpl = parse_quote! {
        impl #to_safe_trait_name for #from_fp_receiver_type_name {
            fn to_safe #from_fp_generics (self) #from_fp_return_type #from_fp_body
        }
    };

    Ok(quote! { 
        #item
        #safe_fp_type
        #adapter_fp_type
        #to_safe_trait
        #to_safe_impl
    }.into())
}

fn create_type(vis: &Visibility, trait_name: &Ident, suffix:&str,  generics: &Generics, fp_type: &TypeBareFn) -> ItemType {
    let name = format!("{}{}", trait_name, suffix);
    let ident = Ident::new(&name, Span::call_site().into());
    parse_quote! { #[allow(type_alias_bounds)]#vis type #ident #generics = #fp_type; }

}

fn find_func(item: &ItemTrait, name: &str) -> Result<TraitItemFn, TokenStream> {
    item.items
        .iter()
        .find_map(|item| {
            if let TraitItem::Fn(func) = item {
                if func.sig.ident.to_string() == name {
                    return Some(func.clone());
                }
            }
            None
        })
        .ok_or(new_error(&format!(
            "Trait must contain function named '{name}'"
        )))
}

fn sig_to_fp(sig: Signature) -> Result<TypeBareFn, TokenStream> {
    let Signature {
        unsafety,
        abi,
        fn_token,
        paren_token,
        inputs,
        variadic,
        output,
        ..
    } = sig;

    Ok(TypeBareFn {
        lifetimes: None,
        unsafety,
        abi,
        fn_token,
        paren_token,
        inputs: args_to_bare_args(inputs)?,
        variadic: variadic_to_bare_variadic(variadic),
        output,
    })
}

fn args_to_bare_args(
    args: Punctuated<FnArg, Comma>,
) -> Result<Punctuated<BareFnArg, Comma>, TokenStream> {
    args.into_iter()
        .map(|arg| {
            let FnArg::Typed(PatType { attrs, ty, .. }) = arg else {
                return Err(new_error("Trait must contain function named 'safe'"));
            };
            Ok(BareFnArg {
                attrs,
                name: None,
                ty: (*ty).clone(),
            })
        })
        .collect()
}

fn variadic_to_bare_variadic(variadic: Option<Variadic>) -> Option<BareVariadic> {
    let Variadic {
        attrs, dots, comma, ..
    } = variadic?;
    Some(BareVariadic {
        attrs,
        name: None,
        dots,
        comma,
    })
}

fn new_error(msg: &str) -> TokenStream {
    syn::Error::new(Span::call_site().into(), msg)
        .to_compile_error()
        .into()
}
