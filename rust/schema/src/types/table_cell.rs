// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::integer::Integer;
use super::string::String;
use super::table_cell_type::TableCellType;

/// A cell within a `Table`.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "TableCell")]
#[html(elem = "td")]
pub struct TableCell {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("TableCell"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The type of cell.
    #[serde(alias = "cell-type", alias = "cell_type")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub cell_type: Option<TableCellType>,

    /// Contents of the table cell.
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    #[patch(format = "all")]
    #[cfg_attr(feature = "proptest-min", proptest(strategy = r#"vec_paragraphs(1)"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"vec_paragraphs(1)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"vec_paragraphs(1)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"vec_paragraphs(1)"#))]
    #[dom(elem = "td")]
    pub content: Vec<Block>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<TableCellOptions>,

    /// A unique identifier for a node within a document
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub struct TableCellOptions {
    /// The name of the cell.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub name: Option<String>,

    /// How many columns the cell extends.
    #[serde(alias = "column-span", alias = "column_span")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "colspan")]
    pub column_span: Option<Integer>,

    /// How many columns the cell extends.
    #[serde(alias = "row-span", alias = "row_span")]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "rowspan")]
    pub row_span: Option<Integer>,
}

impl TableCell {
    const NICK: [u8; 3] = [116, 98, 99];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::TableCell
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(content: Vec<Block>) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }
}
