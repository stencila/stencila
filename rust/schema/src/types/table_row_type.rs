// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// Indicates whether the row is in the header, body or footer of the table.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, SmartDefault, strum::EnumString, ReadNode)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub enum TableRowType {
    HeaderRow,

    #[default]
    BodyRow,

    FooterRow,
}
