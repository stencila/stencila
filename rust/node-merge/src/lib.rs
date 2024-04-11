//! A crate for diffing, patching and merging (diffing then patching) nodes
//!
//! This crate duplicates some functionality that is already in the sibling
//! `node-patch` and `node-map` crates. It is anticipated that this crate
//! will replace at least one of those.

mod diff;
mod merge;
mod patch;

pub use diff::{diff, DiffResult};
pub use merge::merge;
pub use patch::patch;
