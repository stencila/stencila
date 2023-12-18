// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::boolean::Boolean;
use super::string::String;

/// A tag on code that affects its execution.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "ExecutionTag")]
pub struct ExecutionTag {
    /// The type of this item.
    pub r#type: MustBe!("ExecutionTag"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The name of the tag
    pub name: String,

    /// The value of the tag
    pub value: String,

    /// Whether the tag is global to the document
    #[serde(alias = "is-global", alias = "is_global")]
    pub is_global: Boolean,

    /// A unique identifier for this node
    
    #[serde(skip)]
    pub node_id: NodeId
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

impl Entity for ExecutionTag {
    fn node_type() -> NodeType {
        NodeType::ExecutionTag
    }

    fn node_id(&self) -> &NodeId {
        &self.node_id
    }
}
