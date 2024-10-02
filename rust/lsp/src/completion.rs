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

    // Get the source before the cursor
    let source = source.read().await;
    let cursor = params.text_document_position.position;
    let positions = Positions::new(&source);
    let end = positions
        .index_at_position16(position_to_position16(cursor))
        .unwrap_or_else(|| source.chars().count());
    let start = end.saturating_sub(16);
    let take = end - start;
    let source_before: String = source.chars().skip(start).take(take).collect();

    // Dispatch based on source before cursor
    if source_before.ends_with("::: prompt ")
        || source_before.ends_with('@')
        || (trigger_kind == CompletionTriggerKind::TRIGGER_CHARACTER && trigger_character == "@")
    {
        return prompt_completion(&source_before).await;
    }

    Ok(None)
}

/// Provide completion list for prompts of an instruction
async fn prompt_completion(before: &str) -> Result<Option<CompletionResponse>, ResponseError> {
    let itype = if before.contains("::: new") {
        Some(InstructionType::New)
    } else if before.contains("::: edit") {
        Some(InstructionType::Edit)
    } else if before.contains("::: fix") {
        Some(InstructionType::Fix)
    } else if before.contains("::: describe") {
        Some(InstructionType::Describe)
    } else if before.contains("::: prompt") {
        None
    } else {
        return Ok(None);
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

            if let Some(itype) = &itype {
                if !instruction_types.contains(itype) {
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

            // If this is a Stencila prompt then strip the redundant prefix
            let label = if let Some(itype) = &itype {
                let stencila_prefix =
                    ["stencila/", &itype.to_string().to_lowercase(), "/"].concat();
                if let Some(id) = id.strip_prefix(&stencila_prefix) {
                    id.to_string()
                } else {
                    id.to_string()
                }
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
