use std::collections::HashMap;

use lazy_static::lazy_static;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use regex::Regex;

pub fn expand_macro(file_path: String) -> syn::Result<TokenStream> {
    let http_content = std::fs::read_to_string(&file_path).map_err(move |e| {
        syn::Error::new(
            Span::call_site(),
            format!("Failed to read {file_path}: {e}"),
        )
    })?;

    let requests = parse_http_file(&http_content);

    Ok(quote!{#(#requests)*})
}

lazy_static! {
    // 正则表达式解析 HTTP 方法和 URL
    static ref SNIPPET_SPLITTER: Regex = Regex::new(r"#{3,}").unwrap();
    static ref HTTP_RE: Regex = Regex::new(r"###\s*(?<name>\w+)\n(?<method>GET|POST|HEAD|PUT|DELETE|PATCH|HEAD|OPTIONS|TRACE)\s*(?<url>https?://\S+)\n(?<headers>(?:\S+:\s+[^\n]+\n)*)\n?(?<body>[\s\S]*)").unwrap();
    static ref HEADER_RE: Regex = Regex::new(r"(?<key>\S+):\s*(?<value>[^\n]+)").unwrap();
}

fn parse_http_file(content: &str) -> Vec<HttpRequestFn> {
    let snippets: Vec<&str> = SNIPPET_SPLITTER.split(content).collect();
    snippets
        .into_iter()
        .map(|s| s.trim())
        .flat_map(parse_http)
        .collect()
}

fn parse_http(content: &str) -> Vec<HttpRequestFn> {
    let mut request_fns = vec![];
    for caps in HTTP_RE.captures_iter(content) {
        let name = caps.name("name").unwrap().as_str();
        let method = caps.name("method").unwrap().as_str();
        let url = caps.name("url").unwrap().as_str();

        let mut headers = HashMap::new();
        if let Some(headers_str) = caps.name("headers") {
            for caps in HEADER_RE.captures_iter(headers_str.as_str()) {
                let key = caps.name("key").unwrap().as_str();
                let value = caps.name("value").unwrap().as_str();
                headers.insert(key, value);
            }
        }

        // Optional body
        let body = caps.name("body").map(|b| b.as_str());
        request_fns.push(HttpRequestFn {
            name,
            request: HttpRequest {
                method,
                url,
                headers,
                body,
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
    url: &'f str,
    headers: HashMap<&'f str, &'f str>,
    body: Option<&'f str>,
}

impl<'f> ToTokens for HttpRequestFn<'f> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { name, request } = self;
        tokens.extend(quote! {
            pub async fn #name() -> Result<reqwest::Response, reqwest::Error> {
                let client = reqwest::Client::new();
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
        tokens.extend(quote! {client.#method(#url)});
        for (key, value) in headers {
            tokens.extend(quote! {.header(#key, #value)});
        }
        if let Some(body) = body {
            tokens.extend(quote! {.body(#body)});
        }
        tokens.extend(quote!{.send().await})
    }
}
