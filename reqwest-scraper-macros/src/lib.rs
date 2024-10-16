mod css_selector;
mod include_http;
mod utils;
mod xpath;

use include_http::IncludeHttp;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(FromCssSelector, attributes(selector))]
pub fn derive_css_selector(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    css_selector::expand_derive_from_response(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro_derive(FromXPath, attributes(xpath))]
pub fn derive_xpath(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    xpath::expand_derive_from_response(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

#[proc_macro]
pub fn include_http(input: TokenStream) -> TokenStream {
    let input: IncludeHttp = match syn::parse(input) {
        Ok(input) => input,
        Err(e) => return syn::Error::into_compile_error(e).into(),
    };
    include_http::expand_macro(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
