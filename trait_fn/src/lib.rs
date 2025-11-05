use convert_case::Casing;
use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Generics, Ident, ItemFn, ItemTrait, ReturnType, Signature, Token, TraitItem, TraitItemFn, parse::Parse, parse_macro_input, parse_quote, punctuated::Punctuated, spanned::Spanned, token::Fn};

struct AttrArgs {
    trait_ident: Ident,
    trait_fn: Option<Ident>
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
        Ok(AttrArgs { trait_ident, trait_fn })
    }
}

#[proc_macro_attribute]
pub fn trait_fn(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = parse_macro_input!(attr as AttrArgs);
    let func = parse_macro_input!(item as ItemFn);
    let mut impl_func = func.clone();
    if let Some(name) = attr.trait_fn {
        impl_func.sig.ident = name;
    }
    impl_func.vis = syn::Visibility::Inherited;
    let trait_ident = attr.trait_ident;
    let struct_name = Ident::new(&func.sig.ident.to_string()
            .to_case(convert_case::Case::Pascal), Span::call_site().into());
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
    let mut trait_item = parse_macro_input!(item as ItemTrait);
    let adapter = trait_item.items.iter().find_map(find_adapter);
    let Some(adapter) = adapter else {
        return syn::Error::new(Span::call_site().into(), "Trait must contain function named 'adapter'").to_compile_error().into();
    };
    let adapter_args = adapter.sig.inputs;
    let adapter_result = adapter.sig.output;
    let func = parse_quote! { fn as_fp() -> unsafe extern "C" fn (#adapter_args) #adapter_result {
        Self::adapter
    }};
    trait_item.items.push(TraitItem::Fn(func));
    quote! { #trait_item}.into()
}

fn find_adapter(item: &TraitItem) -> Option<TraitItemFn> {
    if let TraitItem::Fn(func) = item {
        if func.sig.ident.to_string() == "adapter" {
            return Some(func.clone())
        }
    }
    None
}