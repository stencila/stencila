use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};

use codecs::{DecodeOptions, EncodeOptions};

use common::{
    eyre::Result,
    serde::{Deserialize, Serialize},
    serde_with::skip_serializing_none,
    similar::{Algorithm, DiffTag, TextDiffConfig},
    tokio::{
        self,
        sync::{
            mpsc::{Receiver, Sender},
            Mutex,
        },
    },
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
    #[tracing::instrument(skip(self, patch_receiver))]
    pub async fn sync_nodes(&self, patch_receiver: Receiver<NodePatch>) -> Result<()> {
        tracing::trace!("Syncing nodes");

        Ok(())
    }
}
