use anyhow::Result;
use reqwest_scraper::xpath::NodeResult;
use reqwest_scraper::ScraperResponse;

#[tokio::main]
async fn main() {
    request().await.expect("request error");
}

async fn request() -> Result<()> {
    let html = reqwest::get("https://github.com/holmofy")
        .await?
        .xpath()
        .await?;

    assert_eq!(
        html.select("//span[contains(@class,'p-name')]")?
            .as_value()?
            .into_string(),
        "holmofy"
    );

    let select_result = html
        .select("//ul[contains(@class,'vcard-details']/li[contains(@class,'vcard-detail']")?
        .as_value()?;

    for detail_item in select_result.as_node()?.into_iter() {
        let attr = detail_item
            .element()
            .unwrap()
            .attribute("aria-label")
            .unwrap();
        println!("{}", attr.value());
    }

    Ok(())
}
