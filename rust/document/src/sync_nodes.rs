use common::{
    eyre::Result,
    serde::Deserialize,
    tokio::{self, sync::mpsc::Receiver},
    tracing,
};
use schema::Node;

use crate::Document;

/**
 * A patch to apply to a node in a document
 * 
 * Intentionally similar to JSON Patch (https://jsonpatch.com/) but adds
 * an `id` field and has a singular `path`.
 */
#[derive(Debug, Deserialize)]
#[serde(crate = "common::serde")]
#[allow(unused)]
pub struct NodePatch {
    /// The id of the document node this is the target of this patch
    id: String,

    /// The patch operation
    op: NodePatchOperation,

    /// The path to the property or item of the target to be patched
    ///
    /// Unlike JSON Patch, this currently only supports paths with a
    /// single item.
    path: NodePatchSlot,

    /// The path to the property or item from which to move or copy
    /// 
    /// Only applies to [`NodePatchOperation::Move`] and [`NodePatchOperation::Copy`]
    /// operations.
    from: Option<NodePatchSlot>,

    /// The value of the property or item to add or replace
    /// 
    /// Only applies to [`NodePatchOperation::Add`] and [`NodePatchOperation::Replace`]
    /// operations.
    value: Option<Node>,
}

/// A patch operation
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase", crate = "common::serde")]
enum NodePatchOperation {
    Add,
    Remove,
    Replace,
    Move,
    Copy,
}

/// A slot in the path of a patch
#[derive(Debug, Deserialize)]
#[serde(untagged, crate = "common::serde")]
enum NodePatchSlot {
    Index(usize),
    Name(String),
}

impl Document {
    /// Synchronize the document with a string buffer
    ///
    /// This function spawns a task to synchronize a document's root node
    /// with an in-memory string buffer.
    #[tracing::instrument(skip(self, patch_receiver))]
    pub async fn sync_nodes(&self, mut patch_receiver: Receiver<NodePatch>) -> Result<()> {
        tracing::trace!("Syncing nodes");

        tokio::spawn(async move {
            while let Some(_patch) = patch_receiver.recv().await {
                tracing::trace!("Received node patch");

                // TODO
            }
        });

        Ok(())
    }
}
