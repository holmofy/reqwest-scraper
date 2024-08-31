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
        let result = inner_fetch("?stype=1".to_string()).await?;

        for item in result {
            log::trace!("fetch proxy: {:?}", item);
            sender.send_proxy(item).await?;
        }

        let result = inner_fetch("?stype=2".to_string()).await?;

        for item in result {
            log::trace!("fetch proxy: {:?}", item);
            sender.send_proxy(item).await?;
        }
        Ok(())
    }
}

async fn inner_fetch(mut query: String) -> Result<Vec<Proxy>> {
    let mut result = vec![];
    loop {
        let html = super::get(format!("http://www.ip3366.net/free/{query}"))
            .await?
            .xpath()
            .await?;

        let next_page = html
            .select("//*[@id='listnav']/ul/a[text()='下一页']/@href")?
            .as_str();

        let items = Proxy::from_xhtml(html)?;
        result.extend(items);

        query = match next_page {
            None => break,
            Some(href) => href,
        };
    }
    Ok(result)
}

inventory::submit! {
    &ProxyFetcher as &dyn super::ProxyFetcher
}
