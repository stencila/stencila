use async_lsp::{
    lsp_types::{InitializeParams, InitializeResult, ServerCapabilities, ServerInfo},
    ResponseError,
};

use common::tracing;

pub const STENCILA_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Initialize the language server
/// 
/// See https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#initialize
#[tracing::instrument]
pub(crate) async fn initialize(
    params: InitializeParams,
) -> Result<InitializeResult, ResponseError> {
    tracing::debug!("Initialize");

    Ok(InitializeResult {
        server_info: Some(ServerInfo {
            name: "Stencila Language Server".to_string(),
            version: Some(STENCILA_VERSION.to_string()),
        }),
        capabilities: ServerCapabilities {
            ..ServerCapabilities::default()
        },
    })
}
