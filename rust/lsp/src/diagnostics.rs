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
use schema::{Author, AuthorRoleName, ExecutionStatus, MessageLevel, NodeType, StringOrNumber};

use crate::text_document::TextNode;

/// A summary of the execution status of a node including
/// its status, execution duration etc
#[derive(Serialize, Deserialize)]
#[serde(crate = "common::serde")]
struct Status {
    range: Range,
    status: ExecutionStatus,
    details: String,
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
    // TODO: consider periodic updates to update times ended
    let mut items = Vec::new();

    if let Some(execution) = node.execution.as_ref() {
        use ExecutionStatus::*;
        let details = match &execution.status {
            Pending | Running => execution.status.to_string(),
            Succeeded => {
                let mut status = if matches!(
                    node.node_type,
                    NodeType::InsertBlock
                        | NodeType::InsertInline
                        | NodeType::ReplaceBlock
                        | NodeType::ReplaceInline
                ) {
                    "Generated"
                } else {
                    "Succeeded"
                }
                .to_string();

                if let Some(duration) = &execution.duration {
                    status.push_str(" in ");
                    status.push_str(&duration.humanize(true));
                }

                if let Some(ended) = &execution.ended {
                    let ended = ended.humanize(false);
                    if ended == "now ago" {
                        status.push_str(", just now");
                    } else {
                        status.push_str(", ");
                        status.push_str(&ended);
                    }
                }

                if let Some(authors) = &execution.authors {
                    status.push_str(", by ");
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
                                if let Some(version) =
                                    &app.options.software_version.clone().or_else(|| {
                                        app.options.version.as_ref().map(|version| match version {
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
                        .join(", ");
                    status.push_str(&list);
                }

                status
            }
            // Ignore other statuses: warning, errors, and exceptions are
            // published as diagnostics
            _ => return vec![],
        };

        items.push(Status {
            range: node.range,
            status: execution.status.clone(),
            details,
        });
    }

    items.append(&mut node.children.iter().flat_map(statuses).collect());

    items
}

/// Create [`Diagnostic`]s for a [`TextNode`]
fn diagnostics(node: &TextNode) -> Vec<Diagnostic> {
    let mut diags = node
        .execution
        .as_ref()
        .map(|execution| {
            execution
                .messages
                .iter()
                .flatten()
                .map(|message| {
                    // Calculate range of diagnostic
                    let range = if let Some(location) = &message.code_location {
                        let mut start_line = node.range.start.line
                            + if matches!(node.node_type, NodeType::CodeChunk) {
                                // Plus one for code chunk back ticks
                                1
                            } else {
                                0
                            };
                        let mut start_column = node.range.start.character;
                        let mut end_line = start_line;
                        let mut end_column = start_column;
                        if let Some(line) = location.start_line {
                            start_line += line as u32;
                        }
                        if let Some(column) = location.start_column {
                            start_column += column as u32;
                        }
                        if let Some(line) = location.end_line {
                            end_line += line as u32;
                        }
                        if let Some(column) = location.end_column {
                            end_column += column as u32;
                        } else {
                            end_column += 100;
                        }

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
        })
        .unwrap_or_default();

    diags.append(&mut node.children.iter().flat_map(diagnostics).collect());

    diags
}
