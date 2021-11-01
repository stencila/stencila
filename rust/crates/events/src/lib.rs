use serde::{Deserialize, Serialize};

/// An event updating progress of some task
#[derive(Default, Debug, Deserialize, Serialize)]
pub struct ProgressEvent {
    /// The id of the task that this progress event relates to
    pub id: Option<String>,

    /// The id of the parent task (if any)
    pub parent: Option<String>,

    /// The event message
    pub message: Option<String>,

    /// The current value
    pub current: Option<i64>,

    /// The expected value when complete
    pub expected: Option<i64>,

    // Whether or not the task is complete
    pub complete: bool,
}
