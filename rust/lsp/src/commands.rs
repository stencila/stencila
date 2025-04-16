use std::{
    fmt::Display,
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
        ProgressParamsValue, Range, ShowMessageParams, TextDocumentEdit, Url, WorkDoneProgress,
        WorkDoneProgressBegin, WorkDoneProgressCancelParams, WorkDoneProgressCreateParams,
        WorkDoneProgressEnd, WorkDoneProgressReport, WorkspaceEdit,
    },
    ClientSocket, Error, ErrorCode, LanguageClient, ResponseError,
};

use codecs::{EncodeOptions, Format};
use common::{
    eyre::{OptionExt, Result},
    itertools::Itertools,
    once_cell::sync::Lazy,
    serde_json::{self, json, Value},
    tokio::{
        self,
        sync::{mpsc, watch::Receiver, RwLock},
        time::timeout,
    },
    tracing,
};
use document::{Command, CommandNodes, CommandScope, CommandStatus, ContentType, Document};
use node_execute::ExecuteOptions;
use node_find::find;
use schema::{
    diff, replicate, AuthorRole, AuthorRoleName, Block, Chat, ExecutionMode, InstructionBlock,
    InstructionMessage, InstructionType, ModelParameters, Node, NodeId, NodePath, NodeProperty,
    NodeType, Patch, PatchNode, PatchOp, PatchValue, PromptBlock, SuggestionBlock, Timestamp,
};

use crate::{
    formatting::format_doc,
    text_document::{SyncState, TextNode},
    ServerState,
};

pub(super) const PATCH_VALUE: &str = "stencila.patch-value";
pub(super) const PATCH_VALUE_EXECUTE: &str = "stencila.patch-value-execute";
pub(super) const PATCH_CLONE: &str = "stencila.patch-clone";
pub(super) const PATCH_CHAT_FOCUS: &str = "stencila.patch-chat-focus";
pub(super) const PATCH_NODE_FORMAT: &str = "stencila.patch-node-format";
pub(super) const VERIFY_NODE: &str = "stencila.verify-node";

pub(super) const RUN_NODE: &str = "stencila.run-node";
pub(super) const RUN_CURR: &str = "stencila.run-curr";
pub(super) const RUN_CHAT: &str = "stencila.run-chat";
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

pub(super) const INSERT_NODE: &str = "stencila.insert-node";
pub(super) const INSERT_CLONES: &str = "stencila.insert-clones";
pub(super) const INSERT_INSTRUCTION: &str = "stencila.insert-instruction";

pub(super) const MERGE_NODE: &str = "stencila.merge-node";
pub(super) const DELETE_NODE: &str = "stencila.delete-node";

pub(super) const CREATE_CHAT: &str = "stencila.create-chat";

pub(super) const EXPORT_DOC: &str = "stencila.export-doc";

