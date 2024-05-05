//! Handling of code lens related messages
//!
//! See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_codeLens

use std::sync::Arc;

use async_lsp::{
    lsp_types::{CodeLens, Command, Url},
    ErrorCode, ResponseError,
};
use common::{itertools::Itertools, serde_json::json, tokio::sync::RwLock};
use schema::NodeType;

use crate::{
    commands::{ACCEPT_NODE, CANCEL_NODE, INSPECT_NODE, REJECT_NODE, RUN_NODE, VIEW_NODE},
    text_document::TextNode,
};

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
                 node_type,
                 node_id,
                 provenance,
                 ..
             }| {
                let lens = |command: &str| CodeLens {
                    range: *range,
                    command: None,
                    data: Some(json!([command, uri, node_id])),
                };

                let mut lenses = match node_type {
                    // Executable block nodes
                    NodeType::CallBlock
                    | NodeType::CodeChunk
                    | NodeType::ForBlock
                    | NodeType::IfBlock
                    | NodeType::IncludeBlock
                    | NodeType::InstructionBlock => {
                        vec![lens(RUN_NODE), lens(CANCEL_NODE), lens(VIEW_NODE)]
                    }
                    // Block suggestions
                    NodeType::InsertBlock | NodeType::ReplaceBlock | NodeType::DeleteBlock => {
                        vec![lens(ACCEPT_NODE), lens(REJECT_NODE), lens(VIEW_NODE)]
                    }
                    _ => vec![],
                };

                if let Some(provenance) = provenance {
                    let desc = provenance
                        .iter()
                        .filter_map(|count| {
                            count.character_percent.map(|percent| {
                                format!("{}:{}%", count.provenance_category, percent)
                            })
                        })
                        .join(" ");
                    lenses.push(CodeLens {
                        range: *range,
                        command: None,
                        data: Some(json!([INSPECT_NODE, uri, node_id, desc])),
                    });
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

    let Some((command, uri, node_id)) = data.next_tuple() else {
        return Err(ResponseError::new(
            ErrorCode::INVALID_REQUEST,
            "Expected three items in code lens data",
        ));
    };

    let arguments = Some(vec![json!(uri), json!(node_id)]);

    let command = match command {
        RUN_NODE => Command::new(
            "$(run) Run".to_string(),
            "stencila.run-node".to_string(),
            arguments,
        ),
        CANCEL_NODE => Command::new(
            "$(stop-circle) Cancel".to_string(),
            "stencila.cancel-node".to_string(),
            arguments,
        ),
        ACCEPT_NODE => Command::new(
            "$(thumbsup) Accept".to_string(),
            "stencila.accept-node".to_string(),
            arguments,
        ),
        REJECT_NODE => Command::new(
            "$(thumbsdown) Reject".to_string(),
            "stencila.reject-node".to_string(),
            arguments,
        ),
        VIEW_NODE => Command::new(
            "$(eye) View".to_string(),
            "stencila.view-node".to_string(),
            arguments,
        ),
        INSPECT_NODE => Command::new(
            format!("$(search) {prov}", prov = data.next().unwrap_or_default()),
            "stencila.inspect-node".to_string(),
            arguments,
        ),
        _ => {
            return Err(ResponseError::new(
                ErrorCode::INVALID_REQUEST,
                "Unknown command for code lens",
            ));
        }
    };

    Ok(CodeLens {
        range,
        command: Some(command),
        data: None,
    })
}
