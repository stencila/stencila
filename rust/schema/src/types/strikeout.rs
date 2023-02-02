//! Generated file, do not edit

use crate::prelude::*;

use super::inline::Inline;

/// Content that is marked as struck out
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Strikeout {
    /// The content that is marked.
    content: Vec<Inline>,
}
