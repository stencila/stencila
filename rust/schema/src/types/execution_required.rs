// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// Whether, and why, the execution of a node is required or not.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, strum::EnumString, Eq, PartialOrd, Ord, Hash, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum ExecutionRequired {
    /// No re-execution is required, the semantics of the node and its dependencies has not changed since it was last executed 
    No,

    /// Execution is required because the node has never been executed (or any previous execution was not persisted in its state). 
    #[default]
    NeverExecuted,

    /// Re-execution is required because the state of the node (e.g. source code) has changed since it was last executed and no semantic digest is available to determine if semantics changed. 
    StateChanged,

    /// Re-execution is required because the semantics of the node has changed since it was last executed. 
    SemanticsChanged,

    /// Re-execution is required because the semantics of one or more dependencies (including transitive dependencies) changed since it was last executed. 
    DependenciesChanged,

    /// Re-execution is required because one or more dependencies (including transitive dependencies) failed when it was last executed. 
    DependenciesFailed,

    /// Re-execution is required because execution failed (there were errors or exceptions) the last time it was executed. 
    ExecutionFailed,

    /// Re-execution may be required because execution was pending but was cancelled. 
    ExecutionCancelled,

    /// Re-execution is required because execution was interrupted the last time it was executed. 
    ExecutionInterrupted,

    /// Re-execution is required because the kernel that the node was last executed in was restarted. 
    KernelRestarted,

    /// Execution is required because it was explicitly requested by a user.
    UserRequested,
}
