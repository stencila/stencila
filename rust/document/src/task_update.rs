use common::{tokio, tracing};
use node_patch::NodePatchReceiver;
use node_store::{ReadNode, WriteNode};
use schema::Node;

use crate::{Document, DocumentStore, DocumentUpdateReceiver, DocumentWatchSender};

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
        mut patch_receiver: NodePatchReceiver,
        store: DocumentStore,
        watch_sender: DocumentWatchSender,
    ) {
        tracing::debug!("Document update task started");

        loop {
            tokio::select! {
                Some(node) = update_receiver.recv() => {
                    tracing::trace!("Document root node update received");

                    // Dump the node to the store
                    let mut store = store.write().await;
                    if let Err(error) = node.dump(&mut store) {
                        tracing::error!("While dumping node to store: {error}");
                    }
                },
                Some(patch) = patch_receiver.recv() => {
                    tracing::trace!("Document node patch received");

                    // Apply the patch to the store
                    let mut store = store.write().await;
                    if let Err(error) = patch.apply(&mut store) {
                        tracing::error!("While applying patch to store: {error}");
                    }
                },
                else => {
                    tracing::debug!("Both update and patch channels closed");
                    break;
                },
            }

            // Load the node from the store.
            let store = store.read().await;
            let node = match Node::load(&*store) {
                Ok(node) => node,
                Err(error) => {
                    tracing::error!("While loading node from store: {error}");
                    continue;
                }
            };

            // Send the node to watchers
            if let Err(error) = watch_sender.send(node) {
                tracing::error!("While notifying watchers: {error}");
            }
        }

        tracing::debug!("Document update task stopped");
    }
}
