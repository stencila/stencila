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
use common::{inflector::Inflector, tokio::sync::RwLock};
use schema::Assistant;

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
        .map(|assistant| {
            let Assistant {
                name,
                version,
                description,
                ..
            } = assistant;

            // This attempts to maintain consistency with the symbols used for
            // `DocumentSymbols` for various node types
            let kind = if name.contains("code") {
                CompletionItemKind::EVENT
            } else if name.contains("math") {
                CompletionItemKind::OPERATOR
            } else if name.contains("styled") {
                CompletionItemKind::COLOR
            } else if name.contains("table") {
                CompletionItemKind::STRUCT
            } else if name.contains("block") {
                CompletionItemKind::CONSTRUCTOR
            } else {
                CompletionItemKind::INTERFACE
            };

            let label = if let Some(name) = name.strip_prefix("stencila/") {
                name.to_string()
            } else {
                name.to_string()
            };

            let detail = Some(format!("{} v{}", name.to_title_case(), version));

            let documentation = Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: description.to_string(),
            }));

            CompletionItem {
                kind: Some(kind),
                label,
                detail,
                documentation,
                ..Default::default()
            }
        })
        .collect();

    Ok(Some(CompletionResponse::Array(items)))
}
