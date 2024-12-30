use std::path::PathBuf;

use codecs::{to_path, DecodeOptions, EncodeOptions, LossesResponse};
use common::{
    eyre::{bail, Report, Result},
    itertools::Itertools,
    tokio::{self, task::JoinHandle},
    tracing,
};
use format::Format;
use node_execute::{compile, execute, interrupt, ExecuteOptions};
use schema::{
    transforms::blocks_to_inlines, Article, Block, ChatMessage, ChatMessageOptions, File, Node,
    NodeId, NodeProperty, Patch, PatchNode, PatchOp, PatchPath,
};

use crate::{
    Command, CommandNodes, CommandStatus, ContentType, Document, DocumentCommandReceiver,
    DocumentCommandStatusSender, DocumentKernels, DocumentPatchSender, DocumentRoot,
    SaveDocumentSidecar, SaveDocumentSource,
};

impl Document {
    /// Asynchronous task to coalesce and perform document commands
    #[tracing::instrument(skip_all)]
    pub(super) async fn command_task(
        mut command_receiver: DocumentCommandReceiver,
        status_sender: DocumentCommandStatusSender,
        home: PathBuf,
        path: Option<PathBuf>,
        root: DocumentRoot,
        kernels: DocumentKernels,
        patch_sender: DocumentPatchSender,
    ) {
        tracing::debug!("Document command task started");

        // Local function to send back command status and log any error when doing so
        fn send_status(sender: &DocumentCommandStatusSender, id: u64, status: CommandStatus) {
            if let Err(error) = sender.send((id, status)) {
                tracing::error!("While sending command status: {error}");
            }
        }

        // The details of the command that is currently running
        let mut current_command_details: Option<(Command, u64, JoinHandle<()>)> = None;

        while let Some((command, command_id)) = command_receiver.recv().await {
            tracing::trace!("Document command `{}` received", command.to_string());

            use Command::*;

            // If there is already a command running, decide whether to ignore the new command,
            // interrupt execution, or wait for the current command to finish.
            if let Some((current_command, current_command_id, current_task)) =
                &current_command_details
            {
                if !current_task.is_finished() {
                    match (&command, current_command) {
                        (CompileDocument | ExecuteDocument(..), ExecuteDocument(..)) => {
                            tracing::debug!("Ignoring document command: already executing");
                            send_status(&status_sender, command_id, CommandStatus::Ignored);

                            continue;
                        }
                        (InterruptDocument, ExecuteDocument(..) | ExecuteNodes(..)) => {
                            tracing::debug!("Interrupting document execution");
                            send_status(&status_sender, command_id, CommandStatus::Running);

                            current_task.abort();

                            let status = if let Err(error) = interrupt(
                                home.clone(),
                                root.clone(),
                                kernels.clone(),
                                Some(patch_sender.clone()),
                                None,
                            )
                            .await
                            {
                                CommandStatus::Failed(format!(
                                    "While interrupting document: {error}"
                                ))
                            } else {
                                send_status(
                                    &status_sender,
                                    *current_command_id,
                                    CommandStatus::Interrupted,
                                );
                                CommandStatus::Succeeded
                            };

                            send_status(&status_sender, command_id, status);
                            continue;
                        }
                        (
                            InterruptNodes(CommandNodes { node_ids, .. }),
                            ExecuteDocument(..) | ExecuteNodes(..),
                        ) => {
                            tracing::debug!("Interrupting node execution");
                            send_status(&status_sender, command_id, CommandStatus::Running);

                            // Abort the current task if it has the same node_ids and scope
                            if &command == current_command {
                                current_task.abort();
                            }

                            let status = if let Err(error) = interrupt(
                                home.clone(),
                                root.clone(),
                                kernels.clone(),
                                Some(patch_sender.clone()),
                                Some(node_ids.clone()),
                            )
                            .await
                            {
                                CommandStatus::Failed(format!("While interrupting nodes: {error}"))
                            } else {
                                send_status(
                                    &status_sender,
                                    *current_command_id,
                                    CommandStatus::Interrupted,
                                );
                                CommandStatus::Succeeded
                            };

                            send_status(&status_sender, command_id, status);
                            continue;
                        }
                        _ => {}
                    }
                }
            }

            let home = home.clone();
            let path = path.clone();
            let root = root.clone();
            let kernels = kernels.clone();
            let patch_sender = patch_sender.clone();

            match command.clone() {
                PatchNode(patch) => {
                    let status = if let Err(error) = patch_sender.send(patch) {
                        CommandStatus::Failed(format!("While sending patch: {error}"))
                    } else {
                        CommandStatus::Succeeded
                    };
                    send_status(&status_sender, command_id, status);
                }
                PatchNodeFormat {
                    node_id,
                    property,
                    format,
                    content,
                    content_type,
                } => {
                    let status = if let Err(error) = async {
                        let Ok(Node::Article(Article { content, .. })) = codecs::from_str(
                            &content,
                            Some(DecodeOptions {
                                format: Some(format),
                                ..Default::default()
                            }),
                        )
                        .await
                        else {
                            bail!("Unexpected node type when patching node with format")
                        };

                        let value = match content_type {
                            ContentType::Block => content
                                .iter()
                                .filter_map(|block| block.to_value().ok())
                                .collect_vec(),
                            ContentType::Inline => blocks_to_inlines(content)
                                .iter()
                                .filter_map(|inline| inline.to_value().ok())
                                .collect_vec(),
                        };

                        patch_sender.send(Patch {
                            node_id,
                            ops: vec![
                                (PatchPath::from(property), PatchOp::Clear),
                                (PatchPath::from(property), PatchOp::Append(value)),
                            ],
                            ..Default::default()
                        })?;

                        Ok(())
                    }
                    .await
                    {
                        CommandStatus::Failed(format!("Failed to patch node with format: {error}"))
                    } else {
                        CommandStatus::Succeeded
                    };
                    send_status(&status_sender, command_id, status);
                }
                CompileDocument => {
                    let status_sender = status_sender.clone();
                    let task = tokio::spawn(async move {
                        let status = if let Err(error) =
                            compile(home, root, kernels, Some(patch_sender), None, None).await
                        {
                            CommandStatus::Failed(format!("While compiling document: {error}"))
                        } else {
                            CommandStatus::Succeeded
                        };
                        send_status(&status_sender, command_id, status);
                    });
                    current_command_details = Some((command, command_id, task));
                }
                ExecuteDocument(options) => {
                    let status_sender = status_sender.clone();
                    let task = tokio::spawn(async move {
                        let status = if let Err(error) =
                            execute(home, root, kernels, Some(patch_sender), None, Some(options))
                                .await
                        {
                            CommandStatus::Failed(format!("While executing document: {error}"))
                        } else {
                            CommandStatus::Succeeded
                        };
                        send_status(&status_sender, command_id, status);
                    });
                    current_command_details = Some((command, command_id, task));
                }

                ExecuteNodes(..) | PatchExecuteNodes(..) | PatchExecuteChat { .. } => {
                    // Extract or generate the patch if necessary
                    let (patch, node_ids, options) = match command {
                        ExecuteNodes((nodes, options)) => (None, nodes.node_ids, options),
                        PatchExecuteNodes((patch, nodes, options)) => {
                            (Some(patch), nodes.node_ids, options)
                        }
                        PatchExecuteChat {
                            chat_id,
                            text,
                            files,
                        } => {
                            let patch = match chat_patch(&chat_id, text, files).await {
                                Ok(message) => message,
                                Err(error) => {
                                    send_status(
                                        &status_sender,
                                        command_id,
                                        CommandStatus::Failed(format!(
                                            "While creating chat patch: {error}"
                                        )),
                                    );
                                    continue;
                                }
                            };

                            (Some(patch), vec![chat_id], ExecuteOptions::default())
                        }
                        _ => unreachable!(),
                    };

                    // Apply the patch if appropriate
                    if let Some(patch) = patch {
                        let root = &mut *root.write().await;
                        if let Err(error) = schema::patch(root, patch) {
                            send_status(
                                &status_sender,
                                command_id,
                                CommandStatus::Failed(format!(
                                    "While applying patch to root: {error}"
                                )),
                            );
                            continue;
                        }
                    }

                    // Execute the node/s
                    let status_sender = status_sender.clone();
                    let task = tokio::spawn(async move {
                        let status = match execute(
                            home,
                            root,
                            kernels,
                            Some(patch_sender),
                            Some(node_ids),
                            Some(options),
                        )
                        .await
                        {
                            Ok(..) => CommandStatus::Succeeded,
                            Err(error) => {
                                CommandStatus::Failed(format!("While executing nodes: {error}"))
                            }
                        };
                        send_status(&status_sender, command_id, status);
                    });
                    current_command_details = Some((
                        Command::ExecuteNodes((CommandNodes::default(), ExecuteOptions::default())),
                        command_id,
                        task,
                    ));
                }

                InterruptDocument | InterruptNodes(..) => {
                    // If these have fallen down to here it means that no execution was happening at the time
                    // so just ignore them
                    send_status(&status_sender, command_id, CommandStatus::Ignored);
                }

                // Note: the following commands are not cancellable so
                // the `current_command_details` variable is not set
                SaveDocument((source, sidecar)) => {
                    // Save the document to its source and sidecar
                    if let Some(path) = &path {
                        let status_sender = status_sender.clone();
                        let path = path.to_path_buf();
                        tokio::spawn(async move {
                            let root = &*root.read().await;
                            let status = match async {
                                if matches!(source, SaveDocumentSource::Yes) {
                                    to_path(
                                        root,
                                        &path,
                                        Some(EncodeOptions {
                                            // Ignore losses because lossless sidecar file is
                                            // encoded next.
                                            losses: LossesResponse::Ignore,
                                            ..Default::default()
                                        }),
                                    )
                                    .await?;
                                }

                                if !matches!(sidecar, SaveDocumentSidecar::No) {
                                    let path = Document::sidecar_path(&path);
                                    if matches!(sidecar, SaveDocumentSidecar::Yes)
                                        || (matches!(sidecar, SaveDocumentSidecar::IfExists)
                                            && path.exists())
                                    {
                                        to_path(root, &path, None).await?;
                                    }
                                }

                                Ok::<(), Report>(())
                            }
                            .await
                            {
                                Ok(..) => CommandStatus::Succeeded,
                                Err(error) => {
                                    CommandStatus::Failed(format!("While saving: {error}"))
                                }
                            };
                            send_status(&status_sender, command_id, status);
                        });
                    } else {
                        send_status(
                            &status_sender,
                            command_id,
                            CommandStatus::Failed("Document does not have a path".to_string()),
                        );
                    }
                }
                ExportDocument((path, options)) => {
                    // Export the document to a path (usually a different format)
                    let status_sender = status_sender.clone();
                    tokio::spawn(async move {
                        let root = &*root.read().await;
                        let status = match to_path(root, &path, Some(options)).await {
                            Ok(..) => CommandStatus::Succeeded,
                            Err(error) => {
                                CommandStatus::Failed(format!("While encoding to path: {error}"))
                            }
                        };
                        send_status(&status_sender, command_id, status);
                    });
                }
            }
        }

        tracing::debug!("Document command task stopped");
    }
}

/// Create a patch for a chat from the fields of a [`Command::PatchExecuteChat`]
async fn chat_patch(chat_id: &NodeId, text: String, files: Option<Vec<File>>) -> Result<Patch> {
    let Ok(Node::Article(Article { content, .. })) = codecs::from_str(
        &text,
        Some(DecodeOptions {
            format: Some(Format::Markdown),
            ..Default::default()
        }),
    )
    .await
    else {
        bail!("Error or unexpected node type when decoding")
    };

    let chat_message = Block::ChatMessage(ChatMessage {
        content,
        options: Box::new(ChatMessageOptions {
            files,
            ..Default::default()
        }),
        ..Default::default()
    });

    Ok(Patch {
        node_id: Some(chat_id.clone()),
        ops: vec![(
            PatchPath::from(NodeProperty::Content),
            PatchOp::Push(chat_message.to_value()?),
        )],
        ..Default::default()
    })
}
