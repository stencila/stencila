//! Publishing of diagnostics and other notifications
//!
//! See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_publishDiagnostics

use async_lsp::{
    lsp_types::{
        notification::Notification, Diagnostic, DiagnosticSeverity, PublishDiagnosticsParams,
        Range, Url,
    },
    ClientSocket, LanguageClient,
};

use common::{
    itertools::Itertools,
    serde::{Deserialize, Serialize},
    tracing,
};
use schema::{ExecutionStatus, MessageLevel};

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
    let mut items = node
        .execution
        .as_ref()
        .map(|execution| {
            use ExecutionStatus::*;
            let details = match &execution.status {
                Pending | Running => execution.status.to_string(),
                Succeeded => {
                    let mut status = "Succeeded".to_string();
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
                    status
                }
                // Ignore other statuses: warning, errors, and exceptions are
                // published as diagnostics
                _ => return vec![],
            };

            vec![Status {
                range: node.range,
                status: execution.status.clone(),
                details,
            }]
        })
        .unwrap_or_default();

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
                    use MessageLevel::*;
                    let severity = Some(match message.level {
                        Error | Exception => DiagnosticSeverity::ERROR,
                        Warning => DiagnosticSeverity::WARNING,
                        Info => DiagnosticSeverity::INFORMATION,
                        Debug | Trace => DiagnosticSeverity::HINT,
                    });

                    let message = message.message.clone();

                    Diagnostic {
                        range: node.range,
                        severity,
                        message,
                        ..Default::default()
                    }
                })
                .collect_vec()
        })
        .unwrap_or_default();

    diags.append(&mut node.children.iter().flat_map(diagnostics).collect());

    diags
}
