// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// Indicates how a block (usually a `CodeChunk`) should be automatically labelled.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, Copy, EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[strum(ascii_case_insensitive)]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub enum LabelType {
    AppendixLabel,

    #[default]
    FigureLabel,

    TableLabel,
}
