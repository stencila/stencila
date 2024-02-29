use common::tracing;
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
    pub(super) async fn update_task(
        mut update_receiver: DocumentUpdateReceiver,
        store: DocumentStore,
        watch_sender: DocumentWatchSender,
    ) {
        tracing::debug!("Document update task started");

        // Receive updates to the root node
        while let Some(node) = update_receiver.recv().await {
            tracing::trace!("Document node updated");

            // Dump the node to the store
            let mut store = store.write().await;
            if let Err(error) = node.dump(&mut store) {
                tracing::error!("While dumping node to store: {error}");
            }

            // Load the node from the store.
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
