use core::{fmt::Display, mem::MaybeUninit, array, fmt};

use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{
    Attribute, BareFnArg, BareVariadic, FnArg, GenericParam, Generics, Ident, ItemFn, ItemImpl, ItemTrait, ItemType, PatType, Signature, Token, TraitItem, TraitItemFn, TypeBareFn, Variadic, Visibility, parse::Parse, parse_macro_input, parse_quote, punctuated::Punctuated, token::Comma
};

struct AttrArgs {
    struct_ident: Ident,
    trait_ident: Ident,
}

impl Parse for AttrArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let trait_ident = input.parse()?;
        let _: Token![for] = input.parse()?;
        let struct_ident = input.parse()?;
        Ok(Self {
            struct_ident,
            trait_ident,
        })
    }
}

trait ArrayIter: Iterator {
    fn collect_array<const N: usize>(self) -> Result<[Self::Item;N], ArrayBoundsError> where Self: Sized{
        let collector: ArrayCollector<_,_> = self.collect();
        collector.0
    }
}

impl<T> ArrayIter for T where T: Iterator + ?Sized {}

struct ArrayCollector<T, const N:usize>(Result<[T;N], ArrayBoundsError>);


impl<T, const N:usize> FromIterator<T> for ArrayCollector<T, N> {
    #[expect(clippy::unnecessary_safety_comment, clippy::undocumented_unsafe_blocks,
        clippy::indexing_slicing, reason = "1 safety comment explains the functions safety")]
    fn from_iter<B: IntoIterator<Item = T>>(iter: B) -> Self {
        // SAFETY: The result items are guaranteed to be uninit until the for loop.
        // After the ith iteration indices from 0 to i are guaranteed to be init by passing through the if branch.
        // In case the iterator ends before N elements got initialized the first i elements that are init get droped.
        // If the iterator contains more then N elements all elements in result get droped.
        // Once the iter is empty and N(all) elements in the array got initialized it is safe to map all elements with assume_init.
        let mut result = array::from_fn::<_, N,_>(|_| MaybeUninit::<T>::uninit());
        let mut iterator = iter.into_iter();
        for i in 0..N {
            let item = iterator.next();
            if let Some(item_t) = item {
                result[i] = MaybeUninit::new(item_t);
            } else {
                result.iter_mut().take(i).for_each(|init_ele| unsafe { init_ele.assume_init_drop()});
                return Self(Err(ArrayBoundsError::Only(i, N)));
            }
        }
        if iterator.next().is_some() {
            result.iter_mut().for_each(|init_ele| unsafe { init_ele.assume_init_drop()});
            return Self(Err(ArrayBoundsError::MoreThan(N)));
        } 
        Self(Ok(result.map(|init_ele| unsafe { init_ele.assume_init()})))
    }
}

enum ArrayBoundsError {
    MoreThan(usize),
    Only(usize, usize),
}

impl Display for ArrayBoundsError {
    #[expect(clippy::arithmetic_side_effects, reason = "adding 1 to display doesn't matter")]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Only(only, from) => write!(f, "Expected {from} element(s), found {only}."),
            Self::MoreThan(from) => write!(f, "Expected {from} elements, found {} or more.", from+1)
        }
    }
}



#[proc_macro_attribute]
pub fn plugin_main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let item_fn  = parse_macro_input!(item as ItemFn);
    let item_name = item_fn.sig.ident.clone();
    quote! {
        use plugin_loader_api::{CUuid, CPluginInfo};
        #[unsafe(no_mangle)]
        pub unsafe extern "C" fn plugin_main(uuid: CUuid) -> CPluginInfo {
            #item_name(uuid.into()).into()
        }

        #item_fn
    }.into()
}

#[proc_macro_attribute]
pub fn trait_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(attr as AttrArgs);
    let item_fn = syn::parse::<ItemFn>(item);
    let struct_name = attr_args.struct_ident;
    let trait_ident = attr_args.trait_ident;
    let result = match item_fn {
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
    match fn_trait_result(trait_item) {
        Ok(ok) => ok,
        Err(err) => err,
    }
}

