// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

/// Status of the most recent, including any current, execution of a document node.
#[derive(Debug, strum::Display, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, SmartDefault, strum::EnumString, ReadNode, DomCodec, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(crate = "common::serde")]
#[strum(ascii_case_insensitive, crate = "common::strum")]
pub enum ExecutionStatus {
    #[default]
    Scheduled,

    ScheduledPreviouslyFailed,

    Running,

    RunningPreviouslyFailed,

    Succeeded,

    Failed,

    Cancelled,
}
