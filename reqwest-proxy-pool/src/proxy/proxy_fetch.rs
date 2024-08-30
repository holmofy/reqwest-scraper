/// Proxy Pool
/// * https://www.docip.net/free
/// * https://www.89ip.cn/index_1.html
/// * http://www.ip3366.net/free/?stype=1
/// * https://www.kuaidaili.com/free/intr/
/// * https://www.zdaye.com/free/1/
/// * https://www.data5u.com/
/// * http://proxydb.net/
/// * https://list.proxylistplus.com/Fresh-HTTP-Proxy-List-1
/// * https://www.iplocation.net/proxy-list

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
