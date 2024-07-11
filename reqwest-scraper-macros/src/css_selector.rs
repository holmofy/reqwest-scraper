use darling::{FromDeriveInput, FromField, FromMeta};
use proc_macro::TokenStream;
use quote::{quote, ToTokens};

#[derive(Debug, FromMeta)]
enum Attr {
    Text,
    Html,
    InnerHtml,
    Attr(String),
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(css_selector))]
struct CssSelectorScraper {
    ident: syn::Ident,
    vis: syn::Visibility,
    generics: syn::Generics,
    data: darling::ast::Data<(), CssSelectorField>,
    selector: Option<String>,
}

#[derive(Debug, FromField)]
#[darling(attributes(css_selector))]
struct CssSelectorField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    selector: Option<String>,
    attr: String,
}

pub fn expand_derive_from_response(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    Ok(TokenStream::from(
        CssSelectorScraper::from_derive_input(&input)?.into_token_stream(),
    ))
}

impl quote::ToTokens for CssSelectorScraper {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let type_name = &self.ident;
        match &self.selector {
            Some(selector) => tokens.extend(quote! {
                impl FromCssSelector for #type_name {
                    type CssSelectorExtractResult = Result<Vec<#type_name>>;
                    fn from(html: XHtml) -> Self::CssSelectorExtractResult {
                        let list = html.select(#selector)?;
                        let result: Vec<#type_name> = Vec::new();

                        for item in list.iter() {
                            
                        }

                        Ok(result)
                    }
                }
            }),
            None => tokens.extend(quote! {
                impl FromCssSelector for #type_name {
                    type CssSelectorExtractResult = Result<#type_name>;
                    fn from(html: XHtml) -> Self::CssSelectorExtractResult {
                        let item = html;



                    }
                }
            }),
        }
    }
}

#[test]
fn test_select_list() {
    let input = r#"#[derive(FromCssSelector)]
#[css_selector(selector = ".list")]
pub struct ExtractListResult {
    #[css_selector(selector = ".list-item>a", attr = "Text")]
    link_text: String,

    #[css_selector(attr = "Text")]
    item_class: String,
}"#;

    let parsed = syn::parse_str(input).unwrap();
    let receiver = CssSelectorScraper::from_derive_input(&parsed).unwrap();
    let tokens = quote!(#receiver);

    println!(
        "INPUT: \n{}\n\nPARSED AS: \n{:#?}\n\nEMITS: \n{}",
        input, receiver, tokens
    );
}

#[test]
fn test_select_item() {
    let input = r#"#[derive(FromCssSelector)]
pub struct ExtractListResult {
    #[css_selector(selector = ".link>a", attr = "Text")]
    link_text: String,

    #[css_selector(selector = ".link>a", attr = "Html")]
    item_html: String,
}"#;

    let parsed = syn::parse_str(input).unwrap();
    let receiver = CssSelectorScraper::from_derive_input(&parsed).unwrap();
    let tokens = quote!(#receiver);

    println!(
        "INPUT: \n{}\n\nPARSED AS: \n{:#?}\n\nEMITS: \n{}",
        input, receiver, tokens
    );
}
