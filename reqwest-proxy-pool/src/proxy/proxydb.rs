/// * http://proxydb.net/
use super::{OkLogErr, ProxySender, SendProxyEx};
use crate::{
    error::Result,
    proxy::{IntoProxy, Privacy, ProxyType},
};
use async_trait::async_trait;
use serde::Deserialize;
use std::{net::SocketAddr, str::FromStr};

#[derive(Debug, Deserialize)]
pub(super) struct Proxy {
    ip: String,
    port: i32,
    #[serde(rename = "type")]
    ty: String,
    anonlvl: i32,
}

impl IntoProxy for Proxy {
    fn make_proxy(self) -> Option<crate::proxy::Proxy> {
        let socket = SocketAddr::from_str(&format!("{}:{}", self.ip, self.port)).ok_log_err()?;
        let ty = ProxyType::from_str(&self.ty).ok_log_err()?;
        let pri = match self.anonlvl {
            4 => Privacy::HighAnonymity,
            2 => Privacy::Anonymity,
            _ => Privacy::Unknown,
        };

        Some(crate::proxy::Proxy { socket, ty, pri })
    }
}

pub(super) struct ProxyFetcher;

#[async_trait]
impl super::ProxyFetcher for ProxyFetcher {
    async fn fetch(&self, sender: ProxySender) -> Result<()> {
        let json = super::post("http://proxydb.net/list").await?.text().await?;
        let items: Vec<Proxy> = serde_json::from_str(&json)?;

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
