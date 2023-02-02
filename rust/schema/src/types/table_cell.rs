//! Generated file, do not edit

use crate::prelude::*;

use super::blocks_or_inlines::BlocksOrInlines;
use super::integer::Integer;
use super::string::String;
use super::table_cell_type::TableCellType;

/// A cell within a `Table`.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct TableCell {
    /// The type of this item
    r#type: MustBe!("TableCell"),

    /// The identifier for this item
    id: String,

    /// Non-core optional fields
    #[serde(flatten)]
    options: Box<TableCellOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct TableCellOptions {
    /// The name of the cell.
    name: Option<String>,

    /// How many columns the cell extends.
    colspan: Option<Integer>,

    /// The type of cell.
    cell_type: Option<TableCellType>,

    /// How many columns the cell extends.
    rowspan: Option<Integer>,

    /// Contents of the table cell.
    content: Option<BlocksOrInlines>,
}
