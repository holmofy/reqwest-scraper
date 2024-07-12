use anyhow::Result;
use reqwest_scraper::{FromXPath, ScraperResponse};

#[tokio::main]
async fn main() {
    request().await.expect("request error");
}

#[derive(Debug, FromXPath)]
#[xpath(path = "//div[@id='user-repositories-list']/ul/li")]
struct Repo {
    #[xpath(path = ".//a[contains(@itemprop,'name')]/text()", default = "<unname>")]
    name: String,

    #[xpath(path = ".//span[contains(@itemprop,'programmingLanguage')]/text()")]
    program_lang: Option<String>,

    #[xpath(path = ".//div[contains(@class,'topics-row-container')]/a/text()")]
    topics: Vec<String>,
}

async fn request() -> Result<()> {
    let html = reqwest::get("https://github.com/holmofy")
        .await?
        .xpath()
        .await?;

    // simple extract element
    let name = html
        .select("//span[contains(@class,'p-name')]")?
        .as_node()
        .unwrap()
        .text();
    assert_eq!(name.trim(), "holmofy");

    // iterate elements
    let select_result = html
        .select("//ul[contains(@class,'vcard-details')]/li[contains(@class,'vcard-detail')]")?
        .as_nodes();

    println!("{}", select_result.len());

    for item in select_result.into_iter() {
        let attr = item.attr("aria-label").unwrap_or_else(|| "".into());
        println!("{}", attr);
        println!("{}", item.text().trim());
    }

    let html = reqwest::get("https://github.com/holmofy?tab=repositories")
        .await?
        .xpath()
        .await?;

    // 2. Select List Element
    println!("\n2. Select List Element");
    let select_result = html.select("//div[@id='user-repositories-list']/ul/li")?;

    for item in select_result.as_nodes() {
        let name = item.findvalue(".//a[contains(@itemprop,'name')]/text()")?;

        let program_lang =
            item.findvalue(".//span[contains(@itemprop,'programmingLanguage')]/text()")?;

        let topics = item.findvalues(".//div[contains(@class,'topics-row-container')]/a/text()")?;

        let item = Repo {
            name,
            program_lang: Some(program_lang),
            topics,
        };

        println!("{:?}", item);
    }

    // 3. Extract By Derived Macros
    println!("\n3. Extract By Derived Macros");

    let items = Repo::from_xhtml(html)?;
    items.iter().for_each(|item| println!("{:?}", item));

    Ok(())
}
