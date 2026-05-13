//! Project Stencila execution metadata into provenance snapshots.
//!
//! Executable Stencila nodes carry execution status, duration, digests,
//! dependencies, kernel identity, and messages. This module normalizes those
//! node-specific fields into the Content Credentials snapshot model.

use chrono::{DateTime, Utc};
use stencila_schema::{
    CompilationDigest, Duration, ExecutionDependency, ExecutionMessage, Node, Timestamp,
};

use crate::{
    DependencySnapshot, ExecutionDigestSnapshot, ExecutionMessageSnapshot, ExecutionSnapshot,
    KernelSnapshot,
};

/// Borrowed view over the execution fields common to executable nodes.
///
/// Stencila stores execution metadata on several node types with slightly
/// different Rust structs. This view lets the projection logic stay uniform
/// without cloning every nested value up front.
struct ExecutableView<'a> {
    status: Option<String>,
    ended_at: Option<&'a Timestamp>,
    duration: Option<&'a Duration>,
    digest: Option<&'a CompilationDigest>,
    execution_count: Option<i64>,
    execution_instance: Option<&'a str>,
    language: Option<&'a str>,
    dependencies: &'a [ExecutionDependency],
    messages: &'a [ExecutionMessage],
}

/// Return the execution metadata view for supported executable nodes.
///
/// The export assertion should only include execution facts that Stencila itself
/// recorded. Unsupported nodes return `None` so plain media and document
/// structure are not accidentally described as executed work.
fn executable_view(node: &Node) -> Option<ExecutableView<'_>> {
    match node {
        Node::Article(article) => Some(ExecutableView {
            status: article
                .options
                .execution_status
                .as_ref()
                .map(ToString::to_string),
            ended_at: article.options.execution_ended.as_ref(),
            duration: article.options.execution_duration.as_ref(),
            digest: article.options.execution_digest.as_ref(),
            execution_count: article.options.execution_count,
            execution_instance: article.options.execution_instance.as_deref(),
            language: None,
            dependencies: article
                .options
                .execution_dependencies
                .as_deref()
                .unwrap_or(&[]),
            messages: article.options.execution_messages.as_deref().unwrap_or(&[]),
        }),
        Node::CodeChunk(chunk) => Some(ExecutableView {
            status: chunk
                .options
                .execution_status
                .as_ref()
                .map(ToString::to_string),
            ended_at: chunk.options.execution_ended.as_ref(),
            duration: chunk.options.execution_duration.as_ref(),
            digest: chunk.options.execution_digest.as_ref(),
            execution_count: chunk.options.execution_count,
            execution_instance: chunk.options.execution_instance.as_deref(),
            language: chunk.programming_language.as_deref(),
            dependencies: chunk
                .options
                .execution_dependencies
                .as_deref()
                .unwrap_or(&[]),
            messages: chunk.options.execution_messages.as_deref().unwrap_or(&[]),
        }),
        Node::CodeExpression(expression) => Some(ExecutableView {
            status: expression
                .options
                .execution_status
                .as_ref()
                .map(ToString::to_string),
            ended_at: expression.options.execution_ended.as_ref(),
            duration: expression.options.execution_duration.as_ref(),
            digest: expression.options.execution_digest.as_ref(),
            execution_count: expression.options.execution_count,
            execution_instance: expression.options.execution_instance.as_deref(),
            language: expression.programming_language.as_deref(),
            dependencies: expression
                .options
                .execution_dependencies
                .as_deref()
                .unwrap_or(&[]),
            messages: expression
                .options
                .execution_messages
                .as_deref()
                .unwrap_or(&[]),
        }),
        _ => None,
    }
}

/// Project execution metadata from a subject node into a provenance snapshot.
///
/// Execution records are omitted when a node has no execution-specific facts.
/// That distinction matters because absence means "not applicable or unknown",
/// while an empty execution record could be read as an attestation that execution
/// was checked and had no details.
pub(super) fn execution_snapshot_for(subject: &Node) -> Option<ExecutionSnapshot> {
    let view = executable_view(subject)?;

    let has_execution = view.status.is_some()
        || view.ended_at.is_some()
        || view.duration.is_some()
        || view.digest.is_some()
        || view.execution_count.is_some()
        || view.execution_instance.is_some()
        || view.language.is_some()
        || !view.dependencies.is_empty()
        || !view.messages.is_empty();

    if !has_execution {
        return None;
    }

    let mut snapshot = ExecutionSnapshot {
        status: view.status,
        ended_at: view.ended_at.and_then(|timestamp| {
            DateTime::<Utc>::try_from(timestamp)
                .ok()
                .map(|time| time.to_rfc3339())
        }),
        duration_ms: view
            .duration
            .and_then(|duration| u64::try_from(duration.to_milliseconds()).ok()),
        digest: view.digest.map(execution_digest_snapshot),
        count: view.execution_count,
        ..Default::default()
    };

    if view.execution_instance.is_some() || view.language.is_some() {
        snapshot.kernel = Some(KernelSnapshot {
            name: view.execution_instance.map(ToString::to_string),
            language: view.language.map(ToString::to_string),
            ..Default::default()
        });
    }

    snapshot.dependencies = view.dependencies.iter().map(dependency_snapshot).collect();
    snapshot.messages = view
        .messages
        .iter()
        .map(execution_message_snapshot)
        .collect();

    Some(snapshot)
}

/// Convert Stencila compilation digests into credential digest fields.
///
/// The source digest values are Stencila's compact semantic fingerprints rather
/// than cryptographic file hashes. Prefixing them as `stencila:` avoids confusing
/// them with SHA-style asset digests while preserving reproducibility signals.
fn execution_digest_snapshot(digest: &CompilationDigest) -> ExecutionDigestSnapshot {
    ExecutionDigestSnapshot {
        state_digest: Some(value_to_digest(digest.state_digest)),
        semantic_digest: digest.semantic_digest.map(value_to_digest),
        dependencies_digest: digest.dependencies_digest.map(value_to_digest),
        dependencies_stale: digest.dependencies_stale,
        dependencies_failed: digest.dependencies_failed,
    }
}

/// Project an execution dependency into the credential dependency record.
///
/// Dependencies explain why an output may become stale when upstream state
/// changes. The projection currently preserves identity and relation while
/// leaving digest empty because the schema dependency value does not carry one.
fn dependency_snapshot(dependency: &ExecutionDependency) -> DependencySnapshot {
    DependencySnapshot {
        node_id: Some(dependency.dependency_id.clone()),
        node_type: Some(dependency.dependency_type.clone()),
        relation: Some(dependency.dependency_relation.to_string()),
        digest: None,
    }
}

/// Project an execution message into a compact credential message.
///
/// Messages are retained because warnings and errors can materially affect trust
/// in generated outputs, but the projection intentionally keeps them as concise
/// structured diagnostics rather than full logs.
fn execution_message_snapshot(message: &ExecutionMessage) -> ExecutionMessageSnapshot {
    ExecutionMessageSnapshot {
        level: Some(message.level.to_string()),
        error_type: message.error_type.clone(),
        message: Some(message.message.clone()),
    }
}

/// Format a Stencila digest integer as a stable namespaced digest string.
///
/// The namespace makes clear that the value is generated by Stencila's internal
/// digest algorithm and should not be treated as a generic cryptographic hash.
fn value_to_digest(value: u64) -> String {
    format!("stencila:{value:016x}")
}
