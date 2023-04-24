use crate::prelude::*;

/// Indicates whether the cell is a header or data.
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Defaults, Strip, Read, Write, ToHtml)]
#[serde(crate = "common::serde")]
#[def = "Data"]
pub enum TableCellType {
    Data,
    Header,
}
