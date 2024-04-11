// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The severity level of a message.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, strum::EnumString, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum MessageLevel {
    #[default]
    Trace,

    Debug,

    Info,

    Warning,

    Error,

    Exception,
}
