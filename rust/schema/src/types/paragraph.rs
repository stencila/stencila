//! Generated file, do not edit

use crate::prelude::*;

use super::inline::Inline;
use super::string::String;

/// Paragraph
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Paragraph {
    /// The type of this item
    r#type: MustBe!("Paragraph"),

    /// The identifier for this item
    id: String,

    /// The contents of the paragraph.
    content: Vec<Inline>,
}
