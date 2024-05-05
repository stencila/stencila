//! Publishing of diagnostics
//!
//! See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_publishDiagnostics

use async_lsp::{
    lsp_types::{Diagnostic, DiagnosticSeverity, PublishDiagnosticsParams, Range, Url},
    ClientSocket, LanguageClient,
};

use common::tracing;
use schema::ExecutionStatus;

use crate::text_document::{TextNode, TextNodeExecution};

/// Publish diagnostics
pub(super) fn publish(uri: &Url, text_node: &TextNode, client: &mut ClientSocket) {
    if let Err(error) = client.publish_diagnostics(PublishDiagnosticsParams {
        uri: uri.clone(),
        diagnostics: diagnostics(text_node),
        version: None,
    }) {
        tracing::error!("While publishing diagnostics: {error}");
    }
}

/// Create [`Diagnostic`]s for a [`TextNode`]
fn diagnostics(node: &TextNode) -> Vec<Diagnostic> {
    let mut diags = node
        .execution
        .as_ref()
        .map(|execution| execution_diagnostics(node.range, execution))
        .unwrap_or_default();

    diags.append(&mut node.children.iter().flat_map(diagnostics).collect());

    diags
}

/// Create [`Diagnostic`]s for an executable node
fn execution_diagnostics(
    range: Range,
    TextNodeExecution { execution_status }: &TextNodeExecution,
) -> Vec<Diagnostic> {
    let severity = match execution_status {
        ExecutionStatus::Errors | ExecutionStatus::Exceptions => DiagnosticSeverity::ERROR,
        ExecutionStatus::Warnings => DiagnosticSeverity::WARNING,
        _ => DiagnosticSeverity::INFORMATION,
    };

    vec![Diagnostic {
        range,
        severity: Some(severity),
        message: execution_status.to_string(),
        ..Default::default()
    }]
}
