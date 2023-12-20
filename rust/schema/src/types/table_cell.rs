// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::integer::Integer;
use super::string::String;
use super::table_cell_type::TableCellType;

/// A cell within a `Table`.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
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
    #[cfg_attr(feature = "proptest-min", proptest(strategy = r#"vec_paragraphs(1)"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"vec_paragraphs(1)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"vec_paragraphs(1)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"vec_paragraphs(1)"#))]
    pub content: Vec<Block>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<TableCellOptions>,

    /// A universally unique identifier for this node
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[serde(skip)]
    pub uuid: NodeUuid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
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
    pub fn new(content: Vec<Block>) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }
}

impl Entity for TableCell {
    const NICK: &'static str = "tab";

    fn node_type(&self) -> NodeType {
        NodeType::TableCell
    }

    fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uuid)
    }
}
