use crate::utils::syn::{get_type_detail, PathType};
use darling::{ast::Data, FromDeriveInput, FromField};
use proc_macro2::TokenStream;
use quote::quote;
use syn::{spanned::Spanned, DeriveInput, Error, Result};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(xpath), supports(struct_named))]
struct XPathScraper {
    ident: syn::Ident,
    generics: syn::Generics,
    data: Data<(), XPathStructField>,
    path: Option<String>,
}

#[derive(Debug, FromField)]
#[darling(attributes(xpath))]
struct XPathStructField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    path: String,
    default: Option<String>,
}

pub fn expand_derive_from_response(input: DeriveInput) -> syn::Result<TokenStream> {
    let scraper = XPathScraper::from_derive_input(&input)?;

    let type_name = scraper.ident;
    let generics = scraper.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let fields = scraper
        .data
        .as_ref()
        .take_struct()
        .ok_or_else(|| Error::new(input.span(), "xpath should never be used on enum"))?
        .fields;

    Ok(match scraper.path {
        Some(xpath) => {
            let field_extractors = generate_list_item_field_extractors(fields)?;
            quote! {
                impl #impl_generics ::reqwest_scraper::FromXPath for #type_name #ty_generics #where_clause {
                    type XPathExtractResult = ::reqwest_scraper::error::Result<std::vec::Vec<Self>>;
                    fn from_xhtml(html: ::reqwest_scraper::xpath::XHtml) -> Self::XPathExtractResult {
                        let list = html.select(#xpath)?.as_nodes();
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
        None => {
            let field_extractors = generate_field_extractors(fields)?;
            quote! {
                impl #impl_generics ::reqwest_scraper::FromXPath for #type_name #ty_generics #where_clause {
                    type XPathExtractResult = ::reqwest_scraper::error::Result<Self>;
                    fn from_xhtml(html: ::reqwest_scraper::xpath::XHtml) -> Self::XPathExtractResult {
                        let item = &html;

                        Ok(Self {
                            #(#field_extractors),*
                        })
                    }
                }
            }
        }
    })
}

fn generate_list_item_field_extractors(fields: Vec<&XPathStructField>) -> Result<Vec<TokenStream>> {
    let mut tokens = Vec::with_capacity(fields.len());
    for f in fields.into_iter() {
        let field_ident = f.ident.as_ref().ok_or_else(|| {
            Error::new(f.ident.span(), "xpath struct should never be tuple struct")
        })?;
        let default = &f.default;
        let ty = get_type_detail(&f.ty);
        if default.is_none() && ty.is_other() {
            return Err(Error::new(
                field_ident.span(),
                "Non-option field need to be given a default value: xpath(default=\"xxx\")",
            ));
        }
        let xpath = &f.path;
        tokens.push(match ty {
            PathType::Option => quote! {
                #field_ident: item.findvalue(#xpath)?
            },
            PathType::Vector => quote! {
                #field_ident: item.findvalues(#xpath)?
            },
            PathType::Other => quote! {
                #field_ident: item.findvalue(#xpath)?.unwrap_or(#default.into())
            },
        })
    }
    Ok(tokens)
}

fn generate_field_extractors(fields: Vec<&XPathStructField>) -> Result<Vec<TokenStream>> {
    let mut tokens = Vec::with_capacity(fields.len());
    for f in fields.into_iter() {
        let field_ident = f.ident.as_ref().ok_or_else(|| {
            Error::new(f.ident.span(), "xpath struct should never be tuple struct")
        })?;
        let default = &f.default;
        let ty = get_type_detail(&f.ty);
        if default.is_none() && ty.is_other() {
            return Err(Error::new(
                field_ident.span(),
                "Non-option field need to be given a default value: xpath(default=\"xxx\")",
            ));
        }
        let xpath = &f.path;
        tokens.push(match ty {
            PathType::Option => quote! {
                #field_ident: item.select(#xpath)?.as_str().map(|s|s.into())
            },
            PathType::Vector => quote! {
                #field_ident: item.select(#xpath)?.as_strs().map(|s|s.into())
            },
            PathType::Other => quote! {
                #field_ident: item.select(#xpath)?.as_str().map(|s|s.into()).unwrap_or(#default)
            },
        });
    }
    Ok(tokens)
}

#[test]
fn test_select_list() -> Result<()> {
    let input = r#"
#[derive(FromXPath)]
#[xpath(path = "//div[id=\"root\"]")]
pub struct ExtractListResult {
    #[xpath(path = "/li/a/@text")]
    link_text: Vec<String>,

    #[xpath(path="/li/a", default="xxx")]
    item_html: String,

    #[xpath(path="/li/a/@class")]
    item_class: Option<String>
}"#;

    let parsed = syn::parse_str(input).unwrap();
    let receiver = XPathScraper::from_derive_input(&parsed)?;
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
#[derive(FromXPath)]
pub struct ExtractListResult {
    #[xpath(path = "/ul[id='list']/li/a/@text", default="xxx")]
    link_text: String,

    #[xpath(path = "/link/a")]
    item_html: Option<String>,
}"#;

    let parsed = syn::parse_str(input).unwrap();
    let receiver = XPathScraper::from_derive_input(&parsed)?;
    let tokens = expand_derive_from_response(parsed)?;

    println!(
        "INPUT: \n{}\n\nPARSED AS: \n{:#?}\n\nEMITS: \n{}",
        input, receiver, tokens
    );

    Ok(())
}
