use std::{fmt::Display, mem::MaybeUninit};

use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{
    Attribute, BareFnArg, BareVariadic, FnArg, GenericParam, Generics, Ident, ItemFn, ItemImpl, ItemTrait, ItemType, PatType, Signature, Token, TraitItem, TraitItemFn, TypeBareFn, Variadic, Visibility, parse::Parse, parse_macro_input, parse_quote, punctuated::Punctuated, token::Comma
};



#[proc_macro_attribute]
pub fn plugin_main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item  = parse_macro_input!(item as ItemFn);
    let item_name = item.sig.ident.clone();
    quote! {
        use finance_together_api::c::{CUuid, CPluginInfo};
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn plugin_main(uuid: CUuid) -> CPluginInfo {
            #item_name(uuid.into()).into()
        }

        #item
    }.into()
}

struct AttrArgs {
    trait_ident: Ident,
    struct_ident: Ident,
}

impl Parse for AttrArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let trait_ident = input.parse()?;
        let _: Token![for] = input.parse()?;
        let struct_ident = input.parse()?;
        Ok(AttrArgs {
            trait_ident,
            struct_ident,
        })
    }
}

#[proc_macro_attribute]
pub fn trait_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as AttrArgs);
    let func = syn::parse::<ItemFn>(item);
    let struct_name = attr.struct_ident;
    let trait_ident = attr.trait_ident;
    let result = match func {
        Ok(func) => {
            let mut impl_func = func.clone();
            impl_func.vis = syn::Visibility::Inherited;
            let struct_vis = func.vis;
            quote! {
                #struct_vis struct #struct_name;
                impl #trait_ident for #struct_name {
                    #impl_func
                }
            }
        }
        Err(_) => {
            quote! {
                struct #struct_name;
                impl #trait_ident for #struct_name {

                }
            }
        }
    };
    result.into()
}

#[proc_macro_attribute]
pub fn fn_trait(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let trait_item = parse_macro_input!(item as ItemTrait);
    let x =match fn_trait_result(trait_item) {
        Ok(ok) => ok,
        Err(err) => err,
    };
    //println!("{}", x);
    x
}

