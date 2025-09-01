// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The type of a `Note` which determines where the note content is displayed within the document.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, Copy, EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[strum(ascii_case_insensitive)]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub enum NoteType {
    #[default]
    Footnote,

    Endnote,

    Sidenote,
}
