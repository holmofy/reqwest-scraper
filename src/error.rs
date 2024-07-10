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

    /// Json Deserialize Error
    #[cfg(feature = "jsonpath")]
    #[error(transparent)]
    JsonDeserializeError(#[from] serde_json::Error),

    /// Css Selector Error
    #[cfg(feature = "css_selector")]
    #[error("css selector error: {0}")]
    CssSelectorError(String),

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
    #[error("http request error:{0}, body text:{1}")]
    HttpError(u16, String),

    /// Illegal Args Error
    #[error("illegal argument:{0}")]
    IllegalArgsError(String),
}

#[cfg(feature = "css_selector")]
impl<'a> From<scraper::error::SelectorErrorKind<'a>> for ScraperError {
    fn from(value: scraper::error::SelectorErrorKind<'a>) -> Self {
        Self::CssSelectorError(value.to_string())
    }
}

/// Result
pub type Result<T> = std::result::Result<T, ScraperError>;
