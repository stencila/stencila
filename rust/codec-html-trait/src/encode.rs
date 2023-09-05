//! Helper functions for encoding to HTML

use html_escape::{encode_double_quoted_attribute, encode_safe};

use common::{inflector::Inflector, itertools::Itertools};

/// Generate a HTML attribute
///
/// Ensures the attribute name is kebab case. Removes any superfluous outer quotes
/// from the values and does necessary escaping of the attribute value.
pub fn attr(name: &str, value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }

    let name = name.to_kebab_case();

    let value = encode_double_quoted_attribute(value.trim_matches('"'));

    [&name, "=\"", &value, "\""].concat()
}

/// Generate escaped HTML text
pub fn text(value: &str) -> String {
    encode_safe(value).to_string()
}

/// Generate a HTML element with given name, attributes and children
pub fn elem(name: &str, attrs: &[String], children: &[String]) -> String {
    if name.is_empty() {
        return String::new()
    }
    
    let attrs = attrs
        .iter()
        .filter(|attr| !attr.trim().is_empty())
        .join(" ");

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
