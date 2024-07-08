use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScraperError {
    #[error(transparent)]
    JsonPathError(#[from] jsonpath_lib::JsonPathError),

    #[error("css selector error: {0}")]
    CssSelectorError(String),

    #[error("xpath error: {0}")]
    XPathError(String),

    #[error(transparent)]
    XPathApplyError(#[from] skyscraper::xpath::ExpressionApplyError),

    #[error(transparent)]
    HtmlParseError(#[from] skyscraper::html::parse::ParseError),

    #[error(transparent)]
    IOError(#[from] reqwest::Error),

    #[error("http request error:{0}, body text:{1}")]
    HttpError(u16, String),
}

impl<'a> From<scraper::error::SelectorErrorKind<'a>> for ScraperError {
    fn from(value: scraper::error::SelectorErrorKind<'a>) -> Self {
        Self::CssSelectorError(value.to_string())
    }
}

impl<'a> From<nom::Err<nom::error::VerboseError<&str>>> for ScraperError {
    fn from(value: nom::Err<nom::error::VerboseError<&str>>) -> Self {
        Self::XPathError(value.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ScraperError>;
