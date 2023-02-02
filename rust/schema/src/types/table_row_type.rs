//! Generated file, do not edit

use crate::prelude::*;

/// Indicates whether the row is in the header, body or footer of the table.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
#[def = "Body"]
pub enum TableRowType {
    Header,
    Body,
    Footer,
}
