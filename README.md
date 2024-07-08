## reqwest-scraper - Scraping integration with reqwest

Expand [reqwest](https://github.com/seanmonstar/reqwest) functionality to support multiple crawling methods.

### Features

* [x] Use [JsonPath](#jsonpath) to select fields in json
* [x] Select elements in HTML using [CSS selector](#css-selector)
* [ ] Evalute the value in HTML using [xpath expression](#xpath)
* [ ] Derive macro extract

### Start Guide

* add dependency
    ```toml
    reqwest = { version = "0.12", features = ["json"] }
    reqwest-scraper="0.1.0"
    ```
* use ScraperResponse
    ```rust
    use reqwest_scraper::ScraperResponse;
    ```


<h3 id="jsonpath">JsonPath</h3>

```rust
use reqwest_scraper::ScraperResponse;

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
```

<h3 id="css-selector">CSS selector</h3>

```rust
use reqwest_scraper::ScraperResponse;

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
```

<h3 id="xpath">XPath</h3>

TODO

## Related Projects

* [reqwest](https://github.com/seanmonstar/reqwest)
* [scraper](https://github.com/causal-agent/scraper)
* [Skyscraper](https://github.com/James-LG/Skyscraper)
* [jsonpath_lib](https://github.com/freestrings/jsonpath)
* [unhtml.rs](https://github.com/Hexilee/unhtml.rs)
* [nipper](https://github.com/importcjj/nipper)