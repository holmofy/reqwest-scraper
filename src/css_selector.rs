//!  Select elements in HTML response using CSS selector
//!
use crate::error::Result;
use itertools::Itertools;
use scraper::ElementRef;

/// Html Response
pub struct Html {
    pub(crate) value: scraper::Html,
}

impl Html {
    /// Select elements in HTML using CSS selector
    pub fn select(&self, selector: &str) -> Result<Selectable<scraper::Html>> {
        Selectable::wrap(selector, &self.value)
    }
}

/// Wrapper object for HTML elements and CSS selectors
pub struct Selectable<'a, T> {
    selector: scraper::Selector,
    node: &'a T,
}

/// Iterator for selected elements in Html
pub struct HtmlSelectIterator<'a, 'b> {
    select: scraper::html::Select<'a, 'b>,
}

/// Iterator for selected elements in Element
pub struct ElementSelectIterator<'a, 'b> {
    select: scraper::element_ref::Select<'a, 'b>,
}

/// HTML elements selected by CSS selector
pub struct SelectItem<'a> {
    element: ElementRef<'a>,
}

impl<'a, T> Selectable<'a, T> {
    fn wrap(selector: &str, html: &'a T) -> Result<Selectable<'a, T>> {
        Ok(Self {
            selector: scraper::Selector::parse(selector)?,
            node: html,
        })
    }
}

/// Selectable Html Document
pub type SelectableHtml<'a> = Selectable<'a, scraper::Html>;

impl<'a> Selectable<'a, scraper::Html> {
    /// iterator
    pub fn iter(&self) -> HtmlSelectIterator {
        HtmlSelectIterator {
            select: self.node.select(&self.selector),
        }
    }

    /// first match item
    pub fn first(&self) -> Option<SelectItem> {
        self.iter().next()
    }
}

/// Selectable Html Element
pub type SelectableElement<'a> = Selectable<'a, ElementRef<'a>>;

impl<'a> Selectable<'a, ElementRef<'a>> {
    /// iterator
    pub fn iter(&self) -> ElementSelectIterator {
        ElementSelectIterator {
            select: self.node.select(&self.selector),
        }
    }

    /// first match item
    pub fn first(&self) -> Option<SelectItem> {
        self.iter().next()
    }
}

impl<'a, 'b> Iterator for HtmlSelectIterator<'a, 'b> {
    type Item = SelectItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.select.next().map(Self::Item::new)
    }
}

impl<'a, 'b> Iterator for ElementSelectIterator<'a, 'b> {
    type Item = SelectItem<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.select.next().map(Self::Item::new)
    }
}

/// Case Sensitivity Match
pub type CaseSensitivity = scraper::CaseSensitivity;
/// Html element class attribute
pub type Classes<'a> = scraper::node::Classes<'a>;
/// Html element attributes
pub type Attrs<'a> = scraper::node::Attrs<'a>;

impl<'a> SelectItem<'a> {
    /// constructor
    pub fn new(element: ElementRef<'a>) -> Self {
        Self { element }
    }

    /// Returns the element name.
    pub fn name(&self) -> &str {
        self.element.value().name()
    }

    /// Returns the element ID.
    pub fn id(&self) -> Option<&str> {
        self.element.value().id()
    }

    /// Returns true if element has the class.
    pub fn has_class(&self, class: &str, case_sensitive: CaseSensitivity) -> bool {
        self.element.value().has_class(class, case_sensitive)
    }

    /// Returns an iterator over the element's classes.
    pub fn classes(&self) -> Classes {
        self.element.value().classes()
    }

    /// Returns an iterator over the element's attributes.
    pub fn attrs(&self) -> Attrs {
        self.element.value().attrs()
    }

    /// Returns the value of an attribute.
    pub fn attr(&self, attr: &str) -> Option<&'a str> {
        self.element.attr(attr)
    }

    /// Returns the text of this element.
    pub fn text(&self) -> String {
        self.element.text().join("").trim().into()
    }

    /// Returns the HTML of this element.
    pub fn html(&self) -> String {
        self.element.html()
    }

    /// Returns the inner HTML of this element.
    pub fn inner_html(&self) -> String {
        self.element.inner_html()
    }

    /// Iterate over all child nodes which are elements
    pub fn children(&self) -> impl Iterator<Item = SelectItem<'a>> {
        self.element.child_elements().map(SelectItem::new)
    }

    /// Use CSS selector to find elements based on the current element
    pub fn select(&self, selector: &str) -> Result<Selectable<'a, ElementRef>> {
        Selectable::wrap(selector, &self.element)
    }
}
