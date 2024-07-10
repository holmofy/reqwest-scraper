mod css_selector;
mod json;
mod xpath;
use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(FromResponse, attributes(xpath))]
pub fn derive(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    xpath::expand_derive_from_response(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
