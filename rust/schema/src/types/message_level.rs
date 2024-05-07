// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The severity level of a message.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, strum::EnumString, Eq, PartialOrd, Ord, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum MessageLevel {
    /// A tracing message
    #[default]
    Trace,

    /// A debug message
    Debug,

    /// An information message
    Info,

    /// A warning message
    Warning,

    /// An error message
    Error,

    /// An exception message
    Exception,
}