#[expect(clippy::single_call_fn, reason = "function extracted for visibility")]
fn fn_trait_result(mut item: ItemTrait) -> Result<TokenStream, TokenStream> {
    let trait_vis = &item.vis;
    let trait_name = &item.ident;
    //signature
    let [sig_func] = find_func_by_attr(item.items.iter_mut(), &parse_quote!(#[sig]))
            .collect_array()
            .map_err(annotation_error("sig"))?;
    let sig_function_type = sig_to_fp(sig_func.sig.clone())?;
    let sig_fp_getter = sig_getter(sig_func, &sig_function_type);
    let sig_fp_type = create_type(trait_vis, trait_name, "SafeFP", &sig_func.sig.generics, &sig_function_type);

    //adapter
    let [adapter_func] = find_func_by_attr(item.items.iter_mut(), &parse_quote!(#[adapter]))
            .collect_array()
            .map_err(annotation_error("adapter"))?;
    let adapter_function_type = sig_to_fp(adapter_func.sig.clone())?;
    let adapter_fp_getter = adapter_getter(adapter_func, &adapter_function_type);
    let adapter_fp_type = create_type(trait_vis, trait_name, "UnsafeFP", &adapter_func.sig.generics, &adapter_function_type);

    //fp_adapter
    
    let [fp_adapter] = find_func_by_attr(item.items.iter_mut(), &parse_quote!(#[fp_adapter]))
        .collect_array()
        .map_err(annotation_error("fp_adapter"))?;
    let (fp_adapter_trait, fp_adapter_trait_impl) = fp_adapter_trait(fp_adapter, trait_name, trait_vis)?;


    //update trait elements
    let fp_adapter_ident = fp_adapter.sig.ident.clone();
    item.items.retain(|trait_item| if let TraitItem::Fn(item_fn) = trait_item { item_fn.sig.ident != fp_adapter_ident} else {true});
    item.items.append(&mut vec![sig_fp_getter, adapter_fp_getter]);
    
    Ok(quote! {
        #item
        #sig_fp_type
        #adapter_fp_type
        #fp_adapter_trait
        #fp_adapter_trait_impl
    }.into())
}

#[expect(clippy::single_call_fn, reason = "function extracted for visibility")]
fn sig_getter(sig_func: &TraitItemFn, sig_fn_type: &TypeBareFn) -> TraitItem {
    let sig_generics = &sig_func.sig.generics;
    let sig_func_name = &sig_func.sig.ident;
    let sig_getter_name = Ident::new(&format!("{}_fp", sig_func.sig.ident), Span::call_site().into());
    parse_quote! {
        #[inline]
        fn #sig_getter_name #sig_generics () -> #sig_fn_type {
            Self:: #sig_func_name
        }
    }
}

#[expect(clippy::single_call_fn, reason = "function extracted for visibility")]
fn adapter_getter(adapter_func: &TraitItemFn, adapter_fn_type: &TypeBareFn) ->TraitItem {
    let adapter_generics = &adapter_func.sig.generics;
    let mut call_site_generics = adapter_generics.clone();
    let adapter_func_name = &adapter_func.sig.ident;
    let adapter_getter_name = Ident::new(&format!("{}_fp", adapter_func.sig.ident), Span::call_site().into());
    call_site_generics.params.iter_mut().for_each(|param| if let GenericParam::Type(type_param) = param {
        type_param.colon_token = None;
        type_param.bounds = Punctuated::default();
        type_param.default = None;
    });
    let call_site_generic_params = call_site_generics.params;
    parse_quote! {
        #[inline]
        fn #adapter_getter_name #adapter_generics () -> #adapter_fn_type {
            Self:: #adapter_func_name ::<#call_site_generic_params>
        }
    }
}

#[expect(clippy::single_call_fn, reason = "function extracted for visibility")]
fn fp_adapter_trait(fp_adapter: &TraitItemFn, trait_name: &Ident, trait_vis: &Visibility) -> Result<(ItemTrait, ItemImpl), TokenStream> {
    
    let fp_adapter_name = &fp_adapter.sig.ident;
    let fp_adapter_trait_name = Ident::new(&format!("{trait_name}FPAdapter"), Span::call_site().into());
    let fp_adapter_generics = &fp_adapter.sig.generics;
    let fp_adapter_return_type = &fp_adapter.sig.output;

    let fp_adapter_arg = fp_adapter.sig.inputs.first().ok_or_else(|| new_error(format!("{} must take one argument.", fp_adapter.sig.ident)))?;
    let FnArg::Receiver(fp_adapter_receiver) = fp_adapter_arg else {
        return Err(new_error(format!("{} must take a self parameter.", fp_adapter.sig.ident)))
    };
    let fp_adapter_receiver_type = &*fp_adapter_receiver.ty;
    let fp_adapter_body = fp_adapter.default.as_ref().ok_or_else(|| new_error(format!("{} must have an implementation.", fp_adapter.sig.ident)))?;

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



#[expect(clippy::manual_inspect, reason = "inspect mut is not in std")]
fn find_func_by_attr<'item>(items: impl Iterator<Item= &'item mut TraitItem>, attr:&Attribute) -> impl Iterator<Item = &'item TraitItemFn> {
    items.filter_map(|item| if let TraitItem::Fn(func) = item { Some(func)} else {None})
            .filter(|func| func.attrs.contains(attr))
            .map(move |func| { func.attrs.retain(|attrib| attrib != attr); func})
            .map(|func| &*func)
}

fn annotation_error(annotation:&str) -> impl Fn(ArrayBoundsError) -> TokenStream {
    move |error| new_error(format!("Invalid number of #[{annotation}] annotated elements. {error}"))
}


fn create_type(vis: &Visibility, trait_name: &Ident, suffix:&str,  generics: &Generics, fp_type: &TypeBareFn) -> ItemType {
    let name = format!("{trait_name}{suffix}");
    let ident = Ident::new(&name, Span::call_site().into());
    parse_quote! { #[allow(type_alias_bounds)]#vis type #ident #generics = #fp_type; }

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

#[expect(clippy::single_call_fn, reason = "function extracted for visibility")]
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

#[expect(clippy::single_call_fn, reason = "function extracted for visibility")]
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
