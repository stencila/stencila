//! Handling of code lens related messages
//!
//! See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_codeLens

use std::sync::Arc;

use async_lsp::{
    lsp_types::{CodeLens, Command, Range, Url},
    ErrorCode, ResponseError,
};
use common::{inflector::Inflector, itertools::Itertools, serde_json::json, tokio::sync::RwLock};
use schema::{ExecutionStatus, NodeType};

use crate::{
    commands::{
        ARCHIVE_NODE, CANCEL_NODE, NEXT_NODE, PATCH_VALUE, PREV_NODE, REVISE_NODE, RUN_NODE,
        VERIFY_NODE,
    },
    text_document::TextNode,
};

/// Lens to view the node. Command implemented on the client
pub(super) const VIEW_NODE: &str = "stencila.view-node";

/// Lens to show the provenance of the node and view it's authors when when clicked.
pub(super) const PROV_NODE: &str = "stencila.view-node-authors";

/// Lens to show the index of the current item in a collection
/// (e.g. active suggestion index ins suggestions for and instruction)
pub(super) const INDEX_OF: &str = "stencila.index-of";

/// Lens to continue a walkthrough
pub(super) const WALKTHROUGH_CONTINUE: &str = "stencila.walkthroughs.continue";

/// Lens to expand a walkthrough
pub(super) const WALKTHROUGH_EXPAND: &str = "stencila.walkthroughs.expand";

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
                 index_of,
                 is_active,
                 provenance,
                 detail,
                 execution,
                 ..
             }| {
                // Do not show lenses for nodes that are not encoded into the document
                // (i.e. those that have a default range)
                if *range == Range::default() {
                    return Vec::new();
                }

                let lens = |command: &str| CodeLens {
                    range: *range,
                    command: None,
                    data: Some(json!([command, uri, node_type, node_id])),
                };
                let lens_parent = |command: &str| CodeLens {
                    range: *range,
                    command: None,
                    data: Some(json!([command, uri, parent_type, parent_id])),
                };
                let lens_index_of = |index: &usize, of: &usize| CodeLens {
                    range: *range,
                    command: None,
                    data: Some(json!([
                        INDEX_OF,
                        uri,
                        node_type,
                        node_id,
                        if *index == 0 {
                            "Original".to_string()
                        } else {
                            format!("{} of {}", index, of)
                        }
                    ])),
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
                        let mut lenses = vec![lens(RUN_NODE), lens(VIEW_NODE)];
                        if let Some((index, of)) = index_of {
                            lenses.append(&mut vec![
                                lens(PREV_NODE),
                                lens_index_of(index, of),
                                lens(NEXT_NODE),
                                lens(ARCHIVE_NODE),
                            ]);
                            if *index > 0 {
                                lenses.push(lens(REVISE_NODE));
                            }
                        }
                        lenses
                    }
                    NodeType::MathBlock | NodeType::RawBlock | NodeType::StyledBlock => {
                        vec![lens(VIEW_NODE)]
                    }
                    NodeType::WalkthroughStep => {
                        if matches!(is_active, Some(true)) {
                            vec![]
                        } else {
                            vec![lens(WALKTHROUGH_CONTINUE), lens_parent(WALKTHROUGH_EXPAND)]
                        }
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
                            if (prov.provenance_category.is_machine_written()
                                && !prov.provenance_category.is_human_edited())
                                || prov.provenance_category.is_machine_edited()
                            {
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
                                        "$(hubot) {machine_percent}%  $(verified) {verified_percent}%"
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
    let arguments = Some(vec![json!(uri), json!(node_type), json!(node_id)]);

    let command = match command.as_str() {
        RUN_NODE => Command::new("$(run) Run".to_string(), command, arguments),
        CANCEL_NODE => Command::new("$(stop-circle) Cancel".to_string(), command, arguments),
        PREV_NODE | NEXT_NODE => {
            let title = match command.as_str() {
                PREV_NODE => "$(arrow-left)",
                NEXT_NODE => "$(arrow-right)",
                _ => unreachable!(),
            }
            .to_string();

            Command::new(title, command, arguments)
        }
        INDEX_OF => Command::new(
            data.next().unwrap_or_default().to_string(),
            String::new(),
            None,
        ),
        ARCHIVE_NODE => {
            let title = match node_type {
                "InstructionBlock" => "$(pass) Accept".to_string(),
                _ => "$(archive) Archive".to_string(),
            };
            Command::new(title, command, arguments)
        }
        REVISE_NODE => Command::new(
            "$(refresh) Revise".to_string(),
            // Call corresponding `invoke` command on the client to collect any feedback from user
            "stencila.invoke.revise-node".to_string(),
            arguments,
        ),
        VIEW_NODE => Command::new("$(preview) View".to_string(), command, arguments),
        VERIFY_NODE => Command::new("$(verified) Verify".to_string(), command, arguments),
        PROV_NODE => Command::new(
            data.next().unwrap_or_default().to_string(),
            command,
            arguments,
        ),
        WALKTHROUGH_CONTINUE => Command::new(
            "$(arrow-right) Next".to_string(),
            PATCH_VALUE.to_string(),
            Some(vec![
                json!(uri),
                json!(node_type),
                json!(node_id),
                json!("isCollapsed"),
                json!(false),
            ]),
        ),
        WALKTHROUGH_EXPAND => Command::new(
            "$(chevron-down) Expand all".to_string(),
            PATCH_VALUE.to_string(),
            Some(vec![
                json!(uri),
                json!(node_type),
                json!(node_id),
                json!("isCollapsed"),
                json!(false),
            ]),
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
