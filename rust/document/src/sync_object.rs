use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use json_patch::{PatchOperation, ReplaceOperation};

use common::{
    eyre::Result,
    indexmap::IndexMap,
    serde::{Deserialize, Serialize},
    serde_json,
    tokio::{
        self,
        sync::{
            mpsc::{Receiver, Sender},
            Mutex,
        },
    },
    tracing,
};
use node_map::{node_map, NodePath};
use schema::{Node, NodeId};

use crate::Document;

/// The state of a Stencila Schema node including a map of node ids to paths
///
/// This struct simply exists to provide a single serializable entity that
/// can be diffed to create patches for both the node and its map.
#[derive(Serialize)]
#[serde(crate = "common::serde")]
struct ObjectState {
    /// The root node which the object represents
    node: Node,

    /// A map between node ids and paths within the root node
    map: IndexMap<NodeId, NodePath>,
}

/// A patch to apply to a JSON object representing the document
///
/// An `ObjectPatch` is a collection of operations with a version which is
/// used to ensure that the operations are applied to the correct version.
///
/// An incoming patch with version `0` is a request for a "reset" patch and is
/// normally only received after a client has missed a patch (i.e. when versions are not sequential).
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, crate = "common::serde")]
pub struct ObjectPatch {
    /// The version of the patch
    version: u32,

    /// The operations in the patch
    ops: Vec<PatchOperation>,
}

impl ObjectPatch {
    /// Create a patch to initialize or reset the whole object
    ///
    /// Note: as per https://datatracker.ietf.org/doc/html/rfc6901/ an empty
    /// path refers to the whole document.
    fn reset(version: u32, value: serde_json::Value) -> ObjectPatch {
        ObjectPatch {
            version,
            ops: vec![PatchOperation::Replace(ReplaceOperation {
                path: String::new(),
                value,
            })],
        }
    }
}

impl Document {
    /// Synchronize the document with a JSON object
    ///
    /// This function spawns a task to synchronize a document's root node
    /// with an in-memory JSON object. Changes to the document are sent to
    /// the client as `ObjectPatch`s using the `patch_sender`.
    ///
    /// Incoming messages for a reset patch from the `patch_receiver` are handled
    /// in another task. All other messages on that receiver are ignored.
    #[tracing::instrument(skip_all)]
    pub async fn sync_object(
        &self,
        mut patch_receiver: Receiver<ObjectPatch>,
        patch_sender: Sender<ObjectPatch>,
    ) -> Result<()> {
        tracing::trace!("Syncing JSON");

        // Get the node and its JSON value
        let node = self.root.read().await;
        let map = node_map(&*node);
        let value = serde_json::to_value(&ObjectState {
            node: node.clone(),
            map,
        })?;

        // Create current state mutex and initialize the version
        let current = Arc::new(Mutex::new(value.clone()));
        let version = Arc::new(AtomicU32::new(1));

        // Start task to receive incoming `ObjectPatch`s from the client and send a
        // reset patch if version==0. All other patches are ignored.
        let current_clone = current.clone();
        let version_clone = version.clone();
        let patch_sender_clone = patch_sender.clone();
        tokio::spawn(async move {
            while let Some(patch) = patch_receiver.recv().await {
                tracing::trace!("Received object patch");

                if patch.version == 0 {
                    let reset = ObjectPatch::reset(
                        version_clone.load(Ordering::SeqCst),
                        current_clone.lock().await.clone(),
                    );
                    if let Err(error) = patch_sender_clone.send(reset).await {
                        tracing::error!("While sending reset object patch: {error}");
                    }
                }
            }
        });

        // Start task to listen for changes to the document's root node,
        // convert them to a `ObjectPatch` and send to the client
        let mut node_receiver = self.watch_receiver.clone();
        tokio::spawn(async move {
            // Send initial patch to set initial content
            let init = ObjectPatch::reset(version.load(Ordering::SeqCst), value);
            if let Err(error) = patch_sender.send(init).await {
                tracing::error!("While sending initial JSON patch: {error}");
            }

            while node_receiver.changed().await.is_ok() {
                tracing::trace!("Root node changed, updating JSON object");

                // Get the new version of the node and its JSON serialization
                let node = node_receiver.borrow_and_update().clone();
                let map = node_map(&node);
                let new = match serde_json::to_value(&ObjectState { node, map }) {
                    Ok(new) => new,
                    Err(error) => {
                        tracing::error!("While serializing node: {error}");
                        continue;
                    }
                };

                let mut current = current.lock().await;

                // Diff the new and current state to create a set of patch ops
                let ops = json_patch::diff(&current, &new).0;

                // Do not send a patch if no operations (i.e. no change in doc)
                if ops.is_empty() {
                    continue;
                }

                // Update current state
                *current = new;
                drop(current);

                // Increment version
                let version = version.fetch_add(1, Ordering::SeqCst) + 1;

                // Create and send a patch
                let patch = ObjectPatch { version, ops };
                if patch_sender.send(patch).await.is_err() {
                    // Most likely receiver has dropped so just finish this task
                    break;
                }
            }
        });

        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use json_patch::{AddOperation, RemoveOperation};

    use common::{eyre::bail, tokio::sync::mpsc::channel};
    use common_dev::pretty_assertions::assert_eq;
    use schema::{
        shortcuts::{art, p, t},
        Article,
    };

    use super::*;

    /// Test sending patches to the client
    #[tokio::test]
    async fn send_patches() -> Result<()> {
        // Create a document and start syncing with Markdown buffer
        let document = Document::new()?;
        {
            let mut root = document.root.write().await;
            *root = Node::Article(Article::default());
        }

        let (.., in_receiver) = channel(1);
        let (out_sender, mut out_receiver) = channel(4);
        document.sync_object(in_receiver, out_sender).await?;

        // First patch should be a reset with empty content
        let patch = out_receiver.recv().await.unwrap();
        assert_eq!(patch.version, 1);
        if let PatchOperation::Replace(ReplaceOperation { path, .. }) = &patch.ops[0] {
            assert_eq!(path, &String::new());
        } else {
            bail!("unexpected patch operation {patch:?}")
        }

        // Test inserting content
        document
            .update(art([p([t("Hello world")])]), None, None)
            .await?;
        let patch = out_receiver.recv().await.unwrap();
        assert_eq!(patch.version, 2);
        if let Some(PatchOperation::Add(AddOperation { path, .. })) = &patch.ops.get(2) {
            assert_eq!(path, "/node/content/0");
        } else {
            bail!("unexpected patch operation {patch:?}")
        }

        // Test replacing content
        document.update(art([p([t("Hello!")])]), None, None).await?;
        let patch = out_receiver.recv().await.unwrap();
        assert_eq!(patch.version, 3);
        if let Some(PatchOperation::Replace(ReplaceOperation { path, .. })) = &patch.ops.get(3) {
            assert_eq!(path, "/node/content/0/content/0/value/string");
        } else {
            bail!("unexpected patch operation {patch:?}")
        }

        // Test removing content
        document.update(art([p([])]), None, None).await?;
        let patch = out_receiver.recv().await.unwrap();
        assert_eq!(patch.version, 4);
        if let Some(PatchOperation::Remove(RemoveOperation { path, .. })) = &patch.ops.first() {
            assert_eq!(path, "/node/content/0/content/0");
        } else {
            bail!("unexpected patch operation {patch:?}")
        }

        Ok(())
    }
}
