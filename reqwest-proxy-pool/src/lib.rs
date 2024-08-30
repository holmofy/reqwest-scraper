mod proxy;

use reqwest::Url;

pub struct ProxyPool {}

impl ProxyPool {
    pub fn new() -> Self {
        Self {}
    }

    pub fn choose(&self) -> Option<Url> {
        reqwest::Url::parse("https://my.prox").ok()
    }

    fn scraper(&self) {}

    fn check(&self) {}
}
