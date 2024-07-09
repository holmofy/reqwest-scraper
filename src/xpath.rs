//!  Evalute the value in HTML response using xpath expression
//!
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
    /// Using xpath to extract results from html
    pub fn select(&self, xpath: &str) -> Result<XPathResult> {
        let context = Context::new(&self.doc)
            .map_err(|_| ScraperError::XPathError(format!("xpath parse failed:{}", xpath)))?;
        let object = context
            .evaluate(xpath)
            .map_err(|_| ScraperError::XPathError(format!("xpath parse failed:{}", xpath)))?;
        Ok(XPathResult { object })
    }
}

/// Html Node
pub struct Node {
    node: libxml::tree::node::Node,
}

impl XPathResult {
    /// return multiple results
    pub fn as_nodes(&self) -> Vec<Node> {
        self.object
            .get_nodes_as_vec()
            .into_iter()
            .map(|node| Node { node })
            .collect_vec()
    }

    /// return multiple results as string
    pub fn as_strs(&self) -> Vec<String> {
        self.object.get_nodes_as_str()
    }

    /// return first result
    pub fn as_node(&self) -> Option<Node> {
        self.object
            .get_nodes_as_vec()
            .first()
            .map(|n| Node { node: n.to_owned() })
    }

    /// return first result as string
    pub fn as_str(&self) -> Option<String> {
        self.object.get_nodes_as_str().first().map(|s| s.to_owned())
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
        self.node
            .get_class_names()
            .into_iter()
            .filter(|c| !c.is_empty())
            .collect()
    }

    /// Returns the value of an attribute.
    pub fn attr(&self, attr: &str) -> Option<String> {
        self.node.get_attribute(attr)
    }

    /// Check if the attribute exists
    pub fn has_attr(&self, attr: &str) -> bool {
        self.node.has_attribute(attr)
    }

    /// Returns the text of this element.
    pub fn text(&self) -> String {
        self.node.get_content()
    }

    /// Returns the HTML of this element.
    pub fn html(&self) -> String {
        todo!()
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

    /// Find nodes based on this node using a relative xpath
    pub fn findnodes(&self, relative_xpath: &str) -> Result<Vec<Node>> {
        Ok(self
            .node
            .findnodes(relative_xpath)
            .map_err(|_| {
                ScraperError::XPathError(format!("relative xpath parse failed:{}", relative_xpath))
            })?
            .into_iter()
            .map(|node| Node { node })
            .collect_vec())
    }

    /// Find values based on this node using a relative xpath
    pub fn findvalues(&self, relative_xpath: &str) -> Result<Vec<String>> {
        self.node.findvalues(relative_xpath).map_err(|_| {
            ScraperError::XPathError(format!("relative xpath parse failed:{}", relative_xpath))
        })
    }

    /// Find first node based on this node using a relative xpath
    pub fn findnode(&self, relative_xpath: &str) -> Result<Node> {
        self.node
            .findnodes(relative_xpath)
            .map_err(|_| {
                ScraperError::XPathError(format!("relative xpath parse failed:{}", relative_xpath))
            })?
            .first()
            .map(|node| Node {
                node: node.to_owned(),
            })
            .ok_or_else(|| {
                ScraperError::XPathError(format!("relative xpath don't found:{}", relative_xpath))
            })
    }

    /// Find first value based on this node using a relative xpath
    pub fn findvalue(&self, relative_xpath: &str) -> Result<String> {
        match self
            .node
            .findvalues(relative_xpath)
            .map_err(|_| {
                ScraperError::XPathError(format!("relative xpath parse failed:{}", relative_xpath))
            })?
            .first()
        {
            Some(str) => Ok(str.to_owned()),
            None => Err(ScraperError::XPathError(format!(
                "relative xpath don't found:{}",
                relative_xpath
            ))),
        }
    }
}
