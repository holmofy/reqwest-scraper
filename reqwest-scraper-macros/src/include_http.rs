use std::collections::HashMap;

use lazy_static::lazy_static;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use regex::Regex;
use syn::Ident;

pub fn expand_macro(file_path: String) -> syn::Result<TokenStream> {
    let http_content = std::fs::read_to_string(&file_path).map_err(move |e| {
        syn::Error::new(
            Span::call_site(),
            format!("Failed to read {file_path}: {e}"),
        )
    })?;

    let requests = parse_http_file(&http_content);
    eprintln!("request count: {}", requests.len());
    Ok(quote! {#(#requests)*})
}

lazy_static! {
    static ref SNIPPET_SPLITTER: Regex = Regex::new(r"#{3,}").unwrap();
    static ref HTTP_RE: Regex = Regex::new(r"(?<name>\w+)\n(?<method>GET|POST|HEAD|PUT|DELETE|PATCH|HEAD|OPTIONS|TRACE)\s*(?<url>https?://\S+)(?:\n(?<headers>(?:\S+:\s+[^\n]+\n)*)\n?(?<body>[\s\S]*))?").unwrap();
    static ref HEADER_RE: Regex = Regex::new(r"(?<key>\S+):\s*(?<value>[^\n]+)").unwrap();
    static ref VARIABLE_RE: Regex = Regex::new(r"{{(?<ident>\w+)(?::\s*(?<ty>\w+))?}}").unwrap();
}

fn parse_http_file(content: &str) -> Vec<HttpRequestFn> {
    let snippets: Vec<&str> = SNIPPET_SPLITTER.split(content).collect();
    snippets
        .into_iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .flat_map(parse_http)
        .collect()
}

fn parse_http(content: &str) -> Vec<HttpRequestFn> {
    let mut request_fns = vec![];
    eprintln!("content: {content}");
    for caps in HTTP_RE.captures_iter(content) {
        let name = caps.name("name").unwrap().as_str();
        let method = caps.name("method").unwrap().as_str();
        let url = caps.name("url").unwrap().as_str();

        eprintln!(">>>pared>>>>>{name}: {method} {url}");
        let mut headers = HashMap::new();
        if let Some(headers_str) = caps.name("headers") {
            for caps in HEADER_RE.captures_iter(headers_str.as_str()) {
                let key = caps.name("key").unwrap().as_str();
                let value = caps.name("value").unwrap().as_str();
                headers.insert(key, StrEnum::new(value));
            }
        }

        // Optional body
        let body = caps.name("body").map(|b| b.as_str());
        request_fns.push(HttpRequestFn {
            name,
            request: HttpRequest {
                method,
                url: StrEnum::new(url),
                headers,
                body: body.map(StrEnum::new),
            },
        })
    }
    request_fns
}

struct HttpRequestFn<'f> {
    name: &'f str,
    request: HttpRequest<'f>,
}

struct HttpRequest<'f> {
    method: &'f str,
    url: StrEnum<'f>,
    headers: HashMap<&'f str, StrEnum<'f>>,
    body: Option<StrEnum<'f>>,
}

impl<'f> ToTokens for HttpRequestFn<'f> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { name, request } = self;
        let method_name_ident = Ident::new(name, Span::call_site());
        tokens.extend(quote! {
            pub async fn #method_name_ident() -> Result<reqwest::Response, reqwest::Error> {
                #request
            }
        });
    }
}

impl<'f> ToTokens for HttpRequest<'f> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            method,
            url,
            headers,
            body,
        } = self;
        let method = method.to_lowercase();
        let method_ident = Ident::new(&method, Span::call_site());
        tokens.extend(quote! {reqwest::Client::new().#method_ident(#url)});
        for (key, value) in headers {
            tokens.extend(quote! {.header(#key, #value)});
        }
        if let Some(body) = body {
            tokens.extend(quote! {.body(#body)});
        }
        tokens.extend(quote! {.send().await})
    }
}

enum StrEnum<'f> {
    RawStr(&'f str),
    Format(FormatInterpolator<'f>),
}

impl<'f> StrEnum<'f> {
    fn new(string: &'f str) -> Self {
        match VARIABLE_RE.captures(string){
            None=>Self::RawStr(string),
            Some(captures)=>{
                None
            }
        }
    }
}

impl<'f> ToTokens for StrEnum<'f> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::RawStr(string) => tokens.extend(quote! {#string}),
            Self::Format(fmt) => tokens.extend(quote! {r#"format!("{}")"#}),
        }
    }
}

struct FormatInterpolator<'f> {
    format: String,
    args: Vec<FormatArg<'f>>,
}

struct FormatArg<'f> {
    name: &'f str,
    ty: &'f str,
}
