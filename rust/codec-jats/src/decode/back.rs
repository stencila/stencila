use roxmltree::Node;

use codec::{schema::Article, Losses};

/// Decode the `<back>` of an `<article>`
pub(super) fn decode_back(
    _parent: &str,
    _node: &Node,
    _article: &mut Article,
    _losses: &mut Losses,
) {
}
