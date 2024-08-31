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
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(reqwest::header::USER_AGENT, "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36 Edg/128.0.0.0".parse().unwrap());
    reqwest::Client::builder()
        .default_headers(headers)
        .build()?
        .get(url)
        .send()
        .await
}

/// * https://www.docip.net/free
mod docip {
    use super::{ProxySender, SendProxyEx};
    use crate::{
        error::Result,
        proxy::{IntoProxy, Privacy, ProxyType},
    };
    use async_trait::async_trait;
    use reqwest_scraper::{FromXPath, ScraperResponse};
    use std::{net::SocketAddr, str::FromStr};

    #[derive(Debug, FromXPath)]
    #[xpath(path = r#"//*[@id="freebody"]/tr"#)]
    pub(super) struct Proxy {
        #[xpath(path = "./th/text()")]
        socket: Option<String>,

        #[xpath(path = "./td[1]/text()")]
        protocol: Option<String>,

        #[xpath(path = "./td[2]/text()")]
        anonymity: Option<String>,
    }

    impl IntoProxy for Proxy {
        fn make_proxy(self) -> Option<crate::proxy::Proxy> {
            let socket = SocketAddr::from_str(&self.socket?).ok()?;
            let ty = ProxyType::from_str(&self.protocol?).ok()?;
            let pri = Privacy::from_str(&self.anonymity?);

            Some(crate::proxy::Proxy { socket, ty, pri })
        }
    }

    pub(super) struct ProxyFetcher;
    #[async_trait]
    impl super::ProxyFetcher for ProxyFetcher {
        async fn fetch(&self, sender: ProxySender) -> Result<()> {
            let html = super::get("https://www.docip.net/free")
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

/// 有反爬虫措施
/// TODO: 第一个请求用混淆的js设置cookie
/// * https://www.89ip.cn/index_1.html
mod www89ip {
    use super::{ProxySender, SendProxyEx};
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
            let socket = SocketAddr::from_str(&format!("{}:{}", self.ip?, self.port?)).ok()?;

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
    use super::{ProxySender, SendProxyEx};
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
            let socket = SocketAddr::from_str(&format!("{}:{}", self.ip?, self.port?)).ok()?;
            let ty = ProxyType::from_str(&self.protocol?).ok()?;
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
    use super::{ProxySender, SendProxyEx};
    use crate::{
        error::Result,
        proxy::{IntoProxy, Privacy, ProxyType},
    };
    use async_trait::async_trait;
    use reqwest_scraper::{FromXPath, ScraperResponse};
    use std::{net::SocketAddr, str::FromStr};

    #[derive(Debug, FromXPath)]
    #[xpath(path = r#"//*[@id="table__free-proxy"]/div/table/tbody/tr"#)]
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
            let socket = SocketAddr::from_str(&format!("{}:{}", self.ip?, self.port?)).ok()?;
            let ty = ProxyType::from_str(&self.protocol?).ok()?;
            let pri = Privacy::from_str(&self.anonymity?);

            Some(crate::proxy::Proxy { socket, ty, pri })
        }
    }

