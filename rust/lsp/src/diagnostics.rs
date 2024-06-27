//! Publishing of diagnostics and other notifications
//!
//! See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_publishDiagnostics

use async_lsp::{
    lsp_types::{
        notification::Notification, Diagnostic, DiagnosticSeverity, Position,
        PublishDiagnosticsParams, Range, Url,
    },
    ClientSocket, LanguageClient,
};

use common::{
    itertools::Itertools,
    serde::{Deserialize, Serialize},
    tracing,
};
use schema::{
    Author, AuthorRoleName, ExecutionRequired, ExecutionStatus, MessageLevel, NodeType,
    StringOrNumber,
};

use crate::text_document::{TextNode, TextNodeExecution};

/// A summary of the execution status of a node including
/// its status, execution duration etc
///
/// Similar to an LSP Diagnostic but intended to be displayed
/// separately to those (because diagnostics imply "problems")
#[derive(Serialize, Deserialize)]
#[serde(crate = "common::serde")]
struct Status {
    range: Range,
    status: String,
    message: String,
}

struct PublishStatus;

#[derive(Serialize, Deserialize)]
#[serde(crate = "common::serde")]
struct PublishStatusParams {
    uri: Url,
    statuses: Vec<Status>,
}

impl Notification for PublishStatus {
    const METHOD: &'static str = "stencila/publishStatus";
    type Params = PublishStatusParams;
}

/// Publish diagnostics
pub(super) fn publish(uri: &Url, text_node: &TextNode, client: &mut ClientSocket) {
    // Publish status notifications. As for diagnostics intentionally publishes an
    // empty set so as to clear existing decorations.
    let statuses = statuses(text_node);
    if let Err(error) = client.notify::<PublishStatus>(PublishStatusParams {
        uri: uri.clone(),
        statuses,
    }) {
        tracing::error!("While publishing status notifications: {error}");
    }

    // Publish diagnostics. This intentionally publishes an empty set so as to clear
    // any existing diagnostics.
    let diagnostics = diagnostics(text_node);
    if let Err(error) = client.publish_diagnostics(PublishDiagnosticsParams {
        uri: uri.clone(),
        diagnostics,
        version: None,
    }) {
        tracing::error!("While publishing diagnostics: {error}");
    }
}

/// Create status notifications
fn statuses(node: &TextNode) -> Vec<Status> {
    let mut items = Vec::new();

    if let Some(execution) = node.execution.as_ref() {
        if let Some(status) = execution_status(node, execution) {
            items.push(status)
        }
    }

    items.append(&mut node.children.iter().flat_map(statuses).collect());

    items
}

/// Create a [`Status`] for a [`TextNodeExecution`]
fn execution_status(node: &TextNode, execution: &TextNodeExecution) -> Option<Status> {
    // Generate status string and message
    let (status, message) = if let Some(
        reason @ (ExecutionRequired::NeverExecuted
        | ExecutionRequired::StateChanged
        | ExecutionRequired::SemanticsChanged
        | ExecutionRequired::DependenciesChanged
        | ExecutionRequired::DependenciesFailed),
    ) = &execution.required
    {
        // Stale nodes: expand reason into message
        // This is first in the if block because any changes since last executed
        // should be indicated (rather than status of last execution).
        use ExecutionRequired::*;
        let message = match reason {
            NeverExecuted => "Not executed".to_string(),
            StateChanged => "Changes since last executed".to_string(),
            SemanticsChanged => "Semantic changes since last executed".to_string(),
            DependenciesChanged => "One or more dependencies have changed".to_string(),
            DependenciesFailed => "One or more dependencies have failed".to_string(),
            _ => reason.to_string(),
        };
        ("Stale".to_string(), message)
    } else if let Some(
        ExecutionStatus::Warnings | ExecutionStatus::Errors | ExecutionStatus::Exceptions,
    ) = &execution.status
    {
        // Do not generate a status for these since we generate an LSP diagnostic (below) for them
        return None;
    } else if let Some(status @ (ExecutionStatus::Pending | ExecutionStatus::Running)) =
        &execution.status
    {
        // Pending or running nodes: just use status name as message
        let status = status.to_string();
        (status.clone(), status)
    } else if let Some(ExecutionStatus::Succeeded) = &execution.status {
        // Succeeded nodes: construct message including duration and authors
        let mut message = if matches!(
            node.node_type,
            NodeType::SuggestionBlock | NodeType::SuggestionInline
        ) {
            "Generated"
        } else {
            "Succeeded"
        }
        .to_string();

        if let Some(duration) = &execution.duration {
            message.push_str(" in ");
            message.push_str(&duration.humanize(true));
        }

        if let Some(ended) = &execution.ended {
            let ended = ended.humanize(false);
            if ended == "now ago" {
                message.push_str(", just now");
            } else {
                message.push_str(", ");
                message.push_str(&ended);
            }
        }

        if let Some(authors) = &execution.authors {
            message.push_str(", by ");
            let list = authors
                .iter()
                .filter_map(|author| match author {
                    Author::AuthorRole(role) => match role.role_name {
                        // Only show generator role
                        AuthorRoleName::Generator => role.to_author(),
                        _ => None,
                    },
                    _ => Some(author.clone()),
                })
                .map(|author| match author {
                    Author::Person(person) => person
                        .given_names
                        .iter()
                        .flatten()
                        .chain(person.family_names.iter().flatten())
                        .join(" "),
                    Author::Organization(org) => org
                        .options
                        .name
                        .clone()
                        .or(org.options.legal_name.clone())
                        .unwrap_or_else(|| "Unnamed Org".to_string()),
                    Author::SoftwareApplication(app) => {
                        let mut name = app.name.clone();
                        if let Some(version) = &app.options.software_version.clone().or_else(|| {
                            app.options.version.as_ref().map(|version| match version {
                                StringOrNumber::String(string) => string.clone(),
                                StringOrNumber::Number(number) => number.to_string(),
                            })
                        }) {
                            name.push_str(" v");
                            name.push_str(version);
                        }
                        name
                    }
                    Author::AuthorRole(_) => String::new(),
                })
                .join(", ");
            message.push_str(&list);
        }

        ("Succeeded".to_string(), message)
    } else {
        return None;
    };

    Some(Status {
        range: node.range,
        status,
        message,
    })
}

