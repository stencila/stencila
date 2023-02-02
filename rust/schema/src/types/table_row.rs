//! Generated file, do not edit

use crate::prelude::*;

use super::string::String;
use super::table_cell::TableCell;
use super::table_row_type::TableRowType;

/// A row within a Table.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct TableRow {
    /// The type of this item
    r#type: MustBe!("TableRow"),

    /// The identifier for this item
    id: String,

    /// An array of cells in the row.
    cells: Vec<TableCell>,

    /// Non-core optional fields
    #[serde(flatten)]
    options: Box<TableRowOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct TableRowOptions {
    /// The type of row.
    row_type: Option<TableRowType>,
}
