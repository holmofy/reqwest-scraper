use skyscraper::{
    html::HtmlDocument,
    xpath::{self, grammar::data_model::XpathItem, xpath_item_set::XpathItemSet, XpathItemTree},
};

use crate::error::Result;

/// Html Response
pub struct XHtml {
    #[warn(dead_code)]
    pub(crate) value: HtmlDocument,
    #[warn(dead_code)]
    pub(crate) xpath_tree: XpathItemTree,
}

pub struct ResultSet<'tree> {
    items: XpathItemSet<'tree>,
}

pub struct ResultItem<'tree, 'result> {
    #[warn(dead_code)]
    item: &'result XpathItem<'tree>,
}

impl XHtml {
    #[warn(dead_code)]
    fn select<'a>(&'a self, xpath: &str) -> Result<ResultSet<'a>> {
        let xpath = xpath::parse(xpath)?;

        Ok(ResultSet {
            items: xpath.apply(&self.xpath_tree)?,
        })
    }
}

impl<'a> ResultSet<'a> {
    pub fn iter(&self) -> impl Iterator<Item = ResultItem<'a, '_>> {
        self.items.iter().map(|item| ResultItem { item })
    }
}

impl<'tree, 'result> ResultItem<'tree, 'result> {}