/// Create [`Diagnostic`]s for a [`TextNode`]
fn diagnostics(node: &TextNode) -> Vec<Diagnostic> {
    let mut diags = Vec::new();

    if let Some(execution) = node.execution.as_ref() {
        diags.append(&mut execution_diagnostic(node, execution));
    }

    diags.append(&mut node.children.iter().flat_map(diagnostics).collect());

    diags
}

/// Create [`Diagnostic`]s for a [`TextNodeExecution`]
fn execution_diagnostic(node: &TextNode, execution: &TextNodeExecution) -> Vec<Diagnostic> {
    // If the node has changed itself since the last execution do not return any
    // diagnostics for it
    if let Some(ExecutionRequired::StateChanged | ExecutionRequired::SemanticsChanged) =
        &execution.required
    {
        return Vec::new();
    }

    // Create a diagnostic for each message
    execution
        .messages
        .iter()
        .flatten()
        .map(|message| {
            // Calculate range of diagnostic
            let range = if let Some(location) = &message.code_location {
                let line_offset = if matches!(node.node_type, NodeType::CodeChunk) {
                    // Plus one for line with code chunk back ticks
                    1
                } else {
                    0
                };

                let start_line = if let Some(line) = location.start_line {
                    node.range.start.line + line_offset + line as u32
                } else {
                    node.range.start.line + line_offset
                };

                let start_column = if let Some(column) = location.start_column {
                    node.range.start.character + column as u32
                } else {
                    node.range.start.character
                };

                let end_line = if let Some(line) = location.end_line {
                    node.range.start.line + line_offset + line as u32
                } else {
                    // End line unknown so assume on same line as start
                    start_line
                };

                let end_column = if let Some(column) = location.start_column {
                    node.range.start.character + column as u32
                } else {
                    // End column unknown so apply to rest of line from start column
                    start_column + 100
                };

                Range::new(
                    Position::new(start_line, start_column),
                    Position::new(end_line, end_column),
                )
            } else {
                // Range is just start (avoids having too many squiggly lines across
                // whole of code chunk)
                Range::new(node.range.start, node.range.start)
            };

            // Translate message level to diagnostic severity
            use MessageLevel::*;
            let severity = Some(match message.level {
                Error | Exception => DiagnosticSeverity::ERROR,
                Warning => DiagnosticSeverity::WARNING,
                Info => DiagnosticSeverity::INFORMATION,
                Debug | Trace => DiagnosticSeverity::HINT,
            });

            // Add error type to message (if any)
            let mut msg = message.message.clone();
            if let Some(error_type) = message.error_type.as_ref() {
                msg.insert_str(0, &[error_type, ": "].concat())
            };
            if let Some(stack_trace) = message.stack_trace.as_ref() {
                msg.push_str("\n\n");
                msg.push_str(&stack_trace);
            };

            Diagnostic {
                range,
                severity,
                message: msg,
                ..Default::default()
            }
        })
        .collect_vec()
}
