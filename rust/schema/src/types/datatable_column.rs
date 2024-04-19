// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::array_validator::ArrayValidator;
use super::primitive::Primitive;
use super::string::String;

/// A column of data within a `Datatable`.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "DatatableColumn")]
pub struct DatatableColumn {
    /// The type of this item.
    pub r#type: MustBe!("DatatableColumn"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The name of the column.
    pub name: String,

    /// The data values of the column.
    #[serde(alias = "value")]
    #[serde(default)]
    pub values: Vec<Primitive>,

    /// The validator to use to validate data in the column.
    pub validator: Option<ArrayValidator>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl DatatableColumn {
    const NICK: [u8; 3] = [100, 116, 99];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::DatatableColumn
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(name: String, values: Vec<Primitive>) -> Self {
        Self {
            name,
            values,
            ..Default::default()
        }
    }
}
