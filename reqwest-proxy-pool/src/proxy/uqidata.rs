/// * https://www.data5u.com/
/// * https://ip.uqidata.com/
use super::{OkLogErr, ProxySender, SendProxyEx};
use crate::{
    error::Result,
    proxy::{IntoProxy, Privacy, ProxyType},
};
use async_trait::async_trait;
use itertools::Itertools;
use reqwest_scraper::{css_selector::SelectItem, FromCssSelector, ScraperResponse};
use std::{net::SocketAddr, str::FromStr};

#[derive(Debug, FromCssSelector)]
#[selector(path = "#main_container > div.inner > table > tbody > tr:nth-child(n+3)")]
pub(super) struct Proxy {
    #[selector(path = "td.ip", map = display_text)]
    ip: Option<String>,

    #[selector(path = "td.port", text)]
    port: Option<String>,

    #[selector(path = "td:nth-child(3)", text)]
    protocol: Option<String>,

    #[selector(path = "td:nth-child(4)", text)]
    anonymity: Option<String>,
}

/// 该网站插入了很多display:none的元素干扰爬虫抓取
fn display_text(e: SelectItem) -> Option<String> {
    Some(
        e.children()
            .filter(|e| {
                e.attr("style")
                    .and_then(|style| Some(!style.contains("none")))
                    .unwrap_or(true)
            })
            .map(|e| e.text())
            .join(""),
    )
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
        let html = super::get("https://ip.uqidata.com/")
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
