use std::{
    ops::ControlFlow,
    path::PathBuf,
    str::FromStr,
    sync::{
        atomic::{AtomicI32, Ordering},
        Arc,
    },
    time::Duration,
};

use async_lsp::{
    lsp_types::{
        ApplyWorkspaceEditParams, DocumentChanges, ExecuteCommandParams, MessageType,
        NumberOrString, OneOf, OptionalVersionedTextDocumentIdentifier, Position, ProgressParams,
        ProgressParamsValue, ShowMessageParams, TextDocumentEdit, Url, WorkDoneProgress,
        WorkDoneProgressBegin, WorkDoneProgressCancelParams, WorkDoneProgressCreateParams,
        WorkDoneProgressEnd, WorkDoneProgressReport, WorkspaceEdit,
    },
    ClientSocket, Error, ErrorCode, LanguageClient, ResponseError,
};

use codecs::{EncodeOptions, Format};
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
use schema::{NodeId, NodeType};

use crate::{formatting::format_doc, text_document::TextNode, ServerState};

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

pub(super) const ACCEPT_NODE: &str = "stencila.accept-node";
pub(super) const REJECT_NODE: &str = "stencila.reject-node";

pub(super) const EXPORT_DOC: &str = "stencila.export-doc";

// These commands are implemented on the client but included here
// for us in the construction of code lenses
pub(super) const VIEW_NODE: &str = "stencila.view-node";
pub(super) const INSPECT_NODE: &str = "stencila.inspect-node";

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
        EXPORT_DOC,
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

    let file_name = PathBuf::from(&uri.to_string())
        .file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().to_string());

    let (command, update) = match command.as_str() {
        RUN_NODE => {
            let node_type = node_type_arg(args.next())?;
            let node_id = node_id_arg(args.next())?;
            (
                Command::ExecuteNodes(CommandNodes::new(vec![node_id], None)),
                matches!(
                    node_type,
                    NodeType::InstructionBlock | NodeType::InstructionInline
                ),
            )
        }
        RUN_CURR => {
            let position = position_arg(args.next())?;
            if let Some(node_id) = root.read().await.node_id_at(position) {
                (
                    Command::ExecuteNodes(CommandNodes::new(vec![node_id], None)),
                    false,
                )
            } else {
                tracing::warn!("No node found at position {position:?}");
                return Ok(None);
            }
        }
        RUN_ALL_DOC => (Command::ExecuteDocument(ExecuteOptions::default()), false),
        EXPORT_DOC => {
            let path = path_buf_arg(args.next())?;
            (
                Command::ExportDocument((path, EncodeOptions::default())),
                false,
            )
        }
        command => {
            return Err(ResponseError::new(
                ErrorCode::INVALID_REQUEST,
                format!("Unknown command `{command}`"),
            ))
        }
    };

    // Send the command to the document with a subscription to receive status updates
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

    // If necessary, create a task to update the text for the node when the command is finished
    // TODO: this is not ideal because it does not handle case where nodes need to be updated after
    // the whole document is run, and because it has to hackily wait for the final patch to be
    // applied. Instead need to set up a patch watcher that allows us to watch for
    // the node types and ids to which a patch was applied.
    if update {
        let mut status_receiver = status_receiver.resubscribe();
        let mut client = client.clone();
        tokio::spawn(async move {
            while let Ok((id, status)) = status_receiver.recv().await {
                if id == command_id && status.is_finished() {
                    // Wait an arbitrary amount of time
                    tokio::time::sleep(Duration::from_millis(100)).await;

                    // Currently this applies a whole document formatting.
                    // TODO: In the future this should be refined to only update the specific node.
                    let edit = match format_doc(doc.clone(), Format::Markdown).await {
                        Ok(edit) => edit,
                        Err(error) => {
                            tracing::error!("While formatting doc after command: {error}");
                            continue;
                        }
                    };
                    client
                        .apply_edit(ApplyWorkspaceEditParams {
                            edit: WorkspaceEdit {
                                document_changes: Some(DocumentChanges::Edits(vec![
                                    TextDocumentEdit {
                                        text_document: OptionalVersionedTextDocumentIdentifier {
                                            uri,
                                            version: None,
                                        },
                                        edits: vec![OneOf::Left(edit)],
                                    },
                                ])),
                                ..Default::default()
                            },
                            label: Some("Update after completion".to_string()),
                        })
                        .await
                        .ok();
                    break;
                }
            }
        });
    }

    // Create a progress notification and spawn a task to update it
    let progress_sender = create_progress(client, format!("Running {file_name}")).await;
    tokio::spawn(async move {
        while let Ok((id, status)) = status_receiver.recv().await {
            if id == command_id && status.is_finished() {
                progress_sender.send((100, None)).ok();
                break;
            }
        }
    });

    Ok(None)
}

/// Extract a document URI from a command arg
pub(super) fn uri_arg(arg: Option<Value>) -> Result<Url, ResponseError> {
    arg.and_then(|value| serde_json::from_value(value).ok())
        .ok_or_else(|| {
            ResponseError::new(
                ErrorCode::INVALID_REQUEST,
                "Document URI argument missing or invalid".to_string(),
            )
        })
}

/// Extract a Stencila [`NodeType`] from a command arg
fn node_type_arg(arg: Option<Value>) -> Result<NodeType, ResponseError> {
    arg.and_then(|value| value.as_str().map(String::from))
        .and_then(|node_id| NodeType::from_str(&node_id).ok())
        .ok_or_else(|| {
            ResponseError::new(
                ErrorCode::INVALID_REQUEST,
                "Node id argument missing or invalid".to_string(),
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

/// Extract a `PathBuf` from a command arg
fn path_buf_arg(arg: Option<Value>) -> Result<PathBuf, ResponseError> {
    arg.and_then(|value| serde_json::from_value(value).ok())
        .ok_or_else(|| {
            ResponseError::new(
                ErrorCode::INVALID_REQUEST,
                "Path argument missing or invalid".to_string(),
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
