//! Handling of completion related messages
//!
//! https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_completion

use std::{ops::Deref, sync::Arc};

use async_lsp::{
    lsp_types::{
        CompletionItem, CompletionItemKind, CompletionParams, CompletionResponse,
        CompletionTriggerKind, Documentation, MarkupContent, MarkupKind,
    },
    ResponseError,
};

use codecs::Positions;
use common::tokio::sync::RwLock;
use schema::{InstructionType, Prompt, StringOrNumber};

use crate::utils::position_to_position16;

pub(super) async fn request(
    params: CompletionParams,
    source: Option<Arc<RwLock<String>>>,
) -> Result<Option<CompletionResponse>, ResponseError> {
    // Get the trigger for the completion
    let trigger_kind = params
        .context
        .as_ref()
        .map(|context| context.trigger_kind)
        .unwrap_or(CompletionTriggerKind::INVOKED);
    let trigger_character = params
        .context
        .and_then(|context| context.trigger_character)
        .unwrap_or_default();

    // Unable to proceed if no source available
    let Some(source) = source else {
        return Ok(None);
    };

    // Get the source before the cursor (up to start of line)
    let source = source.read().await;
    let position = position_to_position16(params.text_document_position.position);
    let positions = Positions::new(&source);
    let Some(end) = positions.index_at_position16(position.clone()) else {
        // Early return if the cursor position can not be resolve in current source
        return Ok(None);
    };
    let start = source[..end].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let take = end - start;
    let before: String = source.chars().skip(start).take(take).collect();

    // Dispatch based on source before cursor
    if before.starts_with(":::") && before.ends_with('@')
        || (trigger_kind == CompletionTriggerKind::TRIGGER_CHARACTER && trigger_character == "@")
    {
        return prompt_completion(&before).await;
    }

    Ok(None)
}

/// Provide completion list for prompts of an instruction
async fn prompt_completion(before: &str) -> Result<Option<CompletionResponse>, ResponseError> {
    let instruction_type = if before.contains("create ") {
        Some(InstructionType::Create)
    } else if before.contains("edit ") {
        Some(InstructionType::Edit)
    } else if before.contains("fix ") {
        Some(InstructionType::Fix)
    } else if before.contains("describe ") {
        Some(InstructionType::Describe)
    } else {
        None
    };

    let items = prompts::list()
        .await
        .iter()
        .filter_map(|prompt| {
            let Prompt {
                id: Some(id),
                name,
                version,
                description,
                instruction_types,
                ..
            } = prompt.deref()
            else {
                return None;
            };

            if let Some(instruction_type) = &instruction_type {
                if !instruction_types.contains(instruction_type) {
                    return None;
                }
            }

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

            let label = prompts::shorten(id, &instruction_type);

            let version = match version {
                StringOrNumber::String(version) => version.to_string(),
                StringOrNumber::Number(version) => version.to_string(),
            };

            let detail = Some(name.to_string());

            let documentation = Some(Documentation::MarkupContent(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("{description}\n\n{id} v{version}"),
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
