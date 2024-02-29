use common::tracing;
use node_store::ReadNode;
use schema::{Article, Node, NodeId};

use crate::{
    Command, Document, DocumentCommandReceiver, DocumentKernels, DocumentStore,
    DocumentUpdateSender,
};

impl Document {
    /// Asynchronous task to coalesce and perform document commands
    pub(super) async fn command_task(
        mut command_receiver: DocumentCommandReceiver,
        store: DocumentStore,
        kernels: DocumentKernels,
        update_sender: DocumentUpdateSender,
    ) {
        tracing::debug!("Document command task started");

        // Receive commands
        while let Some(command) = command_receiver.recv().await {
            tracing::trace!("Document command `{}` received", command.to_string());

            match command {
                Command::ExecuteDocument => execute(&store, &kernels, None, &update_sender).await,
                Command::ExecuteNodes(command) => {
                    execute(&store, &kernels, Some(command.node_ids), &update_sender).await
                }
                _ => {
                    tracing::warn!("TODO: handle {command} command");
                }
            }
        }

        tracing::debug!("Document command task stopped");
    }
}

async fn execute(
    store: &DocumentStore,
    kernels: &DocumentKernels,
    node_ids: Option<Vec<NodeId>>,
    update_sender: &DocumentUpdateSender,
) {
    // Load the root node from the store
    let mut root = {
        // This is within a block to ensure that the lock on `store` gets
        // dropped before execution
        let store = store.read().await;
        Node::load(&*store).unwrap()
    };

    let mut kernels = kernels.write().await;

    // TODO: this executes the entire document and then sends a single update.
    // Instead, have a `node_execute` function that takes a channel which can
    // receive updates for individual nodes after they are updated

    // TODO: remove this temporary hack to get the execution status to running
    // and send an update
    if let Node::Article(Article { options, .. }) = &mut root {
        options.execution_status = Some(schema::ExecutionStatus::Running);
        if let Err(error) = update_sender.send(root.clone()).await {
            tracing::error!("While sending root update: {error}");
        }
    }

    // Execute the root node
    node_execute::execute(&mut root, &mut kernels, node_ids)
        .await
        .unwrap();

    // Send the updated root node to the store
    if let Err(error) = update_sender.send(root).await {
        tracing::error!("While sending root update: {error}");
    }
}
