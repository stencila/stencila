//! Generated file, do not edit

use crate::prelude::*;

use super::inline::Inline;
use super::string::String;

/// Inline text that is underlined.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Underline {
    /// The type of this item
    r#type: MustBe!("Underline"),

    /// The identifier for this item
    id: String,

    /// The content that is marked.
    content: Vec<Inline>,
}
