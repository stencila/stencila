use std::hash::{Hash, Hasher};

use common::seahash::SeaHasher;
pub use common::{eyre::Report, tracing};
pub use parsers::ParseInfo;
pub use schema::{
    Array, Duration, ExecutionMessage, ExecutionRequired, ExecutionStatus, MessageLevel, Node,
    NodeProperty, Null, PatchNode, PatchOp, PatchValue, Primitive, Timestamp, WalkControl,
    WalkNode,
};
use schema::{CompilationDigest, CompilationMessage};

pub(crate) use crate::{Executable, Executor};

/// Add to an existing digest
pub fn add_to_digest(digest: &mut u64, bytes: &[u8]) {
    let mut hash = SeaHasher::new();
    digest.hash(&mut hash);
    bytes.hash(&mut hash);
    *digest = hash.finish()
}

/// Create an `CompilationMessage` from an `eyre::Report`
pub fn error_to_compilation_message(error: Report) -> CompilationMessage {
    CompilationMessage {
        level: MessageLevel::Error,
        message: error.to_string(),
        ..Default::default()
    }
}

/// Create an `ExecutionMessage` from an `eyre::Report`
pub fn error_to_execution_message(context: &str, error: Report) -> ExecutionMessage {
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

/// Create a value for `execution_required` based on execution and compilation digests
pub fn execution_required_digests(
    execution_digest: &Option<CompilationDigest>,
    compilation_digest: &CompilationDigest,
) -> ExecutionRequired {
    // If there is no execution digest then execution is required because
    // it has never been executed
    let Some(execution_digest) = execution_digest else {
        return ExecutionRequired::NeverExecuted;
    };

    // If the compilation digest has a semantic digest then compare it to previous
    if let Some(semantic_digest) = compilation_digest.semantic_digest {
        return if Some(semantic_digest) != execution_digest.semantic_digest {
            ExecutionRequired::SemanticsChanged
        } else {
            ExecutionRequired::No
        };
    }

    // Fallback to comparing the state digests
    if compilation_digest.state_digest != execution_digest.state_digest {
        ExecutionRequired::StateChanged
    } else {
        ExecutionRequired::No
    }
}

/// Create a value for `execution_required` based on an `ExecutionStatus`
pub fn execution_required_status(status: &ExecutionStatus) -> ExecutionRequired {
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

/// Set a property
pub fn set<T: PatchNode>(node_property: NodeProperty, value: T) -> (NodeProperty, PatchOp) {
    (
        node_property,
        PatchOp::Set(value.to_value().unwrap_or_default()),
    )
}

/// Set an optional property to None
pub fn none(node_property: NodeProperty) -> (NodeProperty, PatchOp) {
    (node_property, PatchOp::Set(PatchValue::None))
}

/// Push onto a vector property
pub fn push<T: PatchNode>(node_property: NodeProperty, value: T) -> (NodeProperty, PatchOp) {
    (
        node_property,
        PatchOp::Push(value.to_value().unwrap_or_default()),
    )
}

/// Append to a vector property
pub fn append<T: PatchNode>(
    node_property: NodeProperty,
    values: Vec<T>,
) -> (NodeProperty, PatchOp) {
    (
        node_property,
        PatchOp::Append(
            values
                .into_iter()
                .map(|value| value.to_value().unwrap_or_default())
                .collect(),
        ),
    )
}

/// Clear a vector property
pub fn clear(node_property: NodeProperty) -> (NodeProperty, PatchOp) {
    (node_property, PatchOp::Clear)
}

/// A macro for implementing the `pending` method of [`Executable`] nodes
#[macro_export]
macro_rules! pending_impl {
    ($executor: expr, $node_id: expr) => {
        $executor.patch(
            $node_id,
            [set(NodeProperty::ExecutionStatus, ExecutionStatus::Pending)],
        );
    };
}

/// A macro for implementing the `interrupt` method of [`Executable`] nodes
#[macro_export]
macro_rules! interrupt_impl {
    ($node: expr, $executor: expr, $node_id: expr) => {
        if let Some((status, required)) = interruption(&$node.options.execution_status) {
            $executor.patch(
                $node_id,
                [
                    set(NodeProperty::ExecutionStatus, status),
                    set(NodeProperty::ExecutionRequired, required),
                ],
            );
        }
    };
}
