mod css_selector;
mod json;
mod xpath;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(FromCssSelector, attributes(css_selector))]
pub fn derive_css_selector(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    css_selector::expand_derive_from_response(input)
        .unwrap_or_else(|err| TokenStream::from(err.to_compile_error()))
        .into()
}

#[proc_macro_derive(FromJsonPath, attributes(jsonpath))]
pub fn derive_jsonpath(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    json::expand_derive_from_response(input)
        .unwrap_or_else(|err| TokenStream::from(err.to_compile_error()))
        .into()
}

#[proc_macro_derive(FromXPath, attributes(xpath))]
pub fn derive_xpath(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    xpath::expand_derive_from_response(input)
        .unwrap_or_else(|err| TokenStream::from(err.to_compile_error()))
        .into()
}