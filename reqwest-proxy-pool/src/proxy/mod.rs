mod docip;
mod ip3366;
mod iplocation;
mod kuaidaili;
mod proxydb;
mod proxylistplus;
mod uqidata;
mod www89ip;
mod zdaye;

use crate::error::{ProxyError, Result};
use async_trait::async_trait;
use std::net::SocketAddr;
use tokio::sync::mpsc::Sender;

#[derive(Debug)]
pub struct Proxy {
    socket: SocketAddr,
    ty: ProxyType,
    pri: Privacy,
}

#[derive(Debug)]
pub enum ProxyType {
    Http,
    Https,
    Socks,
}

impl ProxyType {
    fn from_str(protocol: &str) -> Result<ProxyType> {
        match protocol.to_lowercase().as_str() {
            "http" => Ok(Self::Http),
            "https" => Ok(Self::Https),
            "http/s" => Ok(Self::Https),
            "http(s)" => Ok(Self::Https),
            "socks" => Ok(Self::Socks),
            "socks5" => Ok(Self::Socks),
            "socks4" => Err(ProxyError::ProtocolParseErr(
                "socks4 not support".to_string(),
            )),
            _other => Err(ProxyError::ProtocolParseErr(_other.to_string())),
        }
    }
}

#[derive(Debug)]
pub enum Privacy {
    Unknown,
    Anonymity,
    HighAnonymity,
}

impl Privacy {
    fn from_str(privacy: &str) -> Privacy {
        if privacy.contains("高匿") || privacy.contains("high anonymous") {
            return Self::HighAnonymity;
        }
        if privacy.contains("普匿")
            || privacy.contains("普通")
            || privacy.contains("匿名")
            || privacy.contains("anonymous")
        {
            return Self::Anonymity;
        }
        return Self::Unknown;
    }
}

pub(crate) trait IntoProxy {
    fn make_proxy(self) -> Option<Proxy>;
}

/////////////////////////////////////
/// Proxy Pool
pub async fn fetch(sender: ProxySender) {
    for fetcher in inventory::iter::<&dyn ProxyFetcher> {
        let sender_clone = sender.clone();
        tokio::spawn(async move {
            if let Err(e) = fetcher.fetch(sender_clone).await {
                log::error!("fetch proxy ip failed: {}", e);
            }
        });
    }
}

type ProxySender = Sender<Proxy>;

#[async_trait]
trait ProxyFetcher: Sync {
    async fn fetch(&self, sender: ProxySender) -> Result<()>;
}

inventory::collect!(&'static dyn ProxyFetcher);

trait SendProxyEx {
    async fn send_proxy(&self, proxy: impl IntoProxy) -> Result<()>;
}

impl SendProxyEx for ProxySender {
    async fn send_proxy(&self, proxy: impl IntoProxy) -> Result<()> {
        if let Some(proxy) = proxy.make_proxy() {
            return Ok(self.send(proxy).await?);
        }
        Ok(())
    }
}

async fn get<T: reqwest::IntoUrl>(url: T) -> reqwest::Result<reqwest::Response> {
    default_client()?.get(url).send().await
}

async fn post<T: reqwest::IntoUrl>(url: T) -> reqwest::Result<reqwest::Response> {
    default_client()?.post(url).send().await
}

fn default_client() -> reqwest::Result<reqwest::Client> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36 Edg/128.0.0.0".parse().unwrap());
    Ok(reqwest::Client::builder()
        .default_headers(headers)
        .danger_accept_invalid_certs(true)
        .build()?)
}

trait OkLogErr<T> {
    fn ok_log_err(self) -> Option<T>;
}

impl<T, E> OkLogErr<T> for std::result::Result<T, E>
where
    E: std::error::Error,
{
    fn ok_log_err(self) -> Option<T> {
        match self {
            Ok(x) => Some(x),
            Err(e) => {
                log::warn!("{}", e);
                None
            }
        }
    }
}
