// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;

/// A schema specifying that a node must be a boolean value.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "BooleanValidator")]
pub struct BooleanValidator {
    /// The type of this item.
    pub r#type: MustBe!("BooleanValidator"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// A unique identifier for this node
    
    #[serde(skip)]
    pub node_id: NodeId
}

impl BooleanValidator {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl Entity for BooleanValidator {
    fn node_type() -> NodeType {
        NodeType::BooleanValidator
    }

    fn node_id(&self) -> &NodeId {
        &self.node_id
    }
}
