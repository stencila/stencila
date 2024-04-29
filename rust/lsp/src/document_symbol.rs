//! Handling of document symbols messages
//!
//! See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_documentSymbol

use async_lsp::{
    lsp_types::{DocumentSymbol, DocumentSymbolResponse, SymbolKind},
    ResponseError,
};
use schema::NodeType;

use crate::text_document::TextNode;

/// Handle a request for code lenses for a document
///
/// Note that, as recommended for performance reasons, this function returns
/// code lenses without a `command` but with `data` which is used to "resolve"
/// the command in the `resolve` handler below
pub(crate) async fn request(
    root: &TextNode,
) -> Result<Option<DocumentSymbolResponse>, ResponseError> {
    let symbols = symbol(root).children.unwrap_or_default();

    Ok(Some(DocumentSymbolResponse::Nested(symbols)))
}

/// Create a [`DocumentSymbol`] for a [`TextNode`]
fn symbol(node: &TextNode) -> DocumentSymbol {
    let kind = match node.node_type {
        // Primitive node types
        NodeType::Null => SymbolKind::NULL,
        NodeType::Boolean => SymbolKind::BOOLEAN,
        NodeType::Integer => SymbolKind::NUMBER,
        NodeType::Number => SymbolKind::NUMBER,
        NodeType::String => SymbolKind::STRING,
        NodeType::Array => SymbolKind::ARRAY,
        NodeType::Object => SymbolKind::OBJECT,

        // Executable node types
        NodeType::CodeChunk | NodeType::CodeExpression => SymbolKind::EVENT,
        NodeType::IfBlock => SymbolKind::CLASS,
        NodeType::ForBlock => SymbolKind::ENUM,
        NodeType::Parameter => SymbolKind::VARIABLE,

        // No executable node types
        NodeType::Paragraph => SymbolKind::STRING,
        NodeType::CodeBlock | NodeType::CodeInline => SymbolKind::OBJECT,
        NodeType::MathBlock | NodeType::MathInline => SymbolKind::OPERATOR,
        NodeType::Table => SymbolKind::STRUCT,

        _ => SymbolKind::CONSTRUCTOR,
    };

    let range = node.range.clone();
    let selection_range = range.clone();

    let children: Vec<DocumentSymbol> = node.children.iter().map(symbol).collect();
    let children = if children.is_empty() {
        None
    } else {
        Some(children)
    };

    DocumentSymbol {
        name: node.node_type.to_string(),
        kind,
        detail: None,
        tags: None,
        deprecated: None,
        range,
        selection_range,
        children,
    }
}
