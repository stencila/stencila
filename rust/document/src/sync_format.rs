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

/// A patch to apply to a string representing the document in a particular format
///
/// A `FormatPatch` is a collection of [`FormatOperation`]s with a version which is
/// used to ensure that the operations are applied to the correct version.
///
/// An incoming patch with version `0` and empty `ops` is a request for
/// a "reset" patch and is normally only received after a client has
/// missed a patch (i.e. when versions are not sequential).
///
/// Similar to a `StringPatch` in the Stencila Schema which is used for in-document
/// modifications to a string but which lacks the `version` property and
/// used different, longer, names for properties.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, crate = "common::serde")]
pub struct FormatPatch {
    /// The version of the patch
    version: u32,

    /// The operations in the patch
    ops: Vec<FormatOperation>,
}

/// An operation on a string representing the document in a particular format
///
/// Uses a similar data model as a CodeMirror change (see https://codemirror.net/examples/change/)
/// which allows this to be directly passed to/from a browser based code editor.
///
/// Extends the data model with a `type` field to allow us to also use these operations
/// for things like tracking the current selection of the user and applying operation
/// to the nodes that are currently selected.
#[skip_serializing_none]
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(crate = "common::serde")]
pub struct FormatOperation {
    /// The type of operation
    r#type: FormatOperationType,

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

impl FormatOperation {
    /// Create a reset operation
    fn reset<S>(value: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            r#type: FormatOperationType::Reset,
            insert: Some(value.into()),
            ..Default::default()
        }
    }

    /// Create an insert operation
    fn insert<S>(from: usize, value: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            r#type: FormatOperationType::Insert,
            from: Some(from),
            insert: Some(value.into()),
            ..Default::default()
        }
    }

    /// Create a delete operation
    fn delete(from: usize, to: usize) -> Self {
        Self {
            r#type: FormatOperationType::Delete,
            from: Some(from),
            to: Some(to),
            ..Default::default()
        }
    }

    /// Create a replace operation
    fn replace<S>(from: usize, to: usize, value: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            r#type: FormatOperationType::Replace,
            from: Some(from),
            to: Some(to),
            insert: Some(value.into()),
        }
    }
}

/// The type of an operation
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(crate = "common::serde", rename_all = "lowercase")]
enum FormatOperationType {
    /// Reset content
    #[default]
    Reset,

    /// Insert characters (sent by clients and server)
    Insert,

    /// Delete characters (sent by clients and server)
    Delete,

    /// Replace characters (sent by clients and server)
    Replace,

    /// Execute node/s corresponding to character positions (sent by clients only)
    Execute,

    /// Update the current viewport of the user (sent by clients only)
    Viewport,

    /// Update the current selection of the user (sent by clients only)
    Selection,
}

