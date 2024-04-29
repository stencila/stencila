//! Handling of lifecycle related messages
//!
//! See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#lifeCycleMessages

use std::ops::ControlFlow;

use async_lsp::{
    lsp_types::{
        CodeLensOptions, DocumentSymbolOptions, ExecuteCommandOptions, InitializeParams,
        InitializeResult, InitializedParams, OneOf, ServerCapabilities, ServerInfo,
        TextDocumentSyncCapability, TextDocumentSyncKind, WorkDoneProgressOptions,
    },
    Error, ResponseError,
};

use crate::{commands, ServerState};

pub const STENCILA_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the language server and respond with its capabilities
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
            document_symbol_provider: Some(OneOf::Right(DocumentSymbolOptions {
                label: Some("Nodes".to_string()),
                work_done_progress_options: WorkDoneProgressOptions {
                    work_done_progress: None,
                },
            })),
            code_lens_provider: Some(CodeLensOptions {
                resolve_provider: Some(true),
            }),
            execute_command_provider: Some(ExecuteCommandOptions {
                commands: commands::commands(),
                ..Default::default()
            }),
            ..ServerCapabilities::default()
        },
    })
}

/// Handle the notification from the client that the connection has been initialized
pub(super) fn initialized(
    _state: &mut ServerState,
    _params: InitializedParams,
) -> ControlFlow<Result<(), Error>> {
    ControlFlow::Continue(())
}
