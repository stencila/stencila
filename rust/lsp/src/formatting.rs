//! Handling of formatting related messages
//!
//! See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_formatting

use std::sync::Arc;

use async_lsp::{
    lsp_types::{Position, Range, TextEdit},
    ErrorCode, ResponseError,
};

use codecs::{EncodeOptions, Format};
use common::{tokio::sync::RwLock, tracing};
use document::Document;

/// Handle to format a document
#[tracing::instrument(skip(doc))]
pub(crate) async fn request(
    doc: Arc<RwLock<Document>>,
) -> Result<Option<Vec<TextEdit>>, ResponseError> {
    Ok(Some(vec![format_doc(doc, Format::Markdown).await?]))
}

// Create a text edit to replace the whole text document
pub(crate) async fn format_doc(
    doc: Arc<RwLock<Document>>,
    format: Format,
) -> Result<TextEdit, ResponseError> {
    let formatted = match doc
        .read()
        .await
        .export(
            None,
            Some(EncodeOptions {
                format: Some(format.clone()),
                ..Default::default()
            }),
        )
        .await
    {
        Ok(content) => content,
        Err(error) => {
            let message = format!("When encoding document to {format}: {error}");
            tracing::error!("{message}");
            return Err(ResponseError::new(ErrorCode::INTERNAL_ERROR, message));
        }
    };

    let edit = TextEdit::new(
        Range::new(Position::new(0, 0), Position::new(u32::MAX, 0)),
        formatted,
    );

    Ok(edit)
}
