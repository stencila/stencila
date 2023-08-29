// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// Indicates whether the cell is a header or data.
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, SmartDefault, Strip, Read, Write, ToHtml, ToText)]
#[serde(crate = "common::serde")]
pub enum TableCellType {
    #[default]
    Data,
    Header,
}
