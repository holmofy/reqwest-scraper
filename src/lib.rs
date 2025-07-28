#![deny(missing_docs)]

//! reqwest-scraper
//#![doc = include_str!("../README.md")]

#[cfg(feature = "css_selector")]
pub mod css_selector;
pub mod error;
#[cfg(feature = "jsonpath")]
pub mod jsonpath;
#[cfg(feature = "xpath")]
pub mod xpath;

#[cfg(feature = "jsonpath")]
use std::future::Future;

#[cfg(feature = "css_selector")]
use crate::css_selector::Html;
use crate::error::Result;
#[cfg(feature = "jsonpath")]
use crate::jsonpath::Json;
#[cfg(feature = "xpath")]
use crate::xpath::XHtml;
use encoding_rs::{Encoding, UTF_8};
use error::ScraperError;
use mime::Mime;
use regex::Regex;
pub use reqwest::Response;
#[cfg(feature = "json")]
use serde::de::DeserializeOwned;

pub use reqwest_scraper_macros::{include_http, FromCssSelector, FromXPath};

/// Use XPath to extract the HTML response body into the derived struct
#[cfg(feature = "xpath")]
pub trait FromXPath {
    /// extract result by xpath
    type XPathExtractResult;

    /// From Html Response
    fn from_xhtml(html: XHtml) -> Self::XPathExtractResult;
}

/// Use CssSelector to extract the HTML response body into the derived struct
#[cfg(feature = "css_selector")]
pub trait FromCssSelector {
    /// extract result by css selector
    type CssSelectorExtractResult;

    /// From Html Response
    fn from_html(html: Html) -> Self::CssSelectorExtractResult;
}

/// Support extended traits of jsonpath, css selector, and xpath
pub trait ScraperResponse {
    /// Use jsonpath to select the response body
    #[cfg(feature = "jsonpath")]
    fn jsonpath(self) -> impl Future<Output = Result<Json>>;

    /// works with any existing Serde Deserializer and exposes the chain of field names leading to the error.
    /// * https://crates.io/crates/serde_path_to_error
    #[cfg(feature = "json")]
    fn json_with_path_to_err<T: DeserializeOwned>(self) -> impl Future<Output = Result<T>>;

    /// Use CSS selector to select the response body
    #[cfg(feature = "css_selector")]
    fn css_selector(self) -> impl Future<Output = Result<Html>>;

    /// Use XPath to select the response body
    #[cfg(feature = "xpath")]
    fn xpath(self) -> impl Future<Output = Result<XHtml>>;

    /// If there is no Encoding method in the Content-Type of the response header,
    /// try to read the meta information in the HTML to obtain the encoding.
    /// eg: <meta charset="gb2312">
    fn html(self) -> impl Future<Output = Result<String>>;
}

impl ScraperResponse for Response {
    #[cfg(feature = "jsonpath")]
    async fn jsonpath(self) -> Result<Json> {
        if self.status().is_success() {
            let json = self.text().await?;
            Ok(Json::new(json.as_str())?)
        } else {
            let url = self.url().to_string();
            let status_code = self.status().as_u16();
            let response = self.text().await?;
            Err(ScraperError::HttpError(url, status_code, response))
        }
    }

    #[cfg(feature = "json")]
    async fn json_with_path_to_err<T: DeserializeOwned>(self) -> Result<T> {
        let full = self.bytes().await?;
        let mut deserializer = serde_json::Deserializer::from_slice(&full);
        Ok(serde_path_to_error::deserialize(&mut deserializer)?)
    }

    #[cfg(feature = "css_selector")]
    async fn css_selector(self) -> Result<Html> {
        if self.status().is_success() {
            let html_str = self.html().await?;
            Ok(Html::new(html_str.as_str()))
        } else {
            let url = self.url().to_string();
            let status_code = self.status().as_u16();
            let response = self.text().await?;
            Err(ScraperError::HttpError(url, status_code, response))
        }
    }

    #[cfg(feature = "xpath")]
    async fn xpath(self) -> Result<XHtml> {
        if self.status().is_success() {
            let html_str = self.html().await?;
            Ok(XHtml::new(html_str)?)
        } else {
            let url = self.url().to_string();
            let status_code = self.status().as_u16();
            let response = self.text().await?;
            Err(ScraperError::HttpError(url, status_code, response))
        }
    }

    async fn html(self) -> Result<String> {
        let content_type = self
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<Mime>().ok());
        let encoding_name = content_type
            .as_ref()
            .and_then(|mime| mime.get_param("charset").map(|charset| charset.as_str()));

        let full = self.bytes().await?;
        match encoding_name {
            Some(encoding_name) => {
                let encoding = Encoding::for_label(encoding_name.as_bytes()).unwrap_or(UTF_8);
                let (text, _, _) = encoding.decode(&full);
                Ok(text.into_owned())
            }
            None => {
                let (text, _, _) = UTF_8.decode(&full);
                let meta_charset = extract_charset(&text);
                if let Some(meta_charset) = meta_charset {
                    let encoding = Encoding::for_label(meta_charset.as_bytes()).unwrap_or(UTF_8);
                    let (text, _, _) = encoding.decode(&full);
                    Ok(text.into_owned())
                } else {
                    Ok(text.into_owned())
                }
            }
        }
    }
}

fn extract_charset(html: &str) -> Option<String> {
    // 优先匹配 <meta charset="UTF-8"> 或 <meta charset='UTF-8'>
    let re1 = Regex::new(r#"(?i)<meta\s+[^>]*charset=['"]?([a-zA-Z0-9\-_]+)"#).unwrap();
    if let Some(cap) = re1.captures(html) {
        return Some(cap[1].to_string());
    }

    // 兼容 HTML4/XHTML 写法
    // 匹配 <meta http-equiv="Content-Type" content="text/html; charset=GB2312">
    // 支持属性乱序、单双引号
    let re2 = Regex::new(r#"(?i)<meta\s+[^>]*http-equiv=['"]?content-type['"]?[^>]*content=['"][^>]*charset=([a-zA-Z0-9\-_]+)"#).unwrap();
    if let Some(cap) = re2.captures(html) {
        return Some(cap[1].to_string());
    }

    None
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_extract_charset() {
        use super::extract_charset;
        let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset="gb2312">
            <title>Example</title>
        </head>
        <body><p>Hello, world!</p></body>
        </html>
        "#;

        let cs = extract_charset(html);
        assert!(cs.is_some());
        assert_eq!(cs.unwrap(), "gb2312");

        let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta charset='gb2312'>
            <title>Example</title>
        </head>
        <body><p>Hello, world!</p></body>
        </html>
        "#;

        let cs = extract_charset(html);
        assert!(cs.is_some());
        assert_eq!(cs.unwrap(), "gb2312");

        let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta http-equiv="Content-Type" content="text/html; charset=GB2312">
            <title>Example</title>
        </head>
        <body><p>Hello, world!</p></body>
        </html>
        "#;

        let cs = extract_charset(html);
        assert!(cs.is_some());
        assert_eq!(cs.unwrap(), "GB2312");

        let html = r#"
        <!DOCTYPE html>
        <html>
        <head>
            <meta content='text/html; charset=gbk' http-equiv="Content-Type">
            <title>Example</title>
        </head>
        <body><p>Hello, world!</p></body>
        </html>
        "#;

        let cs = extract_charset(html);
        assert!(cs.is_some());
        assert_eq!(cs.unwrap(), "gbk");
    }
}
