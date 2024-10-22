//! Handling of lifecycle related messages
//!
//! See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#lifeCycleMessages

use std::{ops::ControlFlow, process};

use async_lsp::{
    lsp_types::{
        CodeLensOptions, CompletionOptions, DocumentSymbolOptions, ExecuteCommandOptions,
        HoverProviderCapability, InitializeResult, InitializedParams, MessageType, OneOf,
        ServerCapabilities, ServerInfo, ShowMessageParams, TextDocumentSyncCapability,
        TextDocumentSyncKind, WorkDoneProgressOptions,
    },
    Error, LanguageClient, ResponseError,
};

use common::serde_json;

use crate::{commands, ServerState, ServerStatus};

pub const STENCILA_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize options
pub(super) fn initialize_options(state: &mut ServerState, options: serde_json::Value) {
    if let Err(error) = state.options.initialize(options) {
        state
            .client
            .show_message(ShowMessageParams {
                typ: MessageType::ERROR,
                message: format!("Error initializing config options: {error}"),
            })
            .ok();
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
            hover_provider: Some(HoverProviderCapability::Simple(true)),
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

/// Shutdown the language server
///
/// Currently does nothing except change the status of the server.
pub(super) fn shutdown(state: &mut ServerState) -> Result<(), ResponseError> {
    state.status = ServerStatus::Shutdown;

    Ok(())
}

/// Exit the language server
///
/// This exits with a code 0 or 1 as per https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#exit
pub(super) fn exit(state: &mut ServerState) -> ControlFlow<Result<(), Error>> {
    let code = match state.status {
        ServerStatus::Running => 1,
        ServerStatus::Shutdown => 0,
    };
    process::exit(code);

    #[allow(unreachable_code)]
    ControlFlow::Break(Ok(()))
}
