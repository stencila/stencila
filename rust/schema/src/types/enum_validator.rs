// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::node::Node;
use super::string::String;

/// A schema specifying that a node must be one of several values.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
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
    #[serde(default)]
    #[patch(format = "md", format = "smd", format = "myst")]
    pub values: Vec<Node>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl EnumValidator {
    const NICK: [u8; 3] = [101, 110, 118];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::EnumValidator
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(values: Vec<Node>) -> Self {
        Self {
            values,
            ..Default::default()
        }
    }
}
