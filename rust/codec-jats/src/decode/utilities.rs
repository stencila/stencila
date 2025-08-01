use codec::{Losses, common::itertools::Itertools};
use roxmltree::Node;

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

/// Split a string of author given names
///
/// In addition to splitting by whitespace, removes any trailing dot
/// (often on initials) and splits all caps (initials without separator).
pub(super) fn split_given_names(name: &str) -> Vec<String> {
    name.split_ascii_whitespace()
        .flat_map(|n| {
            let trimmed = n.trim_end_matches('.');
            if trimmed.chars().all(|c| c.is_ascii_uppercase()) && trimmed.chars().count() > 1 {
                trimmed
                    .chars()
                    .map(|c| c.to_string())
                    .collect::<Vec<String>>()
            } else {
                vec![trimmed.to_string()]
            }
        })
        .collect()
}
