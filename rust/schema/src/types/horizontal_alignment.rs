// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The horizontal alignment of content.
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
pub enum HorizontalAlignment {
    /// Left align content.
    #[default]
    #[strum(serialize = "left")]
    AlignLeft,

    /// Right align content.
    #[strum(serialize = "right")]
    AlignRight,

    /// Fully justify cell content.
    #[strum(serialize = "justify")]
    AlignJustify,

    /// Center align the cell content.
    #[strum(serialize = "center")]
    AlignCenter,

    /// Align the content on a character.
    #[strum(serialize = "char")]
    AlignCharacter,
}