    pub(super) struct ProxyFetcher;
    #[async_trait]
    impl super::ProxyFetcher for ProxyFetcher {
        async fn fetch(&self, sender: ProxySender) -> Result<()> {
            let html = super::get("https://www.kuaidaili.com/free/intr/")
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

/// * https://www.zdaye.com/free/1/
mod zdaye {
    use super::{ProxySender, SendProxyEx};
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
            let socket = SocketAddr::from_str(&format!("{}:{}", self.ip?, self.port?)).ok()?;
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
    use super::{ProxySender, SendProxyEx};
    use crate::{
        error::Result,
        proxy::{IntoProxy, Privacy, ProxyType},
    };
    use async_trait::async_trait;
    use reqwest_scraper::{FromXPath, ScraperResponse};
    use std::{net::SocketAddr, str::FromStr};

    #[derive(Debug, FromXPath)]
    #[xpath(path = r#"//*[@id="main_container"]/div[1]/table/tbody/tr[position() >= 3]"#)]
    pub(super) struct Proxy {
        #[xpath(path = "./td[1]/text()")]
        ip: Option<String>,

        #[xpath(path = "./td[2]/text()")]
        port: Option<String>,

        #[xpath(path = "./td[3]/text()")]
        protocol: Option<String>,

        #[xpath(path = "./td[4]/text()")]
        anonymity: Option<String>,
    }

    impl IntoProxy for Proxy {
        fn make_proxy(self) -> Option<crate::proxy::Proxy> {
            let socket = SocketAddr::from_str(&format!("{}:{}", self.ip?, self.port?)).ok()?;
            let ty = ProxyType::from_str(&self.protocol?).ok()?;
            let pri = Privacy::from_str(&self.anonymity?);

            Some(crate::proxy::Proxy { socket, ty, pri })
        }
    }

    pub(super) struct ProxyFetcher;
    #[async_trait]
    impl super::ProxyFetcher for ProxyFetcher {
        async fn fetch(&self, sender: ProxySender) -> Result<()> {
            let html = super::get("https://ip.uqidata.com/").await?.xpath().await?;

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

/// * http://proxydb.net/
mod proxydb {
    use super::{ProxySender, SendProxyEx};
    use crate::{
        error::Result,
        proxy::{IntoProxy, Privacy, ProxyType},
    };
    use async_trait::async_trait;
    use reqwest_scraper::{FromXPath, ScraperResponse};
    use std::{net::SocketAddr, str::FromStr};

    #[derive(Debug, FromXPath)]
    #[xpath(path = r#"//*[@id="app"]/div[1]/table/tbody/tr"#)]
    pub(super) struct Proxy {
        #[xpath(path = "./td[1]/text()")]
        socket: Option<String>,

        #[xpath(path = "./td[5]/text()")]
        protocol: Option<String>,

        #[xpath(path = "./td[6]/text()")]
        anonymity: Option<String>,
    }

    impl IntoProxy for Proxy {
        fn make_proxy(self) -> Option<crate::proxy::Proxy> {
            let socket = SocketAddr::from_str(&self.socket?).ok()?;
            let ty = ProxyType::from_str(&self.protocol?).ok()?;
            let pri = Privacy::from_str(&self.anonymity?);

            Some(crate::proxy::Proxy { socket, ty, pri })
        }
    }

    pub(super) struct ProxyFetcher;
    #[async_trait]
    impl super::ProxyFetcher for ProxyFetcher {
        async fn fetch(&self, sender: ProxySender) -> Result<()> {
            let html = super::get("http://proxydb.net/").await?.xpath().await?;

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

/// * https://list.proxylistplus.com/Fresh-HTTP-Proxy-List-1
mod proxylistplus {
    use super::{ProxySender, SendProxyEx};
    use crate::{
        error::Result,
        proxy::{IntoProxy, Privacy, ProxyType},
    };
    use async_trait::async_trait;
    use reqwest_scraper::{FromXPath, ScraperResponse};
    use std::{net::SocketAddr, str::FromStr};

    #[derive(Debug, FromXPath)]
    #[xpath(path = r#"//*[@id="page"]/table[2]/tbody/tr[position() >= 3]"#)]
    pub(super) struct Proxy {
        #[xpath(path = "./td[2]/text()")]
        ip: Option<String>,

        #[xpath(path = "./td[3]/text()")]
        port: Option<String>,

        #[xpath(path = "./td[4]/text()")]
        anonymity: Option<String>,

        #[xpath(path = "./td[7]/text()")]
        https: Option<String>,
    }

    impl IntoProxy for Proxy {
        fn make_proxy(self) -> Option<crate::proxy::Proxy> {
            let socket = SocketAddr::from_str(&format!("{}:{}", self.ip?, self.port?)).ok()?;
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

/// * https://www.iplocation.net/proxy-list
mod iplocation {
    use super::{ProxySender, SendProxyEx};
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
            let socket = SocketAddr::from_str(&format!("{}:{}", self.ip?, self.port?)).ok()?;
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
