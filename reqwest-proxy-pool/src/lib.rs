mod error;
mod proxy;

use proxy::Proxy;
use reqwest::Url;
use tokio::sync::mpsc;

pub struct ProxyPool {}

impl ProxyPool {
    pub fn new() -> Self {
        Self {}
    }

    pub fn choose(&self) -> Option<Url> {
        reqwest::Url::parse("https://my.prox").ok()
    }

    pub async fn scraper(&self) {
        let (tx, mut rx) = mpsc::channel::<Proxy>(32);
        proxy::proxy_fetch::fetch(tx).await;

        while let Some(proxy) = rx.recv().await {
            println!("GOT = {:?}", proxy);
        }
    }

    pub async fn check(&self) {}
}
