use anyhow::Result;
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

    // simple extract element
    let name = html
        .select("//span[contains(@class,'p-name')]")?
        .as_node()
        .unwrap()
        .text();
    println!("{}", name);
    assert_eq!(name.trim(), "holmofy");

    // iterate elements
    let select_result = html
        .select("//ul[contains(@class,'vcard-details')]/li[contains(@class,'vcard-detail')]")?
        .as_nodes();

    println!("{}", select_result.len());

    for item in select_result.into_iter() {
        let attr = item.attr("aria-label").unwrap_or_else(|| "".into());
        println!("{}", attr);
    }

    // attribute extract
    let select_result = html
        .select("//ul[contains(@class,'vcard-details')]/li[contains(@class,'vcard-detail')]/@aria-label")?
        .as_strs();

    println!("{}", select_result.len());
    select_result.into_iter().for_each(|s| println!("{}", s));

    //
    let select_result = html
        .select("//ul[contains(@class,'vcard-details')]/li[contains(@class,'vcard-detail')]/@aria-label")?
        .as_nodes();

    println!("{}", select_result.len());

    select_result
        .into_iter()
        .for_each(|n| println!("{}", n.name()));

    Ok(())
}
