## reqwest-scraper - Scraping integration with reqwest

[![crates.io](https://img.shields.io/crates/v/reqwest-scraper.svg)](https://crates.io/crates/reqwest-scraper)
[![Documentation](https://docs.rs/reqwest-scraper/badge.svg)](https://docs.rs/reqwest-scraper)
[![CI](https://github.com/holmofy/reqwest-scraper/workflows/Publish/badge.svg)](https://github.com/holmofy/reqwest-scraper/actions?query=workflow%3APublish)

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
    reqwest-scraper="0.1.3"
    ```
* use ScraperResponse
    ```rust
    use reqwest_scraper::ScraperResponse;
    ```


<h3 id="jsonpath">JsonPath</h3>

* `Json::select<T: DeserializeOwned>(path: &str) -> Result<Vec<T>>`
* `Json::select_one<T: DeserializeOwned>(path: &str) -> Result<T>`
* `Json::select_as_str(path: &str) -> Result<String>`

[**example**](./examples/json.rs):

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

* `Html::select(selector: &str) -> Result<Selectable>`
* `Selectable::iter() -> impl Iterator<SelectItem>`
* `SelectItem::name() -> &str`
* `SelectItem::id() -> Option<&str>`
* `SelectItem::has_class(class: &str, case_sensitive: CaseSensitivity) -> bool`
* `SelectItem::classes() -> Classes`
* `SelectItem::attrs() -> Attrs`
* `SelectItem::attr(attr: &str) -> Option<&str>`
* `SelectItem::text() -> String`
* `SelectItem::html() -> String`
* `SelectItem::inner_html() -> String`
* `SelectItem::children() -> impl Iterator<SelectItem>`
* `SelectItem::find(selector: &str) -> Result<Selectable>`

[**example**](./examples/html.rs):

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
* [nipper](https://github.com/importcjj/nipper)
* [jsonpath_lib](https://github.com/freestrings/jsonpath)
* [unhtml.rs](https://github.com/Hexilee/unhtml.rs)
* [xpath-scraper](https://github.com/Its-its/xpath-scraper)