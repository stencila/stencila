// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::boolean::Boolean;
use super::string::String;

/// A tag on code that affects its execution.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
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

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl ExecutionTag {
    const NICK: [u8; 3] = [101, 120, 116];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::ExecutionTag
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(name: String, value: String, is_global: Boolean) -> Self {
        Self {
            name,
            value,
            is_global,
            ..Default::default()
        }
    }
}
