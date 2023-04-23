use crate::prelude::*;

/// Indicates whether the row is in the header, body or footer of the table.
#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Defaults, Strip, Read, Write, ToHtml)]
#[serde(crate = "common::serde")]
#[def = "Body"]
pub enum TableRowType {
    Header,
    Body,
    Footer,
}
