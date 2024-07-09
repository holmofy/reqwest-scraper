use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScraperError {
    #[cfg(feature = "jsonpath")]
    #[error(transparent)]
    JsonPathError(#[from] jsonpath_lib::JsonPathError),

    #[cfg(feature = "jsonpath")]
    #[error(transparent)]
    JsonDeserializeError(#[from] serde_json::Error),

    #[cfg(feature = "css_selector")]
    #[error("css selector error: {0}")]
    CssSelectorError(String),

    #[cfg(feature = "xpath")]
    #[error(transparent)]
    HtmlParseError(#[from] sxd_document::parser::Error),

    #[cfg(feature = "xpath")]
    #[error(transparent)]
    XPathParseError(#[from] sxd_xpath::ParserError),

    #[cfg(feature = "xpath")]
    #[error(transparent)]
    XPathExecutionError(#[from] sxd_xpath::ExecutionError),

    #[cfg(feature = "xpath")]
    #[error("{0}")]
    XPathError(String),

    #[error(transparent)]
    IOError(#[from] reqwest::Error),

    #[error("http request error:{0}, body text:{1}")]
    HttpError(u16, String),

    #[error("illegal argument:{0}")]
    IllegalArgsError(String),
}

#[cfg(feature = "css_selector")]
impl<'a> From<scraper::error::SelectorErrorKind<'a>> for ScraperError {
    fn from(value: scraper::error::SelectorErrorKind<'a>) -> Self {
        Self::CssSelectorError(value.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ScraperError>;