impl Document {
    /// Synchronize the document with a string buffer
    ///
    /// This function spawns a task to synchronize a document's root node
    /// with an in-memory string buffer.
    #[tracing::instrument(skip(self, patch_receiver, patch_sender))]
    pub async fn sync_format(
        &self,
        patch_receiver: Option<Receiver<FormatPatch>>,
        patch_sender: Option<Sender<FormatPatch>>,
        decode_options: Option<DecodeOptions>,
        encode_options: Option<EncodeOptions>,
    ) -> Result<()> {
        tracing::trace!("Syncing string");

        // Create initial encoding of the root node
        let node = self.load().await?;
        let content = codecs::to_string(&node, encode_options.clone()).await?;

        // Create the buffer and initialize the version
        let buffer = Arc::new(Mutex::new(content.clone()));
        let version = Arc::new(AtomicU32::new(1));

        // Start task to receive incoming `StringPatch`s from the client, apply them
        // to the buffer, and update the document's root node
        if let Some(mut patch_receiver) = patch_receiver {
            let buffer_clone = buffer.clone();
            let version_clone = version.clone();
            let patch_sender_clone = patch_sender.clone();
            let update_sender = self.update_sender.clone();
            tokio::spawn(async move {
                while let Some(patch) = patch_receiver.recv().await {
                    tracing::trace!("Received string patch");

                    let mut buffer = buffer_clone.lock().await;

                    // If the patch is not for the current version then send a reset patch
                    // (if there is a patch sender) and ignore the patch
                    let current_version = version_clone.load(Ordering::SeqCst);
                    if patch.version != current_version {
                        if let Some(patch_sender) = &patch_sender_clone {
                            let reset = FormatPatch {
                                version: current_version,
                                ops: vec![FormatOperation::reset(&*buffer)],
                            };
                            if let Err(error) = patch_sender.send(reset).await {
                                tracing::error!("While sending reset string patch: {error}");
                            }
                        }
                        continue;
                    }

                    // Apply the patch to the buffer
                    for op in patch.ops {
                        use FormatOperationType::*;
                        match op {
                            FormatOperation { r#type: Reset, .. } => {
                                tracing::warn!("Client attempted to reset buffer")
                            }

                            FormatOperation {
                                r#type: Insert,
                                from: Some(from),
                                to: None,
                                insert: Some(insert),
                            } => buffer.insert_str(from, &insert),

                            FormatOperation {
                                r#type: Delete,
                                from: Some(from),
                                to: Some(to),
                                insert: None,
                            } => buffer.replace_range(from..to, ""),

                            FormatOperation {
                                r#type: Replace,
                                from: Some(from),
                                to: Some(to),
                                insert: Some(insert),
                            } => buffer.replace_range(from..to, &insert),

                            FormatOperation {
                                r#type: Execute,
                                from: Some(from),
                                to: Some(to),
                                insert: None,
                            } => {
                                // TODO
                                tracing::debug!("Execute operation {from}-{to}")
                            }

                            FormatOperation {
                                r#type: Selection,
                                from: Some(from),
                                to: Some(to),
                                insert: None,
                            } => {
                                // TODO
                                tracing::debug!("Selection operation {from}-{to}")
                            }

                            _ => tracing::warn!("Client sent invalid operation"),
                        }
                    }

                    // Increment the buffer's version number
                    version_clone.fetch_add(1, Ordering::SeqCst);

                    // Update the root node
                    // TODO consider debouncing this since `from_str` and the update will be relatively expensive
                    if let Ok(node) = codecs::from_str(&buffer, decode_options.clone()).await {
                        if let Err(error) = update_sender.send(node).await {
                            tracing::error!("While sending node update: {error}");
                        }
                    }
                }
            });
        }

        // Start task to listen for changes to the document's root node,
        // convert them to a `StringPatch` and send to the client
        if let Some(patch_sender) = patch_sender {
            let mut node_receiver = self.watch_receiver.clone();
            tokio::spawn(async move {
                // Send initial patch to set initial content
                let init = FormatPatch {
                    version: version.load(Ordering::SeqCst),
                    ops: vec![FormatOperation::reset(content)],
                };
                if let Err(error) = patch_sender.send(init).await {
                    tracing::error!("While sending initial string patch: {error}");
                }

                // TODO: consider debouncing this
                while node_receiver.changed().await.is_ok() {
                    tracing::trace!("Root node changed, updating string buffer");

                    let node = node_receiver.borrow_and_update().clone();

                    // Encode the node to a string in the format
                    let new = match codecs::to_string(&node, encode_options.clone()).await {
                        Ok(string) => string,
                        Err(error) => {
                            tracing::error!("While encoding node to string: {error}");
                            continue;
                        }
                    };

                    let mut buffer = buffer.lock().await;

                    // Continue if there is no change in the string
                    if new == *buffer {
                        continue;
                    }

                    // Calculate a diff between old and new string contents
                    let diff = TextDiffConfig::default()
                        .algorithm(Algorithm::Patience)
                        .timeout(Duration::from_secs(5))
                        .diff_chars(buffer.as_str(), new.as_str());

                    // Convert the diff to a set of `StringOp`s
                    let mut ops = Vec::new();
                    let mut from = 0usize;
                    for op in diff.ops() {
                        match op.tag() {
                            DiffTag::Insert => {
                                ops.push(FormatOperation::insert(from, &new[op.new_range()]))
                            }
                            DiffTag::Delete => {
                                ops.push(FormatOperation::delete(from, from + op.old_range().len()))
                            }
                            DiffTag::Replace => ops.push(FormatOperation::replace(
                                from,
                                from + op.old_range().len(),
                                new[op.new_range()].to_string(),
                            )),
                            DiffTag::Equal => {}
                        };

                        from += op.new_range().len();
                    }

                    // Update buffer
                    *buffer = new;
                    drop(buffer);

                    // Increment version
                    let version = version.fetch_add(1, Ordering::SeqCst) + 1;

                    // Create and send a `StringPatch`
                    let patch = FormatPatch { version, ops };
                    if patch_sender.send(patch).await.is_err() {
                        // Most likely receiver has dropped so just finish this task
                        break;
                    }
                }
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use common::{eyre::Report, tokio::sync::mpsc::channel};
    use common_dev::{ntest::timeout, pretty_assertions::assert_eq};
    use format::Format;
    use schema::shortcuts::{art, p, t};

    use crate::DocumentType;

    use super::*;

    /// Test receiving patches from a client
    ///
    /// Uses a timeout because errors in patch versions can otherwise
    /// cause this to hang forever on `watch.changed()` (because the
    /// patch is rejected).
    #[tokio::test]
    #[timeout(1000)]
    async fn receive_patches() -> Result<()> {
        // Create a document and start syncing with Markdown buffer
        let document = Document::new(DocumentType::Article)?;
        let (patch_sender, patch_receiver) = channel::<FormatPatch>(1);
        document
            .sync_format(
                Some(patch_receiver),
                None,
                Some(DecodeOptions {
                    format: Some(Format::Markdown),
                    ..Default::default()
                }),
                Some(EncodeOptions {
                    format: Some(Format::Markdown),
                    ..Default::default()
                }),
            )
            .await?;

        // Use document's watch channel to be able to wait for changes to its root node
        let mut watch = document.watch_receiver.clone();

        // Get the Markdown of the document's root node
        // Note: this is NOT the Markdown in the buffer (but it should be the same as)
        let md = || async {
            let md = document
                .export(
                    None,
                    Some(EncodeOptions {
                        format: Some(Format::Markdown),
                        ..Default::default()
                    }),
                )
                .await?;
            Ok::<String, Report>(md)
        };

        // Document's Markdown should start off empty
        assert_eq!(md().await?, "");

        // Test insert operation
        patch_sender
            .send(FormatPatch {
                version: 1,
                ops: vec![FormatOperation::insert(0, "Hello world")],
            })
            .await?;
        watch.changed().await.ok();
        assert_eq!(md().await?, "Hello world");

        // Test delete operation
        patch_sender
            .send(FormatPatch {
                version: 2,
                ops: vec![FormatOperation::delete(6, 9)],
            })
            .await?;
        watch.changed().await.ok();
        assert_eq!(md().await?, "Hello ld");

        // Test replace operation
        patch_sender
            .send(FormatPatch {
                version: 3,
                ops: vec![FormatOperation::replace(6, 7, "frien")],
            })
            .await?;
        watch.changed().await.ok();
        assert_eq!(md().await?, "Hello friend");

        Ok(())
    }

    /// Test sending patches to the client
    #[tokio::test]
    async fn send_patches() -> Result<()> {
        // Create a document and start syncing with Markdown buffer
        let document = Document::new(DocumentType::Article)?;
        let (patch_sender, mut patch_receiver) = channel::<FormatPatch>(4);
        document
            .sync_format(
                None,
                Some(patch_sender),
                Some(DecodeOptions {
                    format: Some(Format::Markdown),
                    ..Default::default()
                }),
                Some(EncodeOptions {
                    format: Some(Format::Markdown),
                    ..Default::default()
                }),
            )
            .await?;

        // First patch should be a reset with empty content
        assert_eq!(
            patch_receiver.recv().await.unwrap(),
            FormatPatch {
                version: 1,
                ops: vec![FormatOperation::reset("")]
            }
        );

        // Test inserting content
        document
            .update_sender
            .send(art([p([t("Hello world")])]))
            .await?;
        assert_eq!(
            patch_receiver.recv().await.unwrap(),
            FormatPatch {
                version: 2,
                ops: vec![FormatOperation::insert(0, "Hello world")]
            }
        );

        // Test deleting content
        document
            .update_sender
            .send(art([p([t("Hello ld")])]))
            .await?;
        assert_eq!(
            patch_receiver.recv().await.unwrap(),
            FormatPatch {
                version: 3,
                ops: vec![FormatOperation::delete(6, 9)]
            }
        );

        // Test replacing content
        document
            .update_sender
            .send(art([p([t("Hello friend")])]))
            .await?;
        assert_eq!(
            patch_receiver.recv().await.unwrap(),
            FormatPatch {
                version: 4,
                ops: vec![FormatOperation::replace(6, 7, "frien")]
            }
        );

        Ok(())
    }
}
