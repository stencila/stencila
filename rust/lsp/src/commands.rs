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
    serde_json::{self, Value},
    tokio::{
        self,
        sync::{mpsc, RwLock},
    },
    tracing,
};
use document::{
    Command, CommandNodes, CommandScope, CommandStatus, Document, SaveDocumentSidecar,
    SaveDocumentSource,
};
use node_execute::ExecuteOptions;
use schema::{
    AuthorRole, AuthorRoleName, NodeId, NodeProperty, NodeType, Patch, PatchOp, PatchPath,
    PatchValue, Timestamp,
};

use crate::{formatting::format_doc, text_document::TextNode, ServerState};

pub(super) const PATCH_NODE: &str = "stencila.patch-node";
pub(super) const VERIFY_NODE: &str = "stencila.verify-node";

pub(super) const RUN_NODE: &str = "stencila.run-node";
pub(super) const RUN_CURR: &str = "stencila.run-curr";
pub(super) const RUN_DOC: &str = "stencila.run-doc";
pub(super) const RUN_CODE: &str = "stencila.run-code";
pub(super) const RUN_INSTRUCT: &str = "stencila.run-instruct";
pub(super) const RUN_ABOVE: &str = "stencila.run-above";
pub(super) const RUN_BELOW: &str = "stencila.run-below";

pub(super) const CANCEL_NODE: &str = "stencila.cancel-node";
pub(super) const CANCEL_CURR: &str = "stencila.cancel-curr";
pub(super) const CANCEL_DOC: &str = "stencila.cancel-doc";

pub(super) const LOCK_CURR: &str = "stencila.lock-curr";
pub(super) const UNLOCK_CURR: &str = "stencila.unlock-curr";

pub(super) const PREV_NODE: &str = "stencila.prev-node";
pub(super) const NEXT_NODE: &str = "stencila.next-node";
pub(super) const ARCHIVE_NODE: &str = "stencila.archive-node";
pub(super) const REVISE_NODE: &str = "stencila.revise-node";

pub(super) const SAVE_DOC: &str = "stencila.save-doc";
pub(super) const EXPORT_DOC: &str = "stencila.export-doc";

