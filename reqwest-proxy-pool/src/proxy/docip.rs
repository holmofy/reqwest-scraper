/// * https://www.docip.net/free
use super::{OkLogErr, ProxySender, SendProxyEx};
use crate::{
    error::Result,
    proxy::{IntoProxy, Privacy, ProxyType},
};
use async_trait::async_trait;
use reqwest_scraper::ScraperResponse;
use serde::Deserialize;
use std::{net::SocketAddr, str::FromStr};

#[derive(Debug, Deserialize)]
pub(super) struct Proxy {
    ip: String,
    proxy_type: String,
}

impl IntoProxy for Proxy {
    fn make_proxy(self) -> Option<crate::proxy::Proxy> {
        let socket = SocketAddr::from_str(&self.ip).ok_log_err()?;
        let ty = match self.proxy_type.as_str() {
            "1" => ProxyType::Https,
            _ => ProxyType::Http,
        };
        let pri = Privacy::HighAnonymity;

        Some(crate::proxy::Proxy { socket, ty, pri })
    }
}

pub(super) struct ProxyFetcher;

#[async_trait]
impl super::ProxyFetcher for ProxyFetcher {
    async fn fetch(&self, sender: ProxySender) -> Result<()> {
        let items: Vec<Proxy> = super::get("https://www2.docip.net/data/free.json")
            .await?
            .jsonpath()
            .await?
            .select("$.data[*]")?;

        for item in items {
            log::trace!("fetch proxy: {:?}", item);
            sender.send_proxy(item).await?;
        }
        Ok(())
    }
}

inventory::submit! {
    &ProxyFetcher as &dyn super::ProxyFetcher
}
