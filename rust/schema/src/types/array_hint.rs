// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::integer::Integer;
use super::primitive::Primitive;
use super::string::String;

/// A hint to the content of an `Array`.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "ArrayHint")]
pub struct ArrayHint {
    /// The type of this item.
    pub r#type: MustBe!("ArrayHint"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The length (number of items) of the array.
    pub length: Integer,

    /// The distinct types of the array items.
    #[serde(alias = "item-types", alias = "item_types", alias = "itemType", alias = "item-type", alias = "item_type")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    pub item_types: Option<Vec<String>>,

    /// The minimum value in the array.
    pub minimum: Option<Primitive>,

    /// The maximum value in the array.
    pub maximum: Option<Primitive>,

    /// The number of `Null` values in the array.
    pub nulls: Option<Integer>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl ArrayHint {
    const NICK: &'static str = "arh";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::ArrayHint
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uid)
    }
    
    pub fn new(length: Integer) -> Self {
        Self {
            length,
            ..Default::default()
        }
    }
}
