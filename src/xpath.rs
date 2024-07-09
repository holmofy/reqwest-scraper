use std::collections::HashSet;

use itertools::Itertools;
use libxml::{
    tree::Document,
    xpath::{Context, Object},
};

use crate::error::{Result, ScraperError};

/// Html Response
pub struct XHtml {
    pub(crate) doc: Document,
}

/// Wrap HTML document and compiled xpath
pub struct XPathResult {
    object: Object,
}

impl XHtml {
    pub fn select(&self, xpath: &str) -> Result<XPathResult> {
        let context = Context::new(&self.doc)
            .map_err(|_| ScraperError::XPathError(format!("xpath parse failed:{}", xpath)))?;
        let object = context
            .evaluate(xpath)
            .map_err(|_| ScraperError::XPathError(format!("xpath parse failed:{}", xpath)))?;
        Ok(XPathResult { object })
    }
}

pub struct Node {
    node: libxml::tree::node::Node,
}

impl XPathResult {
    pub fn as_nodes(&self) -> Vec<Node> {
        self.object
            .get_nodes_as_vec()
            .into_iter()
            .map(|node| Node { node })
            .collect_vec()
    }

    pub fn as_strs(&self) -> Vec<String> {
        self.object.get_nodes_as_str()
    }

    pub fn as_str(&self) -> Option<String> {
        self.object.get_nodes_as_str().first().map(|s| s.to_owned())
    }

    pub fn as_node(&self) -> Option<Node> {
        self.object
            .get_nodes_as_vec()
            .first()
            .map(|n| Node { node: n.to_owned() })
    }
}

impl Node {
    /// Returns the element name.
    pub fn name(&self) -> String {
        self.node.get_name()
    }

    /// Returns the element ID.
    pub fn id(&self) -> Option<String> {
        self.node.get_attribute("id")
    }

    /// Returns the element class.
    pub fn classes(&self) -> HashSet<String> {
        self.node.get_class_names()
    }

    /// Returns the value of an attribute.
    pub fn attr(&self, attr: &str) -> Option<String> {
        self.node.get_attribute(attr)
    }

    pub fn text(&self) -> String {
        todo!()
    }

    /// Returns the HTML of this element.
    pub fn html(&self) -> String {
        // TODO
        self.node.get_content()
    }

    /// Returns the inner HTML of this element.
    pub fn inner_html(&self) -> String {
        todo!()
    }

    /// Iterate over all child nodes which are elements
    pub fn children(&self) -> Vec<Node> {
        self.node
            .get_child_elements()
            .into_iter()
            .map(|node| Node { node })
            .collect_vec()
    }

    pub fn find(&self, relative_xpath: &str) -> Result<XPathResult> {
        let context = Context::from_node(&self.node).map_err(|_| {
            ScraperError::XPathError(format!("relative xpath parse failed:{}", relative_xpath))
        })?;
        let object = context.evaluate(relative_xpath).map_err(|_| {
            ScraperError::XPathError(format!("relative xpath parse failed:{}", relative_xpath))
        })?;
        Ok(XPathResult { object })
    }
}
