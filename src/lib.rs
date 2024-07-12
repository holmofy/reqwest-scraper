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

#[cfg(feature = "css_selector")]
use crate::css_selector::Html;
use crate::error::Result;
#[cfg(feature = "jsonpath")]
use crate::jsonpath::Json;
#[cfg(feature = "xpath")]
use crate::xpath::XHtml;
use async_trait::async_trait;
use error::ScraperError;
pub use reqwest::Response;

pub use reqwest_scraper_macros::{FromCssSelector, FromJsonPath, FromXPath};

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

/// Use JsonPath to extract the Json response body into the derived struct
#[cfg(feature = "jsonpath")]
pub trait FromJsonPath {
    /// extract result by jsonpath
    type JsonPathExtractResult;

    /// From Json Response
    fn from_json(json: Json) -> Self::JsonPathExtractResult;
}

/// Support extended traits of jsonpath, css selector, and xpath
#[async_trait]
pub trait ScraperResponse {
    /// Use jsonpath to select the response body
    #[cfg(feature = "jsonpath")]
    async fn jsonpath(self) -> Result<Json>;

    /// Use CSS selector to select the response body
    #[cfg(feature = "css_selector")]
    async fn css_selector(self) -> Result<Html>;

    /// Use XPath to select the response body
    #[cfg(feature = "xpath")]
    async fn xpath(self) -> Result<XHtml>;
}

#[async_trait]
impl ScraperResponse for Response {
    #[cfg(feature = "jsonpath")]
    async fn jsonpath(self) -> Result<Json> {
        if self.status().is_success() {
            let json_value = self.json().await?;
            Ok(Json { value: json_value })
        } else {
            let status_code = self.status().as_u16();
            let response = self.text().await?;
            Err(ScraperError::HttpError(status_code, response))
        }
    }

    #[cfg(feature = "css_selector")]
    async fn css_selector(self) -> Result<Html> {
        if self.status().is_success() {
            let html_str = self.text().await?;
            Ok(Html {
                value: scraper::Html::parse_fragment(html_str.as_str()),
            })
        } else {
            let status_code = self.status().as_u16();
            let response = self.text().await?;
            Err(ScraperError::HttpError(status_code, response))
        }
    }

    #[cfg(feature = "xpath")]
    async fn xpath(self) -> Result<XHtml> {
        if self.status().is_success() {
            let html_str = self.text().await?;
            let parser = libxml::parser::Parser::default_html();
            let doc = parser.parse_string(html_str)?;
            Ok(XHtml { doc })
        } else {
            let status_code = self.status().as_u16();
            let response = self.text().await?;
            Err(ScraperError::HttpError(status_code, response))
        }
    }
}
