// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::inline::Inline;
use super::string::String;

/// Emphasized content.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, HtmlCodec, TextCodec, StripNode, Read, Write)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Emphasis {
    /// The type of this item
    pub r#type: MustBe!("Emphasis"),

    /// The identifier for this item
    #[strip(id)]
    pub id: Option<String>,

    /// The content that is marked.
    pub content: Vec<Inline>,
}
impl Emphasis {
    pub fn new(content: Vec<Inline>) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }
}
