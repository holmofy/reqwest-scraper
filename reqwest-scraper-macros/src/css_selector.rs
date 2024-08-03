use std::str::FromStr;

use crate::utils::syn::{get_type_detail, PathType};
use darling::{ast::Data, util::Flag, FromDeriveInput, FromField, FromMeta};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use scraper::Selector;
use syn::{spanned::Spanned, DeriveInput, Error, Result};

#[derive(Debug)]
struct CssSelector(String);

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(selector), supports(struct_named))]
struct CssSelectorScraper {
    ident: syn::Ident,
    generics: syn::Generics,
    data: Data<(), CssSelectorStructField>,
    path: Option<CssSelector>,
}

#[derive(Debug, FromField)]
#[darling(attributes(selector))]
struct CssSelectorStructField {
    ident: Option<syn::Ident>,
    ty: syn::Type,
    path: Option<String>,
    default: Option<String>,
    name: Flag,
    id: Flag,
    text: Flag,
    html: Flag,
    inner_html: Flag,
    has_class: Option<String>,
    attr: Option<String>,
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
        .ok_or_else(|| Error::new(input.span(), "css selector should never be used on enum"))?
        .fields;

    let field_extractors = generate_field_extractors(fields)?;

    Ok(match scraper.path {
        Some(selector) => {
            quote! {
                impl #impl_generics ::reqwest_scraper::FromCssSelector for #type_name #ty_generics #where_clause {
                    type CssSelectorExtractResult = ::reqwest_scraper::error::Result<std::vec::Vec<Self>>;
                    fn from_html(html: ::reqwest_scraper::css_selector::Html) -> Self::CssSelectorExtractResult {
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
            impl #impl_generics ::reqwest_scraper::FromCssSelector for #type_name #ty_generics #where_clause {
                type CssSelectorExtractResult = ::reqwest_scraper::error::Result<Self>;
                fn from_html(html: ::reqwest_scraper::css_selector::Html) -> Self::CssSelectorExtractResult {
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
                "css selector struct should never be tuple struct",
            )
        })?;
        let extractor = Extractor::from_field(f)?;
        let default = &f.default;
        let ty = get_type_detail(&f.ty);
        if default.is_none() && ty.is_other() {
            return Err(Error::new(
                field_ident.span(),
                "Non-option field need to be given a default value: selector(default=\"xxx\")",
            ));
        }
        tokens.push(match &f.path {
            Some(selector) => {
                match ty {
                    PathType::Option=>quote! {
                        #field_ident: item.select(#selector)?.first().and_then(#extractor).into()
                    },
                    PathType::Vector=>quote! {
                        #field_ident: item.select(#selector)?.iter()
                                        .map(|item|::std::option::Option::Some(item).and_then(#extractor))
                                        .filter(|o|o.is_some())
                                        .map(|o|o.unwrap().into())
                                        .collect::<::std::vec::Vec<_>>()
                    },
                    PathType::Other=>quote! {
                        #field_ident: item.select(#selector)?.first().and_then(#extractor).unwrap_or(#default.into()).into()
                    }
                }
            }
            None => {
                match ty {
                    PathType::Option=>quote! {
                        #field_ident: ::std::option::Option::Some(item).and_then(#extractor).into()
                    },
                    PathType::Vector=>{
                        return Err(Error::new(
                            field_ident.span(),
                            "Vec field must has selector path: selector(path=\"xxx\")",
                        ));
                    },
                    PathType::Other=>quote! {
                        #field_ident: ::std::option::Option::Some(item).and_then(#extractor).unwrap_or(#default.into()).into()
                    }
                }
            },
        })
    }
    return Ok(tokens);
}

#[derive(Debug)]
pub enum Extractor {
    Name,
    Id,
    Text,
    Html,
    InnerHtml,
    HasClass(String),
    Attr(String),
}

impl Extractor {
    fn from_field(field: &CssSelectorStructField) -> Result<Self> {
        let mut exists = 0;
        let mut result = Self::Html;
        let mut span: Span = field.ident.span();
        if field.id.is_present() {
            exists += 1;
            result = Self::Id;
            span = field.id.span();
        }
        if field.name.is_present() {
            exists += 1;
            result = Self::Name;
            span = field.name.span();
        }
        if field.text.is_present() {
            exists += 1;
            result = Self::Text;
            span = field.text.span();
        }
        if field.html.is_present() {
            exists += 1;
            result = Self::Html;
            span = field.html.span();
        }
        if field.inner_html.is_present() {
            exists += 1;
            result = Self::InnerHtml;
            span = field.inner_html.span();
        }
        if let Some(class) = &field.has_class {
            exists += 1;
            result = Self::HasClass(class.into());
            span = field.has_class.span();
        }
        if let Some(attr) = &field.attr {
            exists += 1;
            result = Self::Attr(attr.into());
            span = field.attr.span();
        }
        if exists <= 1 {
            return Ok(result);
        } else {
            return Err(Error::new(
                span,
                "[id,name,text,html,inner_html,has_class=\"class_name\",attr=\"name\"] must select only one",
            ));
        }
    }
}

impl quote::ToTokens for Extractor {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(match self {
            Self::Name => quote! {|e|Some(e.name().to_string())},
            Self::Id => quote! {|e|e.id().map(|id|id.to_string())},
            Self::Text => quote! {|e|Some(e.text())},
            Self::Html => quote! {|e|Some(e.html())},
            Self::InnerHtml => quote! {|e|Some(e.inner_html())},
            Self::HasClass(class) => quote! {|e|Some(e.has_class(#class,::reqwest_scraper::css_selector::CaseSensitivity::CaseSensitive))},
            Self::Attr(attr) => quote! {|e|e.attr(#attr).map(|v|v.to_string())},
        })
    }
}

impl FromStr for CssSelector {
    type Err = syn::Error;

    fn from_str(selector: &str) -> Result<Self> {
        Selector::parse(selector).map(|_| ()).map_err(|err| {
            syn::Error::new(
                Span::call_site(),
                format!("invalid css selector `{}`: {:?}", selector, err),
            )
        });
        Ok(CssSelector(selector.to_string()))
    }
}

impl FromMeta for CssSelector {
    fn from_string(s: &str) -> darling::Result<Self> {
        s.parse().map_err(darling::Error::from)
    }
}

impl quote::ToTokens for CssSelector {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let selector = &self.0;
        tokens.extend(quote! {#selector})
    }
}

#[test]
fn test_select_list() -> Result<()> {
    let input = r#"
#[derive(FromCssSelector)]
#[selector(path = ".list")]
pub struct ExtractListResult {
    #[selector(path = ".list-item>a", default="link_text_default_value", text)]
    link_text: Vec<String>,

    #[selector(default="item_class_default_value")]
    item_html: String,

    #[selector(attr="class")]
    item_class: Option<String>
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
    #[selector(path = ".link>a", default="\"xxx\"", text)]
    link_text: String,

    #[selector(path = ".link>a", html)]
    item_html: Option<String>,
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
