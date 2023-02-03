//! Generated file, do not edit

use crate::prelude::*;

/// Indicates whether the cell is a header or data.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(untagged, crate = "common::serde")]
#[def = "Data"]
pub enum TableCellType {
    Data,
    Header,
}
