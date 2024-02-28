use common::tracing;

use crate::{Document, DocumentStore, DocumentCommandReceiver};

impl Document {
    /// Asynchronous task to coalesce and perform document commands
    pub(super) async fn command_task(
        store: DocumentStore,
        mut command_receiver: DocumentCommandReceiver
    ) {
        tracing::debug!("Document command task started");

        // Receive commands
        while let Some(command) = command_receiver.recv().await {
            tracing::trace!("Document command `{}` received", command.to_string());
        }

        tracing::debug!("Document command task stopped");
    }
}
