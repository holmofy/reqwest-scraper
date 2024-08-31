/// 有反爬虫措施
/// TODO: 第一个请求用混淆的js设置cookie
/// * https://www.89ip.cn/index_1.html
use super::{OkLogErr, ProxySender, SendProxyEx};
use crate::{
    error::Result,
    proxy::{IntoProxy, Privacy, ProxyType},
};
use async_trait::async_trait;
use reqwest_scraper::{FromXPath, ScraperResponse};
use std::{net::SocketAddr, str::FromStr};

#[derive(Debug, FromXPath)]
#[xpath(path = r#"//table[contains(@class,'layui-table')]/tbody/tr"#)]
pub(super) struct Proxy {
    #[xpath(path = "./td[1]/text()")]
    ip: Option<String>,

    #[xpath(path = "./td[2]/text()")]
    port: Option<String>,
}

impl IntoProxy for Proxy {
    fn make_proxy(self) -> Option<crate::proxy::Proxy> {
        let socket = SocketAddr::from_str(&format!("{}:{}", self.ip?, self.port?)).ok_log_err()?;

        Some(crate::proxy::Proxy {
            socket,
            ty: ProxyType::Http,
            pri: Privacy::Unknown,
        })
    }
}

pub(super) struct ProxyFetcher;

#[async_trait]
impl super::ProxyFetcher for ProxyFetcher {
    async fn fetch(&self, sender: ProxySender) -> Result<()> {
        let html = super::get("https://www.89ip.cn/index_1.html")
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
