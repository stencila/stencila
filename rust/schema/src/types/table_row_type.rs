use crate::prelude::*;

/// Indicates whether the row is in the header, body or footer of the table.
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Defaults, Read, Write, ToHtml)]
#[serde(untagged, crate = "common::serde")]
#[def = "Body"]
pub enum TableRowType {
    Header,
    Body,
    Footer,
}
