use common::{
    eyre::Result,
    serde::{Deserialize, Serialize},
    tokio::sync::mpsc::Receiver,
    tracing,
};

use crate::Document;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, crate = "common::serde")]
pub struct NodePatch {}

impl Document {
    /// Synchronize the document with a string buffer
    ///
    /// This function spawns a task to synchronize a document's root node
    /// with an in-memory string buffer.
    #[tracing::instrument(skip(self, _patch_receiver))]
    pub async fn sync_nodes(&self, _patch_receiver: Receiver<NodePatch>) -> Result<()> {
        tracing::trace!("Syncing nodes");

        Ok(())
    }
}
