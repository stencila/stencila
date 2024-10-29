use std::collections::HashMap;

use async_lsp::lsp_types::{notification, request};
use async_lsp::{
    client_monitor::ClientProcessMonitorLayer, concurrency::ConcurrencyLayer,
    panic::CatchUnwindLayer, router::Router, server::LifecycleLayer, tracing::TracingLayer,
    MainLoop,
};
use async_lsp::{ErrorCode, ResponseError};
use tower::ServiceBuilder;

use common::serde_json;
use tracing_subscriber::filter::LevelFilter;

use crate::{
    code_lens, commands, completion, content, dom, formatting, hover, kernels_, lifecycle, logging,
    models_, prompts_, symbols, text_document, ServerState, ServerStatus,
};

/// Run the language server
pub async fn run(log_level: LevelFilter, log_filter: &str) {
    let (server, _) = MainLoop::new_server(|client| {
        logging::setup(log_level, log_filter, client.clone());

        let mut router = Router::new(ServerState {
            client: client.clone(),
            documents: HashMap::new(),
            options: Default::default(),
            status: ServerStatus::Running,
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

        router.request::<request::HoverRequest, _>(|state, params| {
            let uri = params
                .text_document_position_params
                .text_document
                .uri
                .clone();
            let doc_root = state
                .documents
                .get(&uri)
                .map(|text_doc| (text_doc.doc.clone(), text_doc.root.clone()));
            async move {
                match doc_root {
                    Some((doc, root)) => hover::request(params, uri, doc, root).await,
                    None => Ok(None),
                }
            }
        });

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

        router
            .request::<dom::SubscribeDom, _>(|state, params| {
                let uri = &params.uri;
                let doc = state
                    .documents
                    .get(uri)
                    .map(|text_doc| text_doc.doc.clone());
                let client = state.client.clone();
                async move {
                    match doc {
                        Some(doc) => dom::subscribe(doc, client).await,
                        None => Err(ResponseError::new(
                            ErrorCode::INVALID_PARAMS,
                            "Unknown document",
                        )),
                    }
                }
            })
            .request::<dom::UnsubscribeDom, _>(|_state, params| {
                dom::unsubscribe(params.subscription_id)
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

        router.request::<kernels_::ListKernels, _>(|_state, _params| async {
            Ok(kernels_::list().await)
        });

        router.request::<prompts_::ListPrompts, _>(|_state, _params| async {
            Ok(prompts_::list().await)
        });

        router.request::<models_::ListModels, _>(|_state, _params| async {
            Ok(models_::list().await)
        });

        router.request::<request::Shutdown, _>(|state, _params| {
            let result = lifecycle::shutdown(state);
            async move { result }
        });

        router.notification::<notification::Exit>(|state, _params| lifecycle::exit(state));

        ServiceBuilder::new()
            .layer(TracingLayer::default())
            .layer(LifecycleLayer::default())
            .layer(CatchUnwindLayer::default())
            .layer(ConcurrencyLayer::default())
            .layer(ClientProcessMonitorLayer::new(client))
            .service(router)
    });

    // Prefer truly asynchronous piped stdin/stdout without blocking tasks.
    #[cfg(unix)]
    let (stdin, stdout) = (
        async_lsp::stdio::PipeStdin::lock_tokio().unwrap(),
        async_lsp::stdio::PipeStdout::lock_tokio().unwrap(),
    );

    // Fallback to spawn blocking read/write otherwise.
    #[cfg(not(unix))]
    let (stdin, stdout) = {
        use common::tokio::io;
        (
            tokio_util::compat::TokioAsyncReadCompatExt::compat(io::stdin()),
            tokio_util::compat::TokioAsyncWriteCompatExt::compat_write(io::stdout()),
        )
    };

    server.run_buffered(stdin, stdout).await.unwrap();
}
