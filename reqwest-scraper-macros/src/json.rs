use darling::{FromDeriveInput, FromField};
use proc_macro::TokenStream;

#[derive(Debug, FromDeriveInput)]
struct JsonPathScraper {
    ident: syn::Ident,
    vis: syn::Visibility,
    generics: syn::Generics,
    data: darling::ast::Data<(), JsonPathField>,
}

#[derive(Debug, FromField)]
struct JsonPathField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
}

pub fn expand_derive_from_response(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    todo!()
}
