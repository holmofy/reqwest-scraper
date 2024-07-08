use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScraperError {
    #[cfg(feature = "jsonpath")]
    #[error(transparent)]
    JsonPathError(#[from] jsonpath_lib::JsonPathError),

    #[cfg(feature = "css_selector")]
    #[error("css selector error: {0}")]
    CssSelectorError(String),

    #[cfg(feature = "xpath")]
    #[error("xpath error: {0}")]
    XPathError(String),

    #[cfg(feature = "xpath")]
    #[error(transparent)]
    XPathApplyError(#[from] skyscraper::xpath::ExpressionApplyError),

    #[cfg(feature = "xpath")]
    #[error(transparent)]
    HtmlParseError(#[from] skyscraper::html::parse::ParseError),

    #[error(transparent)]
    IOError(#[from] reqwest::Error),

    #[error("http request error:{0}, body text:{1}")]
    HttpError(u16, String),
}

#[cfg(feature = "css_selector")]
impl<'a> From<scraper::error::SelectorErrorKind<'a>> for ScraperError {
    fn from(value: scraper::error::SelectorErrorKind<'a>) -> Self {
        Self::CssSelectorError(value.to_string())
    }
}

#[cfg(feature = "xpath")]
impl<'a> From<nom::Err<nom::error::VerboseError<&str>>> for ScraperError {
    fn from(value: nom::Err<nom::error::VerboseError<&str>>) -> Self {
        Self::XPathError(value.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ScraperError>;
