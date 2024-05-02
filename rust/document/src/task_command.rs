use std::path::PathBuf;

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
                        (InterruptDocument, ExecuteDocument(..)) => {
                            tracing::debug!("Interrupting document execution");

                            send_status(command_id, CommandStatus::Running);

                            current_task.abort();

                            let status = if let Err(error) = interrupt(
                                home.clone(),
                                root.clone(),
                                kernels.clone(),
                                patch_sender.clone(),
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
                        _ => {}
                    }
                }
            }

            let home = home.clone();
            let store = root.clone();
            let kernels = kernels.clone();
            let patch_sender = patch_sender.clone();
            let status_sender = command_status_sender.clone();

            match command.clone() {
                CompileDocument => {
                    let task = tokio::spawn(async move {
                        let status = if let Err(error) =
                            compile(home, store, kernels, patch_sender, None, None).await
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
                            execute(home, store, kernels, patch_sender, None, Some(options)).await
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
                            store,
                            kernels,
                            patch_sender,
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
                _ => {
                    tracing::warn!("Document command `{command}` not handled yet");
                }
            }
        }

        tracing::debug!("Document command task stopped");
    }
}