/// Get the list of commands that the language server supports
pub(super) fn commands() -> Vec<String> {
    [
        PATCH_NODE,
        VERIFY_NODE,
        RUN_NODE,
        RUN_CURR,
        RUN_DOC,
        RUN_CODE,
        RUN_INSTRUCT,
        RUN_ABOVE,
        RUN_BELOW,
        CANCEL_NODE,
        CANCEL_CURR,
        CANCEL_DOC,
        LOCK_CURR,
        UNLOCK_CURR,
        PREV_NODE,
        NEXT_NODE,
        ARCHIVE_NODE,
        REVISE_NODE,
        SAVE_DOC,
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
    author: AuthorRole,
    format: Format,
    root: Arc<RwLock<TextNode>>,
    doc: Arc<RwLock<Document>>,
    mut client: ClientSocket,
) -> Result<Option<Value>, ResponseError> {
    let mut args = arguments.into_iter();
    let uri = uri_arg(args.next())?;

    let file_name = PathBuf::from(&uri.to_string())
        .file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().to_string());

    let author = AuthorRole {
        last_modified: Some(Timestamp::now()),
        ..author
    };

    let (title, command, cancellable, update_after) = match command.as_str() {
        PATCH_NODE => {
            args.next(); // Skip the currently unused node type arg
            let node_id = node_id_arg(args.next())?;
            let property = node_property_arg(args.next())?;
            let value = args.next();

            let value = match value {
                Some(value) => PatchValue::Json(value),
                None => PatchValue::None,
            };

            (
                "Patching node".to_string(),
                Command::PatchNode(Patch {
                    node_id: Some(node_id),
                    ops: vec![(PatchPath::from(property), PatchOp::Set(value))],
                    authors: Some(vec![author]),
                    ..Default::default()
                }),
                false,
                true,
            )
        }
        VERIFY_NODE => {
            args.next(); // Skip the currently unused node type arg
            let node_id = node_id_arg(args.next())?;

            (
                "Verifying node".to_string(),
                Command::PatchNode(Patch {
                    node_id: Some(node_id),
                    ops: vec![(PatchPath::default(), PatchOp::Verify)],
                    authors: Some(vec![AuthorRole {
                        role_name: AuthorRoleName::Verifier,
                        ..author
                    }]),
                    ..Default::default()
                }),
                false,
                true,
            )
        }
        RUN_NODE => {
            let node_type = node_type_arg(args.next())?;
            let node_id = node_id_arg(args.next())?;
            (
                "Running node".to_string(),
                Command::ExecuteNodes((
                    CommandNodes::new(vec![node_id], CommandScope::Only),
                    ExecuteOptions::default(),
                )),
                true,
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
                    "Running current node".to_string(),
                    Command::ExecuteNodes((
                        CommandNodes::new(vec![node_id], CommandScope::Only),
                        ExecuteOptions::default(),
                    )),
                    true,
                    true,
                )
            } else {
                tracing::error!("No node to run at current position");
                return Ok(None);
            }
        }
        RUN_DOC => (
            format!("Running {file_name}"),
            Command::ExecuteDocument(ExecuteOptions::default()),
            true,
            false,
        ),
        CANCEL_NODE => {
            args.next(); // Skip the currently unused node type arg
            let node_id = node_id_arg(args.next())?;
            (
                "Cancelling node".to_string(),
                Command::InterruptNodes(CommandNodes::new(vec![node_id], CommandScope::Only)),
                false,
                false,
            )
        }
        LOCK_CURR => {
            let position = position_arg(args.next())?;
            let node_id = if let Some(node_id) = root.read().await.node_id_at(position) {
                node_id
            } else {
                tracing::error!("No node to lock at current position");
                return Ok(None);
            };

            (
                "Locking node".to_string(),
                Command::PatchNode(Patch {
                    node_id: Some(node_id),
                    ops: vec![(
                        PatchPath::from(NodeProperty::ExecutionMode),
                        PatchOp::Set(PatchValue::String("Locked".to_string())),
                    )],
                    ..Default::default()
                }),
                false,
                true,
            )
        }
        UNLOCK_CURR => {
            let position = position_arg(args.next())?;
            let node_id = if let Some(node_id) = root.read().await.node_id_at(position) {
                node_id
            } else {
                tracing::error!("No node to unlock at current position");
                return Ok(None);
            };

            (
                "Unlocking node".to_string(),
                Command::PatchNode(Patch {
                    node_id: Some(node_id),
                    ops: vec![(
                        PatchPath::from(NodeProperty::ExecutionMode),
                        PatchOp::Set(PatchValue::None),
                    )],
                    ..Default::default()
                }),
                false,
                true,
            )
        }
        PREV_NODE | NEXT_NODE | ARCHIVE_NODE => {
            // Second arg (after document URI) is either current position (when invoked
            // via keybinding) or node type (when invoked via code lens). So resolve
            // instruction id on that basis
            let instruction_id = match position_arg(args.next()) {
                Ok(position) => match root.read().await.instruction_at(position) {
                    Some(id) => id,
                    None => {
                        tracing::error!("No command at current position");
                        return Ok(None);
                    }
                },
                Err(..) => node_id_arg(args.next())?,
            };

            let (title, path, op) = match command.as_str() {
                PREV_NODE => (
                    "Previous suggestion".to_string(),
                    PatchPath::from(NodeProperty::ActiveSuggestion),
                    PatchOp::Decrement,
                ),
                NEXT_NODE => (
                    "Next suggestion".to_string(),
                    PatchPath::from(NodeProperty::ActiveSuggestion),
                    PatchOp::Increment,
                ),
                ARCHIVE_NODE => (
                    "Accepting suggestion and archiving command".to_string(),
                    PatchPath::new(),
                    PatchOp::Archive,
                ),
                _ => unreachable!(),
            };

            (
                title,
                Command::PatchNode(Patch {
                    node_id: Some(instruction_id),
                    ops: vec![(path, op)],
                    authors: Some(vec![author]),
                    ..Default::default()
                }),
                false,
                true,
            )
        }
        REVISE_NODE => {
            // As above, get instruction id
            let instruction_id = match position_arg(args.next()) {
                Ok(position) => match root.read().await.instruction_at(position) {
                    Some(id) => id,
                    None => {
                        tracing::error!("No command at current position");
                        return Ok(None);
                    }
                },
                Err(..) => node_id_arg(args.next())?,
            };

            // Next arg is the feedback for the instruction's active suggestion
            // it may be empty (e.g. just a plain retry without the entering any feedback)
            let feedback = args
                .next()
                .map(PatchValue::Json)
                .unwrap_or(PatchValue::None);

            (
                "Revising suggestion".to_string(),
                Command::PatchExecuteNodes((
                    Patch {
                        node_id: Some(instruction_id.clone()),
                        ops: vec![(
                            // Instructions do not have a feedback property but have
                            // a custom patch implem that will intercept this and apply
                            // it to the active suggestion
                            PatchPath::from(NodeProperty::Feedback),
                            PatchOp::Set(feedback),
                        )],
                        authors: Some(vec![author]),
                        ..Default::default()
                    },
                    CommandNodes::new(vec![instruction_id], CommandScope::Only),
                    ExecuteOptions {
                        retain_suggestions: true,
                        ..Default::default()
                    },
                )),
                false,
                true,
            )
        }
        SAVE_DOC => (
            "Saving document with sidecar".to_string(),
            Command::SaveDocument((SaveDocumentSource::Yes, SaveDocumentSidecar::Yes)),
            false,
            false,
        ),
        EXPORT_DOC => {
            let path = path_buf_arg(args.next())?;
            (
                "Exporting document".to_string(),
                Command::ExportDocument((path, EncodeOptions::default())),
                false,
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
    if update_after {
        let mut status_receiver = status_receiver.resubscribe();
        let mut client = client.clone();
        let uri = uri.clone();
        tokio::spawn(async move {
            while let Ok((id, status)) = status_receiver.recv().await {
                if id == command_id && status.finished() {
                    // Wait an arbitrary amount of time for any patches to be applied (see note above)
                    tokio::time::sleep(Duration::from_millis(100)).await;

                    // Currently this applies a whole document formatting.
                    // TODO: In the future this should be refined to only update the specific node.
                    let edit = match format_doc(doc.clone(), format.clone()).await {
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
                    client.code_lens_refresh(()).await.ok();
                    break;
                }
            }
        });
    }

    // Create a progress notification and spawn a task to update it
    let progress_sender = create_progress(client.clone(), title, cancellable).await;
    tokio::spawn(async move {
        while let Ok((id, status)) = status_receiver.recv().await {
            if id == command_id && status.finished() {
                progress_sender.send((100, None)).ok();

                // Notify the user if the command failed
                if let CommandStatus::Failed(error) = status {
                    client
                        .show_message(ShowMessageParams {
                            typ: MessageType::ERROR,
                            message: format!("{error}\n\n{uri}"),
                        })
                        .ok();
                }

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
                "Node type argument missing or invalid".to_string(),
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

/// Extract a Stencila [`NodeProperty`] from a command arg
fn node_property_arg(arg: Option<Value>) -> Result<NodeProperty, ResponseError> {
    arg.and_then(|value| value.as_str().map(String::from))
        .and_then(|node_id| NodeProperty::from_str(&node_id).ok())
        .ok_or_else(|| {
            ResponseError::new(
                ErrorCode::INVALID_REQUEST,
                "Node property argument missing or invalid".to_string(),
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
    cancellable: bool,
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
                cancellable: Some(cancellable),
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
