// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// Status of the most recent, including any current, execution of a document node.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(crate = "common::serde")]
pub enum ExecutionStatus {
    Scheduled,

    ScheduledPreviouslyFailed,

    Running,

    RunningPreviouslyFailed,

    Succeeded,

    Failed,

    Cancelled,
}
