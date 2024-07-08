use reqwest_scraper::error::Result;
use reqwest_scraper::ScraperResponse;

#[tokio::main]
async fn main() {
    request().await.expect("request error");
}

async fn request() -> Result<()> {
    let html = reqwest::get("https://github.com/holmofy")
        .await?
        .css_selector()
        .await?;

    assert_eq!(
        html.select(".p-name")?.iter().nth(0).unwrap().text().trim(),
        "holmofy"
    );

    let select_result = html.select(".vcard-details > li.vcard-detail")?;

    for detail_item in select_result.iter() {
        println!("{}", detail_item.attr("aria-label").unwrap())
    }

    Ok(())
}
