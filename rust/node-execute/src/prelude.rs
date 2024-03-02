pub use common::{eyre::Report, tracing};
pub use node_patch::{Property, Value};
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
pub fn execution_status(messages: &Option<Vec<ExecutionMessage>>) -> ExecutionStatus {
    let Some(messages) = messages else {
        return ExecutionStatus::Succeeded;
    };
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

    if has_exceptions {
        ExecutionStatus::Exceptions
    } else if has_errors {
        ExecutionStatus::Errors
    } else if has_warnings {
        ExecutionStatus::Warnings
    } else {
        ExecutionStatus::Succeeded
    }
}

/// Create a value for `execution_required` from an `ExecutionStatus`
pub fn execution_required(status: &ExecutionStatus) -> ExecutionRequired {
    match status {
        ExecutionStatus::Errors | ExecutionStatus::Exceptions => ExecutionRequired::ExecutionFailed,
        _ => ExecutionRequired::No,
    }
}

/// Create a value for `execution_duration` from start and end timestamps
pub fn execution_duration(started: &Timestamp, ended: &Timestamp) -> Duration {
    ended
        .duration(started)
        .expect("should use compatible timestamps")
}

pub fn interruption(
    status: &Option<ExecutionStatus>,
) -> Option<(ExecutionStatus, ExecutionRequired)> {
    match status {
        Some(ExecutionStatus::Running) => Some((
            ExecutionStatus::Interrupted,
            ExecutionRequired::ExecutionInterrupted,
        )),
        Some(ExecutionStatus::Pending) => Some((
            ExecutionStatus::Cancelled,
            ExecutionRequired::ExecutionCancelled,
        )),
        _ => None,
    }
}

/// A macro for implementing the `pending` method of [`Executable`] nodes
#[macro_export]
macro_rules! pending_impl {
    ($executor: expr, $node_id: expr) => {
        $executor.replace_properties(
            $node_id,
            [(Property::ExecutionStatus, ExecutionStatus::Pending.into())],
        );
    };
}

/// A macro for implementing the `interrupt` method of [`Executable`] nodes
#[macro_export]
macro_rules! interrupt_impl {
    ($node: expr, $executor: expr, $node_id: expr) => {
        if let Some((status, required)) = interruption(&$node.options.execution_status) {
            $executor.replace_properties(
                $node_id,
                [
                    (Property::ExecutionStatus, status.into()),
                    (Property::ExecutionRequired, required.into()),
                ],
            );
        }
    };
}
