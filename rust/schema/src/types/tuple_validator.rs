// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;
use super::validator::Validator;

/// A validator specifying constraints on an array of heterogeneous items.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "TupleValidator")]
pub struct TupleValidator {
    /// The type of this item.
    pub r#type: MustBe!("TupleValidator"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// An array of validators specifying the constraints on each successive item in the array.
    #[serde(alias = "item")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    pub items: Option<Vec<Validator>>,

    /// A universally unique identifier for this node
    
    #[serde(skip)]
    pub uuid: NodeUuid
}

impl TupleValidator {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl Entity for TupleValidator {
    const NICK: &'static str = "tup";

    fn node_type(&self) -> NodeType {
        NodeType::TupleValidator
    }

    fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uuid)
    }
}
