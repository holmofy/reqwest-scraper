use darling::{FromDeriveInput, FromField, FromMeta};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use scraper::Selector;

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
    attr: Option<String>,
    default: Option<String>,
}

pub fn expand_derive_from_response(input: syn::DeriveInput) -> syn::Result<TokenStream> {
    let scraper = CssSelectorScraper::from_derive_input(&input)?;

    let type_name = scraper.ident;

    let fields = scraper
        .data
        .as_ref()
        .take_struct()
        .expect("Should never be enum")
        .fields;

    let field_extractors = fields
        .into_iter()
        .map(|f| {
            let field_name = f.ident.as_ref().expect("Should never be tuple struct");
            let field_name_str = field_name.to_string();
            let attr = f.attr.as_ref().unwrap_or(&field_name_str);
            let default = &f.default;
            //f.ty.
            match &f.selector {
                Some(selector) => quote! {
                    #field_name: item.select(#selector)?.first()?.attr(#attr).unwrap_or(#default)
                },
                None => quote! {
                    #field_name: item.attr(#attr).unwrap_or(#default)
                },
            }
        })
        .collect::<Vec<_>>();

    Ok(match scraper.selector {
        Some(selector) => {
            check_selector(&selector)?;
            quote! {
                impl reqwest_scraper::FromCssSelector for #type_name {
                    type CssSelectorExtractResult = reqwest_scraper::error::Result<std::vec::Vec<Self>>;
                    fn from(html: reqwest_scraper::css_selector::Html) -> Self::CssSelectorExtractResult {
                        let list = html.select(#selector)?;
                        let result: Vec<Self> = std::vec::Vec::new();

                        for item in list.iter() {
                            let extract_item = Self {
                                #(#field_extractors),*
                            }
                            result.push(extract_item);
                        }

                        Ok(result)
                    }
                }
            }
        }
        None => quote! {
            impl reqwest_scraper::FromCssSelector for #type_name {
                type CssSelectorExtractResult = reqwest_scraper::error::Result<Self>;
                fn from(html: reqwest_scraper::css_selector::Html) -> Self::CssSelectorExtractResult {
                    let item = &html;

                    Ok(Self {
                        #(#field_extractors),*
                    })
                }
            }
        },
    })
}

fn check_selector(selector: &str) -> syn::Result<()> {
    Selector::parse(selector).map(|_| ()).map_err(|err| {
        syn::Error::new(
            Span::call_site(),
            format!("invalid css selector `{}`: {:?}", selector, err),
        )
    })
}

#[test]
fn test_select_list() {
    let input = r#"
#[derive(FromCssSelector)]
#[css_selector(selector = ".list")]
pub struct ExtractListResult {
    value: Option<String>,

    #[css_selector(selector = ".list-item>a", attr = "text")]
    link_text: String,

    #[css_selector(attr = "text")]
    item_class: String,

    #[css_selector(attr = "data")]
    item_data: Option<String>
}"#;

    let parsed = syn::parse_str(input).unwrap();
    let receiver = CssSelectorScraper::from_derive_input(&parsed).unwrap();
    let tokens = expand_derive_from_response(parsed).unwrap();

    println!(
        "INPUT: \n{}\n\nPARSED AS: \n{:#?}\n\nEMITS: \n{}",
        input, receiver, tokens
    );
}

#[test]
fn test_select_item() {
    let input = r#"
#[derive(FromCssSelector)]
pub struct ExtractListResult {
    #[css_selector(selector = ".link>a", attr = "Text")]
    link_text: String,

    #[css_selector(selector = ".link>a", attr = "Html")]
    item_html: String,
}"#;

    let parsed = syn::parse_str(input).unwrap();
    let receiver = CssSelectorScraper::from_derive_input(&parsed).unwrap();
    let tokens = expand_derive_from_response(parsed).unwrap();

    println!(
        "INPUT: \n{}\n\nPARSED AS: \n{:#?}\n\nEMITS: \n{}",
        input, receiver, tokens
    );
}
