//! Handling of text document synchronization related messages
//!
//! See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_synchronization

use std::ops::ControlFlow;

use async_lsp::{
    lsp_types::{
        DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
        DidSaveTextDocumentParams,
    },
    Error,
};

use common::tracing;

use crate::ServerState;

pub(crate) fn did_open(
    _state: &mut ServerState,
    params: DidOpenTextDocumentParams,
) -> ControlFlow<Result<(), Error>> {
    tracing::info!(
        "did_open: {} {}",
        params.text_document.uri,
        params.text_document.language_id
    );

    // TODO: Create a new document using text

    ControlFlow::Continue(())
}

pub(super) fn did_change(
    _state: &mut ServerState,
    params: DidChangeTextDocumentParams,
) -> ControlFlow<Result<(), Error>> {
    tracing::info!("did_change: {}", params.text_document.uri);

    // TODO: Change document source

    ControlFlow::Continue(())
}

pub(super) fn did_save(
    _state: &mut ServerState,
    params: DidSaveTextDocumentParams,
) -> ControlFlow<Result<(), Error>> {
    tracing::info!("did_save: {}", params.text_document.uri);

    ControlFlow::Continue(())
}

pub(super) fn did_close(
    _state: &mut ServerState,
    params: DidCloseTextDocumentParams,
) -> ControlFlow<Result<(), Error>> {
    tracing::info!("did_close: {}", params.text_document.uri);

    // TODO: Remove document

    ControlFlow::Continue(())
}
