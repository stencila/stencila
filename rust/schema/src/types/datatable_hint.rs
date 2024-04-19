// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::datatable_column_hint::DatatableColumnHint;
use super::integer::Integer;
use super::string::String;

/// A hint to the structure of a table of data.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "DatatableHint")]
pub struct DatatableHint {
    /// The type of this item.
    pub r#type: MustBe!("DatatableHint"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The number of rows of data.
    pub rows: Integer,

    /// A hint for each column of data.
    #[serde(alias = "column")]
    #[serde(deserialize_with = "one_or_many")]
    pub columns: Vec<DatatableColumnHint>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

impl DatatableHint {
    const NICK: [u8; 3] = [100, 116, 104];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::DatatableHint
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(rows: Integer, columns: Vec<DatatableColumnHint>) -> Self {
        Self {
            rows,
            columns,
            ..Default::default()
        }
    }
}
