/// Proxy Pool

/// * https://www.docip.net/free
mod docip {
    use reqwest_scraper::{FromXPath, ScraperResponse};

    #[derive(Debug, FromXPath)]
    #[xpath(path = r#"//*[@id="freebody"]/tr"#)]
    struct Proxy {
        #[xpath(path = "./th/text()")]
        socket: String,

        #[xpath(path = "./td[1]/text()")]
        protocol: String,

        #[xpath(path = "./td[2]/text()")]
        anonymity: String,
    }

    async fn fetch() {
        let html = reqwest::get("https://www.docip.net/free")
            .await?
            .xpath()
            .await?;

        let items = Proxy::from_xhtml(html)?;
    }
}

/// * https://www.89ip.cn/index_1.html
mod www89ip {
    use reqwest_scraper::{FromXPath, ScraperResponse};

    #[derive(Debug, FromXPath)]
    #[xpath(path = r#"//table[contains(@class,'layui-table')]/tbody/tr"#)]
    struct Proxy {
        #[xpath(path = "./td[1]/text()")]
        ip: String,

        #[xpath(path = "./td[2]/text()")]
        port: String,
    }

    async fn fetch() {
        let html = reqwest::get("https://www.89ip.cn/index_1.html")
            .await?
            .xpath()
            .await?;

        let items = Proxy::from_xhtml(html)?;
    }
}

/// * http://www.ip3366.net/free/?stype=1
mod ip3366 {
    use reqwest_scraper::{FromXPath, ScraperResponse};

    #[derive(Debug, FromXPath)]
    #[xpath(path = r#"//*[@id="list"]/table/tbody/tr"#)]
    struct Proxy {
        #[xpath(path = "./td[1]/text()")]
        ip: String,

        #[xpath(path = "./td[2]/text()")]
        port: String,

        #[xpath(path = "./td[3]/text()")]
        anonymity: String,

        #[xpath(path = "./td[4]/text()")]
        protocol: String,
    }

    async fn fetch() {
        let html = reqwest::get("http://www.ip3366.net/free/?stype=1")
            .await?
            .xpath()
            .await?;

        let items = Proxy::from_xhtml(html)?;
    }
}

/// * https://www.kuaidaili.com/free/intr/
mod ip3366 {
    use reqwest_scraper::{FromXPath, ScraperResponse};

    #[derive(Debug, FromXPath)]
    #[xpath(path = r#"//*[@id="table__free-proxy"]/div/table/tbody/tr"#)]
    struct Proxy {
        #[xpath(path = "./td[1]/text()")]
        ip: String,

        #[xpath(path = "./td[2]/text()")]
        port: String,

        #[xpath(path = "./td[3]/text()")]
        anonymity: String,

        #[xpath(path = "./td[4]/text()")]
        protocol: String,
    }

    async fn fetch() {
        let html = reqwest::get("https://www.kuaidaili.com/free/intr/")
            .await?
            .xpath()
            .await?;

        let items = Proxy::from_xhtml(html)?;
    }
}

/// * https://www.zdaye.com/free/1/
mod zdaye {
    use reqwest_scraper::{FromXPath, ScraperResponse};

    #[derive(Debug, FromXPath)]
    #[xpath(path = r#"//*[@id="ipc"]/tbody/tr"#)]
    struct Proxy {
        #[xpath(path = "./td[1]/text()")]
        ip: String,

        #[xpath(path = "./td[2]/text()")]
        port: String,

        #[xpath(path = "./td[3]/text()")]
        anonymity: String,

        #[xpath(path = "./td[6]/div[contains(@class,'iyes')]")]
        https: bool,
    }

    async fn fetch() {
        let html = reqwest::get("https://www.zdaye.com/free/1/")
            .await?
            .xpath()
            .await?;

        let items = Proxy::from_xhtml(html)?;
    }
}

/// * https://www.data5u.com/
/// * https://ip.uqidata.com/
mod uqidata {
    use reqwest_scraper::{FromXPath, ScraperResponse};

    #[derive(Debug, FromXPath)]
    #[xpath(path = r#"//*[@id="main_container"]/div[1]/table/tbody/tr[position() >= 3]"#)]
    struct Proxy {
        #[xpath(path = "./td[1]/text()")]
        ip: String,

        #[xpath(path = "./td[2]/text()")]
        port: String,

        #[xpath(path = "./td[3]/text()")]
        protocol: String,

        #[xpath(path = "./td[4]/text()")]
        anonymity: String,
    }

    async fn fetch() {
        let html = reqwest::get("https://ip.uqidata.com/")
            .await?
            .xpath()
            .await?;

        let items = Proxy::from_xhtml(html)?;
    }
}

/// * http://proxydb.net/
mod proxydb {
    use reqwest_scraper::{FromXPath, ScraperResponse};

    #[derive(Debug, FromXPath)]
    #[xpath(path = r#"//*[@id="app"]/div[1]/table/tbody/tr"#)]
    struct Proxy {
        #[xpath(path = "./td[1]/text()")]
        socket: String,

        #[xpath(path = "./td[5]/text()")]
        protocol: String,

        #[xpath(path = "./td[6]/text()")]
        anonymity: String,
    }

    async fn fetch() {
        let html = reqwest::get("http://proxydb.net/").await?.xpath().await?;

        let items = Proxy::from_xhtml(html)?;
    }
}

/// * https://list.proxylistplus.com/Fresh-HTTP-Proxy-List-1
mod proxylistplus {
    use reqwest_scraper::{FromXPath, ScraperResponse};

    #[derive(Debug, FromXPath)]
    #[xpath(path = r#"//*[@id="page"]/table[2]/tbody/tr[position() >= 3]"#)]
    struct Proxy {
        #[xpath(path = "./td[2]/text()")]
        ip: String,

        #[xpath(path = "./td[3]/text()")]
        port: String,

        #[xpath(path = "./td[4]/text()")]
        anonymity: String,

        #[xpath(path = "./td[7]/text()")]
        https: bool,
    }

    async fn fetch() {
        let html = reqwest::get("https://list.proxylistplus.com/Fresh-HTTP-Proxy-List-1")
            .await?
            .xpath()
            .await?;

        let items = Proxy::from_xhtml(html)?;
    }
}

/// * https://www.iplocation.net/proxy-list
mod iplocation {
    use reqwest_scraper::{FromXPath, ScraperResponse};

    #[derive(Debug, FromXPath)]
    #[xpath(
        path = r#"//*[@id="wrapper"]/div[1]/div[2]/div/section/div/div/div[1]/div[5]/div/div/table/tbody/tr"#
    )]
    struct Proxy {
        #[xpath(path = "./td[2]/text()")]
        ip: String,

        #[xpath(path = "./td[3]/text()")]
        port: String,

        #[xpath(path = "./td[5]/span[contains(@class,'my_http_status_green')]")]
        http: bool,

        #[xpath(path = "./td[6]/span[contains(@class,'my_https_status_red')]")]
        https: bool,
    }

    async fn fetch() {
        let html = reqwest::get("https://www.iplocation.net/proxy-list")
            .await?
            .xpath()
            .await?;

        let items = Proxy::from_xhtml(html)?;
    }
}
