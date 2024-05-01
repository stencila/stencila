//! Handling of completion related messages
//!
//! https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_completion

use async_lsp::{
    lsp_types::{
        CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse,
        CompletionTriggerKind, Documentation, MarkupContent, MarkupKind,
    },
    ResponseError,
};

use common::itertools::Itertools;

pub(super) async fn request(
    params: CompletionParams,
    source_before: Option<String>,
) -> Result<Option<CompletionResponse>, ResponseError> {
    // Get the trigger for the completion
    let trigger_kind = params.context.as_ref().map(|context| context.trigger_kind);
    let trigger_character = params.context.and_then(|context| context.trigger_character);

    if (Some(CompletionTriggerKind::TRIGGER_CHARACTER), Some("@"))
        == (trigger_kind, trigger_character.as_deref())
        || source_before
            .map(|source| source.ends_with('@'))
            .unwrap_or_default()
    {
        return assignee_completion().await;
    }

    Ok(None)
}

/// Provide completion list for assignees of an instruction
async fn assignee_completion() -> Result<Option<CompletionResponse>, ResponseError> {
    let items = assistants::list(false)
        .await
        .iter()
        // Filter out the generic assistants and this that are not available
        .filter(|assistant| assistant.preference_rank() > 0 && assistant.is_available())
        // Sort by descending order of preference rank
        .sorted_by(|a, b| a.preference_rank().cmp(&b.preference_rank()).reverse())
        .map(|assistant| {
            let name = assistant.name();

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
                name
            };

            let detail = Some(format!("{} v{}", assistant.title(), assistant.version()));

            let documentation = assistant.description().map(|desc| {
                Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: desc,
                })
            });

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
