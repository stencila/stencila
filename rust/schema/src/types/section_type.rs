// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The type of a `Section`.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, Copy, strum::EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub enum SectionType {
    #[default]
    Abstract,

    Summary,

    Introduction,

    Materials,

    Methods,

    Cases,

    Results,

    Discussion,

    Conclusions,

    SupplementaryMaterials,

    Main,

    Header,

    Footer,

    /// A section representing an iteration of a `ForBlock`.
    Iteration,

    /// A section representing an island of island content.
    Island,
}
