//! Scraping Error
//!
use thiserror::Error;

/// Scraping Error
#[derive(Error, Debug)]
pub enum ScraperError {
    /// JsonPath Error
    #[cfg(feature = "jsonpath")]
    #[error(transparent)]
    JsonPathError(#[from] jsonpath_lib::JsonPathError),

    /// JsonPath Match Error
    #[cfg(feature = "jsonpath")]
    #[error("jsonpath match error:{0}")]
    JsonPathMatchError(String),

    /// Json Deserialize Error
    #[cfg(feature = "jsonpath")]
    #[error(transparent)]
    JsonDeserializeError(#[from] serde_json::Error),

    /// Css Selector Error
    #[cfg(feature = "css_selector")]
    #[error("css selector error: {0}")]
    CssSelectorError(String),

    /// JsonPath Match Error
    #[cfg(feature = "css_selector")]
    #[error("css selector match error:{0}")]
    CssSelectorMatchError(String),

    /// Html Document Parse Error
    #[cfg(feature = "xpath")]
    #[error(transparent)]
    HtmlParseError(#[from] libxml::parser::XmlParseError),

    /// XPath Evaluate Error
    #[cfg(feature = "xpath")]
    #[error("{0}")]
    XPathError(String),

    /// IO Error
    #[error(transparent)]
    IOError(#[from] reqwest::Error),

    /// Http response failed
    #[error("http request for \"{0}\" error code:{1}, body text:{2}")]
    HttpError(String, u16, String),
}

#[cfg(feature = "css_selector")]
impl<'a> From<scraper::error::SelectorErrorKind<'a>> for ScraperError {
    fn from(value: scraper::error::SelectorErrorKind<'a>) -> Self {
        Self::CssSelectorError(value.to_string())
    }
}

/// Result
pub type Result<T> = std::result::Result<T, ScraperError>;
