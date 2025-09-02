//! Handling of lifecycle related messages
//!
//! See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#lifeCycleMessages

use std::{ops::ControlFlow, process};

use async_lsp::{
    ClientSocket, Error, ErrorCode, LanguageClient, ResponseError,
    lsp_types::{
        CodeLensOptions, CompletionOptions, DocumentSymbolOptions, ExecuteCommandOptions,
        HoverProviderCapability, InitializeResult, InitializedParams, MessageActionItem,
        MessageType, OneOf, ServerCapabilities, ServerInfo, ShowMessageParams,
        ShowMessageRequestParams, TextDocumentSyncCapability, TextDocumentSyncKind,
        WorkDoneProgressOptions,
    },
};
use async_trait::async_trait;
use eyre::{Result, bail};
use tokio::sync::Mutex;

use crate::{ServerState, ServerStatus, commands};

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
#[tracing::instrument]
pub(super) async fn initialize(client: ClientSocket) -> Result<InitializeResult, ResponseError> {
    tracing::debug!("Initializing language server connection");

    stencila_ask::setup_lsp(AskClient::new(client))
        .await
        .map_err(|error| ResponseError::new(ErrorCode::INTERNAL_ERROR, error.to_string()))?;

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
                trigger_characters: Some(vec![
                    ".".into(),
                    "/".into(),
                    ":".into(),
                    "@".into(),
                    "[".into(),
                    "(".into(),
                    ",".into(),
                    "`".into(),
                    " ".into(),
                ]),
                resolve_provider: Some(true),
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
#[tracing::instrument]
pub(super) fn initialized(
    state: &mut ServerState,
    params: InitializedParams,
) -> ControlFlow<Result<(), Error>> {
    tracing::debug!("Language server connection initialized");

    ControlFlow::Continue(())
}

/// Shutdown the language server
///
/// Currently does nothing except change the status of the server.
#[tracing::instrument]
pub(super) fn shutdown(state: &mut ServerState) -> Result<(), ResponseError> {
    tracing::debug!("Shutting down language server");

    state.status = ServerStatus::Shutdown;

    Ok(())
}

/// Exit the language server
///
/// This exits with a code 0 or 1 as per https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#exit
#[tracing::instrument]
pub(super) fn exit(state: &mut ServerState) -> ControlFlow<Result<(), Error>> {
    tracing::debug!("Exiting language server");

    let code = match state.status {
        ServerStatus::Running => 1,
        ServerStatus::Shutdown => 0,
    };
    process::exit(code);

    #[allow(unreachable_code)]
    ControlFlow::Break(Ok(()))
}

/// Client for asking user questions in the editor
struct AskClient {
    client: Mutex<ClientSocket>,
}

impl AskClient {
    fn new(client: ClientSocket) -> Self {
        Self {
            client: Mutex::new(client),
        }
    }
}

#[async_trait]
impl stencila_ask::LspClient for AskClient {
    async fn show_message_request(
        &self,
        params: ShowMessageRequestParams,
    ) -> Result<Option<MessageActionItem>> {
        Ok(self
            .client
            .lock()
            .await
            .show_message_request(params)
            .await?)
    }

    async fn request_password_input(&self, _prompt: &str) -> Result<Option<String>> {
        // TODO: VSCode extension to handle custom notification/request for password input
        bail!("Password input via editor is not yet implemented.")
    }
}
