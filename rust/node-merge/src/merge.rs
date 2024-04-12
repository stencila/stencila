use common::eyre::Result;
use schema::Node;

use crate::{diff, patch};

/// Merge `new` node into `old` node
///
/// This function simply combines a call to [`diff`] with
/// a call to `patch`.
pub fn merge(old: &mut Node, new: &Node) -> Result<()> {
    let ops = diff(old, new).ops();
    patch(old, new, ops)
}
