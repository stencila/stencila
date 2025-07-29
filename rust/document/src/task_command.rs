use std::path::PathBuf;

use codecs::DecodeOptions;
use common::{
    eyre::{Result, bail},
    itertools::Itertools,
    tokio::{self, task::JoinHandle},
    tracing,
};
use format::Format;
use node_execute::{ExecuteOptions, compile, execute, interrupt, lint};
use schema::{
    Article, Block, ChatMessage, ChatMessageOptions, CodeChunk, CodeExpression, File, Inline, Node,
    NodeId, NodePath, NodeProperty, Paragraph, Patch, PatchNode, PatchOp,
    transforms::blocks_to_inlines,
};

use crate::{
    Command, CommandNodes, CommandStatus, ContentType, Document, DocumentCommandReceiver,
    DocumentCommandStatusSender, DocumentKernels, DocumentPatchSender, DocumentRoot,
};

impl Document {
    /// Asynchronous task to coalesce and perform document commands
    #[tracing::instrument(skip_all)]
    pub(super) async fn command_task(
        mut command_receiver: DocumentCommandReceiver,
        home: PathBuf,
        root: DocumentRoot,
        kernels: DocumentKernels,
        patch_sender: DocumentPatchSender,
        decode_options: Option<DecodeOptions>,
    ) {
        tracing::debug!("Document command task started");

        // Local function to send back command status and log any error when doing so
        async fn send_status(sender: &Option<DocumentCommandStatusSender>, status: CommandStatus) {
            if let Some(sender) = sender {
                if let Err(error) = sender.send(status).await {
                    tracing::error!("While sending command status: {error}");
                }
            }
        }

        // The details of the command that is currently running
        let mut current_command_details: Option<(
            Command,
            Option<DocumentCommandStatusSender>,
            JoinHandle<()>,
        )> = None;

        while let Some((command, status_sender)) = command_receiver.recv().await {
            tracing::trace!("Document command `{}` received", command.to_string());

            use Command::*;

            // If there is already a command running, decide whether to ignore the new command,
            // interrupt execution, or wait for the current command to finish.
            if let Some((current_command, current_status_sender, current_task)) =
                &current_command_details
            {
                if !current_task.is_finished() {
                    match (&command, current_command) {
                        (CompileDocument { .. } | ExecuteDocument(..), ExecuteDocument(..)) => {
                            tracing::debug!("Ignoring document command: already executing");
                            send_status(&status_sender, CommandStatus::Ignored).await;

                            continue;
                        }
                        (InterruptDocument, ExecuteDocument(..) | ExecuteNodes(..)) => {
                            tracing::debug!("Interrupting document execution");
                            send_status(&status_sender, CommandStatus::Running).await;

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
                                send_status(current_status_sender, CommandStatus::Interrupted)
                                    .await;
                                CommandStatus::Succeeded
                            };

                            send_status(&status_sender, status).await;
                            continue;
                        }
                        (
                            InterruptNodes(CommandNodes { node_ids, .. }),
                            ExecuteDocument(..) | ExecuteNodes(..),
                        ) => {
                            tracing::debug!("Interrupting node execution");
                            send_status(&status_sender, CommandStatus::Running).await;

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
                                send_status(current_status_sender, CommandStatus::Interrupted)
                                    .await;
                                CommandStatus::Succeeded
                            };

                            send_status(&status_sender, status).await;
                            continue;
                        }
                        _ => {}
                    }
                }
            }

            let home = home.clone();
            let root = root.clone();
            let kernels = kernels.clone();
            let patch_sender = patch_sender.clone();

            match command.clone() {
                PatchNode(patch) => {
                    let status = if let Err(error) = patch_sender.send((patch, None)) {
                        CommandStatus::Failed(format!("While sending patch: {error}"))
                    } else {
                        CommandStatus::Succeeded
                    };
                    send_status(&status_sender, status).await;
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

                        patch_sender.send((
                            Patch {
                                node_id,
                                ops: vec![
                                    (NodePath::from(property), PatchOp::Clear),
                                    (NodePath::from(property), PatchOp::Append(value)),
                                ],
                                ..Default::default()
                            },
                            None,
                        ))?;

                        Ok(())
                    }
                    .await
                    {
                        CommandStatus::Failed(format!("Failed to patch node with format: {error}"))
                    } else {
                        CommandStatus::Succeeded
                    };
                    send_status(&status_sender, status).await;
                }

