// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;

/// Represents additional information about a relationship or property.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "Role")]
pub struct Role {
    /// The type of this item.
    pub r#type: MustBe!("Role"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,
}

impl Role {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
