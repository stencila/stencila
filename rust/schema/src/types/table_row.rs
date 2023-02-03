//! Generated file, do not edit

use crate::prelude::*;

use super::string::String;
use super::table_cell::TableCell;
use super::table_row_type::TableRowType;

/// A row within a Table.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct TableRow {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    pub r#type: MustBe!("TableRow"),

    /// The identifier for this item
    #[key]
    pub id: Option<String>,

    /// An array of cells in the row.
    pub cells: Vec<TableCell>,

    /// Non-core optional fields
    #[serde(flatten)]
    pub options: Box<TableRowOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct TableRowOptions {
    /// The type of row.
    pub row_type: Option<TableRowType>,
}
