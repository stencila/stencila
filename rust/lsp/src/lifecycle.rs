//! Handling of lifecycle related messages
//!
//! See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#lifeCycleMessages

use std::ops::ControlFlow;

use async_lsp::{
    lsp_types::{
        CodeLensOptions, CompletionOptions, DocumentSymbolOptions, ExecuteCommandOptions,
        InitializeParams, InitializeResult, InitializedParams, MessageType, OneOf,
        ServerCapabilities, ServerInfo, ShowMessageParams, TextDocumentSyncCapability,
        TextDocumentSyncKind, WorkDoneProgressOptions,
    },
    Error, LanguageClient, ResponseError,
};

use common::serde_json;

use crate::{commands, ServerState};

pub const STENCILA_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize options
pub(super) fn initialize_options(state: &mut ServerState, options: serde_json::Value) {
    match serde_json::from_value(options) {
        Ok(options) => state.options = Some(options),
        Err(error) => {
            state
                .client
                .show_message(ShowMessageParams {
                    typ: MessageType::ERROR,
                    message: format!("Error parsing config options: {error}"),
                })
                .ok();
        }
    }
}

/// Initialize the language server and respond with its capabilities
pub(super) async fn initialize() -> Result<InitializeResult, ResponseError> {
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
            completion_provider: Some(CompletionOptions {
                trigger_characters: Some(vec!["@".to_string()]),
                ..Default::default()
            }),
            code_lens_provider: Some(CodeLensOptions {
                resolve_provider: Some(true),
            }),
            execute_command_provider: Some(ExecuteCommandOptions {
                commands: commands::commands(),
                ..Default::default()
            }),
            document_formatting_provider: Some(OneOf::Left(true)),
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
