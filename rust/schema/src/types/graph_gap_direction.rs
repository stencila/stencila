// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The direction of a relationship qualified by a gap.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, InspectNode, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, Copy, EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[strum(ascii_case_insensitive)]
pub enum GraphGapDirection {
    /// A relationship into the containing item.
    #[default]
    Incoming,

    /// A relationship from the containing item.
    Outgoing,
}
