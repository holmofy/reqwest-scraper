pub mod css_selector;
pub mod error;
pub mod jsonpath;
pub mod xpath;

use crate::css_selector::Html;
use crate::error::Result;
use crate::jsonpath::Json;
use crate::xpath::XHtml;
use async_trait::async_trait;
use error::ScraperError;
pub use reqwest::Response;

/// Support extended traits of jsonpath, css selector, and xpath
#[async_trait]
pub trait ScraperResponse {
    /// Use jsonpath to select the response body
    async fn jsonpath(self) -> Result<Json>;

    /// Use CSS selector to select the response body
    async fn css_selector(self) -> Result<Html>;

    /// Use XPath to select the response body
    async fn xpath(self) -> Result<XHtml>;
}

#[async_trait]
impl ScraperResponse for Response {
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

    async fn xpath(self) -> Result<XHtml> {
        if self.status().is_success() {
            let html_str = self.text().await?;
            let html_value = skyscraper::html::parse(html_str.as_str())?;
            let xpath_tree = skyscraper::xpath::XpathItemTree::from(&html_value);
            Ok(XHtml {
                value: html_value,
                xpath_tree,
            })
        } else {
            let status_code = self.status().as_u16();
            let response = self.text().await?;
            Err(ScraperError::HttpError(status_code, response))
        }
    }
}
