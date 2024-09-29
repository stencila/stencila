//! Handling of code lens related messages
//!
//! See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_codeLens

use std::sync::Arc;

use async_lsp::{
    lsp_types::{CodeLens, Command, Url},
    ErrorCode, ResponseError,
};
use common::{inflector::Inflector, itertools::Itertools, serde_json::json, tokio::sync::RwLock};
use schema::NodeType;

use crate::{
    commands::{
        ACCEPT_NODE, ARCHIVE_NODE, CANCEL_NODE, REJECT_NODE, REVISE_NODE, RUN_NODE, VERIFY_NODE,
    },
    text_document::TextNode,
};

/// Lens to view the node. Command implemented on the client
pub(super) const VIEW_NODE: &str = "stencila.view-node";

/// Lens to show the provenance of the node and view it's authors when when clicked.
pub(super) const PROV_NODE: &str = "stencila.view-node-authors";

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
                 ..
             }| {
                let lens = |command: &str| CodeLens {
                    range: *range,
                    command: None,
                    data: Some(json!([command, uri, node_type, node_id])),
                };
                let lens_with_parent = |command: &str| CodeLens {
                    range: *range,
                    command: None,
                    data: Some(json!([command, uri, node_type, node_id, parent_id])),
                };

                let mut lenses = match node_type {
                    NodeType::CallBlock
                    | NodeType::CodeChunk
                    | NodeType::ForBlock
                    | NodeType::IfBlock
                    | NodeType::IncludeBlock
                    | NodeType::PromptBlock => {
                        // It would be nice to show/hide the run and cancel buttons
                        // based on the execution status of the node but doing this
                        // while avoiding race conditions is difficult.
                        // TODO: A cancel lens is not provided because this is currently
                        // not fully implemented
                        vec![lens(RUN_NODE), lens(VIEW_NODE)]
                    }
                    NodeType::InstructionBlock => {
                        vec![lens(RUN_NODE), lens(ARCHIVE_NODE), lens(VIEW_NODE)]
                    }
                    NodeType::SuggestionBlock => {
                        vec![
                            lens_with_parent(ACCEPT_NODE),
                            lens(REJECT_NODE),
                            lens(REVISE_NODE),
                            lens(VIEW_NODE),
                        ]
                    }
                    NodeType::InsertBlock | NodeType::ReplaceBlock | NodeType::DeleteBlock => {
                        vec![lens(ACCEPT_NODE), lens(REJECT_NODE), lens(VIEW_NODE)]
                    }
                    NodeType::MathBlock | NodeType::RawBlock | NodeType::StyledBlock => {
                        vec![lens(VIEW_NODE)]
                    }
                    _ => vec![],
                };

                if let Some(provenance) = provenance {
                    // Only show provenance code lens for certain node types
                    // (e.g. not for paragraphs or list in list items, or in captions of
                    // code chunks, tables etc)
                    if !(matches!(
                        node_type,
                        NodeType::InstructionBlock | NodeType::SuggestionBlock
                    ) || matches!(parent_type, NodeType::ListItem)
                        || (matches!(node_type, NodeType::Paragraph)
                            && matches!(
                                parent_type,
                                NodeType::CodeChunk | NodeType::Figure | NodeType::Table
                            )))
                    {
                        let machine_percent = provenance.iter().fold(0u64, |sum, prov| {
                            if prov.provenance_category.is_machine_written() {
                                sum + prov.character_percent.unwrap_or_default()
                            } else {
                                sum
                            }
                        });
                        let verified_percent = provenance.iter().fold(0u64, |sum, prov| {
                            if prov.provenance_category.is_verified() {
                                sum + prov.character_percent.unwrap_or_default()
                            } else {
                                sum
                            }
                        });

                        if machine_percent > 0 {
                            // Remove any existing VIEW_NODE lens because that can be done via
                            // the PROV_NODE lens that we're about to add
                            lenses.retain(|lens| {
                                let is_view_node = lens
                                    .data
                                    .as_ref()
                                    .and_then(|data| data.as_array())
                                    .and_then(|items| items.first())
                                    .and_then(|first| first.as_str())
                                    .map(|first| first == VIEW_NODE)
                                    .unwrap_or_default();
                                !is_view_node
                            });

                            // Add "Verify" lens if not yet fully verified
                            if verified_percent < 100 {
                                lenses.push(lens(VERIFY_NODE));
                            }

                            // Add provenance lens attributing authorship and showing
                            // percent verified
                            lenses.push(CodeLens {
                                range: *range,
                                command: None,
                                data: Some(json!([
                                    PROV_NODE,
                                    uri,
                                    node_type,
                                    node_id,
                                    format!(
                                        "$(hubot) {machine_percent}%  $(check) {verified_percent}%"
                                    )
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
        VERIFY_NODE => Command::new("$(pass) Verify".to_string(), command, arguments),
        RUN_NODE => Command::new("$(run) Run".to_string(), command, arguments),
        CANCEL_NODE => Command::new("$(stop-circle) Cancel".to_string(), command, arguments),
        ARCHIVE_NODE => Command::new("$(archive) Archive".to_string(), command, arguments),
        ACCEPT_NODE | REJECT_NODE | REVISE_NODE => {
            if let (Some(arguments), Some(parent_id)) = (arguments.as_mut(), data.next()) {
                arguments.push(json!(parent_id));
            }

            let title = match command.as_str() {
                ACCEPT_NODE => "$(thumbsup) Accept",
                REJECT_NODE => "$(thumbsdown) Reject",
                REVISE_NODE => "$(refresh) Revise",
                _ => unreachable!(),
            }
            .to_string();

            Command::new(title, command, arguments)
        }
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
