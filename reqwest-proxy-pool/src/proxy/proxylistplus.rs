/// * https://list.proxylistplus.com/Fresh-HTTP-Proxy-List-1
use super::{OkLogErr, ProxySender, SendProxyEx};
use crate::{
    error::Result,
    proxy::{IntoProxy, Privacy, ProxyType},
};
use async_trait::async_trait;
use reqwest_scraper::{FromCssSelector, ScraperResponse};
use std::{net::SocketAddr, str::FromStr};

#[derive(Debug, FromCssSelector)]
#[selector(path = "#page > table.bg > tbody > tr:nth-child(n+3)")]
pub(super) struct Proxy {
    #[selector(path = "td:nth-child(2)", text)]
    ip: Option<String>,

    #[selector(path = "td:nth-child(3)", text)]
    port: Option<String>,

    #[selector(path = "td:nth-child(4)", text)]
    anonymity: Option<String>,

    #[selector(path = "td:nth-child(7)", text)]
    https: Option<String>,
}

impl IntoProxy for Proxy {
    fn make_proxy(self) -> Option<crate::proxy::Proxy> {
        let socket = SocketAddr::from_str(&format!("{}:{}", self.ip?, self.port?)).ok_log_err()?;
        let ty = if let Some(https_text) = self.https {
            if https_text.contains("yes") {
                ProxyType::Https
            } else {
                ProxyType::Http
            }
        } else {
            ProxyType::Http
        };
        let pri = Privacy::from_str(&self.anonymity?);

        Some(crate::proxy::Proxy { socket, ty, pri })
    }
}

pub(super) struct ProxyFetcher;

#[async_trait]
impl super::ProxyFetcher for ProxyFetcher {
    async fn fetch(&self, sender: ProxySender) -> Result<()> {
        let html = super::get("https://list.proxylistplus.com/Fresh-HTTP-Proxy-List-1")
            .await?
            .css_selector()
            .await?;

        let items = Proxy::from_html(html)?;
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
