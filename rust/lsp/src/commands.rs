use std::{
    ops::ControlFlow,
    path::PathBuf,
    sync::atomic::{AtomicI32, Ordering},
    time::Duration,
};

use async_lsp::{
    lsp_types::{
        ExecuteCommandParams, NumberOrString, ProgressParams, ProgressParamsValue,
        WorkDoneProgress, WorkDoneProgressBegin, WorkDoneProgressCancelParams,
        WorkDoneProgressCreateParams, WorkDoneProgressEnd, WorkDoneProgressReport,
    },
    ClientSocket, Error, ErrorCode, LanguageClient, ResponseError,
};

use common::{
    once_cell::sync::Lazy,
    serde_json::Value,
    tokio::{self, sync::mpsc},
    tracing,
};

use crate::ServerState;

pub(super) const RUN_NODE: &str = "stencila.run-node";
pub(super) const RUN_CURR: &str = "stencila.run-curr";
pub(super) const RUN_ALL_DOC: &str = "stencila.run-all-doc";
pub(super) const RUN_CODE_DOC: &str = "stencila.run-code-doc";
pub(super) const RUN_ASSIST_DOC: &str = "stencila.run-assist-doc";
pub(super) const RUN_ALL_ABOVE: &str = "stencila.run-all-above";
pub(super) const RUN_ALL_BELOW: &str = "stencila.run-all-below";

pub(super) const CANCEL_NODE: &str = "stencila.cancel-node";
pub(super) const CANCEL_CURR: &str = "stencila.cancel-curr";
pub(super) const CANCEL_ALL_DOC: &str = "stencila.cancel-all-doc";

pub(super) const ACCEPT_NODE: &str = "stencila.accept_node";
pub(super) const REJECT_NODE: &str = "stencila.reject_node";

// This command is implemented on the client but included here
// for us in the construction of code lenses
pub(super) const INSPECT_NODE: &str = "stencila.inspect_node";

/// Get the list of commands that the language server supports
pub(super) fn commands() -> Vec<String> {
    [
        RUN_NODE,
        RUN_CURR,
        RUN_ALL_DOC,
        RUN_CODE_DOC,
        RUN_ASSIST_DOC,
        RUN_ALL_ABOVE,
        RUN_ALL_BELOW,
        CANCEL_NODE,
        CANCEL_CURR,
        CANCEL_ALL_DOC,
        ACCEPT_NODE,
        REJECT_NODE,
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

/// Execute a command
pub(super) async fn execute_command(
    client: ClientSocket,
    params: ExecuteCommandParams,
) -> Result<Option<Value>, ResponseError> {
    match params.command.as_str() {
        RUN_ALL_DOC => run_all_doc(client, params.arguments).await,
        command => Err(ResponseError::new(
            ErrorCode::INVALID_REQUEST,
            format!("Unknown command `{command}`"),
        )),
    }
}

/// Run all nodes in a document
async fn run_all_doc(
    client: ClientSocket,
    args: Vec<Value>,
) -> Result<Option<Value>, ResponseError> {
    let path = args
        .first()
        .and_then(|value| value.as_str())
        .ok_or_else(|| {
            ResponseError::new(
                ErrorCode::INVALID_REQUEST,
                "No document path in args".to_string(),
            )
        })?;

    let path = PathBuf::from(path);

    let file_name = path
        .file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().to_string());

    let progress_sender = create_progress(client, format!("Running {file_name}")).await;

    // TODO: replace this
    tokio::spawn(async move {
        let mut percentage = 10;
        while percentage <= 100 {
            percentage += 10;
            if progress_sender.send((percentage, None)).is_err() {
                break;
            };
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    Ok(None)
}

static PROGRESS_TOKEN: Lazy<AtomicI32> = Lazy::new(AtomicI32::default);

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

/// Handle a notification from the client to cancel a task previously associated
/// with `WorkDoneProgressBegin`
pub(crate) fn cancel_progress(
    _state: &mut ServerState,
    params: WorkDoneProgressCancelParams,
) -> ControlFlow<Result<(), Error>> {
    tracing::info!("cancel_progress: {:?}", params.token);

    // TODO: Cancel the task associated with the token

    ControlFlow::Continue(())
}
