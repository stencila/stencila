//! Handling of lifecycle related messages
//!
//! See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#lifeCycleMessages

use std::ops::ControlFlow;

use async_lsp::{
    lsp_types::{
        InitializeParams, InitializeResult, InitializedParams, ServerCapabilities, ServerInfo,
        TextDocumentSyncCapability, TextDocumentSyncKind,
    },
    Error, ResponseError,
};

use crate::ServerState;

pub const STENCILA_VERSION: &str = env!("CARGO_PKG_VERSION");

pub(super) async fn initialize(
    _params: InitializeParams,
) -> Result<InitializeResult, ResponseError> {
    Ok(InitializeResult {
        server_info: Some(ServerInfo {
            name: "Stencila Language Server".to_string(),
            version: Some(STENCILA_VERSION.to_string()),
        }),
        capabilities: ServerCapabilities {
            text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
            ..ServerCapabilities::default()
        },
    })
}

pub(super) fn initialized(
    _state: &mut ServerState,
    _params: InitializedParams,
) -> ControlFlow<Result<(), Error>> {
    ControlFlow::Continue(())
}
