// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// Indicates whether the row is in the header, body or footer of the table.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, Copy, EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[strum(ascii_case_insensitive)]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub enum TableRowType {
    HeaderRow,

    #[default]
    BodyRow,

    FooterRow,
}
