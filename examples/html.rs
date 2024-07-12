use anyhow::Result;
use reqwest_scraper::{FromCssSelector, ScraperResponse};
use reqwest_scraper_macros::FromCssSelector;

#[tokio::main]
async fn main() {
    request().await.expect("request error");
}

#[derive(Debug, FromCssSelector)]
#[css_selector(selector = ".vcard-details > li.vcard-detail")]
struct ExtractItem {
    #[css_selector(attr = "aria-label", default = "\"label_default_value\"")]
    aria_label: String,
    #[css_selector(
        selector = "svg.octicon",
        attr = "class",
        default = "\"svg_default_value\""
    )]
    svg_icon: String,
}

async fn request() -> Result<()> {
    let html = reqwest::get("https://github.com/holmofy")
        .await?
        .css_selector()
        .await?;

    // Simple extract
    assert_eq!(
        html.select(".p-name")?
            .first()
            .map(|e| e.text())
            .unwrap_or("xxx".into()),
        "holmofy"
    );

    // Select List Element
    let select_result = html.select(".vcard-details > li.vcard-detail")?;

    for detail_item in select_result.iter() {
        let label = detail_item.attr("aria-label").unwrap_or_else(|| "");
        let svg_element = detail_item.select("svg.octicon")?;
        let svg_class = svg_element
            .first()
            .and_then(|e| e.attr("class"))
            .unwrap_or("default_value");

        let item = ExtractItem {
            aria_label: label.into(),
            svg_icon: svg_class.into(),
        };

        println!("{:?}", item);
    }

    // 3. Extract By Derived Macros
    let items = ExtractItem::from_html(html)?;
    items.iter().for_each(|item| println!("{:?}", item));

    Ok(())
}
