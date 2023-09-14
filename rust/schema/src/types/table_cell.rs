// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::blocks_or_inlines::BlocksOrInlines;
use super::integer::Integer;
use super::string::String;
use super::table_cell_type::TableCellType;

/// A cell within a `Table`.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[html(elem = "td")]
pub struct TableCell {
    /// The type of this item
    pub r#type: MustBe!("TableCell"),

    /// The identifier for this item
    #[strip(id)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    pub options: Box<TableCellOptions>,
}

#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct TableCellOptions {
    /// The name of the cell.
    pub name: Option<String>,

    /// How many columns the cell extends.
    pub colspan: Option<Integer>,

    /// The type of cell.
    pub cell_type: Option<TableCellType>,

    /// How many columns the cell extends.
    pub rowspan: Option<Integer>,

    /// Contents of the table cell.
    pub content: Option<BlocksOrInlines>,
}

impl TableCell {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
