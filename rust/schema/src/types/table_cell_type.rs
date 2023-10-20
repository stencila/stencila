// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// Indicates whether the cell is a header or data.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, SmartDefault, ReadNode)]
#[serde(crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub enum TableCellType {
    #[default]
    Data,

    Header,
}
