use codec::{common::itertools::Itertools, Losses};
use roxmltree::Node;

/// Extend a path with a new child tag name
pub(super) fn extend_path(path: &str, tag: &str) -> String {
    [path, "/", tag].concat()
}

/// Record the attributes of a node that are lost when encoding to JATS
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

/// Record that a whole node was lost
pub(super) fn record_node_lost(path: &str, node: &Node, losses: &mut Losses) {
    if node.is_element() {
        losses.add(path)
    } else if node.is_text() {
        losses.add(format!("{path}/text()"))
    }
}
