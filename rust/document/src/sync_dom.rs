use std::{
    ops::DerefMut,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use codecs::EncodeOptions;
use common::{
    eyre::{bail, Result},
    itertools::Itertools,
    serde::{Deserialize, Serialize},
    serde_with::skip_serializing_none,
    similar::{capture_diff_slices_deadline, Algorithm, DiffTag},
    tokio::{
        self,
        sync::{
            mpsc::{Receiver, Sender},
            Mutex,
        },
    },
    tracing,
};
use format::Format;
use node_find::find;
use schema::NodeId;

use crate::Document;

/// A patch to apply to the DOM HTML representation of a document
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default, crate = "common::serde")]
pub struct DomPatch {
    /// The version of the patch
    version: u32,

    /// The operations in the patch
    ops: Vec<DomOperation>,
}

impl DomPatch {
    /// Create a patch that is a request from the client for a reset patch
    /// to be sent by the server
    ///
    /// Used when the client receives a patch from the server that is not
    /// in sequential order.
    pub fn reset_request() -> Self {
        Self {
            version: 0,
            ops: Vec::new(),
        }
    }
}

/// An operation on the DOM HTML representation of a document
#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(crate = "common::serde")]
pub struct DomOperation {
    /// The type of operation
    r#type: DomOperationType,

    /// The position in the string from which the operation is applied
    from: Option<usize>,

    /// The position in the string to which the operation is applied
    ///
    /// May be omitted for additions.
    to: Option<usize>,

    /// The string to insert between `from` and `to`.
    ///
    /// For additions and replacements; may be omitted for deletions.
    insert: Option<String>,
}

impl DomOperation {
    /// Create a content reset operation
    fn reset_content<S>(value: S) -> Self
    where
        S: Into<String>,
    {
        DomOperation {
            r#type: DomOperationType::Reset,
            insert: Some(value.into()),
            ..Default::default()
        }
    }

    /// Create a content insert operation
    fn insert_content<S>(from: usize, value: S) -> Self
    where
        S: Into<String>,
    {
        DomOperation {
            r#type: DomOperationType::Insert,
            from: Some(from),
            insert: Some(value.into()),
            ..Default::default()
        }
    }

    /// Create a content delete operation
    fn delete_content(from: usize, to: usize) -> Self {
        DomOperation {
            r#type: DomOperationType::Delete,
            from: Some(from),
            to: Some(to),
            ..Default::default()
        }
    }

    /// Create a content replace operation
    fn replace_content<S>(from: usize, to: usize, value: S) -> Self
    where
        S: Into<String>,
    {
        DomOperation {
            r#type: DomOperationType::Replace,
            from: Some(from),
            to: Some(to),
            insert: Some(value.into()),
        }
    }
}

/// The type of an operation
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase", crate = "common::serde")]
enum DomOperationType {
    /// Reset content
    #[default]
    Reset,

    /// Insert characters
    Insert,

    /// Delete characters
    Delete,

    /// Replace characters
    Replace,
}

