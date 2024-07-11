use darling::{FromDeriveInput, FromField};
use proc_macro::TokenStream;

#[derive(Debug, FromDeriveInput)]
struct XPathScraper {
    ident: syn::Ident,
    vis: syn::Visibility,
    data: darling::ast::Data<(), XPathField>,
}

#[derive(Debug, FromField)]
struct XPathField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
}

pub fn expand_derive_from_response(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let scraper = XPathScraper::from_derive_input(&input)?;
    todo!()
}
