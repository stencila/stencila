// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The horizontal alignment of content.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, Copy, strum::EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[strum(ascii_case_insensitive, crate = "common::strum")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub enum HorizontalAlignment {
    /// Left align content.
    #[default]
    AlignLeft,

    /// Right align content.
    AlignRight,

    /// Fully justify cell content.
    AlignJustify,

    /// Center align the cell content.
    AlignCenter,

    /// Align the content on a character.
    AlignCharacter,
}
