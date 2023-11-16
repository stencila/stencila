// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::node::Node;
use super::string::String;

/// A schema specifying that a node must be one of several values.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "EnumValidator")]
pub struct EnumValidator {
    /// The type of this item.
    pub r#type: MustBe!("EnumValidator"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// A node is valid if it is equal to any of these values.
    #[serde(alias = "value")]
    pub values: Vec<Node>,
}

impl EnumValidator {
    pub fn new(values: Vec<Node>) -> Self {
        Self {
            values,
            ..Default::default()
        }
    }
}
