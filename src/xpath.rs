//!  Evalute the value in HTML response using xpath expression
//!
use crate::error::{Result, ScraperError};
use itertools::Itertools;
use libxml::{
    tree::Document,
    xpath::{Context, Object},
};
use std::collections::HashSet;

/// Html Response
#[derive(Clone)]
pub struct XHtml {
    doc: Document,
}

/// Wrap HTML document and compiled xpath
pub struct XPathResult {
    object: Object,
}

impl XHtml {
    /// constructor
    pub fn new<S: Into<String>>(html_str: S) -> Result<Self> {
        let parser = libxml::parser::Parser::default_html();
        let doc = parser.parse_string(html_str.into())?;
        Ok(Self { doc })
    }
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
            .map(Node::new)
            .collect::<Vec<_>>()
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
            .map(|n| Node::new(n.to_owned()))
    }

    /// return first result as string
    pub fn as_str(&self) -> Option<String> {
        self.object
            .get_nodes_as_str()
            .first()
            .map(ToOwned::to_owned)
    }
}

impl Node {
    /// constructor
    pub fn new(node: libxml::tree::node::Node) -> Self {
        Self { node }
    }

    /// Returns the element name.
    pub fn name(&self) -> String {
        self.node.get_name()
    }

    /// Returns the element ID.
    pub fn id(&self) -> Option<String> {
        self.node.get_attribute("id").map(|s| s.trim().into())
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
        self.node.get_attribute(attr).map(|s| s.trim().into())
    }

    /// Check if the attribute exists
    pub fn has_attr(&self, attr: &str) -> bool {
        self.node.has_attribute(attr)
    }

    /// Returns the text of this element.
    pub fn text(&self) -> String {
        self.node.get_content().trim().into()
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
            .map(Node::new)
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
            .map(Node::new)
            .collect_vec())
    }

    /// Find values based on this node using a relative xpath
    pub fn findvalues(&self, relative_xpath: &str) -> Result<Vec<String>> {
        match self.node.findvalues(relative_xpath) {
            Ok(vec) => Ok(vec.into_iter().map(|s| s.trim().to_string()).collect_vec()),
            Err(_) => Err(ScraperError::XPathError(format!(
                "relative xpath parse failed:{}",
                relative_xpath
            ))),
        }
    }

    /// Find first node based on this node using a relative xpath
    pub fn findnode(&self, relative_xpath: &str) -> Result<Option<Node>> {
        Ok(self
            .node
            .findnodes(relative_xpath)
            .map_err(|_| {
                ScraperError::XPathError(format!("relative xpath parse failed:{}", relative_xpath))
            })?
            .first()
            .map(|node| Node::new(node.to_owned())))
    }

    /// Find first value based on this node using a relative xpath
    pub fn findvalue(&self, relative_xpath: &str) -> Result<Option<String>> {
        Ok(self
            .node
            .findvalues(relative_xpath)
            .map_err(|_| {
                ScraperError::XPathError(format!("relative xpath parse failed:{}", relative_xpath))
            })?
            .first()
            .map(|v| v.trim().into()))
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_select_xpath() {
        use super::*;
        let html_str = r#"
        <html>
            <body>
                <div id="content">
                    <p>Hello, World!</p>
                    <p>This is a test.</p>
                </div>
            </body>
        </html>
        "#;

        let xhtml = XHtml::new(html_str).expect("parse xhtml failed");

        let content = xhtml.select("//div[@id='content']").ok();
        assert!(content.is_some());

        let content = content.unwrap().as_node();
        assert!(content.is_some());

        let content = content.unwrap();
        assert_eq!(content.attr("id").unwrap(), "content");
        let children = content.children();
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].text(), "Hello, World!");
        assert_eq!(children[1].text(), "This is a test.");

        let p1 = content.findvalue("./p[position()=1]").unwrap();
        assert!(p1.is_some());
        assert_eq!(p1.unwrap(), "Hello, World!");
    }
}
