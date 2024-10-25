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
    Author, AuthorRoleName, ExecutionKind, ExecutionRequired, ExecutionStatus, MessageLevel,
    NodeType, StringOrNumber,
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

    if matches!(node.node_type, NodeType::IfBlockClause) && matches!(node.is_active, Some(true)) {
        items.push(Status {
            range: node.range,
            status: "Active".to_string(),
            message: "Active".to_string(),
        });
    }

    items.append(&mut node.children.iter().flat_map(statuses).collect());

    items
}

/// Create a [`Status`] for a [`TextNodeExecution`]
fn execution_status(node: &TextNode, execution: &TextNodeExecution) -> Option<Status> {
    // Do not generate status for `IfBlock`s since we generate for its individual clauses
    if matches!(node.node_type, NodeType::IfBlock) {
        return None;
    }

    // Generate status string and message
    let (status, message) =
        if let Some(status @ (ExecutionStatus::Pending | ExecutionStatus::Running)) =
            &execution.status
        {
            // Pending or running nodes: just use status name as message
            // This comes first because it reflects something currently in progress
            let status = status.to_string();
            (status.clone(), status)
        } else if let Some(
            reason @ (ExecutionRequired::NeverExecuted
            | ExecutionRequired::KernelRestarted
            | ExecutionRequired::StateChanged
            | ExecutionRequired::SemanticsChanged
            | ExecutionRequired::DependenciesChanged
            | ExecutionRequired::DependenciesFailed),
        ) = &execution.required
        {
            // Stale nodes: expand reason into message
            // This comes before other execution status variants because any changes since last executed
            // should be indicated (rather than status of last execution).
            use ExecutionRequired::*;
            let status = match reason {
                NeverExecuted => "Unexecuted".to_string(),
                _ => "Stale".to_string(),
            };
            let message = match reason {
                NeverExecuted => "Not executed".to_string(),
                KernelRestarted => {
                    "Stale: not yet executed in the current kernel instance".to_string()
                }
                StateChanged => "Stale: changes since last executed".to_string(),
                SemanticsChanged => "Stale: semantic changes since last executed".to_string(),
                DependenciesChanged => "Stale: one or more dependencies have changed".to_string(),
                DependenciesFailed => "Stale: one or more dependencies have failed".to_string(),
                _ => reason.to_string(),
            };
            (status, message)
        } else if let Some(
            ExecutionStatus::Warnings | ExecutionStatus::Errors | ExecutionStatus::Exceptions,
        ) = &execution.status
        {
            // Do not generate a status for these since we generate an LSP diagnostic (below) for them
            return None;
        } else if let Some(
            status @ (ExecutionStatus::Skipped
            | ExecutionStatus::Locked
            | ExecutionStatus::Rejected),
        ) = &execution.status
        {
            // Skipped nodes
            let mut message = "Skipped: ".to_string();

            if matches!(status, ExecutionStatus::Locked) {
                message += "locked"
            } else if matches!(status, ExecutionStatus::Rejected) {
                message += "rejected suggestion";
            }

            if let Some(ended) = &execution.ended {
                message.push_str(", succeeded ");
                let ended = ended.humanize(false);
                if ended == "now ago" {
                    message.push_str("just now");
                } else {
                    message.push_str(&ended);
                }
            }

            ("Skipped".to_string(), message)
        } else if let Some(ExecutionStatus::Succeeded) = &execution.status {
            // Succeeded nodes: construct message including duration and authors
            let mut message = if matches!(
                node.node_type,
                NodeType::InstructionBlock
                    | NodeType::InstructionInline
                    | NodeType::SuggestionBlock
                    | NodeType::SuggestionInline
            ) {
                "Generated"
            } else {
                "Succeeded"
            }
            .to_string();

            if let Some(outputs) = &execution.outputs {
                message.push_str(" with ");
                if outputs == &1 {
                    message.push_str("1 output");
                } else {
                    message.push_str(&outputs.to_string());
                    message.push_str(" outputs");
                }
            } else if let Some(authors) = &execution.authors {
                message.push_str(" by ");
                let list = authors
                    .iter()
                    .filter_map(|author| match author {
                        Author::AuthorRole(role) => match role.role_name {
                            // Only show prompt and generator roles
                            AuthorRoleName::Prompter | AuthorRoleName::Generator => {
                                role.to_author()
                            }
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
                            .name
                            .clone()
                            .or(org.options.legal_name.clone())
                            .unwrap_or_else(|| "Unnamed Org".to_string()),
                        Author::SoftwareApplication(app) => {
                            let mut name = app.name.clone();
                            if let Some(version) =
                                &app.options.software_version.clone().or_else(|| {
                                    app.version.as_ref().map(|version| match version {
                                        StringOrNumber::String(string) => string.clone(),
                                        StringOrNumber::Number(number) => number.to_string(),
                                    })
                                })
                            {
                                name.push_str(" v");
                                name.push_str(version);
                            }
                            name
                        }
                        Author::AuthorRole(_) => String::new(),
                    })
                    .collect_vec();
                message.push_str(&list.join(if list.len() == 2 { " and " } else { ", " }));
            }

            if let Some(duration) = &execution.duration {
                message.push_str(", in ");
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

            let status = if let Some(ExecutionKind::Fork) = &execution.kind {
                message.push_str(" in forked kernel");
                "SucceededFork"
            } else {
                "Succeeded"
            }
            .to_string();

            (status, message)
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
    // If the node has changed since the last execution do not return any
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
                // Use the available code location offset from range of the code if it is available
                let code_range = execution.code_range.unwrap_or(node.range);

                let start_line = if let Some(line) = location.start_line {
                    code_range.start.line + line as u32
                } else {
                    code_range.start.line
                };

                let start_column = if let Some(column) = location.start_column {
                    code_range.start.character + column as u32
                } else {
                    code_range.start.character
                };

                let end_line = if let Some(line) = location.end_line {
                    code_range.start.line + line as u32
                } else {
                    // End line unknown so assume on same line as start
                    start_line
                };

                let end_column = if let Some(column) = location.start_column {
                    code_range.start.character + column as u32
                } else {
                    // End column unknown so if on a single line use code range end
                    // otherwise use a largish number to take to the end of the line
                    if end_line == code_range.end.line {
                        code_range.end.character
                    } else {
                        start_column + 100
                    }
                };

                Range::new(
                    Position::new(start_line, start_column),
                    Position::new(end_line, end_column),
                )
            } else if let Some(code_range) = execution.code_range {
                // Use the range of the code
                code_range
            } else {
                // Range is just start or node (avoids having too many squiggly lines across
                // whole of the node)
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

            // Format the message
            let message = message.formatted();

            Diagnostic {
                range,
                severity,
                message,
                ..Default::default()
            }
        })
        .collect_vec()
}
