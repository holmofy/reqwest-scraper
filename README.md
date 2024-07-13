## reqwest-scraper - Web scraping integration with reqwest

[![crates.io](https://img.shields.io/crates/v/reqwest-scraper.svg)](https://crates.io/crates/reqwest-scraper)
[![Documentation](https://docs.rs/reqwest-scraper/badge.svg)](https://docs.rs/reqwest-scraper)
[![CI](https://github.com/holmofy/reqwest-scraper/workflows/Publish/badge.svg)](https://github.com/holmofy/reqwest-scraper/actions?query=workflow%3APublish)

Extends [reqwest](https://github.com/seanmonstar/reqwest) to support multiple web scraping methods.

### Features

* [x] Use [JsonPath](#jsonpath) to select fields in json response
* [x] Select elements in HTML response using [CSS selector](#css-selector)
* [x] Evalute the value in HTML response using [xpath expression](#xpath)
* [x] [Derive macro extract](#macros)

### Start Guide

* add dependency
    ```toml
    reqwest = { version = "0.12", features = ["json"] }
    reqwest-scraper="0.2.1"
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
* `Selectable::first() -> Option<SelectItem>`
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

* `XHtml::select(xpath: &str) -> Result<XPathResult>`
* `XPathResult::as_nodes() -> Vec<Node>`
* `XPathResult::as_strs() -> Vec<String>`
* `XPathResult::as_node() -> Option<Node>`
* `XPathResult::as_str() -> Option<String>`
* `Node::name() -> String`
* `Node::id() -> Option<String>`
* `Node::classes() -> HashSet<String>`
* `Node::attr(attr: &str) -> Option<String>`
* `Node::has_attr(attr: &str) -> bool`
* `Node::text() -> String`
* TODO: `Node::html() -> String`
* TODO: `Node::inner_html() -> String`
* `Node::children() -> Vec<Node>`
* `Node::findnodes(relative_xpath: &str) -> Result<Vec<Node>>`
* `Node::findvalues(relative_xpath: &str) -> Result<Vec<String>>`
* `Node::findnode(relative_xpath: &str) -> Result<Option<Node>>`
* `Node::findvalue(relative_xpath: &str) -> Result<Option<String>>`

[**example**](./examples/xpath.rs):

```rust
async fn request() -> Result<()> {
    let html = reqwest::get("https://github.com/holmofy")
        .await?
        .xpath()
        .await?;

    // simple extract element
    let name = html
        .select("//span[contains(@class,'p-name')]")?
        .as_node()
        .unwrap()
        .text();
    println!("{}", name);
    assert_eq!(name.trim(), "holmofy");

    // iterate elements
    let select_result = html
        .select("//ul[contains(@class,'vcard-details')]/li[contains(@class,'vcard-detail')]")?
        .as_nodes();

    println!("{}", select_result.len());

    for item in select_result.into_iter() {
        let attr = item.attr("aria-label").unwrap_or_else(|| "".into());
        println!("{}", attr);
        println!("{}", item.text());
    }

    // attribute extract
    let select_result = html
        .select("//ul[contains(@class,'vcard-details')]/li[contains(@class,'vcard-detail')]/@aria-label")?
        .as_strs();

    println!("{}", select_result.len());
    select_result.into_iter().for_each(|s| println!("{}", s));

    Ok(())
}
```

<h3 id="macros">Derive macro extract</h3>

**use `FromCssSelector` & `selector` to extract html element into struct**
```rust
// define struct and derive the FromCssSelector trait
#[derive(Debug, FromCssSelector)]
#[selector(path = "#user-repositories-list > ul > li")]
struct Repo {
    #[selector(path = "a[itemprop~='name']", default = "<unname>", text)]
    name: String,

    #[selector(path = "span[itemprop~='programmingLanguage']", text)]
    program_lang: Option<String>,

    #[selector(path = "div.topics-row-container>a", text)]
    topics: Vec<String>,
}

// request
let html = reqwest::get("https://github.com/holmofy?tab=repositories")
    .await?
    .css_selector()
    .await?;

// Use the generated `from_html` method to extract data into the struct
let items = Repo::from_html(html)?;
items.iter().for_each(|item| println!("{:?}", item));
```

**use `FromXPath` & `xpath` to extract html element into struct**
```rust
// define struct and derive the FromXPath trait
#[derive(Debug, FromXPath)]
#[xpath(path = "//div[@id='user-repositories-list']/ul/li")]
struct Repo {
    #[xpath(path = ".//a[contains(@itemprop,'name')]/text()", default = "<unname>")]
    name: String,

    #[xpath(path = ".//span[contains(@itemprop,'programmingLanguage')]/text()")]
    program_lang: Option<String>,

    #[xpath(path = ".//div[contains(@class,'topics-row-container')]/a/text()")]
    topics: Vec<String>,
}

let html = reqwest::get("https://github.com/holmofy?tab=repositories")
    .await?
    .xpath()
    .await?;

// Use the generated `from_xhtml` method to extract data into the struct
let items = Repo::from_xhtml(html)?;
items.iter().for_each(|item| println!("{:?}", item));
```


## Related Projects

* [reqwest](https://github.com/seanmonstar/reqwest)
* [scraper](https://github.com/causal-agent/scraper)
* [nipper](https://github.com/importcjj/nipper)
* [jsonpath_lib](https://github.com/freestrings/jsonpath)
* [unhtml.rs](https://github.com/Hexilee/unhtml.rs)
* [xpath-scraper](https://github.com/Its-its/xpath-scraper)