                CompileDocument { config } => {
                    let status_sender_clone = status_sender.clone();
                    let decode_options = decode_options.clone();
                    let task = tokio::spawn(async move {
                        let status = if let Err(error) = compile(
                            home,
                            root,
                            kernels,
                            Some(patch_sender),
                            config,
                            decode_options,
                        )
                        .await
                        {
                            CommandStatus::Failed(format!("While compiling document: {error}"))
                        } else {
                            CommandStatus::Succeeded
                        };
                        send_status(&status_sender_clone, status).await;
                    });
                    current_command_details = Some((command, status_sender, task));
                }

                LintDocument {
                    format,
                    fix,
                    config,
                } => {
                    let status_sender_clone = status_sender.clone();
                    let task = tokio::spawn(async move {
                        let status = if let Err(error) =
                            lint(home, root, kernels, Some(patch_sender), format, fix, config).await
                        {
                            CommandStatus::Failed(format!("While linting document: {error}"))
                        } else {
                            CommandStatus::Succeeded
                        };
                        send_status(&status_sender_clone, status).await;
                    });
                    current_command_details = Some((command, status_sender, task));
                }

                ExecuteDocument(options) => {
                    let status_sender_clone = status_sender.clone();
                    let task = tokio::spawn(async move {
                        let status = if let Err(error) =
                            execute(home, root, kernels, Some(patch_sender), None, Some(options))
                                .await
                        {
                            CommandStatus::Failed(format!("While executing document: {error}"))
                        } else {
                            CommandStatus::Succeeded
                        };
                        send_status(&status_sender_clone, status).await;
                    });
                    current_command_details = Some((command, status_sender, task));
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
                                Ok(patch) => patch,
                                Err(error) => {
                                    send_status(
                                        &status_sender,
                                        CommandStatus::Failed(format!(
                                            "While creating chat patch: {error}"
                                        )),
                                    )
                                    .await;
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
                                CommandStatus::Failed(format!(
                                    "While applying patch to root: {error}"
                                )),
                            )
                            .await;
                            continue;
                        }
                    }

                    // Execute the node/s
                    let status_sender_clone = status_sender.clone();
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
                        send_status(&status_sender_clone, status).await;
                    });
                    current_command_details = Some((
                        Command::ExecuteNodes((CommandNodes::default(), ExecuteOptions::default())),
                        status_sender,
                        task,
                    ));
                }

                InterruptDocument | InterruptNodes(..) => {
                    // If these have fallen down to here it means that no execution was happening at the time
                    // so just ignore them
                    send_status(&status_sender, CommandStatus::Ignored).await;
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

    let content = unwrap_docsql(content);

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
            NodePath::from(NodeProperty::Content),
            PatchOp::Push(chat_message.to_value()?),
        )],
        ..Default::default()
    })
}

/// Unwrap DocsQL [`CodeExpression`] inlines into a [`CodeChunk`] so that
/// it may have multiple [`Excerpt`] outputs
///
/// Applied so that users can write single line messages with double brace enclosed DocsQL
/// queries within them.
fn unwrap_docsql(blocks: Vec<Block>) -> Vec<Block> {
    let mut expanded = Vec::with_capacity(blocks.len());

    for block in blocks {
        if let Block::Paragraph(Paragraph { content, .. }) = block {
            let mut inlines = Vec::with_capacity(content.len());
            for inline in content {
                if let Inline::CodeExpression(
                    CodeExpression {
                        programming_language: Some(lang),
                        code,
                        ..
                    },
                    ..,
                ) = &inline
                {
                    if lang == "docsql" {
                        if !inlines.is_empty() {
                            expanded.push(Block::Paragraph(Paragraph {
                                content: std::mem::take(&mut inlines),
                                ..Default::default()
                            }));
                        }

                        expanded.push(Block::CodeChunk(CodeChunk {
                            programming_language: Some(lang.clone()),
                            code: code.clone(),
                            ..Default::default()
                        }));
                    } else {
                        inlines.push(inline)
                    }
                } else {
                    inlines.push(inline);
                }
            }
            if !inlines.is_empty() {
                expanded.push(Block::Paragraph(Paragraph {
                    content: inlines,
                    ..Default::default()
                }));
            }
        } else {
            expanded.push(block)
        }
    }

    expanded
}
