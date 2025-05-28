use reqwest::Client;
// use reqwest_middleware::ClientWithMiddleware;
use reqwest_scraper::{error::Result, include_http, ScraperResponse};

// fn client() -> ClientWithMiddleware {
//     ClientWithMiddleware::default()
// }

fn client() -> Client {
    Client::default()
}

include_http!("examples/example.http", client, {phone="18720232389", password="101010"});

#[tokio::main]
async fn main() -> Result<()> {
    let json = papers(20).await?.jsonpath().await?;
    println!("{:?}", json);
    Ok(())
}
