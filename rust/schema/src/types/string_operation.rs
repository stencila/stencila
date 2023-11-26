// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;
use super::unsigned_integer::UnsignedInteger;

/// An operation that modifies a string.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "StringOperation")]
pub struct StringOperation {
    /// The type of this item.
    pub r#type: MustBe!("StringOperation"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The start position in the string of the operation.
    #[serde(alias = "start-position", alias = "start_position")]
    pub start_position: UnsignedInteger,

    /// The end position in the string of the operation.
    #[serde(alias = "end-position", alias = "end_position")]
    pub end_position: Option<UnsignedInteger>,

    /// The string value to insert or use as the replacement.
    pub value: Option<String>,
}

impl StringOperation {
    pub fn new(start_position: UnsignedInteger) -> Self {
        Self {
            start_position,
            ..Default::default()
        }
    }
}
