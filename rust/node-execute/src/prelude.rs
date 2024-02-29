pub use std::ops::AddAssign;

pub use common::{eyre::Report, tracing};
pub use schema::{
    walk::{WalkControl, WalkNode},
    Array, Duration, ExecutionMessage, ExecutionRequired, ExecutionStatus, MessageLevel, Node,
    Null, Primitive, Timestamp,
};

pub(crate) use crate::{Executable, Executor};

/// Create an `ExecutionMessage` from an `eyre::Report`
pub fn error_to_message(context: &str, error: Report) -> ExecutionMessage {
    ExecutionMessage {
        level: MessageLevel::Error,
        message: error.to_string(),
        stack_trace: Some(context.to_string()),
        ..Default::default()
    }
}

/// Create a value for `execution_status` based on a vector of `ExecutionMessage`s
pub fn execution_status(messages: &[ExecutionMessage]) -> Option<ExecutionStatus> {
    let mut has_warnings = false;
    let mut has_errors = false;
    let mut has_exceptions = false;
    for ExecutionMessage { level, .. } in messages {
        match level {
            MessageLevel::Warning => {
                has_warnings = true;
            }
            MessageLevel::Error => {
                has_errors = true;
            }
            MessageLevel::Exception => {
                has_exceptions = true;
            }
            _ => {}
        }
    }

    let status = if has_exceptions {
        ExecutionStatus::Exceptions
    } else if has_errors {
        ExecutionStatus::Errors
    } else if has_warnings {
        ExecutionStatus::Warnings
    } else {
        ExecutionStatus::Succeeded
    };

    Some(status)
}

/// Create a value for `execution_required` from an `Option<ExecutionStatus>`
pub fn execution_required(
    execution_status: &Option<ExecutionStatus>,
) -> Option<ExecutionRequired> {
    let Some(status) = execution_status else {
        return None;
    };

    match status {
        ExecutionStatus::Errors | ExecutionStatus::Exceptions => {
            Some(ExecutionRequired::ExecutionFailed)
        }
        _ => Some(ExecutionRequired::No),
    }
}

/// Create a value for `execution_messages` from a vector of `ExecutionMessage`s
pub fn execution_messages(messages: Vec<ExecutionMessage>) -> Option<Vec<ExecutionMessage>> {
    if !messages.is_empty() {
        Some(messages)
    } else {
        None
    }
}

/// Create a value for `execution_duration` from start and end timestamps
pub fn execution_duration(started: &Timestamp, ended: &Timestamp) -> Option<Duration> {
    Some(
        ended
            .duration(started)
            .expect("should use compatible timestamps"),
    )
}
