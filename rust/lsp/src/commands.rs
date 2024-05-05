use std::{
    ops::ControlFlow,
    path::PathBuf,
    str::FromStr,
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc,
    },
};

use async_lsp::{
    lsp_types::{
        ExecuteCommandParams, MessageType, NumberOrString, Position, ProgressParams,
        ProgressParamsValue, ShowMessageParams, WorkDoneProgress, WorkDoneProgressBegin,
        WorkDoneProgressCancelParams, WorkDoneProgressCreateParams, WorkDoneProgressEnd,
        WorkDoneProgressReport,
    },
    ClientSocket, Error, ErrorCode, LanguageClient, ResponseError,
};

use common::{
    eyre::Result,
    once_cell::sync::Lazy,
    serde_json,
    serde_json::Value,
    tokio::{
        self,
        sync::{mpsc, RwLock},
    },
    tracing,
};
use document::{Command, CommandNodes, Document};
use node_execute::ExecuteOptions;
use schema::NodeId;

use crate::{text_document::TextNode, ServerState};

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
    ExecuteCommandParams {
        command, arguments, ..
    }: ExecuteCommandParams,
    root: Arc<RwLock<TextNode>>,
    doc: Arc<RwLock<Document>>,
    mut client: ClientSocket,
) -> Result<Option<Value>, ResponseError> {
    let mut args = arguments.into_iter();
    let uri = uri_arg(args.next())?;

    let file_name = PathBuf::from(&uri)
        .file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().to_string());

    let command = match command.as_str() {
        RUN_NODE => {
            let node_id = node_id_arg(args.next())?;
            Command::ExecuteNodes(CommandNodes::new(vec![node_id], None))
        }
        RUN_CURR => {
            let position = position_arg(args.next())?;
            if let Some(node_id) = root.read().await.node_id_at(position) {
                Command::ExecuteNodes(CommandNodes::new(vec![node_id], None))
            } else {
                tracing::warn!("No node found at position {position:?}");
                return Ok(None);
            }
        }
        RUN_ALL_DOC => Command::ExecuteDocument(ExecuteOptions::default()),
        command => {
            return Err(ResponseError::new(
                ErrorCode::INVALID_REQUEST,
                format!("Unknown command `{command}`"),
            ))
        }
    };

    let (command_id, mut status_receiver) = match doc.read().await.command_subscribe(command).await
    {
        Ok(receiver) => receiver,
        Err(error) => {
            client
                .show_message(ShowMessageParams {
                    typ: MessageType::ERROR,
                    message: format!("While sending command to {uri}: {error}"),
                })
                .ok();
            return Ok(None);
        }
    };

    let progress_sender = create_progress(client, format!("Running {file_name}")).await;
    tokio::spawn(async move {
        while let Ok((id, status)) = status_receiver.recv().await {
            if id == command_id && status.is_finished() && progress_sender.send((100, None)).is_err() {
                break;
            }
        }
    });

    Ok(None)
}

/// Extract a document URI from a command arg
pub(super) fn uri_arg(arg: Option<Value>) -> Result<String, ResponseError> {
    arg.and_then(|value| value.as_str().map(String::from))
        .ok_or_else(|| {
            ResponseError::new(
                ErrorCode::INVALID_REQUEST,
                "Document URI argument missing or invalid".to_string(),
            )
        })
}

/// Extract a Stencila [`NodeId`] from a command arg
fn node_id_arg(arg: Option<Value>) -> Result<NodeId, ResponseError> {
    arg.and_then(|value| value.as_str().map(String::from))
        .and_then(|node_id| NodeId::from_str(&node_id).ok())
        .ok_or_else(|| {
            ResponseError::new(
                ErrorCode::INVALID_REQUEST,
                "Node id argument missing or invalid".to_string(),
            )
        })
}

/// Extract a position from a command arg
fn position_arg(arg: Option<Value>) -> Result<Position, ResponseError> {
    arg.and_then(|value| serde_json::from_value(value).ok())
        .ok_or_else(|| {
            ResponseError::new(
                ErrorCode::INVALID_REQUEST,
                "Position argument missing or invalid".to_string(),
            )
        })
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
