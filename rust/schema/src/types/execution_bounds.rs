// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// The bounds placed on the execution of a document node.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, strum::EnumString, Eq, PartialOrd, Ord, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum ExecutionBounds {
    /// Use the bounds configured in the document, workspace or user settings.
    #[default]
    Default,

    /// Execute within the main set of kernels.
    Main,

    /// Execute within a forked set of kernels.
    Fork,

    /// Execute within a forked set of kernels with limited capabilities.
    Limit,

    /// Execute within a forked set of kernels within a sandbox.
    Box,

    /// Skip execution.
    Skip,
}
