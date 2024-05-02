use common::{
    tokio::{self},
    tracing,
};
use schema;

use crate::{
    Document, DocumentPatchReceiver, DocumentRoot, DocumentUpdateReceiver, DocumentWatchSender,
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
        watch_sender: DocumentWatchSender,
    ) {
        tracing::debug!("Document update task started");

        loop {
            tokio::select! {
                Some(node) = update_receiver.recv() => {
                    tracing::trace!("Document root node update received");

                    let root = &mut *root.write().await;
                    if let Err(error) = schema::merge(root, &node, None) { // TODO: supply authors
                        tracing::error!("While merging update into root: {error}");
                    }
                },
                Some(patch) = patch_receiver.recv() => {
                    tracing::trace!("Document root node patch received");

                    let root = &mut *root.write().await;
                    if let Err(error) = schema::patch(root, patch) {
                        tracing::error!("While applying patch to root: {error}");
                    }
                },
                else => {
                    tracing::debug!("Both update and patch channels closed");
                    break;
                },
            }

            // Send the node to watchers
            if watch_sender.receiver_count() > 0 {
                let root = root.read().await.clone();

                if let Err(error) = watch_sender.send(root) {
                    tracing::error!("While notifying watchers: {error}");
                }
            }
        }

        tracing::debug!("Document update task stopped");
    }
}
