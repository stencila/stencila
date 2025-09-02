use std::path::PathBuf;

use stencila_node_execute::{CompileOptions, ExecuteOptions};
use stencila_schema::{Config, Node, PatchNode, PatchOp, authorship};

use crate::{
    Command, CommandNodes, CommandScope, Document, DocumentCommandSender, DocumentPatchReceiver,
    DocumentRoot, DocumentUpdateReceiver, DocumentWatchSender,
};

impl Document {
    /// Asynchronous task to update the document's store and notify watchers of the update
    ///
    /// The root node is received on the `update_receiver` channel, dumped into
    /// the store, loaded back from the store, and sent to watchers on the `watch_sender` channel.
    ///
    /// Loading back from the store, rather than just sending watchers the received node, is
    /// necessary because the incoming node may be partial (e.g. from a format such as Markdown)
    /// but watchers need complete nodes (e.g with `executionStatus` and `output` properties).
    ///
    /// This task takes a write lock on the document's `store` for each update.
    #[tracing::instrument(skip_all)]
    pub(super) async fn update_task(
        mut update_receiver: DocumentUpdateReceiver,
        mut patch_receiver: DocumentPatchReceiver,
        root: DocumentRoot,
        path: Option<PathBuf>,
        watch_sender: DocumentWatchSender,
        command_sender: DocumentCommandSender,
    ) {
        tracing::debug!("Document update task started");

        // Get the initial config associated with the document.
        // This avoids doing this relatively expensive operation within the loop.
        let config = match Document::config_for(&root, &path).await {
            Ok((config, ..)) => config,
            Err(error) => {
                tracing::warn!("While getting document config: {error}");
                Config::default()
            }
        };

        loop {
            let (compile, lint, execute, ack) = tokio::select! {
                Some((update, ack)) = update_receiver.recv() => {
                    tracing::trace!("Document root node update received");

                    let root = &mut *root.write().await;
                    if matches!(root, Node::Null(..)) {
                        // If the root is null, then just set it to the node, rather than merging
                        let mut node = update.node;
                        if let Some(authors) = update.authors {
                            // In this case ensure authors are applied to the new node
                            if let Err(error) = authorship(&mut node, authors.clone()) {
                                tracing::error!("While apply authors to updated node: {error}");
                            }
                        }
                        *root = node;
                    } else if let Err(error) = stencila_schema::merge(root, &update.node, update.format, update.authors) {
                        tracing::error!("While merging update into root: {error}");
                    }

                    (update.compile, update.lint, update.execute, ack)
                },
                Some((mut patch, ack)) = patch_receiver.recv() => {
                    tracing::trace!("Document root node patch received");

                    let compile = patch.compile;
                    let lint = patch.lint;
                    let execute = patch.execute.clone();

                    let root = &mut *root.write().await;
                    if matches!(root, Node::Null(..)) && patch.node_id.is_none() && matches!(patch.ops.first().map(|(path, op)| (path.is_empty(), op)), Some((true,PatchOp::Set(..)))){
                        // If the root is null and the patch want to set it then do so
                        if let Some((..,PatchOp::Set(value))) = patch.ops.pop() {
                            match Node::from_value(value) {
                                Ok(node) => {
                                    *root = node
                                },
                                Err(error) => {
                                    tracing::error!("While converting value: {error}")
                                }
                            }
                        }
                    } else if let Err(error) = stencila_schema::patch(root, patch) {
                        tracing::debug!("While applying patch to root: {error}");
                    }

                    (compile, lint, execute, ack)
                },
                else => {
                    tracing::debug!("Both update and patch channels closed");
                    break;
                },
            };

            // If acknowledgement requested, acknowledge that the update was applied
            if let Some(ack) = ack
                && ack.send(()).is_err()
            {
                tracing::error!("Error sending update acknowledgement");
            }

            // Send the node to watchers
            if watch_sender.receiver_count() > 0 {
                let root = root.read().await.clone();

                if let Err(error) = watch_sender.send(root) {
                    tracing::error!("While notifying watchers: {error}");
                }
            }

            // Lint, or just compile, if requested.
            if lint || compile {
                let config = Document::config_merge_root(config.clone(), &root).await;

                let command = Command::CompileDocument {
                    config,
                    compile_options: CompileOptions {
                        should_lint: lint,
                        ..Default::default()
                    },
                };
                if let Err(error) = command_sender.send((command, None)).await {
                    tracing::error!("While sending command to document: {error}");
                    continue;
                }
            }

            // Execute if requested
            if let Some(node_ids) = execute {
                let command = Command::ExecuteNodes((
                    CommandNodes {
                        node_ids,
                        scope: CommandScope::Only,
                    },
                    ExecuteOptions::default(),
                ));
                if let Err(error) = command_sender.send((command, None)).await {
                    tracing::error!("While sending command to document: {error}");
                }
            }
        }

        tracing::debug!("Document update task stopped");
    }
}
