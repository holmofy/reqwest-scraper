use anyhow::Result;
use reqwest_scraper::{css_selector::SelectItem, ScraperResponse};

#[tokio::main]
async fn main() {
    request().await.expect("request error");
}

struct ExtractItem {
    aria_label: String,
    svg_icon: String,
}

async fn request() -> Result<()> {
    let html = reqwest::get("https://github.com/holmofy")
        .await?
        .css_selector()
        .await?;

    // simple
    assert_eq!(
        html.select(".p-name")?.iter().nth(0).unwrap().text().trim(),
        "holmofy"
    );

    // select list element
    let select_result = html.select(".vcard-details > li.vcard-detail")?;

    let result: Vec<ExtractItem> = Vec::new();
    for detail_item in select_result.iter() {
        println!("{}", detail_item.attr("aria-label").unwrap_or_else(|| ""));
        println!(
            "{}",
            detail_item
                .find("svg.octicon")?
                .iter()
                .nth(0)
                .unwrap()
                .attr("class")
                .unwrap()
        );
    }

    Ok(())
}
