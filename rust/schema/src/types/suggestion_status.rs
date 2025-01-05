// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The status of an instruction.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, strum::EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum SuggestionStatus {
    /// The suggestion is the original content.
    #[default]
    Original,

    /// The suggestion has been accepted.
    Accepted,

    /// The suggestion has been rejected.
    Rejected,
}
