//! Handling of code lens related messages
//!
//! See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_codeLens

use std::sync::Arc;

use async_lsp::{
    lsp_types::{CodeLens, Command, Url},
    ErrorCode, ResponseError,
};
use common::{inflector::Inflector, itertools::Itertools, serde_json::json, tokio::sync::RwLock};
use schema::{ExecutionMode, NodeType, ProvenanceCategory};

use crate::{
    commands::{
        ACCEPT_NODE, CANCEL_NODE, CHOOSE_NODE, HIDE_SUGGESTIONS_NODE, LOCK_EXEC, REJECT_NODE,
        RUN_NODE, SHOW_SUGGESTIONS_NODE, UNLOCK_EXEC,
    },
    text_document::TextNode,
};

// Preview node. Command implemented on the client
pub(super) const VIEW_NODE: &str = "stencila.view-doc";

// Do not run a command, just display provenance
pub(super) const PROV_NODE: &str = "";

/// Handle a request for code lenses for a document
///
/// Note that, as recommended for performance reasons, this function returns
/// code lenses without a `command` but with `data` which is used to "resolve"
/// the command in the `resolve` handler below
pub(crate) async fn request(
    uri: Url,
    root: Arc<RwLock<TextNode>>,
) -> Result<Option<Vec<CodeLens>>, ResponseError> {
    let code_lenses = root
        .read()
        .await
        .flatten()
        .flat_map(
            |TextNode {
                 range,
                 parent_type,
                 parent_id,
                 node_type,
                 node_id,
                 provenance,
                 execution,
                 ..
             }| {
                let lens = |command: &str| CodeLens {
                    range: *range,
                    command: None,
                    data: Some(json!([command, uri, node_type, node_id])),
                };

                let mut lenses = match node_type {
                    // Executable block nodes
                    NodeType::CallBlock
                    | NodeType::CodeChunk
                    | NodeType::ForBlock
                    | NodeType::IfBlock
                    | NodeType::IncludeBlock => {
                        // It would be nice to show/hide the run and cancel buttons
                        // based on the execution status of the node but doing this
                        // while avoiding race conditions is difficult.
                        // TODO: A cancel lens is not provided because this is currently
                        // not fully implemented
                        let execution_mode = execution.as_ref().and_then(|exec| exec.mode.as_ref());
                        vec![
                            lens(RUN_NODE),
                            if matches!(execution_mode, Some(ExecutionMode::Locked)) {
                                lens(UNLOCK_EXEC)
                            } else {
                                lens(LOCK_EXEC)
                            },
                            lens(VIEW_NODE),
                        ]
                    }
                    NodeType::InstructionBlock => {
                        vec![
                            lens(RUN_NODE),
                            lens(HIDE_SUGGESTIONS_NODE),
                            lens(SHOW_SUGGESTIONS_NODE),
                            lens(VIEW_NODE),
                        ]
                    }
                    // Block suggestions
                    NodeType::SuggestionBlock => {
                        vec![
                            CodeLens {
                                range: *range,
                                command: None,
                                data: Some(json!([
                                    CHOOSE_NODE,
                                    uri,
                                    node_type,
                                    node_id,
                                    parent_id
                                ])),
                            },
                            lens(REJECT_NODE),
                            lens(VIEW_NODE),
                        ]
                    }
                    // Block suggestions
                    NodeType::InsertBlock | NodeType::ReplaceBlock | NodeType::DeleteBlock => {
                        vec![lens(ACCEPT_NODE), lens(REJECT_NODE), lens(VIEW_NODE)]
                    }
                    _ => vec![],
                };

                if let Some(provenance) = provenance {
                    // Only show provenance code lens for certain node types and for the
                    // machine written and not human edited categories (summed)
                    if !matches!(node_type, NodeType::InstructionBlock)
                        && !matches!(parent_type, NodeType::ListItem)
                    {
                        let percent = provenance.iter().fold(0u64, |sum, prov| {
                            use ProvenanceCategory::*;
                            if matches!(prov.provenance_category, Mw | MwMe | MwMv | MwMeMv) {
                                sum + prov.character_percent.unwrap_or_default()
                            } else {
                                sum
                            }
                        });

                        if percent > 0 {
                            lenses.push(CodeLens {
                                range: *range,
                                command: None,
                                data: Some(json!([
                                    PROV_NODE,
                                    uri,
                                    node_type,
                                    node_id,
                                    format!("$(hubot) {percent}%")
                                ])),
                            });
                        }
                    }
                }

                lenses
            },
        )
        .collect();

    Ok(Some(code_lenses))
}

/// Handle a request to resolve the command for a code lens
pub(crate) async fn resolve(
    CodeLens { range, data, .. }: CodeLens,
) -> Result<CodeLens, ResponseError> {
    let Some(mut data) = data
        .as_ref()
        .and_then(|value| value.as_array())
        .map(|array| array.iter().filter_map(|value| value.as_str()))
    else {
        return Err(ResponseError::new(
            ErrorCode::INVALID_REQUEST,
            "No, or invalid, code lens data",
        ));
    };

    let Some((command, uri, node_type, node_id)) = data.next_tuple() else {
        return Err(ResponseError::new(
            ErrorCode::INVALID_REQUEST,
            "Expected three items in code lens data",
        ));
    };

    let command = command.to_string();
    let mut arguments = Some(vec![json!(uri), json!(node_type), json!(node_id)]);

    let command = match command.as_str() {
        RUN_NODE => Command::new("$(run) Run".to_string(), command, arguments),
        LOCK_EXEC => Command::new("$(lock) Lock".to_string(), command, arguments),
        UNLOCK_EXEC => Command::new("$(unlock) Unlock".to_string(), command, arguments),
        CANCEL_NODE => Command::new("$(stop-circle) Cancel".to_string(), command, arguments),
        HIDE_SUGGESTIONS_NODE => Command::new(
            "$(eye-closed) Hide suggestion".to_string(),
            command,
            arguments,
        ),
        SHOW_SUGGESTIONS_NODE => {
            Command::new("$(eye) Show suggestions".to_string(), command, arguments)
        }
        CHOOSE_NODE => {
            if let (Some(arguments), Some(parent_id)) = (arguments.as_mut(), data.next()) {
                arguments.push(json!(parent_id));
            }
            Command::new("$(thumbsup) Accept".to_string(), command, arguments)
        }
        ACCEPT_NODE => Command::new("$(thumbsup) Accept".to_string(), command, arguments),
        REJECT_NODE => Command::new("$(thumbsdown) Reject".to_string(), command, arguments),
        VIEW_NODE => Command::new("$(preview) View".to_string(), command, arguments),
        PROV_NODE => Command::new(
            data.next().unwrap_or_default().to_string(),
            command,
            arguments,
        ),
        _ => Command::new(
            command.replace("stencila.", "").to_title_case(),
            command,
            arguments,
        ),
    };

    Ok(CodeLens {
        range,
        command: Some(command),
        data: None,
    })
}
