use anyhow::Result;
use reqwest_scraper::{FromCssSelector, ScraperResponse};

#[tokio::main]
async fn main() {
    request().await.expect("request error");
}

#[derive(Debug, FromCssSelector)]
#[selector(path = "#user-repositories-list > ul > li")]
struct Repo {
    #[selector(path = "a[itemprop~='name']", default = "<unname>", text)]
    name: String,

    #[selector(path = "span[itemprop~='programmingLanguage']", text)]
    program_lang: Option<String>,

    #[selector(path = "div.topics-row-container>a", text)]
    topics: Vec<String>,
}

async fn request() -> Result<()> {
    let html = reqwest::get("https://github.com/holmofy")
        .await?
        .css_selector()
        .await?;

    // 1. Simple extract
    assert_eq!(
        html.select(".p-name")?
            .first()
            .map(|e| e.text())
            .unwrap_or("xxx".into()),
        "holmofy"
    );

    let html = reqwest::get("https://github.com/holmofy?tab=repositories")
        .await?
        .css_selector()
        .await?;

    // 2. Select List Element
    println!("\n2. Select List Element");
    let select_result = html.select("#user-repositories-list > ul > li")?;

    for item in select_result.iter() {
        let name = item
            .select("a[itemprop~='name']")?
            .first()
            .map(|e| e.text())
            .unwrap_or("<unname>".into());

        let program_lang = item
            .select("span[itemprop~='programmingLanguage']")?
            .first()
            .map(|e| e.text());

        let topics = item
            .select("div.topics-row-container>a")?
            .iter()
            .map(|e| e.text())
            .collect::<Vec<_>>();

        let item = Repo {
            name,
            program_lang,
            topics,
        };

        println!("{:?}", item);
    }

    // 3. Extract By Derived Macros
    println!("\n3. Extract By Derived Macros");

    let items = Repo::from_html(html)?;
    items.iter().for_each(|item| println!("{:?}", item));

    Ok(())
}
