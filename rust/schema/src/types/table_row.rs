// Generated file. Do not edit; see `schema-gen` crate.

use crate::prelude::*;

use super::string::String;
use super::table_cell::TableCell;
use super::table_row_type::TableRowType;

/// A row within a Table.
#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct TableRow {
    /// The type of this item
    pub r#type: MustBe!("TableRow"),

    /// The identifier for this item
    pub id: Option<String>,

    /// An array of cells in the row.
    pub cells: Vec<TableCell>,

    /// Non-core optional fields
    #[serde(flatten)]
    pub options: Box<TableRowOptions>,
}

#[skip_serializing_none]
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct TableRowOptions {
    /// The type of row.
    pub row_type: Option<TableRowType>,
}

impl TableRow {
    #[rustfmt::skip]
    pub fn new(cells: Vec<TableCell>) -> Self {
        Self {
            cells,
            ..Default::default()
        }
    }
}
