// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The kind of execution of a document node.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, strum::EnumString, Eq, PartialOrd, Ord, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum ExecutionKind {
    /// The node was executed within the main set of kernels for the document.
    #[default]
    Main,

    /// The node was executed within a forked set of kernels.
    Fork,
}
