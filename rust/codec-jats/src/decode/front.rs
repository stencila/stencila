use roxmltree::Node;

use codec::{schema::Article, Losses};

/// Decode the `<front>` of an `<article>`
pub(super) fn front(_node: &Node, _article: &mut Article, _losses: &mut Losses) {}
