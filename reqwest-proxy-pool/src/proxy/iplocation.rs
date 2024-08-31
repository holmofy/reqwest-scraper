/// * https://www.iplocation.net/proxy-list
use super::{OkLogErr, ProxySender, SendProxyEx};
use crate::{
    error::Result,
    proxy::{IntoProxy, Privacy, ProxyType},
};
use async_trait::async_trait;
use reqwest_scraper::{FromXPath, ScraperResponse};
use std::{net::SocketAddr, str::FromStr};

#[derive(Debug, FromXPath)]
#[xpath(
    path = r#"//*[@id="wrapper"]/div[1]/div[2]/div/section/div/div/div[1]/div[5]/div/div/table/tbody/tr"#
)]
pub(super) struct Proxy {
    #[xpath(path = "./td[1]/a/text()")]
    ip: Option<String>,

    #[xpath(path = "./td[2]/text()")]
    port: Option<String>,

    #[xpath(path = "./td[6]/span/@class")]
    https: Option<String>,
}

impl IntoProxy for Proxy {
    fn make_proxy(self) -> Option<crate::proxy::Proxy> {
        let socket = SocketAddr::from_str(&format!("{}:{}", self.ip?, self.port?)).ok_log_err()?;
        let ty = if let Some(https_class) = self.https {
            if https_class.contains("my_https_status_green") {
                ProxyType::Https
            } else {
                ProxyType::Http
            }
        } else {
            ProxyType::Http
        };

        Some(crate::proxy::Proxy {
            socket,
            ty,
            pri: Privacy::Unknown,
        })
    }
}

pub(super) struct ProxyFetcher;

#[async_trait]
impl super::ProxyFetcher for ProxyFetcher {
    async fn fetch(&self, sender: ProxySender) -> Result<()> {
        let html = super::get("https://www.iplocation.net/proxy-list")
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
