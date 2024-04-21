// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::node::Node;
use super::string::String;

/// A validator specifying a constant value that a node must have.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
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
    #[patch(format = "md")]
    pub value: Box<Node>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl ConstantValidator {
    const NICK: [u8; 3] = [99, 111, 118];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::ConstantValidator
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(value: Box<Node>) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }
}
