// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::string::String;
use super::table_cell::TableCell;
use super::table_row_type::TableRowType;

/// A row within a Table.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "TableRow")]
#[html(elem = "tr")]
pub struct TableRow {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("TableRow"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// An array of cells in the row.
    #[serde(alias = "cell")]
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    #[patch(format = "md", format = "smd", format = "myst")]
    #[cfg_attr(feature = "proptest-min", proptest(strategy = r#"vec(TableCell::arbitrary(), size_range(1..=1))"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"vec(TableCell::arbitrary(), size_range(2..=2))"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"vec(TableCell::arbitrary(), size_range(4..=4))"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"vec(TableCell::arbitrary(), size_range(1..=8))"#))]
    pub cells: Vec<TableCell>,

    /// The type of row.
    #[serde(alias = "row-type", alias = "row_type")]
    #[patch(format = "md", format = "smd", format = "myst")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub row_type: Option<TableRowType>,

    /// A unique identifier for a node within a document
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[serde(skip)]
    pub uid: NodeUid
}

impl TableRow {
    const NICK: [u8; 3] = [116, 98, 114];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::TableRow
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(cells: Vec<TableCell>) -> Self {
        Self {
            cells,
            ..Default::default()
        }
    }
}
