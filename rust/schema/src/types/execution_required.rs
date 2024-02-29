// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// Whether, and why, the execution of a node is required or not.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, strum::EnumString, ReadNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum ExecutionRequired {
    No,

    #[default]
    NeverExecuted,

    SemanticsChanged,

    DependenciesChanged,

    DependenciesFailed,

    ExecutionFailed,

    KernelRestarted,

    UserRequested,
}
