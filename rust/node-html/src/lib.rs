//! Provides the `ToHtml` trait for generating  HTML for Stencila Schema nodes
//!
//! Note that this trait can not be in the `codec-html` crate (like, for examples, the
//! `ToJSon` trait is in the `codec-json` crate) because `ToHtml` is required by
//! the `schema` crate, which is itself a dependency of the `codec` crate (i.e. it woulf
//! create a circular dependency).

use html_escape::{encode_double_quoted_attribute, encode_safe};

use common::inflector::Inflector;

mod prelude;

mod boolean;
mod r#box;
mod integer;
mod number;
mod option;
mod string;
mod unsigned_integer;
mod vec;

pub trait ToHtml {
    /// Generate an HTML representation of the document node
    fn to_html(&self) -> String;
}

/// Generate a name for a HTML custom element for a Stencila Schema node type
///
/// Generates a kebab cased custom element name prefixed with `stencila-`
pub fn name(typ: &str) -> String {
    ["stencila-", &typ.to_kebab_case()].concat()
}

/// Generate a HTML attribute
///
/// Ensures the attribute name is kebab case and does necessary escaping of the attribute value
pub fn attr(name: &str, value: &str) -> String {
    [
        &name.to_kebab_case(),
        "=\"",
        &encode_double_quoted_attribute(value),
        "\"",
    ]
    .concat()
}

/// Generate a HTML attribute if an option has a value
pub fn attr_maybe<T>(name: &str, value: &Option<T>) -> String
where
    T: ToString,
{
    match value {
        Some(value) => attr(name, &value.to_string()),
        None => String::new(),
    }
}

/// Generate escaped HTML text
pub fn text(value: &str) -> String {
    encode_safe(value).to_string()
}

/// Generate a HTML element with given name, attributes and children
pub fn elem(name: &str, attrs: &[String], children: &[String]) -> String {
    let attrs = attrs.join(" ");
    let children = children.join("");

    [
        "<",
        name,
        if attrs.is_empty() { "" } else { " " },
        &attrs,
        ">",
        &children,
        "</",
        name,
        ">",
    ]
    .concat()
}
