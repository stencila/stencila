use itertools::Itertools;
use roxmltree::Node;

use stencila_codec::{Losses, stencila_schema::IntegerOrString};

/// The XLink namespace, used for `xlink:href` and related attributes
pub(super) const XLINK: &str = "http://www.w3.org/1999/xlink";

/// Extend a path with a new child tag name
pub(super) fn extend_path(path: &str, tag: &str) -> String {
    [path, "/", tag].concat()
}

/// Record the attributes of a XML node that are lost when decoding from JATS
///
/// Pass the names of the of the attributes (not namespaced) that _are_
/// decoded in the `not_lost` parameter.
pub(super) fn record_attrs_lost<'lt, I>(path: &str, node: &Node, not_lost: I, losses: &mut Losses)
where
    I: IntoIterator<Item = &'lt str>,
{
    let not_lost = not_lost.into_iter().collect_vec();
    for attribute in node.attributes() {
        let name = attribute.name();
        if !not_lost.contains(&name) {
            losses.add(format!("{path}/@{name}"));
        }
    }
}

/// Record that a whole XML node was lost when decoding from JATS
pub(super) fn record_node_lost(path: &str, node: &Node, losses: &mut Losses) {
    if node.is_element() {
        losses.add(extend_path(path, node.tag_name().name()))
    } else if node.is_text() {
        // Ignore loss of whitespace only text which can arise in indented JATS
        if !node
            .text()
            .map(|text| text.trim().is_empty())
            .unwrap_or(true)
        {
            losses.add(extend_path(path, "text()"))
        }
    }
}

/// Get the trimmed text of the text nodes that are direct children of an
/// element, if it has any
pub(super) fn non_empty_text(node: &Node) -> Option<String> {
    let text = node
        .children()
        .filter(Node::is_text)
        .filter_map(|child| child.text())
        .collect::<String>();
    let text = text.trim();
    (!text.is_empty()).then(|| text.to_string())
}

/// Get the text of an element and all of its descendants, with whitespace
/// collapsed
///
/// Text nodes are concatenated before whitespace is normalized so that inline
/// markup within a word (e.g. `<italic>Guideline</italic>s`) does not split it.
pub(super) fn non_empty_text_deep(node: &Node) -> Option<String> {
    let text = node
        .descendants()
        .filter(Node::is_text)
        .filter_map(|descendant| descendant.text())
        .collect::<String>();
    let text = text.split_whitespace().join(" ");
    (!text.is_empty()).then_some(text)
}

/// Convert the text of an element into an integer or a string
///
/// Only uses an integer when doing so does not change the value, so that a
/// value such as the page "056708" keeps its leading zero.
pub(super) fn integer_or_string(value: &str) -> IntegerOrString {
    match IntegerOrString::from(value) {
        IntegerOrString::Integer(integer) if integer.to_string() == value => {
            IntegerOrString::Integer(integer)
        }
        _ => IntegerOrString::String(value.to_string()),
    }
}
