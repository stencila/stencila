use roxmltree::Node;

use codec::{schema::Article, Losses};

/// Decode the `<front>` of an `<article>`
pub(super) fn decode_front(_node: &Node, _article: &mut Article, _losses: &mut Losses) {}
