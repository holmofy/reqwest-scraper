use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(FromResponse)]
pub fn derive(input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as DeriveInput);

    expand_derive_from_response(input)
}

fn expand_derive_from_response(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    
}
