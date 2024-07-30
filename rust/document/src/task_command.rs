use std::path::PathBuf;

use codecs::to_path;
use common::{
    tokio::{self, task::JoinHandle},
    tracing,
};
use node_execute::{compile, execute, interrupt, ExecuteOptions};

use crate::{
    Command, CommandNodes, CommandStatus, Document, DocumentCommandReceiver,
    DocumentCommandStatusSender, DocumentKernels, DocumentPatchSender, DocumentRoot,
};

impl Document {
    /// Asynchronous task to coalesce and perform document commands
    #[tracing::instrument(skip_all)]
    pub(super) async fn command_task(
        mut command_receiver: DocumentCommandReceiver,
        command_status_sender: DocumentCommandStatusSender,
        home: PathBuf,
        root: DocumentRoot,
        kernels: DocumentKernels,
        patch_sender: DocumentPatchSender,
    ) {
        tracing::debug!("Document command task started");

        let send_status = |id, status| {
            if let Err(error) = command_status_sender.send((id, status)) {
                tracing::error!("While sending command status: {error}");
            }
        };

        let mut current: Option<(Command, u64, JoinHandle<()>)> = None;
        while let Some((command, command_id)) = command_receiver.recv().await {
            tracing::trace!("Document command `{}` received", command.to_string());

            use Command::*;

            // If there is already a command running, decide whether to ignore the new command,
            // interrupt execution, or wait for the current command to finish.
            if let Some((current_command, current_command_id, current_task)) = &current {
                if !current_task.is_finished() {
                    match (&command, current_command) {
                        (CompileDocument | ExecuteDocument(..), ExecuteDocument(..)) => {
                            tracing::debug!("Ignoring document command: already executing");
                            send_status(command_id, CommandStatus::Ignored);
                            continue;
                        }
                        (InterruptDocument, ExecuteDocument(..) | ExecuteNodes(..)) => {
                            tracing::debug!("Interrupting document execution");
                            send_status(command_id, CommandStatus::Running);

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
                                tracing::error!("While interrupting document: {error}");
                                CommandStatus::Failed
                            } else {
                                send_status(*current_command_id, CommandStatus::Interrupted);
                                CommandStatus::Succeeded
                            };

                            send_status(command_id, status);
                            continue;
                        }
                        (
                            InterruptNodes(CommandNodes { node_ids, .. }),
                            ExecuteDocument(..) | ExecuteNodes(..),
                        ) => {
                            tracing::debug!("Interrupting node execution");
                            send_status(command_id, CommandStatus::Running);

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
                                tracing::error!("While interrupting nodes: {error}");
                                CommandStatus::Failed
                            } else {
                                send_status(*current_command_id, CommandStatus::Interrupted);
                                CommandStatus::Succeeded
                            };

                            send_status(command_id, status);
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
            let status_sender = command_status_sender.clone();

            match command.clone() {
                PatchNode(patch) => {
                    let status = if let Err(error) = patch_sender.send(patch) {
                        tracing::error!("While sending patch: {error}");
                        CommandStatus::Failed
                    } else {
                        CommandStatus::Succeeded
                    };
                    send_status(command_id, status);
                }
                CompileDocument => {
                    let task = tokio::spawn(async move {
                        let status = if let Err(error) =
                            compile(home, root, kernels, Some(patch_sender), None, None).await
                        {
                            tracing::error!("While compiling document: {error}");
                            CommandStatus::Failed
                        } else {
                            CommandStatus::Succeeded
                        };

                        status_sender.send((command_id, status)).ok();
                    });
                    current = Some((command, command_id, task));
                }
                ExecuteDocument(options) => {
                    let task = tokio::spawn(async move {
                        let status = if let Err(error) =
                            execute(home, root, kernels, Some(patch_sender), None, Some(options))
                                .await
                        {
                            tracing::error!("While executing document: {error}");
                            CommandStatus::Failed
                        } else {
                            CommandStatus::Succeeded
                        };

                        status_sender.send((command_id, status)).ok();
                    });
                    current = Some((command, command_id, task));
                }
                ExecuteNodes(CommandNodes { node_ids, .. }) => {
                    let task = tokio::spawn(async move {
                        let options = ExecuteOptions::default();
                        // TODO: set other options based on scope

                        let status = if let Err(error) = execute(
                            home,
                            root,
                            kernels,
                            Some(patch_sender),
                            Some(node_ids),
                            Some(options),
                        )
                        .await
                        {
                            tracing::error!("While executing nodes: {error}");
                            CommandStatus::Failed
                        } else {
                            CommandStatus::Succeeded
                        };

                        status_sender.send((command_id, status)).ok();
                    });
                    current = Some((
                        Command::ExecuteNodes(CommandNodes::default()),
                        command_id,
                        task,
                    ));
                }
                ExportDocument((path, options)) => {
                    let task = tokio::spawn(async move {
                        let root = &*root.read().await;
                        let status = match to_path(root, &path, Some(options)).await {
                            Ok(..) => CommandStatus::Succeeded,
                            Err(error) => {
                                // TODO: This and (other errors) should go back in failed status
                                tracing::error!("While encoding to path: {error}");
                                CommandStatus::Failed
                            }
                        };
                        status_sender.send((command_id, status)).ok();
                    });
                    current = Some((command, command_id, task));
                }
                InterruptDocument | InterruptNodes(..) => {
                    // If these have fallen down to here it means that no execution was happening at the time
                    // so just ignore them
                    status_sender
                        .send((command_id, CommandStatus::Ignored))
                        .ok();
                }
                _ => {
                    tracing::warn!("Document command `{command}` not handled yet");
                    status_sender.send((command_id, CommandStatus::Failed)).ok();
                }
            }
        }

        tracing::debug!("Document command task stopped");
    }
}
