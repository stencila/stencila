// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// A review status for a suggestion.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, ProbeNode, StripNode, WalkNode, WriteNode, SmartDefault, Copy, EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, MarkdownCodec, TextCodec)]
#[strum(ascii_case_insensitive)]
pub enum SuggestionStatus {
    /// The suggestion is the original content.
    #[default]
    Original,

    /// The suggestion has been accepted.
    Accepted,

    /// The suggestion has been rejected.
    Rejected,
}
