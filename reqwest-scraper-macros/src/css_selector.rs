use crate::utils::syn::is_option;
use darling::{ast::Data, FromDeriveInput, FromField};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use scraper::Selector;
use syn::{spanned::Spanned, DeriveInput, Error, Result};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(css_selector), supports(struct_named))]
struct CssSelectorScraper {
    ident: syn::Ident,
    generics: syn::Generics,
    data: Data<(), CssSelectorStructField>,
    selector: Option<String>,
}

#[derive(Debug, FromField)]
#[darling(attributes(css_selector))]
struct CssSelectorStructField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    selector: Option<String>,
    attr: Option<String>,
    default: Option<syn::Expr>,
}

pub fn expand_derive_from_response(input: DeriveInput) -> syn::Result<TokenStream> {
    let scraper = CssSelectorScraper::from_derive_input(&input)?;

    let type_name = scraper.ident;
    let generics = scraper.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let fields = scraper
        .data
        .as_ref()
        .take_struct()
        .ok_or_else(|| Error::new(input.span(), "css_selector should never be used on enum"))?
        .fields;

    let field_extractors = generate_field_extractors(fields)?;

    Ok(match scraper.selector {
        Some(selector) => {
            check_selector(&selector)?;
            quote! {
                impl #impl_generics reqwest_scraper::FromCssSelector for #type_name #ty_generics #where_clause {
                    type CssSelectorExtractResult = reqwest_scraper::error::Result<std::vec::Vec<Self>>;
                    fn from_html(html: reqwest_scraper::css_selector::Html) -> Self::CssSelectorExtractResult {
                        let list = html.select(#selector)?;
                        let mut result: Vec<Self> = std::vec::Vec::new();

                        for item in list.iter() {
                            let extract_item = Self {
                                #(#field_extractors),*
                            };
                            result.push(extract_item);
                        }

                        Ok(result)
                    }
                }
            }
        }
        None => quote! {
            impl #impl_generics reqwest_scraper::FromCssSelector for #type_name #ty_generics #where_clause {
                type CssSelectorExtractResult = reqwest_scraper::error::Result<Self>;
                fn from_html(html: reqwest_scraper::css_selector::Html) -> Self::CssSelectorExtractResult {
                    let item = &html;

                    Ok(Self {
                        #(#field_extractors),*
                    })
                }
            }
        },
    })
}

fn generate_field_extractors(fields: Vec<&CssSelectorStructField>) -> Result<Vec<TokenStream>> {
    let mut tokens = Vec::with_capacity(fields.len());
    for f in fields.into_iter() {
        let field_ident = f.ident.as_ref().ok_or_else(|| {
            Error::new(
                f.ident.span(),
                "css_selector struct should never be tuple struct",
            )
        })?;
        let field_name = field_ident.to_string();
        let attr = f.attr.as_ref().unwrap_or(&field_name);
        let default = &f.default;
        let is_option = is_option(&f.ty);
        if default.is_none() && !is_option {
            return Err(Error::new(
                field_ident.span(),
                "Non-option field need to be given a default value: css_selector(default=\"xxx\")",
            ));
        }
        tokens.push(match &f.selector {
            Some(selector) => {
                check_selector(&selector)?;
                if is_option {
                    quote! {
                        #field_ident: item.select(#selector)?.first().and_then(|e|e.attr(#attr)).attr(#attr)
                    }
                }else{
                    quote! {
                        #field_ident: item.select(#selector)?.first().and_then(|e|e.attr(#attr)).unwrap_or(#default).into()
                    }
                }
            }
            None => {
                if is_option {
                    quote! {
                        #field_ident: item.attr(#attr)
                    }
                }else{
                    quote! {
                        #field_ident: item.attr(#attr).unwrap_or(#default).into()
                    }
                }
            },
        })
    }
    return Ok(tokens);
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
fn test_select_list() -> Result<()> {
    let input = r#"
#[derive(FromCssSelector)]
#[css_selector(selector = ".list")]
pub struct ExtractListResult {
    value: Option<String>,

    #[css_selector(selector = ".list-item>a", attr = "text", default="link_text_default_value")]
    link_text: String,

    #[css_selector(attr = "text",default="item_class_default_value")]
    item_class: String,

    #[css_selector(attr = "data")]
    item_data: Option<String>
}"#;

    let parsed = syn::parse_str(input).unwrap();
    let receiver = CssSelectorScraper::from_derive_input(&parsed)?;
    let tokens = expand_derive_from_response(parsed)?;

    println!(
        "INPUT: \n{}\n\nPARSED AS: \n{:#?}\n\nEMITS: \n{}",
        input, receiver, tokens
    );

    Ok(())
}

#[test]
fn test_select_item() -> Result<()> {
    let input = r#"
#[derive(FromCssSelector)]
pub struct ExtractListResult {
    #[css_selector(selector = ".link>a", attr = "Text")]
    link_text: String,

    #[css_selector(selector = ".link>a", attr = "Html")]
    item_html: String,
}"#;

    let parsed = syn::parse_str(input).unwrap();
    let receiver = CssSelectorScraper::from_derive_input(&parsed)?;
    let tokens = expand_derive_from_response(parsed)?;

    println!(
        "INPUT: \n{}\n\nPARSED AS: \n{:#?}\n\nEMITS: \n{}",
        input, receiver, tokens
    );

    Ok(())
}
