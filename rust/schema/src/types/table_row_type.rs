// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// Indicates whether the row is in the header, body or footer of the table.
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, SmartDefault, Strip, Read, Write, ToHtml, ToText)]
#[serde(crate = "common::serde")]
pub enum TableRowType {
    Header,
    #[default]
    Body,
    Footer,
}
