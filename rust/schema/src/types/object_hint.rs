// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::hint::Hint;
use super::integer::Integer;
use super::string::String;

/// A hint to the structure of an `Object`.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "ObjectHint")]
pub struct ObjectHint {
    /// The type of this item.
    pub r#type: MustBe!("ObjectHint"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The length (number of entires) of the object.
    pub length: Integer,

    /// The keys of the object entries.
    #[serde(alias = "key")]
    #[serde(deserialize_with = "one_or_many")]
    pub keys: Vec<String>,

    /// The types of the object entries.
    #[serde(alias = "value")]
    #[serde(deserialize_with = "one_or_many")]
    pub values: Vec<Hint>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl ObjectHint {
    const NICK: &'static str = "obh";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::ObjectHint
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uid)
    }
    
    pub fn new(length: Integer, keys: Vec<String>, values: Vec<Hint>) -> Self {
        Self {
            length,
            keys,
            values,
            ..Default::default()
        }
    }
}
