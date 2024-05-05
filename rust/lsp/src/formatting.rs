//! Handling of formatting related messages
//!
//! See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_formatting

use std::sync::Arc;

use async_lsp::{
    lsp_types::{MessageType, Position, Range, ShowMessageParams, TextEdit},
    ClientSocket, LanguageClient, ResponseError,
};

use codecs::{EncodeOptions, Format};
use common::{tokio::sync::RwLock, tracing};
use document::Document;

/// Handle to format a document
#[tracing::instrument(skip(doc))]
pub(crate) async fn request(
    doc: Arc<RwLock<Document>>,
    mut client: ClientSocket,
) -> Result<Option<Vec<TextEdit>>, ResponseError> {
    let format = Format::Markdown;

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
            client
                .show_message(ShowMessageParams {
                    typ: MessageType::ERROR,
                    message,
                })
                .ok();
            return Ok(None);
        }
    };

    // Create a text edit to replace the whole document
    let edit = TextEdit::new(
        Range::new(Position::new(0, 0), Position::new(u32::MAX, 0)),
        formatted,
    );

    Ok(Some(vec![edit]))
}
