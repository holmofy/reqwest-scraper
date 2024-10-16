use reqwest::Client;
use reqwest_scraper::{error::Result, include_http, ScraperResponse};

fn client() -> Client{
    Client::new()
}

include_http!("examples/example.http", client);

#[tokio::main]
async fn main() -> Result<()> {
    let json = papers(20).await?.jsonpath().await?;
    println!("{:?}", json);
    Ok(())
}
