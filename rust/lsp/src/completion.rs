//! Handling of completion related messages
//!
//! https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_completion

use std::sync::Arc;

use async_lsp::{
    lsp_types::{
        CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse,
        CompletionTriggerKind, Documentation, MarkupContent, MarkupKind,
    },
    ResponseError,
};

use codecs::Positions;
use common::tokio::sync::RwLock;
use schema::{Assistant, StringOrNumber};

use crate::utils::position_to_position16;

pub(super) async fn request(
    params: CompletionParams,
    source: Option<Arc<RwLock<String>>>,
) -> Result<Option<CompletionResponse>, ResponseError> {
    // Get the trigger for the completion
    let trigger_kind = params.context.as_ref().map(|context| context.trigger_kind);
    let trigger_character = params.context.and_then(|context| context.trigger_character);

    // Shortcuts that do not require getting the source
    if (Some(CompletionTriggerKind::TRIGGER_CHARACTER), Some("@"))
        == (trigger_kind, trigger_character.as_deref())
    {
        return assignee_completion().await;
    }

    // Unable to proceed if no source available
    let Some(source) = source else {
        return Ok(None);
    };

    // Get the source before the cursor
    let source = source.read().await;
    let cursor = params.text_document_position.position;
    let positions = Positions::new(&source);
    let end = positions
        .index_at_position16(position_to_position16(cursor))
        .unwrap_or_else(|| source.chars().count());
    let start = end.saturating_sub(10);
    let take = end - start;
    let source_before: String = source.chars().skip(start).take(take).collect();

    // Dispatch based on source before cursor
    if source_before.ends_with('@') {
        return assignee_completion().await;
    }

    Ok(None)
}

/// Provide completion list for assignees of an instruction
async fn assignee_completion() -> Result<Option<CompletionResponse>, ResponseError> {
    let items = assistants::list()
        .await
        .iter()
        .filter_map(|assistant| {
            let Assistant {
                id: Some(id),
                name,
                version,
                description,
                ..
            } = assistant
            else {
                return None;
            };

            // This attempts to maintain consistency with the symbols used for
            // `DocumentSymbols` for various node types
            let kind = if id.contains("code") {
                CompletionItemKind::EVENT
            } else if id.contains("math") {
                CompletionItemKind::OPERATOR
            } else if id.contains("styled") {
                CompletionItemKind::COLOR
            } else if id.contains("table") {
                CompletionItemKind::STRUCT
            } else if id.contains("block") {
                CompletionItemKind::CONSTRUCTOR
            } else {
                CompletionItemKind::INTERFACE
            };

            let label = if let Some(id) = id.strip_prefix("stencila/") {
                id.to_string()
            } else {
                id.to_string()
            };

            let version = match version {
                StringOrNumber::String(version) => version.to_string(),
                StringOrNumber::Number(version) => version.to_string(),
            };

            let detail = Some(format!("{} v{}", name, version));

            let documentation = Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: description.to_string(),
            }));

            Some(CompletionItem {
                kind: Some(kind),
                label,
                detail,
                documentation,
                ..Default::default()
            })
        })
        .collect();

    Ok(Some(CompletionResponse::Array(items)))
}