fn fn_trait_result(mut item: ItemTrait) -> Result<TokenStream, TokenStream> {
    let trait_vis = &item.vis;
    let trait_name = &item.ident;
    //signature
    let [sig_func] = find_func_by_attr::<ArrayCollector<_,1>>(item.items.iter_mut(), &parse_quote!(#[sig])).0
            .map_err(annotation_error("sig"))?;
    let sig_fn_type = sig_to_fp(sig_func.sig.clone())?;
    let sig_fp_getter = sig_getter(&sig_func, &sig_fn_type)?;
    let sig_fp_type = create_type(&trait_vis, &trait_name, "SafeFP", &sig_func.sig.generics, &sig_fn_type);

    //adapter
    let [adapter_func] = find_func_by_attr::<ArrayCollector<_, 1>>(item.items.iter_mut(), &parse_quote!(#[adapter])).0
            .map_err(annotation_error("adapter"))?;
    let adapter_fn_type = sig_to_fp(adapter_func.sig.clone())?;
    let adapter_fp_getter = adapter_getter(&adapter_func, &adapter_fn_type)?;
    let adapter_fp_type = create_type(&trait_vis, &trait_name, "UnsafeFP", &adapter_func.sig.generics, &adapter_fn_type);

    //fp_adapter
    
    let [fp_adapter] = find_func_by_attr::<ArrayCollector<_,1>>(item.items.iter_mut(), &parse_quote!(#[fp_adapter])).0
        .map_err(annotation_error("fp_adapter"))?;
    let (fp_adapter_trait, fp_adapter_trait_impl) = fp_adapter_trait(fp_adapter, trait_name, trait_vis)?;


    //update trait elements
    let fp_adapter_ident = fp_adapter.sig.ident.clone();
    item.items.retain(|item| if let TraitItem::Fn(item) = item { item.sig.ident != fp_adapter_ident} else {true});
    item.items.append(&mut vec![sig_fp_getter, adapter_fp_getter]);
    
    Ok(quote! {
        #item
        #sig_fp_type
        #adapter_fp_type
        #fp_adapter_trait
        #fp_adapter_trait_impl
    }.into())
}

fn sig_getter(sig_func: &TraitItemFn, sig_fn_type: &TypeBareFn) -> Result<TraitItem, TokenStream> {
    let sig_generics = &sig_func.sig.generics;
    let sig_func_name = &sig_func.sig.ident;
    let sig_getter_name = Ident::new(&format!("{}_fp", sig_func.sig.ident), Span::call_site().into());
    Ok(parse_quote! {
        fn #sig_getter_name #sig_generics () -> #sig_fn_type {
            Self:: #sig_func_name
        }
    })
}

fn adapter_getter(adapter_func: &TraitItemFn, adapter_fn_type: &TypeBareFn) -> Result<TraitItem, TokenStream> {
    let adapter_generics = &adapter_func.sig.generics;
    let mut call_site_generics = adapter_generics.clone();
    let adapter_func_name = &adapter_func.sig.ident;
    let adapter_getter_name = Ident::new(&format!("{}_fp", adapter_func.sig.ident), Span::call_site().into());
    call_site_generics.params.iter_mut().for_each(|param| if let GenericParam::Type(t) = param {
        t.colon_token = None;
        t.bounds = Punctuated::default();
        t.default = None;
    });
    let call_site_generics = call_site_generics.params;
    Ok(parse_quote! {
        fn #adapter_getter_name #adapter_generics () -> #adapter_fn_type {
            Self:: #adapter_func_name ::<#call_site_generics>
        }
    })
}

fn fp_adapter_trait(fp_adapter: &TraitItemFn, trait_name: &Ident, trait_vis: &Visibility) -> Result<(ItemTrait, ItemImpl), TokenStream> {
    
    let fp_adapter_name = &fp_adapter.sig.ident;
    let fp_adapter_trait_name = Ident::new(&format!("{}FPAdapter", trait_name), Span::call_site().into());
    let fp_adapter_generics = &fp_adapter.sig.generics;
    let fp_adapter_return_type = &fp_adapter.sig.output;

    let fp_adapter_receiver = fp_adapter.sig.inputs.first().ok_or(new_error(&format!("{} must take one argument.", fp_adapter.sig.ident)))?;
    let FnArg::Receiver(fp_adapter_receiver) = fp_adapter_receiver else {
        return Err(new_error(&format!("{} must take a self parameter.", fp_adapter.sig.ident)))
    };
    let fp_adapter_receiver_type = &*fp_adapter_receiver.ty;
    let fp_adapter_body = fp_adapter.default.as_ref().ok_or(new_error(&format!("{} must have an implementation.", fp_adapter.sig.ident)))?;

    Ok((parse_quote! {
        #trait_vis trait #fp_adapter_trait_name {
            fn #fp_adapter_name #fp_adapter_generics (self) #fp_adapter_return_type;
        }
    },
    parse_quote! {
        impl #fp_adapter_trait_name for #fp_adapter_receiver_type {
            fn #fp_adapter_name #fp_adapter_generics (self) #fp_adapter_return_type #fp_adapter_body
        }
    }
    ))
}



fn find_func_by_attr<'a, B: FromIterator<&'a TraitItemFn>>(items: impl Iterator<Item= &'a mut TraitItem>, attr:&Attribute) -> B {
    items.filter_map(|item| if let TraitItem::Fn(func) = item { Some(func)} else {None})
            .filter(|func| func.attrs.contains(attr))
            .map(|func| { func.attrs.retain(|attrib| attrib != attr); func})
            .map(|func| &*func)
            .collect()
}

struct ArrayCollector<T, const N:usize>(Result<[T;N], ArrayBoundsError>);


impl<T, const N:usize> FromIterator<T> for ArrayCollector<T, N> {
    fn from_iter<B: IntoIterator<Item = T>>(iter: B) -> Self {
        let mut result = std::array::from_fn::<_, N,_>(|_| MaybeUninit::<T>::uninit());
        let mut iter = iter.into_iter();
        for i in 0..N {
            let item = iter.next();
            if let Some(item) = item {
                result[i] = MaybeUninit::new(item);
            } else {
                result.iter_mut().take(i).for_each(|init_ele| unsafe { init_ele.assume_init_drop()});
                return ArrayCollector(Err(ArrayBoundsError::Only(i, N)));
            }
        }
        if let Some(_) = iter.next() {
            result.iter_mut().for_each(|init_ele| unsafe { init_ele.assume_init_drop()});
            return ArrayCollector(Err(ArrayBoundsError::MoreThan(N)));
        } 
        ArrayCollector(Ok(result.map(|init_ele| unsafe { init_ele.assume_init()})))
    }
}



enum ArrayBoundsError {
    Only(usize, usize),
    MoreThan(usize)
}

impl Display for ArrayBoundsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArrayBoundsError::Only(only, from) => write!(f, "Expected {from} element(s), found {only}."),
            ArrayBoundsError::MoreThan(from) => write!(f, "Expected {from} elements, found {} or more.", from+1)
        }
    }
}

