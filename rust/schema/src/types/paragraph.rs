// Generated file. Do not edit; see `schema-gen` crate.

use crate::prelude::*;

use super::inline::Inline;
use super::string::String;

/// Paragraph
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Read, Write)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Paragraph {
    /// The type of this item
    pub r#type: MustBe!("Paragraph"),

    /// The identifier for this item
    pub id: Option<String>,

    /// The contents of the paragraph.
    pub content: Vec<Inline>,
}

impl Paragraph {
    #[rustfmt::skip]
    pub fn new(content: Vec<Inline>) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }
}
