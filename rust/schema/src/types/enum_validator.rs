// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::node::Node;
use super::string::String;

/// A schema specifying that a node must be one of several values.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
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

    /// A universally unique identifier for this node
    
    #[serde(skip)]
    pub uuid: NodeUuid
}

impl EnumValidator {
    pub fn new(values: Vec<Node>) -> Self {
        Self {
            values,
            ..Default::default()
        }
    }
}

impl Entity for EnumValidator {
    const NICK: &'static str = "enu";

    fn node_type(&self) -> NodeType {
        NodeType::EnumValidator
    }

    fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uuid)
    }
}