impl Document {
    /// Synchronize the document with the browser DOM
    ///
    /// This function spawns a task to synchronize a document's root node
    /// with an in-memory string of DOM HTML. It differs from `sync_format` in that
    /// it only does diffing if the HTML content is above a a certain length.
    /// Also, it does diffing on UTF16 slices since that is what browsers use.
    #[tracing::instrument(skip_all)]
    pub async fn sync_dom(
        &self,
        node_id: Option<NodeId>,
        mut patch_receiver: Receiver<DomPatch>,
        patch_sender: Sender<DomPatch>,
    ) -> Result<String> {
        tracing::trace!("Syncing DOM");

        // The minimum length of the content that is diffed. Below this length, the entire
        // content will be sent in a patch is a single `replace` operation. The rationale
        // for having this is that is below some size it will be faster to send the whole
        // DOM rather than doing the diffing here and the patching in the browser.
        // This optimum value for this const needs to be determined.
        const MINIMUM_DIFF_LEN: usize = 1_000;

        /// Maximum number of seconds for diffing
        const MAXIMUM_DIFF_SECS: u64 = 1;

        let encode_options = Some(EncodeOptions {
            format: Some(Format::Dom),
            ..Default::default()
        });

        // Create initial encoding of the root node
        let initial_content = match node_id.as_ref() {
            Some(node_id) => {
                let root = self.root.read().await;
                let Some(node) = find(&*root, node_id.clone()) else {
                    bail!("Unable to find node `{node_id}` in document")
                };
                codecs::to_string(&node, encode_options.clone()).await?
            }
            None => {
                let node = self.root.read().await;
                codecs::to_string(&node, encode_options.clone()).await?
            }
        };

        // Create the mutex for the current content and initialize the version
        let current = Arc::new(Mutex::new(initial_content.clone()));
        let version = Arc::new(AtomicU32::new(1));

        // Start task to receive incoming patches from the client
        // Currently only "reset request patches" are handled - those that
        // have a version number of zero, and trigger a reset patch to be sent back.
        let current_clone = current.clone();
        let version_clone = version.clone();
        let patch_sender_clone = patch_sender.clone();
        tokio::spawn(async move {
            while let Some(patch) = patch_receiver.recv().await {
                tracing::trace!("Received DOM patch");

                let mut current = current_clone.lock().await;
                let current_content = current.deref_mut();

                // If the patch is not for the current version then send a reset patch
                // (if there is a patch sender) and ignore the patch
                let current_version = version_clone.load(Ordering::SeqCst);
                if patch.version != current_version {
                    let reset = DomPatch {
                        version: current_version,
                        ops: vec![DomOperation::reset_content(&*current_content)],
                    };
                    if let Err(error) = patch_sender_clone.send(reset).await {
                        tracing::error!("While sending content reset patch: {error}");
                    }
                    continue;
                }
            }
        });

        // Start task to listen for changes to the document's root node,
        // convert them to a patch and send to the client
        let mut node_receiver = self.watch_receiver.clone();
        let reset_content = initial_content.clone();
        tokio::spawn(async move {
            // Send initial patch to set initial content
            let init = DomPatch {
                version: version.load(Ordering::SeqCst),
                ops: vec![DomOperation::reset_content(reset_content)],
            };
            if let Err(error) = patch_sender.send(init).await {
                tracing::error!("While sending initial string patch: {error}");
            }

            // TODO: consider debouncing this
            while node_receiver.changed().await.is_ok() {
                tracing::trace!("Root node changed, updating string buffer");

                let root = node_receiver.borrow_and_update().clone();

                // Encode the node to a string in the format
                let new_content = match node_id.as_ref() {
                    Some(node_id) => {
                        let Some(node) = find(&root, node_id.clone()) else {
                            // If node has been removed from the document, end the task
                            tracing::debug!(
                                "Unable to find node `{node_id}`, stopping `sync_dom` task"
                            );
                            return;
                        };
                        codecs::to_string(&node, encode_options.clone()).await
                    }
                    None => codecs::to_string(&root, encode_options.clone()).await,
                };
                let new_content = match new_content {
                    Ok(string) => string,
                    Err(error) => {
                        tracing::error!("While encoding node to string: {error}");
                        continue;
                    }
                };

                let mut current = current.lock().await;
                let current_content = current.deref_mut();

                let mut ops = Vec::new();

                if new_content != *current_content {
                    if new_content.len() < MINIMUM_DIFF_LEN {
                        ops.push(DomOperation::reset_content(new_content.clone()))
                    } else {
                        let current_utf16 = current_content.encode_utf16().collect_vec();
                        let new_utf16 = new_content.encode_utf16().collect_vec();

                        let diff_ops = capture_diff_slices_deadline(
                            Algorithm::Myers,
                            &current_utf16[..],
                            &new_utf16[..],
                            Some(Instant::now() + Duration::from_secs(MAXIMUM_DIFF_SECS)),
                        );

                        // Convert the diff to a set of operations
                        let mut from = 0usize;
                        for op in diff_ops {
                            match op.tag() {
                                DiffTag::Insert => ops.push(DomOperation::insert_content(
                                    from,
                                    String::from_utf16_lossy(&new_utf16[op.new_range()]),
                                )),
                                DiffTag::Delete => ops.push(DomOperation::delete_content(
                                    from,
                                    from + op.old_range().len(),
                                )),
                                DiffTag::Replace => ops.push(DomOperation::replace_content(
                                    from,
                                    from + op.old_range().len(),
                                    String::from_utf16_lossy(&new_utf16[op.new_range()]),
                                )),
                                DiffTag::Equal => {}
                            };

                            from += op.new_range().len();
                        }
                    }

                    // Increment version
                    version.fetch_add(1, Ordering::SeqCst);

                    // Update current content
                    *current_content = new_content;
                }

                if !ops.is_empty() {
                    // Create and send a patch for the content
                    let version = version.load(Ordering::SeqCst);
                    let patch = DomPatch { version, ops };
                    if patch_sender.send(patch).await.is_err() {
                        // Most likely receiver has dropped so just finish this task
                        break;
                    }
                }
            }
        });

        Ok(initial_content)
    }
}
