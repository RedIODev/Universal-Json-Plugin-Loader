use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{DeriveInput, Ident, ItemFn, Token, parse::Parse, parse_macro_input};

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
    let item2 = item.clone();
    let attr = parse_macro_input!(attr as AttrArgs);
    let func = parse_macro_input!(item as ItemFn);
    let mut impl_func = func.clone();
    if let Some(name) = attr.trait_fn {
        impl_func.sig.ident = name;
    }
    impl_func.vis = syn::Visibility::Inherited;
    let trait_ident = attr.trait_ident;
    let struct_name = Ident::new(&func.sig.ident.to_string().to_uppercase(), Span::call_site().into());
    let struct_vis = func.vis;
    let implementation = quote! {
        #struct_vis struct #struct_name;
        impl #trait_ident for #struct_name {
            #impl_func
        }
    };
    implementation.into()
}