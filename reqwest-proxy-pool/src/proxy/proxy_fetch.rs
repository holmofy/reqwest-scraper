use super::IntoProxy;
use crate::error::Result;
use crate::proxy::Proxy;
use async_trait::async_trait;
use tokio::sync::mpsc::Sender;

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

/// * https://www.docip.net/free
mod docip {
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
    // inventory::submit! {
    //     &ProxyFetcher as &dyn super::ProxyFetcher
    // }
}

/// 有反爬虫措施
/// TODO: 第一个请求用混淆的js设置cookie
/// * https://www.89ip.cn/index_1.html
mod www89ip {
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
            let socket =
                SocketAddr::from_str(&format!("{}:{}", self.ip?, self.port?)).ok_log_err()?;

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
}

/// * http://www.ip3366.net/free/?stype=1
mod ip3366 {
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
            let socket =
                SocketAddr::from_str(&format!("{}:{}", self.ip?, self.port?)).ok_log_err()?;
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
    // inventory::submit! {
    //     &ProxyFetcher as &dyn super::ProxyFetcher
    // }
}

/// * https://www.kuaidaili.com/free/intr/
mod kuaidaili {
    use super::{OkLogErr, ProxySender, SendProxyEx};
    use crate::{
        error::Result,
        proxy::{utils::substr_between, IntoProxy, Privacy, ProxyType},
    };
    use async_trait::async_trait;
    use serde::Deserialize;
    use std::{net::SocketAddr, str::FromStr};

    #[derive(Debug, Deserialize)]
    pub(super) struct FreeProxy {
        ip: String,
        port: String,
    }

    impl IntoProxy for FreeProxy {
        fn make_proxy(self) -> Option<crate::proxy::Proxy> {
            let socket =
                SocketAddr::from_str(&format!("{}:{}", self.ip, self.port)).ok_log_err()?;

            Some(crate::proxy::Proxy {
                socket,
                ty: ProxyType::Http,
                pri: Privacy::HighAnonymity,
            })
        }
    }

    #[derive(Debug, Deserialize)]
    pub(super) struct HttpsProxy {
        ip: String,
        port: String,
    }

    impl IntoProxy for HttpsProxy {
        fn make_proxy(self) -> Option<crate::proxy::Proxy> {
            let socket =
                SocketAddr::from_str(&format!("{}:{}", self.ip, self.port)).ok_log_err()?;

            Some(crate::proxy::Proxy {
                socket,
                ty: ProxyType::Https,
                pri: Privacy::HighAnonymity,
            })
        }
    }

    pub(super) struct ProxyFetcher;
    #[async_trait]
    impl super::ProxyFetcher for ProxyFetcher {
        async fn fetch(&self, sender: ProxySender) -> Result<()> {
            let html = super::get("https://www.kuaidaili.com/free/intr/")
                .await?
                .text()
                .await?;

            let json = match substr_between(&html, "const fpsList = ", ";") {
                Some(json) => json,
                None => {
                    return Ok(log::warn!("json not found"));
                }
            };

            let items: Vec<FreeProxy> = serde_json::from_str(json)?;

            for item in items {
                log::trace!("fetch proxy: {:?}", item);
                sender.send_proxy(item).await?;
            }
            Ok(())
        }
    }
    // inventory::submit! {
    //     &ProxyFetcher as &dyn super::ProxyFetcher
    // }
}

/// * https://www.zdaye.com/free/1/
mod zdaye {
    use super::{OkLogErr, ProxySender, SendProxyEx};
    use crate::{
        error::Result,
        proxy::{IntoProxy, Privacy, ProxyType},
    };
    use async_trait::async_trait;
    use reqwest_scraper::{FromXPath, ScraperResponse};
    use std::{net::SocketAddr, str::FromStr};

    #[derive(Debug, FromXPath)]
    #[xpath(path = r#"//*[@id="ipc"]/tbody/tr"#)]
    pub(super) struct Proxy {
        #[xpath(path = "./td[1]/text()")]
        ip: Option<String>,

        #[xpath(path = "./td[2]/text()")]
        port: Option<String>,

        #[xpath(path = "./td[3]/text()")]
        anonymity: Option<String>,

        #[xpath(path = "./td[6]/div/@class")]
        https: Option<String>,
    }

    impl IntoProxy for Proxy {
        fn make_proxy(self) -> Option<crate::proxy::Proxy> {
            let socket =
                SocketAddr::from_str(&format!("{}:{}", self.ip?, self.port?)).ok_log_err()?;
            let ty = if let Some(http_class) = self.https {
                if http_class.contains("iyes") {
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
            let html = super::get("https://www.zdaye.com/free/1/")
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
    // inventory::submit! {
    //     &ProxyFetcher as &dyn super::ProxyFetcher
    // }
}

/// * https://www.data5u.com/
/// * https://ip.uqidata.com/
mod uqidata {
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
            let socket =
                SocketAddr::from_str(&format!("{}:{}", self.ip?, self.port?)).ok_log_err()?;
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
    // inventory::submit! {
    //     &ProxyFetcher as &dyn super::ProxyFetcher
    // }
}

/// * http://proxydb.net/
mod proxydb {
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
            let socket =
                SocketAddr::from_str(&format!("{}:{}", self.ip, self.port)).ok_log_err()?;
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
    // inventory::submit! {
    //     &ProxyFetcher as &dyn super::ProxyFetcher
    // }
}

/// * https://list.proxylistplus.com/Fresh-HTTP-Proxy-List-1
mod proxylistplus {
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
            let socket =
                SocketAddr::from_str(&format!("{}:{}", self.ip?, self.port?)).ok_log_err()?;
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
}

/// * https://www.iplocation.net/proxy-list
mod iplocation {
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
            let socket =
                SocketAddr::from_str(&format!("{}:{}", self.ip?, self.port?)).ok_log_err()?;
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

    // inventory::submit! {
    //     &ProxyFetcher as &dyn super::ProxyFetcher
    // }
}