/// Get the list of commands that the language server supports
pub(super) fn commands() -> Vec<String> {
    [
        PATCH_VALUE,
        PATCH_VALUE_EXECUTE,
        PATCH_CLONE,
        PATCH_CHAT_FOCUS,
        PATCH_NODE_FORMAT,
        VERIFY_NODE,
        RUN_NODE,
        RUN_CURR,
        RUN_CHAT,
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
        INSERT_NODE,
        INSERT_CLONES,
        INSERT_INSTRUCTION,
        MERGE_NODE,
        DELETE_NODE,
        CREATE_CHAT,
        EXPORT_DOC,
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

/// Execute a command
#[allow(clippy::too_many_arguments)]
pub(super) async fn execute_command(
    ExecuteCommandParams {
        command, arguments, ..
    }: ExecuteCommandParams,
    author: AuthorRole,
    format: Format,
    source: Arc<RwLock<String>>,
    root: Arc<RwLock<TextNode>>,
    doc: Arc<RwLock<Document>>,
    mut sync_state_receiver: Receiver<SyncState>,
    source_doc: Option<Arc<RwLock<Document>>>,
    mut client: ClientSocket,
) -> Result<Option<Value>, ResponseError> {
    let command = command.as_str();

    // Before running command wait for document to be in sync.
    // Use a timeout so that user is not confused if command does not run. Should be same as or longer
    // that the delay in `TextDocument::diagnostics_task`.
    // TODO: Make this timeout configurable https://github.com/stencila/stencila/issues/2405
    const TIMEOUT_MILLIS: u64 = 3000;
    match timeout(
        Duration::from_millis(TIMEOUT_MILLIS),
        sync_state_receiver.wait_for(|sync_state| matches!(sync_state, SyncState::Updated)),
    )
    .await
    {
        Err(..) => {
            client.show_message(ShowMessageParams {
                typ: MessageType::ERROR,
                message: format!("Unable to run command `{command}` because document is out-of-sync with source, probably due to syntax errors"),
            }).ok();
            return Ok(None);
        }
        Ok(Err(..)) => {
            return Err(ResponseError::new(
                ErrorCode::INTERNAL_ERROR,
                "Unable to wait for is_synced",
            ))
        }
        _ => {}
    };

    let mut args = arguments.into_iter();
    let uri = uri_arg(args.next())?;

    let file_name = PathBuf::from(&uri.to_string())
        .file_name()
        .map_or_else(String::new, |name| name.to_string_lossy().to_string());

    let author = AuthorRole {
        last_modified: Some(Timestamp::now()),
        ..author
    };

    let mut return_value = None;

    let (title, command, cancellable, update_after) = match command {
        PATCH_VALUE | PATCH_CLONE | PATCH_CHAT_FOCUS => {
            let node_type = node_type_arg(args.next())?;

            let node_position_or_id = args
                .next()
                .ok_or_else(|| invalid_request("Node position or id arg missing"))?;
            let node_id = match position_arg(Some(node_position_or_id.clone())) {
                Ok(position) => match root.read().await.node_type_ancestor(node_type, position) {
                    Some(id) => id,
                    None => {
                        tracing::error!("No node of type {node_type} at current position");
                        return Ok(None);
                    }
                },
                Err(..) => node_id_arg(Some(node_position_or_id))?,
            };

            let path = args
                .next()
                .ok_or_eyre("Patch path arg missing")
                .and_then(NodePath::try_from)
                .map_err(invalid_request)?;

            let value = match command {
                PATCH_CLONE | PATCH_CHAT_FOCUS => {
                    let node_id = node_id_arg(args.next())?;

                    let clone = doc
                        .read()
                        .await
                        .find(node_id)
                        .await
                        .ok_or_eyre("Node not found in source document")
                        .and_then(|node| replicate(&node))
                        .map_err(invalid_request)?;

                    match command {
                        PATCH_CHAT_FOCUS => Block::try_from(clone).and_then(|block| {
                            SuggestionBlock {
                                content: vec![block],
                                ..Default::default()
                            }
                            .to_value()
                        }),
                        _ => clone.to_value(),
                    }
                    .map_err(invalid_request)?
                }
                _ => args
                    .next()
                    .map(PatchValue::Json)
                    .unwrap_or(PatchValue::None),
            };

            let op = match command {
                PATCH_CHAT_FOCUS => PatchOp::Push(value),
                _ => PatchOp::Set(value),
            };

            (
                "Patching node".to_string(),
                Command::PatchNode(Patch {
                    node_id: Some(node_id),
                    ops: vec![(path, op)],
                    authors: Some(vec![author]),
                    lint: true,
                    ..Default::default()
                }),
                false,
                true,
            )
        }
        PATCH_VALUE_EXECUTE => {
            let _node_type = node_type_arg(args.next())?;
            let node_id = node_id_arg(args.next())?;

            let path = args
                .next()
                .ok_or_eyre("Patch path arg missing")
                .and_then(NodePath::try_from)
                .map_err(invalid_request)?;

            let op = PatchOp::Set(
                args.next()
                    .map(PatchValue::Json)
                    .unwrap_or(PatchValue::None),
            );

            (
                "Patching and executing node".to_string(),
                Command::PatchExecuteNodes((
                    Patch {
                        node_id: Some(node_id.clone()),
                        ops: vec![(path, op)],
                        authors: Some(vec![author]),
                        ..Default::default()
                    },
                    CommandNodes::new(vec![node_id], CommandScope::Only),
                    ExecuteOptions::default(),
                )),
                false,
                false,
            )
        }
        PATCH_NODE_FORMAT => {
            let node_id = Some(node_id_arg(args.next())?);
            let property = node_property_arg(args.next())?;
            let format = args
                .next()
                .and_then(|arg| arg.as_str().map(Format::from_name))
                .unwrap_or_default();
            let content = args
                .next()
                .and_then(|arg| arg.as_str().map(String::from))
                .unwrap_or_default();
            let content_type = args
                .next()
                .and_then(|arg| {
                    arg.as_str()
                        .and_then(|value| ContentType::from_str(value).ok())
                })
                .unwrap_or_default();

            (
                "Patching node format".to_string(),
                Command::PatchNodeFormat {
                    node_id,
                    property,
                    format,
                    content,
                    content_type,
                },
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
                    ops: vec![(NodePath::default(), PatchOp::Verify)],
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
            let scope = args
                .next()
                .and_then(|value| serde_json::from_value(value).ok())
                .unwrap_or_default();

            // Only update if running an instruction or chat message (since these update
            // the content of the document)
            let update = matches!(
                node_type,
                NodeType::InstructionBlock | NodeType::InstructionInline | NodeType::ChatMessage
            );

            (
                format!("Running node {node_type}"),
                Command::ExecuteNodes((
                    CommandNodes::new(vec![node_id], scope),
                    ExecuteOptions::default(),
                )),
                true,
                update,
            )
        }
        RUN_CURR => {
            let position = position_arg(args.next())?;
            let root = root.read().await;
            if let Some(mut node_id) = root.node_id_closest(position) {
                let mut node_type = NodeType::try_from(&node_id).map_err(internal_error)?;

                // If the node type is an `IfBlockClause` then find the parent `IfBlock` to execute
                if matches!(node_type, NodeType::IfBlockClause) {
                    if let Some(if_block_node_id) =
                        root.node_type_ancestor(NodeType::IfBlock, position)
                    {
                        node_type = NodeType::IfBlock;
                        node_id = if_block_node_id;
                    }
                }

                // Only update if running an instruction or chat message (in a standalone chat) since
                // these update the content of the document
                let update = matches!(
                    node_type,
                    NodeType::InstructionBlock
                        | NodeType::InstructionInline
                        | NodeType::ChatMessage
                );

                // Return the node type and id so that the client can do something with it if
                // necessary (e.g. opening a view)
                return_value = Some(json!([node_type.to_string(), node_id.to_string()]));

                (
                    "Running current node".to_string(),
                    Command::ExecuteNodes((
                        CommandNodes::new(vec![node_id], CommandScope::Only),
                        ExecuteOptions::default(),
                    )),
                    true,
                    update,
                )
            } else {
                tracing::error!("No node to run at current position");
                return Ok(None);
            }
        }
        RUN_CHAT => {
            let chat_id = node_id_arg(args.next())?;

            let text = args
                .next()
                .and_then(|arg| arg.as_str().map(String::from))
                .unwrap_or_default();

            let files = args.next().and_then(|arg| serde_json::from_value(arg).ok());

            (
                "Adding chat message".to_string(),
                Command::PatchExecuteChat {
                    chat_id,
                    text,
                    files,
                },
                false,
                true,
            )
        }
        RUN_DOC => (
            format!("Running {file_name}"),
            Command::ExecuteDocument(ExecuteOptions::default()),
            true,
            // Always update because document may include instructions that were executed
            true,
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
            let node_id = if let Some(node_id) = root.read().await.node_id_closest(position) {
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
                        NodePath::from(NodeProperty::ExecutionMode),
                        PatchOp::Set(ExecutionMode::Lock.to_value().unwrap_or_default()),
                    )],
                    ..Default::default()
                }),
                false,
                true,
            )
        }
        UNLOCK_CURR => {
            let position = position_arg(args.next())?;
            let node_id = if let Some(node_id) = root.read().await.node_id_closest(position) {
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
                        NodePath::from(NodeProperty::ExecutionMode),
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
                Ok(position) => match root.read().await.instruction_ancestor(position) {
                    Some(id) => id,
                    None => {
                        tracing::error!("No command at current position");
                        return Ok(None);
                    }
                },
                Err(..) => node_id_arg(args.next())?,
            };

            let (title, path, op) = match command {
                PREV_NODE => (
                    "Previous suggestion".to_string(),
                    NodePath::from(NodeProperty::ActiveSuggestion),
                    PatchOp::Decrement,
                ),
                NEXT_NODE => (
                    "Next suggestion".to_string(),
                    NodePath::from(NodeProperty::ActiveSuggestion),
                    PatchOp::Increment,
                ),
                ARCHIVE_NODE => (
                    "Accepting suggestion and archiving command".to_string(),
                    NodePath::new(),
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
                Ok(position) => match root.read().await.instruction_ancestor(position) {
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
                            NodePath::from(NodeProperty::Feedback),
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
        INSERT_NODE => {
            // Required args
            let position = position_arg(args.next())?;
            let node_type = node_type_arg(args.next())?;

            // Optional args for `InstructionBlock`s
            let instruction_type = args
                .next()
                .and_then(|value| serde_json::from_value(value).ok())
                .unwrap_or_default();
            let prompt = args
                .next()
                .and_then(|value| serde_json::from_value(value).ok())
                .map(|value: String| PromptBlock {
                    target: Some(value),
                    ..Default::default()
                })
                .unwrap_or_default();
            let message = args
                .next()
                .and_then(|value| serde_json::from_value(value).ok())
                .map(|msg: String| InstructionMessage::from(msg))
                .unwrap_or_default();
            let model_parameters = args
                .next()
                .and_then(|value| serde_json::from_value(value).ok())
                .map(|model_ids| {
                    Box::new(ModelParameters {
                        model_ids,
                        ..Default::default()
                    })
                })
                .unwrap_or_default();

            // Create the new node
            let block = match node_type {
                NodeType::Chat => Block::Chat(Chat {
                    is_embedded: Some(true),
                    content: Vec::new(),
                    ..Default::default()
                }),
                NodeType::InstructionBlock => Block::InstructionBlock(InstructionBlock {
                    instruction_type,
                    prompt,
                    message,
                    model_parameters,
                    ..Default::default()
                }),
                _ => return Err(invalid_request(format!("Unhandled node type: {node_type}"))),
            };

            // Return the node's id so that the client can subscribe to its DOM
            return_value = block
                .node_id()
                .map(|id| serde_json::Value::String(id.to_string()));

            // Create a patch to add to chat to the document's `content`
            let value = block.to_value().map_err(|error| {
                internal_error(format!("While converting block to patch value: {error}"))
            })?;

            // Find where to insert the block based on the position in the text document
            // falling back to appending to the end of the document's root node's content.
            let (node_id, op) = match root.read().await.block_content_index(position) {
                Some((node_id, index)) => {
                    let op = match block {
                        // For edit and fix instructions, wrap the node at the index
                        Block::InstructionBlock(InstructionBlock {
                            instruction_type: InstructionType::Edit | InstructionType::Fix,
                            ..
                        }) => PatchOp::Wrap((index..(index + 1), value, NodeProperty::Content)),
                        // For all other blocks, insert at the index
                        _ => PatchOp::Insert(vec![(index, value)]),
                    };
                    (Some(node_id), op)
                }
                None => (None, PatchOp::Push(value)),
            };

            // Patch the `content` of the document
            let patch = Patch {
                node_id,
                ops: vec![(NodePath::from(NodeProperty::Content), op)],
                ..Default::default()
            };

            (
                format!("Inserting {node_type}"),
                Command::PatchNode(patch),
                false,
                true,
            )
        }
        INSERT_CLONES | INSERT_INSTRUCTION => {
            let position = position_arg(args.next())?;
            args.next(); // Skip the argument for the URI of the source document (already used)
            let node_ids: Vec<NodeId> = args
                .next()
                .and_then(|value| serde_json::from_value(value).ok())
                .ok_or_else(|| invalid_request("node ids arg missing"))?;
            let instruction_type = args
                .next()
                .and_then(|value| serde_json::from_value(value).ok())
                .unwrap_or_default();
            let execution_mode = args
                .next()
                .and_then(|value| serde_json::from_value(value).ok());

            // Get the root node from the source document
            let source_doc = source_doc
                .ok_or_else(|| invalid_request("Source document URI missing or invalid"))?;
            let source_doc = source_doc.read().await;

            // Clone the nodes from the source
            // Using the `.inspect()` method with `find()`, rather than using `.find()`
            // methods meads we only need to take read lock once.
            let nodes = source_doc
                .inspect(|root| {
                    let mut nodes: Vec<Node> = Vec::new();
                    for node_id in node_ids.clone() {
                        if let Some(node) = find(root, node_id) {
                            nodes.push(node)
                        }
                    }
                    nodes
                })
                .await;

            // Convert the nodes into blocks (if necessary) and replicate (to avoid having duplicate ids)
            let blocks: Vec<Block> = nodes
                .into_iter()
                .map(|node| match node {
                    Node::CodeChunk(mut chunk) => {
                        // Remove any execution bounds on code chunks (will usually be `Fork` or `Box`)
                        // when inserting into main document
                        chunk.execution_bounds = None;
                        Ok(Block::CodeChunk(chunk))
                    }
                    _ => replicate(&Block::try_from(node)?),
                })
                .try_collect()
                .map_err(internal_error)?;

            // If appropriate, wrap in a command
            let blocks = if matches!(command, INSERT_INSTRUCTION) {
                vec![Block::InstructionBlock(InstructionBlock {
                    instruction_type,
                    execution_mode,
                    content: Some(blocks),
                    ..Default::default()
                })]
            } else {
                blocks
            };

            // Convert blocks to patch values
            let values = blocks.into_iter().map(PatchValue::Block);

            // Find where to insert the block based on the position in the text document
            // falling back to appending to the end of the document's root node's content.
            let (node_id, op) = match root.read().await.block_content_index(position) {
                Some((node_id, index)) => {
                    let values = values
                        .enumerate()
                        .map(|(offset, value)| (index + offset, value))
                        .collect_vec();
                    (Some(node_id), PatchOp::Insert(values))
                }
                None => (None, PatchOp::Append(values.collect())),
            };

            // Patch the content of the destination document
            let patch = Patch {
                node_id,
                ops: vec![(NodePath::from(NodeProperty::Content), op)],
                ..Default::default()
            };

            (
                "Cloning nodes".to_string(),
                Command::PatchNode(patch),
                false,
                true,
            )
        }
        MERGE_NODE => {
            let old_id = node_id_arg(args.next())?;
            let new_id = node_id_arg(args.next())?;

            let doc = doc.read().await;
            let old = doc
                .find(old_id.clone())
                .await
                .ok_or_else(|| internal_error("Unable to find old node"))?;
            let new = doc
                .find(new_id)
                .await
                .ok_or_else(|| internal_error("Unable to find new node"))?;

            let mut patch = diff(&old, &new, None, None).map_err(internal_error)?;
            patch.node_id = Some(old_id);

            (
                "Merging node".to_string(),
                Command::PatchNode(patch),
                false,
                true,
            )
        }
        DELETE_NODE => {
            args.next(); // Node type arg not currently used
            let node_id = node_id_arg(args.next())?;

            (
                "Deleting node".to_string(),
                Command::PatchNode(Patch {
                    node_id: Some(node_id.clone()),
                    ops: vec![(NodePath::new(), PatchOp::Delete)],
                    ..Default::default()
                }),
                false,
                true,
            )
        }
        CREATE_CHAT => {
            let range = args
                .next()
                .and_then(|value| serde_json::from_value(value).ok());
            let instruction_type = args
                .next()
                .and_then(|value| serde_json::from_value(value).ok());
            let node_type = node_type_arg(args.next()).ok();
            let prompt = args
                .next()
                .and_then(|value| serde_json::from_value::<String>(value).ok());
            let execute_chat: bool = args
                .next()
                .and_then(|value| serde_json::from_value::<bool>(value).ok())
                .unwrap_or_default();

            let root = root.read().await;

            // Get the ids and types of any blocks spanning the range to infer prompt
            let (node_ids, node_types) =
                if matches!(instruction_type, Some(InstructionType::Create)) {
                    // If an explicit create instruction, then ignore nodes spanning
                    // range (likely that cursor accidentally on boundary and user does not
                    // want to use them as suggestions etc)
                    (
                        Vec::new(),
                        node_type
                            .map(|node_type| vec![node_type])
                            .unwrap_or_default(),
                    )
                } else if let Some(range) = range {
                    // Get any blocks spanning the range
                    let node_ids = root.block_ids_spanning(range);

                    let node_types: Vec<NodeType> = node_ids
                        .iter()
                        .map(NodeType::try_from)
                        .try_collect()
                        .map_err(internal_error)?;

                    (node_ids, node_types)
                } else {
                    (Vec::new(), Vec::new())
                };

            // Infer the instruction type based on the number of blocks selected
            // and whether they have any errors
            let instruction_type = if instruction_type.is_some() {
                instruction_type
            } else if node_types.is_empty() {
                None
            } else if let (1, Some(NodeType::CodeChunk | NodeType::MathBlock)) =
                (node_types.len(), node_types.first())
            {
                // Check if the node has warnings or errors and
                if let Some(node_id) = node_ids.first() {
                    if match doc.read().await.find(node_id.clone()).await {
                        Some(Node::CodeChunk(node)) => node.has_warnings_errors_or_exceptions(),
                        Some(Node::MathBlock(node)) => node.has_warnings_errors_or_exceptions(),
                        _ => false,
                    } {
                        Some(InstructionType::Fix)
                    } else {
                        Some(InstructionType::Edit)
                    }
                } else {
                    Some(InstructionType::Edit)
                }
            } else {
                Some(InstructionType::Edit)
            };

            let node_types = (!node_types.is_empty()).then_some(
                node_types
                    .iter()
                    .map(|node_type| node_type.to_string())
                    .collect_vec(),
            );

            // If nodes selected, replicate them into the chat.
            let (target_nodes, content) = if !node_ids.is_empty() {
                let target_nodes = node_ids.iter().map(|id| id.to_string()).collect_vec();

                // Get clones of the blocks
                let content = {
                    let doc = doc.read().await;
                    let mut content = Vec::new();
                    for node_id in node_ids {
                        if let Some(block) = doc
                            .find(node_id)
                            .await
                            .and_then(|node| Block::try_from(node).ok())
                        {
                            content.push(block);
                        }
                    }
                    content
                };

                // Replicate to avoid duplicate ids
                let content = replicate(&content).map_err(internal_error)?;

                (Some(target_nodes), content)
            } else {
                (None, Vec::new())
            };

            // If no prompt provided then infer one from the instruction type, node type etc
            let target = match prompt {
                Some(prompt) => Some(prompt),
                None => prompts::infer(&instruction_type, &node_types, &None)
                    .await
                    .map(|prompt| [&prompt.name, "?"].concat()),
            };

            let chat = Chat {
                prompt: PromptBlock {
                    // Do not set `node_types` since already used to infer prompt
                    // if necessary and can be confusing, especially if more than
                    // on node is selected
                    instruction_type,
                    target,
                    ..Default::default()
                },
                is_embedded: Some(true),
                target_nodes,
                content,
                ..Default::default()
            };

            let chat_id = chat.node_id().clone();

            return_value = Some(serde_json::Value::String(chat_id.to_string()));

            // Find where to insert the chat based on the position in the text document
            // falling back to appending to the end of the document's root node's content.

            let chat = Block::Chat(chat).to_value().map_err(internal_error)?;
            let (node_id, op) = match range
                .map(|range| range.start)
                .and_then(|position| root.block_content_index(position))
            {
                Some((node_id, index)) => (
                    Some(node_id),
                    // Insert before the selected range
                    PatchOp::Insert(vec![(index.saturating_sub(1), chat)]),
                ),
                None => (None, PatchOp::Push(chat)),
            };

            // Patch the content of the destination document
            let patch = Patch {
                node_id,
                ops: vec![(NodePath::from(NodeProperty::Content), op)],
                // Run compile so that that chat's prompt block is compiled
                // to infer the target prompt
                compile: true,
                // Execute if specified
                execute: execute_chat.then_some(vec![chat_id]),
                ..Default::default()
            };

            (
                "Creating chat".to_string(),
                Command::PatchNode(patch),
                false,
                true, // Update after so that the new embedded chat is visible
            )
        }
        EXPORT_DOC => {
            let path = path_buf_arg(args.next())?;

            let doc = doc.read().await;
            doc.export(&path, Some(EncodeOptions::default()))
                .await
                .map_err(internal_error)?;

            return Ok(None);
        }
        command => return Err(invalid_request(format!("Unknown command `{command}`"))),
    };

    // Send the command to the document with a subscription to receive status updates
    let mut status_receiver = match doc.read().await.command_subscribe(command).await {
        Ok(receiver) => receiver,
        Err(error) => {
            client
                .show_message(ShowMessageParams {
                    typ: MessageType::ERROR,
                    message: format!("While sending command to {uri}: {error}"),
                })
                .ok();
            return Ok(return_value);
        }
    };

    // Create a progress notification and spawn a task to update it
    let progress_sender = create_progress(client.clone(), title, cancellable).await;
    let mut client = client.clone();
    let uri = uri.clone();
    tokio::spawn(async move {
        while let Some(status) = status_receiver.recv().await {
            if status.finished() {
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

                if !update_after {
                    return;
                }

                // If necessary, create a task to update the text for the node when the command is finished
                // TODO: this is not ideal because it does not handle case where nodes need to be updated after
                // the whole document is run, and because it has to hackily wait for the final patch to be
                // applied. Instead need to set up a patch watcher that allows us to watch for
                // the node types and ids to which a patch was applied.
                tokio::time::sleep(Duration::from_millis(100)).await;

                // Format the doc and apply any edits
                let edits = match format_doc(doc.clone(), format.clone(), source.clone()).await {
                    Ok(Some(edits)) => edits,
                    Ok(None) => continue,
                    Err(error) => {
                        tracing::error!("While formatting doc after command: {error}");
                        continue;
                    }
                };

                let edits = edits.into_iter().map(OneOf::Left).collect();
                client
                    .apply_edit(ApplyWorkspaceEditParams {
                        edit: WorkspaceEdit {
                            document_changes: Some(DocumentChanges::Edits(vec![
                                TextDocumentEdit {
                                    text_document: OptionalVersionedTextDocumentIdentifier {
                                        uri,
                                        version: None,
                                    },
                                    edits,
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

    Ok(return_value)
}

/// Create an invalid request error
fn invalid_request<T: Display>(value: T) -> ResponseError {
    ResponseError::new(ErrorCode::INVALID_REQUEST, value.to_string())
}

/// Create an internal error
fn internal_error<T: Display>(value: T) -> ResponseError {
    ResponseError::new(ErrorCode::INTERNAL_ERROR, value.to_string())
}

/// Extract a document URI from a command arg
pub(super) fn uri_arg(arg: Option<Value>) -> Result<Url, ResponseError> {
    arg.and_then(|value| serde_json::from_value(value).ok())
        .ok_or_else(|| invalid_request("Document URI argument missing or invalid"))
}

/// Extract a Stencila [`NodeType`] from a command arg
fn node_type_arg(arg: Option<Value>) -> Result<NodeType, ResponseError> {
    arg.and_then(|value| value.as_str().map(String::from))
        .and_then(|node_id| NodeType::from_str(&node_id).ok())
        .ok_or_else(|| invalid_request("Node type argument missing or invalid"))
}

/// Extract a Stencila [`NodeId`] from a command arg
fn node_id_arg(arg: Option<Value>) -> Result<NodeId, ResponseError> {
    arg.and_then(|value| value.as_str().map(String::from))
        .and_then(|node_id| NodeId::from_str(&node_id).ok())
        .ok_or_else(|| invalid_request("Node id argument missing or invalid"))
}

/// Extract a Stencila [`NodeProperty`] from a command arg
fn node_property_arg(arg: Option<Value>) -> Result<NodeProperty, ResponseError> {
    arg.and_then(|value| value.as_str().map(String::from))
        .and_then(|node_id| NodeProperty::from_str(&node_id).ok())
        .ok_or_else(|| invalid_request("Node property argument missing or invalid"))
}

/// Extract a position from a command arg
fn position_arg(arg: Option<Value>) -> Result<Position, ResponseError> {
    arg.and_then(|value| serde_json::from_value(value).ok())
        .ok_or_else(|| invalid_request("Position argument missing or invalid"))
}

/// Extract a range from a command arg
#[allow(unused)]
fn range_arg(arg: Option<Value>) -> Result<Range, ResponseError> {
    arg.and_then(|value| serde_json::from_value(value).ok())
        .ok_or_else(|| invalid_request("Range argument missing or invalid"))
}

/// Extract a `PathBuf` from a command arg
fn path_buf_arg(arg: Option<Value>) -> Result<PathBuf, ResponseError> {
    arg.and_then(|value| serde_json::from_value(value).ok())
        .ok_or_else(|| invalid_request("Path argument missing or invalid"))
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
