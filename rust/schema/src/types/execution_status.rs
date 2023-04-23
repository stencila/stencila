use crate::prelude::*;

/// Status of the most recent, including any current, execution of a document node.
#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Strip, Read, Write, ToHtml)]
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
