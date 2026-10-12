// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The scope of a graph limitation.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, InspectNode, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, Copy, EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[strum(ascii_case_insensitive)]
pub enum GraphGapTargetKind {
    /// The containing item or account as a whole.
    #[default]
    Item,

    /// The named property path.
    Property,

    /// The named edge kind and direction, optionally narrowed by a role.
    Relationship,
}
