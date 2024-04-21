// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;
use super::validator::Validator;

/// A validator specifying constraints on an array of heterogeneous items.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, TextCodec)]
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
    #[patch(format = "md")]
    pub items: Option<Vec<Validator>>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl TupleValidator {
    const NICK: [u8; 3] = [116, 117, 118];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::TupleValidator
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
