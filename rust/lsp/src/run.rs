use std::collections::HashMap;

use async_lsp::lsp_types::{notification, request};
use async_lsp::{
    client_monitor::ClientProcessMonitorLayer, concurrency::ConcurrencyLayer,
    panic::CatchUnwindLayer, router::Router, server::LifecycleLayer, tracing::TracingLayer,
    MainLoop,
};
use tower::ServiceBuilder;

use common::tracing;

use crate::{code_lens, commands, lifecycle, text_document, ServerState};

/// Run the language server
pub async fn run() {
    let (server, _) = MainLoop::new_server(|client| {
        let mut router = Router::new(ServerState {
            client: client.clone(),
            documents: HashMap::new(),
        });

        router
            .request::<request::Initialize, _>(|_, params| lifecycle::initialize(params))
            .notification::<notification::Initialized>(lifecycle::initialized);

        router
            .notification::<notification::DidOpenTextDocument>(text_document::did_open)
            .notification::<notification::DidChangeTextDocument>(text_document::did_change)
            .notification::<notification::DidSaveTextDocument>(text_document::did_save)
            .notification::<notification::DidCloseTextDocument>(text_document::did_close);

        router
            .request::<request::ExecuteCommand, _>(|state, params| {
                let client = state.client.clone();
                async move { commands::execute_command(client, params).await }
            })
            .notification::<notification::WorkDoneProgressCancel>(commands::cancel_progress);

        router
            .request::<request::CodeLensRequest, _>(|state, params| {
                let nodes = state
                    .documents
                    .get(&params.text_document.uri.to_string())
                    .map(|doc| doc.nodes.clone());
                async move {
                    match nodes {
                        Some(nodes) => code_lens::request(nodes.read().await.as_ref()).await,
                        None => Ok(None),
                    }
                }
            })
            .request::<request::CodeLensResolve, _>(|_, code_lens| code_lens::resolve(code_lens));

        ServiceBuilder::new()
            .layer(TracingLayer::default())
            .layer(LifecycleLayer::default())
            .layer(CatchUnwindLayer::default())
            .layer(ConcurrencyLayer::default())
            .layer(ClientProcessMonitorLayer::new(client))
            .service(router)
    });

    // Setup printing of tracing logs
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::TRACE)
        .with_ansi(false)
        .with_writer(std::io::stderr)
        .init();

    // Prefer truly asynchronous piped stdin/stdout without blocking tasks.
    #[cfg(unix)]
    let (stdin, stdout) = (
        async_lsp::stdio::PipeStdin::lock_tokio().unwrap(),
        async_lsp::stdio::PipeStdout::lock_tokio().unwrap(),
    );

    // Fallback to spawn blocking read/write otherwise.
    #[cfg(not(unix))]
    let (stdin, stdout) = (
        tokio_util::compat::TokioAsyncReadCompatExt::compat(tokio::io::stdin()),
        tokio_util::compat::TokioAsyncWriteCompatExt::compat_write(tokio::io::stdout()),
    );

    server.run_buffered(stdin, stdout).await.unwrap();
}
