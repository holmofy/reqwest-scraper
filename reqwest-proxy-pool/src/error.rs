use thiserror::Error;
use crate::proxy::Proxy;

pub type Result<T> = std::result::Result<T, ProxyError>;

/// Scraping Error
#[derive(Error, Debug)]
pub enum ProxyError {
    #[error(transparent)]
    ReqwestErr(#[from] reqwest::Error),

    #[error(transparent)]
    ScraperErr(#[from] reqwest_scraper::error::ScraperError),

    #[error("protocol parse error: {0}")]
    ProtocolParseErr(String),

    #[error(transparent)]
    SendErr(#[from] tokio::sync::mpsc::error::SendError<Proxy>),
}
