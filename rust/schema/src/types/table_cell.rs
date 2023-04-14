// Generated file. Do not edit; see `schema-gen` crate.

use crate::prelude::*;

use super::blocks_or_inlines::BlocksOrInlines;
use super::integer::Integer;
use super::string::String;
use super::table_cell_type::TableCellType;

/// A cell within a `Table`.
#[rustfmt::skip]
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Strip, Read, Write, ToHtml)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct TableCell {
    /// The type of this item
    pub r#type: MustBe!("TableCell"),

    /// The identifier for this item
    pub id: Option<String>,

    /// Non-core optional fields
    #[serde(flatten)]
    pub options: Box<TableCellOptions>,
}

#[rustfmt::skip]
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Strip, Read, Write, ToHtml)]
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
    #[rustfmt::skip]
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
