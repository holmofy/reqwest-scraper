/// * http://www.ip3366.net/free/?stype=1
use super::{OkLogErr, ProxySender, SendProxyEx};
use crate::{
    error::Result,
    proxy::{IntoProxy, Privacy, ProxyType},
};
use async_trait::async_trait;
use reqwest_scraper::{FromXPath, ScraperResponse};
use std::{net::SocketAddr, str::FromStr};

#[derive(Debug, FromXPath)]
#[xpath(path = r#"//*[@id="list"]/table/tbody/tr"#)]
pub(super) struct Proxy {
    #[xpath(path = "./td[1]/text()")]
    ip: Option<String>,

    #[xpath(path = "./td[2]/text()")]
    port: Option<String>,

    #[xpath(path = "./td[3]/text()")]
    anonymity: Option<String>,

    #[xpath(path = "./td[4]/text()")]
    protocol: Option<String>,
}

impl IntoProxy for Proxy {
    fn make_proxy(self) -> Option<crate::proxy::Proxy> {
        let socket = SocketAddr::from_str(&format!("{}:{}", self.ip?, self.port?)).ok_log_err()?;
        let ty = ProxyType::from_str(&self.protocol?).ok_log_err()?;
        let pri = Privacy::from_str(&self.anonymity?);

        Some(crate::proxy::Proxy { socket, ty, pri })
    }
}

pub(super) struct ProxyFetcher;

#[async_trait]
impl super::ProxyFetcher for ProxyFetcher {
    async fn fetch(&self, sender: ProxySender) -> Result<()> {
        let html = super::get("http://www.ip3366.net/free/?stype=1")
            .await?
            .xpath()
            .await?;

        let items = Proxy::from_xhtml(html)?;
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
