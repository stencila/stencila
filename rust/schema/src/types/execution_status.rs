// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// Status of the most recent, including any current, execution of a document node.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, strum::EnumString, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum ExecutionStatus {
    /// Execution of the node has been scheduled for some time in the future.
    #[default]
    Scheduled,

    /// Execution of the node is pending.
    Pending,

    /// Execution of the node or node type was explicitly skipped by the user.
    Skipped,

    /// Execution of the node was skipped because it is locked.
    Locked,

    /// Execution of the node was skipped because it is a rejected suggestion.
    Rejected,

    /// Execution of the node was skipped because it has code, or other property, that is empty.
    Empty,

    /// The node is currently being executed.
    Running,

    /// Execution of the node completed without warning, error, or exception messages.
    Succeeded,

    /// Execution of the node completed but with warning messages.
    Warnings,

    /// Execution of the node completed but with error messages.
    Errors,

    /// Execution of the node did not complete because there was an exception message.
    Exceptions,

    /// Execution of the node was pending but was cancelled.
    Cancelled,

    /// Execution of the node was running but was interrupted.
    Interrupted,
}
