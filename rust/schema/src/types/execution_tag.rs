// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::boolean::Boolean;
use super::string::String;

/// A tag on code that affects its execution
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct ExecutionTag {
    /// The type of this item
    pub r#type: MustBe!("ExecutionTag"),

    /// The identifier for this item
    #[strip(id)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The name of the tag
    pub name: String,

    /// The value of the tag
    pub value: String,

    /// Whether the tag is global to the document
    pub is_global: Boolean,
}

impl ExecutionTag {
    pub fn new(name: String, value: String, is_global: Boolean) -> Self {
        Self {
            name,
            value,
            is_global,
            ..Default::default()
        }
    }
}
