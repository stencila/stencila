//! Functions for transforming nodes between types
//!
//! These functions implement transformations for `Vec` and other types
//! where, due to Rust's "orphan rule", we can not implement the `From`
//! trait. If possible, prefer to implement `From`, or `Into` in the `implem/*.rs`
//! file for the type, rather than add another function here.

use crate::{Block, Inline};

/// Transform a vector of [`Block`]s into a vector of [`Inline`]s
pub fn blocks_to_inlines(blocks: Vec<Block>) -> Vec<Inline> {
    blocks
        .into_iter()
        .flat_map(|block| -> Vec<Inline> { block.into() })
        .collect()
}
