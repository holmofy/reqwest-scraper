pub(crate) mod proxy_fetch;
pub(crate) mod utils;

use crate::error::{ProxyError, Result};
use std::net::SocketAddr;

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
