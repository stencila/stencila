// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;

/// A schema specifying that a node must be a boolean value.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
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

    /// A universally unique identifier for this node
    
    #[serde(skip)]
    pub uuid: NodeUuid
}

impl BooleanValidator {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl Entity for BooleanValidator {
    const NICK: &'static str = "boo";

    fn node_type(&self) -> NodeType {
        NodeType::BooleanValidator
    }

    fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uuid)
    }
}
