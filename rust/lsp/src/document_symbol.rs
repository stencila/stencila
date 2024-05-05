//! Handling of document symbols messages
//!
//! See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_documentSymbol

use std::sync::Arc;

use async_lsp::{
    lsp_types::{DocumentSymbol, DocumentSymbolResponse, SymbolKind},
    ResponseError,
};
use common::tokio::sync::RwLock;
use schema::NodeType;

use crate::text_document::TextNode;

/// Handle a request for document symbols
pub(crate) async fn request(
    root: Arc<RwLock<TextNode>>,
) -> Result<Option<DocumentSymbolResponse>, ResponseError> {
    let symbols = symbol(&*root.read().await)
        .and_then(|symbol| symbol.children)
        .unwrap_or_default();

    Ok(Some(DocumentSymbolResponse::Nested(symbols)))
}

/// Create a [`DocumentSymbol`] for a [`TextNode`]
fn symbol(node: &TextNode) -> Option<DocumentSymbol> {
    use NodeType::*;
    let kind = match node.node_type {
        // Primitive node types
        Null => SymbolKind::NULL,
        Boolean => SymbolKind::BOOLEAN,
        Integer => SymbolKind::NUMBER,
        Number => SymbolKind::NUMBER,
        String => SymbolKind::STRING,
        Array => SymbolKind::ARRAY,
        Object => SymbolKind::OBJECT,

        // Executable node types
        CodeChunk | CodeExpression | IfBlockClause => SymbolKind::EVENT,
        IfBlock => SymbolKind::CLASS,
        ForBlock => SymbolKind::ENUM,
        Parameter => SymbolKind::VARIABLE,

        // Non-executable node types
        Paragraph => SymbolKind::STRING,
        CodeBlock | CodeInline => SymbolKind::OBJECT,
        MathBlock | MathInline => SymbolKind::OPERATOR,
        Table => SymbolKind::STRUCT,

        // Skip generating symbols for table cells and text nodes
        // (of which there are likely to be many)
        TableCell | Text => return None,

        _ => SymbolKind::CONSTRUCTOR,
    };

    let detail = if let Some(detail) = &node.detail {
        const MAX_LEN: usize = 24;
        if detail.len() > MAX_LEN && MAX_LEN > 3 {
            Some(format!("{}...", &detail[..MAX_LEN - 3]))
        } else {
            Some(detail.to_string())
        }
    } else {
        None
    };

    let range = node.range;
    let selection_range = range;

    let children: Vec<DocumentSymbol> = node.children.iter().filter_map(symbol).collect();
    let children = if children.is_empty() {
        None
    } else {
        Some(children)
    };

    #[allow(deprecated)]
    Some(DocumentSymbol {
        name: node.node_type.to_string(),
        kind,
        detail,
        tags: None,
        range,
        selection_range,
        children,

        // Annoyingly this is deprecated but needs to be specified
        // because `DocumentSymbol` does not implement `Default`
        deprecated: None,
    })
}
