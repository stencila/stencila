//! Handling of hover requests
//!
//! https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_completion

use std::sync::Arc;

use async_lsp::{
    lsp_types::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind},
    ResponseError,
};

use codec_markdown_trait::{MarkdownCodec, MarkdownEncodeContext};
use codecs::Format;
use common::tokio::sync::RwLock;
use document::Document;
use node_find::find;
use schema::{CodeChunk, CodeExpression, Node};

use crate::text_document::TextNode;

/// Handle a request for a hover display over a position of the document
pub(super) async fn request(
    params: HoverParams,
    doc: Arc<RwLock<Document>>,
    root: Arc<RwLock<TextNode>>,
) -> Result<Option<Hover>, ResponseError> {
    // Check if there is a node at the position
    let Some(text_node) = root
        .read()
        .await
        .text_node_at(params.text_document_position_params.position)
    else {
        return Ok(None);
    };

    // Check if it has any outputs
    if text_node
        .execution
        .and_then(|execution| execution.outputs)
        .unwrap_or_default()
        == 0
    {
        return Ok(None);
    };

    // Find the node in the document
    let doc = doc.read().await;
    let root = doc.root_read().await;
    let Some(node) = find(&*root, text_node.node_id) else {
        return Ok(None);
    };

    // Transform its outputs to Markdown
    let Some(markdown) = (match node {
        Node::CodeChunk(node) => code_chunk(node),
        Node::CodeExpression(node) => code_expression(node),
        _ => None,
    }) else {
        return Ok(None);
    };

    let contents = HoverContents::Markup(MarkupContent {
        kind: MarkupKind::Markdown,
        value: markdown,
    });

    Ok(Some(Hover {
        contents,
        range: None,
    }))
}

/// Render the outputs of a code chunk as Markdown
fn code_chunk(node: CodeChunk) -> Option<String> {
    let outputs = node.outputs?;

    if outputs.is_empty() {
        return None;
    };

    let mut context = MarkdownEncodeContext::new(Some(Format::Markdown), Some(true));
    for (index, output) in outputs.iter().enumerate() {
        if index > 0 {
            context.push_str("\n\n---\n\n");
        }
        match output {
            Node::String(string) => {
                // Use a code block to preserve whitespace
                context.push_str("```\n").push_str(string);
                if !string.ends_with("\n") {
                    context.push_str("\n");
                };
                context.push_str("```\n\n");
            }
            Node::ImageObject(image) => {
                // Create an image tag with 240px height which the the maximum height
                // which does not require use of the vertical scroll bar
                // (the height of the hover is 250px max and there are margins around the figure)
                context
                    .push_str(r#"<img src=""#)
                    .push_str(&image.content_url)
                    .push_str(r#"" height="240px">"#);
            }
            _ => output.to_markdown(&mut context),
        }
    }
    Some(context.content)
}

/// Render the output of a code expression as Markdown
fn code_expression(node: CodeExpression) -> Option<String> {
    let output = node.output?;

    let mut context = MarkdownEncodeContext::new(Some(Format::Markdown), Some(true));
    output.to_markdown(&mut context);
    Some(context.content)
}
