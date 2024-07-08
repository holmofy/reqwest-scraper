use reqwest_scraper::error::Result;
use reqwest_scraper::ScraperResponse;

#[tokio::main]
async fn main() {
    request().await.expect("request error");
}

pub async fn request() -> Result<()> {
    let json = reqwest::Client::builder()
        .build()?
        .get("https://api.github.com/search/repositories?q=rust")
        .header("User-Agent", "Rust Reqwest")
        .send()
        .await?
        .jsonpath()
        .await?;

    let total_count = json.select_as_str("$.total_count")?;
    let names: Vec<String> = json.select("$.items[*].full_name")?;

    println!("{}", total_count);
    println!("{}", names.join("\t"));

    Ok(())
}
