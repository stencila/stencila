use std::{
    path::PathBuf,
    sync::atomic::{AtomicI32, Ordering},
    time::Duration,
};

use async_lsp::{
    lsp_types::{
        ExecuteCommandParams, NumberOrString, ProgressParams, ProgressParamsValue,
        WorkDoneProgress, WorkDoneProgressBegin, WorkDoneProgressCreateParams, WorkDoneProgressEnd,
        WorkDoneProgressReport,
    },
    ClientSocket, ErrorCode, LanguageClient, ResponseError,
};

use common::{
    once_cell::sync::Lazy,
    serde_json::Value,
    tokio::{self, sync::mpsc},
};

const RUN_DOCUMENT: &str = "stencila.run-document";

/// Get the list of commands that the language server supports
pub(super) fn commands() -> Vec<String> {
    vec![RUN_DOCUMENT.to_string()]
}

/// Execute a command
pub(super) async fn execute_command(
    client: ClientSocket,
    params: ExecuteCommandParams,
) -> Result<Option<Value>, ResponseError> {
    match params.command.as_str() {
        RUN_DOCUMENT => run_document(client, params.arguments).await,
        command => Err(ResponseError::new(
            ErrorCode::INVALID_REQUEST,
            format!("Unknown command `{command}`"),
        )),
    }
}

/// Run a document
async fn run_document(
    client: ClientSocket,
    args: Vec<Value>,
) -> Result<Option<Value>, ResponseError> {
    let path = args
        .first()
        .and_then(|value| value.as_str())
        .ok_or_else(|| {
            ResponseError::new(
                ErrorCode::INVALID_REQUEST,
                format!("No document path in args"),
            )
        })?;

    let path = PathBuf::from(path);

    let file_name = path
        .file_name()
        .map_or_else(|| String::new(), |name| name.to_string_lossy().to_string());

    let progress_sender = create_progress(client, format!("Running {file_name}")).await;

    // TODO: replace this
    tokio::spawn(async move {
        let mut percentage = 10;
        while percentage <= 100 {
            percentage += 10;
            if let Err(..) = progress_sender.send((percentage, None)) {
                break;
            };
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    Ok(None)
}

static PROGRESS_TOKEN: Lazy<AtomicI32> = Lazy::new(|| AtomicI32::default());

/// Create and begin a progress notification
async fn create_progress(
    mut client: ClientSocket,
    title: String,
) -> mpsc::UnboundedSender<(u32, Option<String>)> {
    // Create the token for the progress
    let token = NumberOrString::Number(PROGRESS_TOKEN.fetch_add(1, Ordering::Relaxed));

    // Request that the progress be created
    client
        .work_done_progress_create(WorkDoneProgressCreateParams {
            token: token.clone(),
        })
        .await
        .ok();

    // Begin the progress
    client
        .progress(ProgressParams {
            token: token.clone(),
            value: ProgressParamsValue::WorkDone(WorkDoneProgress::Begin(WorkDoneProgressBegin {
                title,
                cancellable: Some(true),
                ..Default::default()
            })),
        })
        .ok();

    // Create channel and async task to update progress
    let (sender, mut receiver) = mpsc::unbounded_channel::<(u32, Option<String>)>();
    tokio::spawn(async move {
        while let Some((percentage, message)) = receiver.recv().await {
            let work_done = if percentage >= 100 {
                WorkDoneProgress::End(WorkDoneProgressEnd {
                    ..Default::default()
                })
            } else {
                WorkDoneProgress::Report(WorkDoneProgressReport {
                    percentage: Some(percentage),
                    message: Some(message.unwrap_or_else(|| format!("{percentage}%"))),
                    ..Default::default()
                })
            };

            client
                .progress(ProgressParams {
                    token: token.clone(),
                    value: ProgressParamsValue::WorkDone(work_done),
                })
                .ok();
        }
    });

    sender
}
