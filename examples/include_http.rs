use reqwest::Client;
use reqwest_scraper::{error::Result, include_http, ScraperResponse};

fn client() -> Client {
    Client::new()
}

include_http!("examples/example.http", client, {phone="18720232389", password="101010"});

#[tokio::main]
async fn main() -> Result<()> {
    let json = papers(20).await?.jsonpath().await?;
    println!("{:?}", json);
    Ok(())
}
