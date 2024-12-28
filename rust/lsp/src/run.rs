use std::{collections::HashMap, str::FromStr};

use async_lsp::{
    client_monitor::ClientProcessMonitorLayer,
    concurrency::ConcurrencyLayer,
    lsp_types::{notification, request},
    panic::CatchUnwindLayer,
    router::Router,
    server::LifecycleLayer,
    tracing::TracingLayer,
    ErrorCode, MainLoop, ResponseError,
};
use schema::NodeId;
use tower::ServiceBuilder;
use tracing_subscriber::filter::LevelFilter;

use common::serde_json;

use crate::{
    code_lens,
    commands::{self, INSERT_CLONE, INSERT_INSTRUCTION},
    completion, content, dom, formatting, hover, kernels_, lifecycle, logging, models_, node_ids,
    prompts_, symbols, text_document, ServerState, ServerStatus,
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
            let sync_root = state
                .documents
                .get(&uri)
                .map(|text_doc| (text_doc.sync_state(), text_doc.root.clone()));
            async move {
                match sync_root {
                    Some((sync, root)) => symbols::request(sync, root).await,
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

                let source_doc =
                    if matches!(params.command.as_str(), INSERT_CLONE | INSERT_INSTRUCTION) {
                        params
                            .arguments
                            .get(2)
                            .and_then(|value| serde_json::from_value(value.clone()).ok())
                            .and_then(|uri| {
                                state
                                    .documents
                                    .get(&uri)
                                    .map(|text_doc| text_doc.doc.clone())
                            })
                    } else {
                        None
                    };

                let client = state.client.clone();
                async move {
                    match doc_props {
                        Some((author, format, root, doc)) => {
                            commands::execute_command(
                                params, author, format, root, doc, source_doc, client,
                            )
                            .await
                        }
                        None => Err(ResponseError::new(
                            ErrorCode::INVALID_PARAMS,
                            "Invalid document URI",
                        )),
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
                let (uri, node_id) = match params.uri.fragment() {
                    Some(fragment) => {
                        let mut uri = params.uri.clone();
                        uri.set_fragment(None);
                        (uri, NodeId::from_str(fragment).ok())
                    }
                    None => (params.uri.clone(), None),
                };
                let doc = state
                    .documents
                    .get(&uri)
                    .map(|text_doc| text_doc.doc.clone());
                let client = state.client.clone();
                async move {
                    match doc {
                        Some(doc) => dom::subscribe(doc, node_id, client).await,
                        None => Err(ResponseError::new(
                            ErrorCode::INVALID_PARAMS,
                            "Unknown document",
                        )),
                    }
                }
            })
            .request::<dom::ResetDom, _>(|_state, params| dom::reset(params.subscription_id))
            .request::<dom::UnsubscribeDom, _>(|_state, params| {
                dom::unsubscribe(params.subscription_id)
            });

        router
            .request::<node_ids::NodeIdsForLines, _>(|state, params| {
                let root = state
                    .documents
                    .get(&params.uri)
                    .map(|text_doc| text_doc.root.clone());
                async move {
                    match root {
                        Some(root) => node_ids::node_ids_for_lines(root, params.lines).await,
                        None => Err(ResponseError::new(
                            ErrorCode::INVALID_PARAMS,
                            "Unknown document",
                        )),
                    }
                }
            })
            .request::<node_ids::LinesForNodeIds, _>(|state, params| {
                let root = state
                    .documents
                    .get(&params.uri)
                    .map(|text_doc| text_doc.root.clone());
                async move {
                    match root {
                        Some(root) => node_ids::lines_for_node_ids(root, params.ids).await,
                        None => Err(ResponseError::new(
                            ErrorCode::INVALID_PARAMS,
                            "Unknown document",
                        )),
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
