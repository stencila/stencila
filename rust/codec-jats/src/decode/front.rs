use roxmltree::Node;

use codec::{schema::Article, Losses};

use super::utilities::record_node_lost;

/// Decode the `<front>` of an `<article>`
pub(super) fn decode_front(path: &str, node: &Node, _article: &mut Article, losses: &mut Losses) {
    for child in node.children() {
        let _tag = child.tag_name().name();
        {
            record_node_lost(path, &child, losses);
            continue;
        };
    }
}
