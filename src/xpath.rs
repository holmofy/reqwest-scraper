use sxd_document::{dom::Document, Package};
use sxd_xpath::{Context, Factory, XPath};

use crate::error::{Result, ScraperError};

/// Html Response
pub struct XHtml {
    pub(crate) value: Package,
}

// Wrap HTML document and compiled xpath
pub struct Selectable<'p> {
    doc: Document<'p>,
    xpath: XPath,
}

impl XHtml {
    pub fn select<'a>(&'a self, xpath: &str) -> Result<Selectable<'a>> {
        Ok(Selectable::wrap(&self.value, xpath)?)
    }
}

impl<'p> Selectable<'p> {
    fn wrap(package: &'p Package, xpath: &str) -> Result<Selectable<'p>> {
        let factory = Factory::new();
        let xpath = factory
            .build(xpath)?
            .ok_or_else(|| ScraperError::XPathError(format!("xpath is empty: {0}", xpath)))?;
        Ok(Self {
            doc: package.as_document(),
            xpath,
        })
    }
}

impl<'p> Selectable<'p> {
    pub fn as_value(&self) -> Result<Value<'p>> {
        let context = Context::new();

        Ok(self.xpath.evaluate(&context, self.doc.root())?)
    }
}

pub type Value<'d> = sxd_xpath::Value<'d>;

pub type Nodeset<'d> = sxd_xpath::nodeset::Nodeset<'d>;

pub trait NodeResult<'d> {
    fn as_node(&'d self) -> Result<&'d Nodeset<'d>>;
}

impl<'d> NodeResult<'d> for Value<'d> {
    fn as_node(&'d self) -> Result<&'d Nodeset<'d>> {
        match self {
            Value::Nodeset(nodeset) => Ok(nodeset),
            _ => Err(ScraperError::XPathError(format!(
                "The result of the xpath match is not a DOM node:{}",
                self.string()
            ))),
        }
    }
}
