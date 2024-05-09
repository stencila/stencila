use std::collections::HashMap;

use async_lsp::lsp_types::{notification, request};
use async_lsp::{
    client_monitor::ClientProcessMonitorLayer, concurrency::ConcurrencyLayer,
    panic::CatchUnwindLayer, router::Router, server::LifecycleLayer, tracing::TracingLayer,
    MainLoop,
};
use tower::ServiceBuilder;

use common::{serde_json, tracing};

use crate::{
    code_lens, commands, completion, content, formatting, lifecycle, symbols, text_document,
    ServerState,
};

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

        router.request::<request::DocumentSymbolRequest, _>(|state, params| {
            let uri = params.text_document.uri;
            let root = state
                .documents
                .get(&uri)
                .map(|text_doc| text_doc.root.clone());
            async move {
                match root {
                    Some(root) => symbols::request(root).await,
                    None => Ok(None),
                }
            }
        });

        router.request::<request::Completion, _>(|state, params| {
            let uri = &params.text_document_position.text_document.uri;
            let source = state
                .documents
                .get(uri)
                .map(|text_doc| text_doc.source.clone());
            async move { completion::request(params, source).await }
        });

        router
            .request::<request::CodeLensRequest, _>(|state, params| {
                let uri = params.text_document.uri;
                let root = state
                    .documents
                    .get(&uri)
                    .map(|text_doc| text_doc.root.clone());
                async move {
                    match root {
                        Some(root) => code_lens::request(uri, root).await,
                        None => Ok(None),
                    }
                }
            })
            .request::<request::CodeLensResolve, _>(|_, code_lens| code_lens::resolve(code_lens));

        router
            .request::<request::ExecuteCommand, _>(|state, params| {
                let root_doc = params
                    .arguments
                    .first()
                    .and_then(|value| serde_json::from_value(value.clone()).ok())
                    .and_then(|uri| {
                        state
                            .documents
                            .get(&uri)
                            .map(|text_doc| (text_doc.root.clone(), text_doc.doc.clone()))
                    });
                let client = state.client.clone();
                async move {
                    match root_doc {
                        Some((root, doc)) => {
                            commands::execute_command(params, root, doc, client).await
                        }
                        None => Ok(None),
                    }
                }
            })
            .notification::<notification::WorkDoneProgressCancel>(commands::cancel_progress);

        router.request::<request::Formatting, _>(|state, params| {
            let uri = params.text_document.uri;
            let doc = state
                .documents
                .get(&uri)
                .map(|text_doc| text_doc.doc.clone());
            async move {
                match doc {
                    Some(doc) => formatting::request(doc).await,
                    None => Ok(None),
                }
            }
        });

        router.request::<content::SubscribeContent, _>(|state, params| {
            let uri = &params.uri;
            let doc = state
                .documents
                .get(uri)
                .map(|text_doc| text_doc.doc.clone());
            let client = state.client.clone();
            async move {
                match doc {
                    Some(doc) => content::subscribe(doc, params, client).await,
                    None => Ok(String::new()),
                }
            }
        });

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
