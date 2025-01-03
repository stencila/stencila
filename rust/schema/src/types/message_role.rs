// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The role of a message.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, strum::EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum MessageRole {
    /// A system message
    System,

    /// A user message
    #[default]
    User,

    /// A message from a model
    Model,
}
