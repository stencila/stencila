// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// Indicates whether the row is in the header, body or footer of the table.
#[derive(Debug, Display, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, TextCodec, SmartDefault, Read, Write)]
#[serde(crate = "common::serde")]
pub enum TableRowType {
    Header,
    #[default]
    Body,
    Footer,
}