fn annotation_error(annotation:&str) -> impl Fn(ArrayBoundsError) -> TokenStream {
    move |e| new_error(format!("Invalid number of #[{}] annotated elements. {}", annotation, e))
}


// fn fn_trait_result(mut item: ItemTrait) -> Result<TokenStream, TokenStream> {
//     let adapter = find_func(&item, "adapter")?;
//     let safe = find_func(&item, "safe")?;
//     let safe_generics = safe.sig.generics.clone();
//     let adapter_generics = adapter.sig.generics.clone();
//     let adapter_fp_type = sig_to_fp(adapter.sig)?;
//     let safe_fp_type = sig_to_fp(safe.sig)?;
//     let mut call_site_adapter_generics = adapter_generics.clone();
//     call_site_adapter_generics.params.iter_mut().for_each(|param| if let GenericParam::Type(t) = param {
//         t.colon_token = None;
//         t.bounds = Punctuated::default();
//         t.default = None;
//     });
//     let params = call_site_adapter_generics.params;
//     let adapter_func = parse_quote! {
//         fn unsafe_fp #adapter_generics () -> #adapter_fp_type {
//             Self::adapter::<#params>
//         }
//     };
//     let safe_func = parse_quote! {
//         fn safe_fp #safe_generics () -> #safe_fp_type {
//             Self::safe
//         }
//     };

//     item.items.append(&mut vec![adapter_func, safe_func]);



//     let trait_vis = item.vis.clone();
//     let trait_name = item.ident.clone();
//     let safe_fp_type = create_type(&trait_vis, &trait_name, "SafeFP", &safe_generics, &safe_fp_type);
//     let adapter_fp_type = create_type(&trait_vis, &trait_name, "UnsafeFP", &Generics::default(), &adapter_fp_type);


//     let from_fp = find_func(&item, "from_fp")?;
//     item.items.retain(|item| *item != TraitItem::Fn(from_fp.clone()));
//     let to_safe_trait_name = Ident::new(&format!("{}ToSafe", trait_name), Span::call_site().into());

//     let from_fp_generics = from_fp.sig.generics;
//     let from_fp_return_type = from_fp.sig.output;
//     let from_fp_body = from_fp.default.ok_or(new_error("from_fp needs to have a block."))?;

//     let to_safe_trait:ItemTrait = parse_quote! {
//         #trait_vis trait #to_safe_trait_name {
//             fn to_safe #from_fp_generics (self) #from_fp_return_type;
//         } 
//     };

//     let from_fp_receiver = from_fp.sig.inputs.first().ok_or(new_error("from_fp needs to take a parameter."))?;
//     let FnArg::Receiver(from_fp_receiver) = from_fp_receiver else {
//         return Err(new_error("from_fp needs to take a self parameter."));
//     };
//     let from_fp_receiver_type_name = from_fp_receiver.ty.clone();
//     //let adapter_fp_type_name = adapter_fp_type.ident.clone();

//     let to_safe_impl: ItemImpl = parse_quote! {
//         impl #to_safe_trait_name for #from_fp_receiver_type_name {
//             fn to_safe #from_fp_generics (self) #from_fp_return_type #from_fp_body
//         }
//     };

//     Ok(quote! { 
//         #item
//         #safe_fp_type
//         #adapter_fp_type
//         #to_safe_trait
//         #to_safe_impl
//     }.into())
// }

fn create_type(vis: &Visibility, trait_name: &Ident, suffix:&str,  generics: &Generics, fp_type: &TypeBareFn) -> ItemType {
    let name = format!("{}{}", trait_name, suffix);
    let ident = Ident::new(&name, Span::call_site().into());
    parse_quote! { #[allow(type_alias_bounds)]#vis type #ident #generics = #fp_type; }

}

// fn find_func(item: &ItemTrait, name: &str) -> Result<TraitItemFn, TokenStream> {
//     item.items
//         .iter()
//         .find_map(|item| {
//             if let TraitItem::Fn(func) = item {
//                 if func.sig.ident.to_string() == name {
//                     return Some(func.clone());
//                 }
//             }
//             None
//         })
//         .ok_or(new_error(&format!(
//             "Trait must contain function named '{name}'"
//         )))
// }

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

fn new_error<S: Display>(msg: S) -> TokenStream {
    syn::Error::new(Span::call_site().into(), msg)
        .to_compile_error()
        .into()
}
