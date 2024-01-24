// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// Indicates how a block (usually a `CodeChunk`) should be automatically labelled.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, SmartDefault, strum::EnumString, ReadNode)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub enum LabelType {
    #[default]
    FigureLabel,

    TableLabel,
}
