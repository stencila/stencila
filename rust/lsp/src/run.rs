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
            options: Default::default(),
        });

        router
            .request::<request::Initialize, _>(|state, params| {
                if let Some(options) = params.initialization_options {
                    lifecycle::initialize_options(state, options);
                }
                async move { lifecycle::initialize().await }
            })
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
                let doc_props = params
                    .arguments
                    .first()
                    .and_then(|value| serde_json::from_value(value.clone()).ok())
                    .and_then(|uri| {
                        state.documents.get(&uri).map(|text_doc| {
                            (
                                text_doc.author.clone(),
                                text_doc.format.clone(),
                                text_doc.root.clone(),
                                text_doc.doc.clone(),
                            )
                        })
                    });
                let client = state.client.clone();
                async move {
                    match doc_props {
                        Some((author, format, root, doc)) => {
                            commands::execute_command(params, author, format, root, doc, client)
                                .await
                        }
                        None => Ok(None),
                    }
                }
            })
            .notification::<notification::WorkDoneProgressCancel>(commands::cancel_progress);

        router.request::<request::Formatting, _>(|state, params| {
            let uri = params.text_document.uri;
            let doc_format = state
                .documents
                .get(&uri)
                .map(|text_doc| (text_doc.doc.clone(), text_doc.format.clone()));
            async move {
                match doc_format {
                    Some((doc, format)) => formatting::request(doc, format).await,
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
        .with_max_level(tracing::Level::DEBUG)
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
