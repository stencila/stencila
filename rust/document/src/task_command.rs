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
                    current = Some((command, command_id, task));
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
                    current = Some((command, command_id, task));
                }
                ExecuteNodes(CommandNodes { node_ids, .. }) => {
                    let status_sender = status_sender.clone();
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
                            CommandStatus::Failed(format!("While executing nodes: {error}"))
                        } else {
                            CommandStatus::Succeeded
                        };

                        send_status(&status_sender, command_id, status);
                    });
                    current = Some((
                        Command::ExecuteNodes(CommandNodes::default()),
                        command_id,
                        task,
                    ));
                }
                InterruptDocument | InterruptNodes(..) => {
                    // If these have fallen down to here it means that no execution was happening at the time
                    // so just ignore them
                    send_status(&status_sender, command_id, CommandStatus::Ignored);
                }

                // Note: the following commands are not cancellable; the `current` variable is not set
                SaveDocument => {
                    // Save the document to its source and sidecar
                    if let Some(path) = &path {
                        let status_sender = status_sender.clone();
                        let path = path.to_path_buf();
                        tokio::spawn(async move {
                            let root = &*root.read().await;
                            let status = match async {
                                to_path(root, &path, None).await?;
                                to_path(root, &Document::sidecar_path(&path), None).await
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
