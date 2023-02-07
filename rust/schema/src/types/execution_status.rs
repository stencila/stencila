//! Generated file, do not edit

use crate::prelude::*;

/// Status of the most recent, including any current, execution of a document node.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged, crate = "common::serde")]
#[def = "Unknown"]
pub enum ExecutionStatus {
    Scheduled,
    ScheduledPreviouslyFailed,
    Running,
    RunningPreviouslyFailed,
    Succeeded,
    Failed,
    Cancelled,
    Unknown,
}
