// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::integer::Integer;
use super::string::String;

/// A schema specifying constraints on a string node.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct StringValidator {
    /// The type of this item.
    pub r#type: MustBe!("StringValidator"),

    /// The identifier for this item.
    #[strip(id)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The minimum length for a string node.
    pub min_length: Option<Integer>,

    /// The maximum length for a string node.
    pub max_length: Option<Integer>,

    /// A regular expression that a string node must match.
    pub pattern: Option<String>,
}

impl StringValidator {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
