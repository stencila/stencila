// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::integer::Integer;
use super::primitive::Primitive;
use super::string::String;

/// A hint to the type and values in a `DatatableColumn`.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "DatatableColumnHint")]
pub struct DatatableColumnHint {
    /// The type of this item.
    pub r#type: MustBe!("DatatableColumnHint"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The name of the column.
    pub name: String,

    /// The type of items in the column.
    #[serde(alias = "item-type", alias = "item_type")]
    pub item_type: String,

    /// The minimum value in the column.
    pub minimum: Option<Primitive>,

    /// The maximum value in the column.
    pub maximum: Option<Primitive>,

    /// The number of `Null` values in the column.
    pub nulls: Option<Integer>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl DatatableColumnHint {
    const NICK: &'static str = "dch";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::DatatableColumnHint
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uid)
    }
    
    pub fn new(name: String, item_type: String) -> Self {
        Self {
            name,
            item_type,
            ..Default::default()
        }
    }
}
