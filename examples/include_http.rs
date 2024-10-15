use reqwest_scraper::{error::Result, include_http, ScraperResponse};

include_http!("examples/example.http");

#[tokio::main]
async fn main() -> Result<()> {
    let json = papers(20).await?.jsonpath().await?;
    println!("{:?}", json);
    Ok(())
}
