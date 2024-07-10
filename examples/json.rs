use anyhow::Result;
use reqwest_scraper::ScraperResponse;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Owner {
    login: String,
    id: i64,
    node_id: String,
    avatar_url: String,
    gravatar_id: String,
    url: String,
    html_url: String,
    followers_url: String,
    following_url: String,
    gists_url: String,
    starred_url: String,
    subscriptions_url: String,
    organizations_url: String,
    repos_url: String,
    events_url: String,
    received_events_url: String,
    #[serde(alias = "type")]
    _type: String,
    site_admin: bool,
}

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

    let total_count_str = json.select_as_str("$.total_count")?;
    let total_count_int: i32 = json.select_one("$.total_count")?;
    let names: Vec<String> = json.select("$.items[*].full_name")?;
    let owners: Vec<Owner> = json.select("$.items[*].owner")?;

    println!("{}", total_count_str);
    println!("{}", total_count_int);
    println!("{}", names.join("\t"));
    owners.iter().for_each(|o| println!("{:?}", o));

    Ok(())
}
