// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::integer::Integer;
use super::string::String;

/// A schema specifying constraints on a string node.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "StringValidator")]
pub struct StringValidator {
    /// The type of this item.
    pub r#type: MustBe!("StringValidator"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The minimum length for a string node.
    #[serde(alias = "min-length", alias = "min_length")]
    pub min_length: Option<Integer>,

    /// The maximum length for a string node.
    #[serde(alias = "max-length", alias = "max_length")]
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
