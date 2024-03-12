use std::path::PathBuf;

use common::{
    tokio::{self, task::JoinHandle},
    tracing,
};
use node_execute::{execute, interrupt};

use crate::{
    Command, CommandNodeIds, Document, DocumentCommandReceiver, DocumentKernels,
    DocumentPatchSender, DocumentStore,
};

impl Document {
    /// Asynchronous task to coalesce and perform document commands
    #[tracing::instrument(skip_all)]
    pub(super) async fn command_task(
        mut command_receiver: DocumentCommandReceiver,
        home: PathBuf,
        store: DocumentStore,
        kernels: DocumentKernels,
        patch_sender: DocumentPatchSender,
    ) {
        tracing::debug!("Document command task started");

        let mut current: Option<(Command, JoinHandle<()>)> = None;
        while let Some(new_command) = command_receiver.recv().await {
            tracing::trace!("Document command `{}` received", new_command.to_string());

            use Command::*;

            if let Some((current_command, current_task)) = &current {
                if !current_task.is_finished() {
                    match (&new_command, current_command) {
                        (ExecuteDocument(..), ExecuteDocument(..)) => {
                            tracing::info!(
                                "Ignoring document execution command: already executing"
                            );

                            continue;
                        }
                        (InterruptDocument, ExecuteDocument(..)) => {
                            tracing::info!("Interrupting document execution");

                            current_task.abort();
                            if let Err(error) = interrupt(
                                home.clone(),
                                store.clone(),
                                kernels.clone(),
                                patch_sender.clone(),
                                None,
                            )
                            .await
                            {
                                tracing::error!("While interrupting document: {error}")
                            }

                            continue;
                        }
                        _ => {}
                    }
                }
            }

            match new_command.clone() {
                ExecuteDocument(options) => {
                    let home = home.clone();
                    let store = store.clone();
                    let kernels = kernels.clone();
                    let patch_sender = patch_sender.clone();
                    let task = tokio::spawn(async move {
                        if let Err(error) =
                            execute(home, store, kernels, patch_sender, None, Some(options)).await
                        {
                            tracing::error!("While executing document: {error}")
                        }
                    });
                    current = Some((new_command, task));
                }
                ExecuteNodes(command) => {
                    let home = home.clone();
                    let store = store.clone();
                    let kernels = kernels.clone();
                    let patch_sender = patch_sender.clone();
                    let task = tokio::spawn(async move {
                        if let Err(error) = execute(
                            home,
                            store,
                            kernels,
                            patch_sender,
                            Some(command.node_ids),
                            None,
                        )
                        .await
                        {
                            tracing::error!("While executing nodes: {error}")
                        }
                    });
                    current = Some((Command::ExecuteNodes(CommandNodeIds::default()), task));
                }
                _ => {
                    tracing::warn!("TODO: handle {new_command} command");
                }
            }
        }

        tracing::debug!("Document command task stopped");
    }
}
