use lazy_static::lazy_static;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use regex::Regex;
use std::collections::HashMap;
use syn::{Ident, LitStr, Token, Type};

pub fn expand_macro(input: IncludeHttp) -> syn::Result<TokenStream> {
    let IncludeHttp {
        file_path,
        client_supplier,
        variables,
    } = input;
    let raw_file_content = std::fs::read_to_string(&file_path).map_err(move |e| {
        syn::Error::new(
            Span::call_site(),
            format!("Failed to read {file_path}: {e}"),
        )
    })?;

    let mut http_content = parse_and_substitute(&raw_file_content);

    // 2. 追加宏里的变量定义
    if let Some(punctuated) = variables {
        for pair in punctuated {
            let path = &pair.path;
            let value = &pair.value;
            let key = format!("{{{{{}}}}}", quote::quote!(#path).to_string());
            let value = match value {
                syn::Expr::Lit(expr_lit) => {
                    if let syn::Lit::Str(ref lit_str) = expr_lit.lit {
                        lit_str.value()
                    } else {
                        "".to_string()
                    }
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        value,
                        "only string literal values are supported",
                    ))
                }
            };
            http_content = http_content.replace(&key, &value);
        }
    }

    let requests = parse_http_file(&http_content, &client_supplier);

    Ok(quote! {#(#requests)*})
}

// 替换@variable变量
fn parse_and_substitute(input: &str) -> String {
    let var_def_re = Regex::new(r"(?m)^@(\w+)\s*=\s*(.+)$").unwrap();
    let var_use_re = Regex::new(r"\{\{(\w+)\}\}").unwrap();

    let mut vars = HashMap::new();

    // 1. 提取所有变量定义
    for cap in var_def_re.captures_iter(&input) {
        let key = cap[1].to_string();
        let value = cap[2].trim().to_string();
        vars.insert(key, value);
    }

    // 2. 递归展开变量值
    fn resolve_var(
        key: &str,
        vars: &HashMap<String, String>,
        resolving: &mut Vec<String>,
    ) -> String {
        if resolving.contains(&key.to_string()) {
            return format!("{{{{{}}}}}", key); // 防止循环引用
        }
        if let Some(val) = vars.get(key) {
            resolving.push(key.to_string());
            let resolved = Regex::new(r"\{\{(\w+)\}\}")
                .unwrap()
                .replace_all(val, |caps: &regex::Captures| {
                    let inner_key = &caps[1];
                    resolve_var(inner_key, vars, resolving)
                })
                .to_string();
            resolving.pop();
            resolved
        } else {
            format!("{{{{{}}}}}", key)
        }
    }

    let resolved_vars: HashMap<_, _> = vars
        .keys()
        .map(|k| (k.clone(), resolve_var(k, &vars, &mut vec![])))
        .collect();

    // 3. 替换正文中的 {{变量}}
    let input_without_defs = var_def_re.replace_all(&input, "").to_string();
    let http_content = var_use_re.replace_all(&input_without_defs, |caps: &regex::Captures| {
        let key = &caps[1];
        resolved_vars
            .get(key)
            .cloned()
            .unwrap_or_else(|| format!("{{{{{}}}}}", key))
    });

    http_content.to_string()
}

pub struct IncludeHttp {
    file_path: String,
    client_supplier: Option<Ident>,
    variables: Option<syn::punctuated::Punctuated<syn::MetaNameValue, syn::token::Comma>>,
}

impl syn::parse::Parse for IncludeHttp {
    fn parse(args: syn::parse::ParseStream) -> syn::Result<Self> {
        let op = |mut err: syn::Error| {
            err.combine(syn::Error::new(
                err.span(),
                r#"invalid include_http args, expected include_http("<file_path>", [client_supplier], {[variable1=value]...})"#,
            ));

            err
        };
        let file_path = args.parse::<LitStr>().map_err(op)?.value();

        if !args.peek(Token![,]) {
            return Ok(Self {
                file_path,
                client_supplier: None,
                variables: None,
            });
        }

        args.parse::<Token![,]>().map_err(op)?;

        let client_supplier = if args.peek(syn::Ident) {
            let client_supplier = args.parse::<syn::Ident>().map_err(op)?;

            if !args.peek(Token![,]) {
                return Ok(Self {
                    file_path,
                    client_supplier: Some(client_supplier),
                    variables: None,
                });
            }
            args.parse::<Token![,]>().map_err(op)?;
            Some(client_supplier)
        } else {
            None
        };
        let variables;
        syn::braced!(variables in args);
        // zero or more options: name = "foo"
        let variables = variables
            .parse_terminated(syn::MetaNameValue::parse, Token![,])
            .map_err(op)?;
        Ok(Self {
            file_path,
            client_supplier,
            variables: Some(variables),
        })
    }
}

lazy_static! {
    // HTTP片段分割标识
    static ref SNIPPET_SPLITTER: Regex = Regex::new(r"#{3,}\s*@?").unwrap();
    // Http Request
    static ref HTTP_RE: Regex = Regex::new(r"(?<name>\w+)\n(?<method>GET|POST|HEAD|PUT|DELETE|PATCH|HEAD|OPTIONS|TRACE)\s*(?<url>https?://\S+)(?:\s+HTTP/[\d.]+)?(?:\n(?<headers>(?:\S+:\s+[^\n]+\n)*)(?:\n(?<body>[\s\S]*))?)?").unwrap();
    // Http header
    static ref HEADER_RE: Regex = Regex::new(r"(?<key>\S+):\s*(?<value>[^\n]+)").unwrap();
    // 变量插值
    static ref VARIABLE_RE: Regex = Regex::new(r"\{\{(?<env>\$)?(?<ident>\w+)(?::\s*(?<ty>\w+))?\}\}").unwrap();
}

fn parse_http_file<'f, 'c>(
    content: &'f str,
    client_supplier: &'c Option<Ident>,
) -> Vec<HttpRequestFn<'f, 'c>> {
    let snippets: Vec<&str> = SNIPPET_SPLITTER.split(content).collect();
    snippets
        .into_iter()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .flat_map(|s| parse_http(s, client_supplier))
        .collect()
}

fn parse_http<'f, 'c>(
    content: &'f str,
    client_supplier: &'c Option<Ident>,
) -> Vec<HttpRequestFn<'f, 'c>> {
    let mut request_fns = vec![];
    for caps in HTTP_RE.captures_iter(content) {
        let name = caps
            .name("name")
            .expect("Method name is not defined")
            .as_str();
        let method = caps
            .name("method")
            .expect("Http method is not defined")
            .as_str();
        let url = caps.name("url").expect("url is not defined").as_str();

        let mut headers = HashMap::new();
        if let Some(headers_str) = caps.name("headers") {
            for caps in HEADER_RE.captures_iter(headers_str.as_str()) {
                let key = caps
                    .name("key")
                    .expect("Http header key is not defined")
                    .as_str();
                let value = caps.name("value").map(|m| m.as_str()).unwrap_or("");
                headers.insert(key, StrEnum::new(value));
            }
        }

        // Optional body
        let body = caps.name("body").map(|b| b.as_str());
        request_fns.push(HttpRequestFn {
            name,
            client_supplier,
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

struct HttpRequestFn<'f, 'c> {
    name: &'f str,
    client_supplier: &'c Option<Ident>,
    request: HttpRequest<'f>,
}

struct HttpRequest<'f> {
    method: &'f str,
    url: StrEnum<'f>,
    headers: HashMap<&'f str, StrEnum<'f>>,
    body: Option<StrEnum<'f>>,
}

impl<'f> HttpRequest<'f> {
    fn collect_args(&self) -> Vec<FormatArg> {
        let Self {
            url, headers, body, ..
        } = self;
        let mut args = vec![];
        if let StrEnum::Format(fmt) = url {
            args = Self::push_arg_while_unique_name(args, fmt);
        }
        for (_, value) in headers {
            if let StrEnum::Format(fmt) = value {
                args = Self::push_arg_while_unique_name(args, fmt);
            }
        }
        if let Some(StrEnum::Format(fmt)) = body {
            args = Self::push_arg_while_unique_name(args, fmt);
        }
        args
    }

    fn push_arg_while_unique_name(
        mut args: Vec<FormatArg<'f>>,
        fmt: &FormatInterpolator<'f>,
    ) -> Vec<FormatArg<'f>> {
        for fmt_args in &fmt.args {
            if args.iter().any(|a| a.name == fmt_args.name) {
                continue;
            }
            args.push(fmt_args.clone());
        }
        args
    }

    fn collect_envs(&self) -> Vec<EnvVariable> {
        let Self {
            url, headers, body, ..
        } = self;
        let mut envs = vec![];
        if let StrEnum::Format(fmt) = url {
            envs = Self::push_env_while_unique_name(envs, fmt);
        }
        for (_, value) in headers {
            if let StrEnum::Format(fmt) = value {
                envs = Self::push_env_while_unique_name(envs, fmt);
            }
        }
        if let Some(StrEnum::Format(fmt)) = body {
            envs = Self::push_env_while_unique_name(envs, fmt);
        }
        envs
    }

    fn push_env_while_unique_name(
        mut envs: Vec<EnvVariable<'f>>,
        fmt: &FormatInterpolator<'f>,
    ) -> Vec<EnvVariable<'f>> {
        for fmt_env in &fmt.envs {
            if envs.iter().any(|a| a.name == fmt_env.name) {
                continue;
            }
            envs.push(fmt_env.clone());
        }
        envs
    }
}

impl<'f, 'c> ToTokens for HttpRequestFn<'f, 'c> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            name,
            client_supplier,
            request,
        } = self;
        let method_name_ident = Ident::new(name, Span::call_site());
        let args = request.collect_args();
        let envs = request.collect_envs();
        let client = match client_supplier {
            None => quote! {let client = reqwest::Client::new();},
            Some(supplier) => quote! {let client = #supplier();},
        };
        tokens.extend(quote! {
            pub async fn #method_name_ident(#(#args),*) -> ::std::result::Result<::reqwest::Response, ::reqwest::Error> {
                #client
                #(#envs)*
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
        tokens.extend(quote! {client.#method_ident(#url)});
        for (key, value) in headers {
            tokens.extend(quote! {.header(#key, #value)});
        }
        if let Some(body) = body {
            tokens.extend(quote! {.body(#body)});
        }
        tokens.extend(quote! {.send().await})
    }
}

#[derive(Debug, Clone)]
enum StrEnum<'f> {
    RawStr(&'f str),
    Format(FormatInterpolator<'f>),
}

impl<'f> StrEnum<'f> {
    fn new(string: &'f str) -> Self {
        match VARIABLE_RE.captures(string) {
            None => Self::RawStr(string),
            Some(_) => {
                let mut fmt = String::with_capacity(string.len());
                let mut args = vec![];
                let mut envs = vec![];
                let mut last_match = 0;
                for caps in VARIABLE_RE.captures_iter(string) {
                    let matched = caps.get(0).unwrap();
                    let name = match caps.name("env") {
                        Some(_) => {
                            let name = caps.name("ident").unwrap().as_str();
                            let default_value = caps.name("ty").map(|ty| ty.as_str());
                            envs.push(EnvVariable {
                                name,
                                default_value,
                            });
                            name.to_lowercase()
                        }
                        None => {
                            let name = caps.name("ident").unwrap().as_str();
                            let ty = caps.name("ty").map(|ty| ty.as_str());
                            args.push(FormatArg { name, ty });
                            name.to_string()
                        }
                    };
                    // format!转义，要保留原始{}，得{{}}
                    fmt.push_str(
                        &string[last_match..matched.start()]
                            .replace("{", "{{")
                            .replace("}", "}}"),
                    );
                    fmt.push_str(&format!(r"{{{name}}}"));
                    last_match = matched.end();
                }
                fmt.push_str(&string[last_match..].replace("{", "{{").replace("}", "}}"));
                Self::Format(FormatInterpolator { fmt, args, envs })
            }
        }
    }
}

impl<'f> ToTokens for StrEnum<'f> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::RawStr(string) => tokens.extend(quote! {#string}),
            Self::Format(FormatInterpolator { fmt, .. }) => tokens.extend(quote! {format!(#fmt)}),
        }
    }
}

#[derive(Debug, Clone)]
struct FormatInterpolator<'f> {
    fmt: String,
    args: Vec<FormatArg<'f>>,
    envs: Vec<EnvVariable<'f>>,
}

#[derive(Debug, Clone, Copy)]
struct FormatArg<'f> {
    name: &'f str,
    ty: Option<&'f str>,
}

impl<'f> ToTokens for FormatArg<'f> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { name, ty } = self;
        let name = Ident::new(name, Span::call_site());
        let ty = ty.unwrap_or("&str");
        let ty: Type = syn::parse_str(ty).expect(&format!("type is invalid: {ty}"));
        tokens.extend(quote! {#name: #ty});
    }
}

#[derive(Debug, Clone, Copy)]
struct EnvVariable<'f> {
    name: &'f str,
    default_value: Option<&'f str>,
}

impl<'f> EnvVariable<'f> {
    fn lowercase_name(&self) -> String {
        self.name.to_lowercase()
    }
}

impl<'f> ToTokens for EnvVariable<'f> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self {
            name,
            default_value,
        } = self;
        let variable_name = name;
        let name = Ident::new(&self.lowercase_name(), Span::call_site());
        match default_value {
            Some(default_value) => {
                tokens.extend(quote! {
                    let #name = std::env::var(#variable_name).ok().unwrap_or_else(||#default_value.to_string());
                });
            }
            None => {
                tokens.extend(
                    quote! { let #name = ::std::env::var(#variable_name).ok().unwrap_or_default();},
                );
            }
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_http() {
        let req = r####"
### request_baidu
GET https://www.baidu.com
"####;
        let http = parse_http(&req, &None);
        assert_eq!(http.len(), 1);
        let req = http.get(0).unwrap();
        assert_eq!(req.name, "request_baidu");
        assert_eq!(req.request.method, "GET");

        assert_eq!(
            match req.request.url {
                StrEnum::RawStr(url) => url,
                StrEnum::Format(_) => "fmt",
            },
            "https://www.baidu.com"
        );

        let req = r####"
###//comment
### request_baidu
GET https://www.baidu.com/s?kw=xxx
User-Agent: reqwest
token: xxxx1234ABCD
"####;
        let http = parse_http(&req, &None);
        assert_eq!(http.len(), 1);
        let req = http.get(0).unwrap();
        assert_eq!(req.name, "request_baidu");
        let request = &req.request;
        assert_eq!(request.method, "GET");
        assert_eq!(
            match request
                .headers
                .get("User-Agent")
                .expect("User-Agent not exists")
            {
                StrEnum::RawStr(agent) => agent,
                StrEnum::Format(_) => "fmt",
            },
            "reqwest"
        );
        assert_eq!(
            match request.headers.get("token").expect("token not exists") {
                StrEnum::RawStr(agent) => agent,
                StrEnum::Format(_) => "fmt",
            },
            "xxxx1234ABCD"
        );

        let req = r####"
###//comment
### request_baidu
GET https://www.baidu.com/s?kw=xxx
User-Agent: reqwest
Content-Type:   application/json

{"body":"msg"}
"####;
        let http = parse_http(&req, &None);
        assert_eq!(http.len(), 1);
        let req = http.get(0).unwrap();
        assert_eq!(req.name, "request_baidu");
        let request = &req.request;
        assert_eq!(request.method, "GET");
        let headers = &request.headers;
        assert_eq!(
            match headers.get("User-Agent").expect("User-Agent not exists") {
                StrEnum::RawStr(agent) => agent,
                StrEnum::Format(_) => "fmt",
            },
            "reqwest"
        );
        assert_eq!(
            match headers
                .get("Content-Type")
                .expect("Content-Type not exists")
            {
                StrEnum::RawStr(agent) => agent,
                StrEnum::Format(_) => "fmt",
            },
            "application/json"
        );

        let body = request.body.clone();
        assert_eq!(
            match body.clone().expect("body not exists") {
                StrEnum::RawStr(body) => body,
                StrEnum::Format(_) => "fmt",
            },
            r#"{"body":"msg"}
"#
        )
    }

    #[test]
    fn test_parse_and_substitute() {
        let input = r#####"
@hostname = api.example.com
@port = 8080
@host = {{hostname}}:{{port}}
@contentType = application/json

###
POST https://{{host}}/verify-otp HTTP/1.1
Content-Type: {{contentType}}

{
    "username": "alice",
    "otp": "123456"
}
    "#####;

        let output = parse_and_substitute(input);
        assert_eq!(
            output.trim(),
            r#####"
###
POST https://api.example.com:8080/verify-otp HTTP/1.1
Content-Type: application/json

{
    "username": "alice",
    "otp": "123456"
}
        "#####
                .trim()
        )
    }
}
