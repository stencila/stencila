// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The vertical alignment of content.
#[derive(
    Debug,
    strum::Display,
    Clone,
    PartialEq,
    Serialize,
    Deserialize,
    StripNode,
    WalkNode,
    WriteNode,
    SmartDefault,
    strum::EnumString,
    Eq,
    PartialOrd,
    Ord,
    ReadNode,
    PatchNode,
    DomCodec,
    HtmlCodec,
    JatsCodec,
    MarkdownCodec,
    TextCodec,
)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub enum VerticalAlignment {
    /// Aligned to a baseline.
    #[default]
    #[strum(serialize = "baseline")]
    AlignBaseline,

    /// Aligned with bottom.
    #[strum(serialize = "bottom")]
    AlignBottom,

    /// Aligned with top.
    #[strum(serialize = "top")]
    AlignTop,

    /// Centered vertically.
    #[strum(serialize = "middle")]
    AlignMiddle,
}