// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The type of a suggestion.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, Copy, EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[strum(ascii_case_insensitive)]
pub enum SuggestionType {
    /// The suggestion is an insertion of new content.
    #[default]
    Insert,

    /// The suggestion is a deletion of existing content.
    Delete,

    /// The suggestion is a replacement of existing content with new content.
    Replace,
}
