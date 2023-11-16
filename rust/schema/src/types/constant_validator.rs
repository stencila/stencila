// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::node::Node;
use super::string::String;

/// A validator specifying a constant value that a node must have.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "ConstantValidator")]
pub struct ConstantValidator {
    /// The type of this item.
    pub r#type: MustBe!("ConstantValidator"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The value that the node must have.
    pub value: Box<Node>,
}

impl ConstantValidator {
    pub fn new(value: Box<Node>) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }
}
