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

/// An operation on a string
///
/// Uses the same data model as a CodeMirror change (see https://codemirror.net/examples/change/)
/// which allows a `StringChange` to be serialized to/from a browser based code editor.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(crate = "common::serde")]
pub struct StringOp {
    /// The position in the string from which the change applied
    from: usize,

    /// The position in the string to which the change applied
    ///
    /// May be omitted for additions.
    to: Option<usize>,

    /// The string to insert between `from` and `to`.
    ///
    /// For additions and replacements; may be omitted for deletions.
    insert: Option<String>,
}

impl StringOp {
    /// Create an insert operation
    fn insert<S>(from: usize, value: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            from,
            insert: Some(value.into()),
            ..Default::default()
        }
    }

    /// Create a delete operation
    fn delete(from: usize, to: usize) -> Self {
        Self {
            from,
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
            from,
            to: Some(to),
            insert: Some(value.into()),
            ..Default::default()
        }
    }

    /// Create a reset operation
    fn reset<S>(value: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            from: 0,
            to: Some(0),
            insert: Some(value.into()),
            ..Default::default()
        }
    }
}

/// A patch on a string
///
/// A `StringPatch` is a collection of [`StringOp`]s with a version which is
/// used to ensure that the operations are applied to the correct version
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(crate = "common::serde")]
pub struct StringPatch {
    /// The version of the patch
    version: u32,

    /// The operations in the patch
    ops: Vec<StringOp>,
}

impl Document {
    /// Synchronize the document with a string buffer
    ///
    /// This function spawns a task to synchronize a document's root node
    /// with an in-memory string buffer.
    #[tracing::instrument(skip(self))]
    pub async fn sync_string(
        &self,
        patch_receiver: Option<Receiver<StringPatch>>,
        patch_sender: Option<Sender<StringPatch>>,
        decode_options: Option<DecodeOptions>,
        encode_options: Option<EncodeOptions>,
    ) -> Result<()> {
        tracing::trace!("Syncing string");

        // Create initial encoding of the root node
        let node = self.load().await?;
        let content = codecs::to_string(&node, encode_options.clone()).await?;

        // Create the buffer and initialize the version
        let buffer = Arc::new(Mutex::new(content.clone()));
        let version = Arc::new(AtomicU32::new(0));

        // Start task to receive incoming `StringPatch`s from the client, apply them
        // to the buffer, and update the document's root node
        if let Some(mut patch_receiver) = patch_receiver {
            let buffer_clone = buffer.clone();
            let version_clone = version.clone();
            let patch_sender_clone = patch_sender.clone();
            let update_sender = self.update_sender.clone();
            tokio::spawn(async move {
                while let Some(patch) = patch_receiver.recv().await {
                    tracing::trace!("Received patch");

                    let mut buffer = buffer_clone.lock().await;

                    // If the patch is not for the current version then send a reset patch
                    // (if there is a patch sender) and ignore the patch
                    let current_version = version_clone.load(Ordering::SeqCst);
                    if patch.version != current_version {
                        if let Some(patch_sender) = &patch_sender_clone {
                            let reset = StringPatch {
                                version: current_version,
                                ops: vec![StringOp::reset(&*buffer)],
                            };
                            if let Err(error) = patch_sender.send(reset).await {
                                tracing::error!("While sending reset string patch: {error}");
                            }
                        }
                        continue;
                    }

                    // Apply the patch to the buffer
                    for op in patch.ops {
                        match op {
                            // Insert
                            StringOp {
                                from,
                                to: None,
                                insert: Some(insert),
                            } => buffer.insert_str(from, &insert),

                            // Delete
                            StringOp {
                                from,
                                to: Some(to),
                                insert: None,
                            } => buffer.replace_range(from..to, ""),

                            // Replace
                            StringOp {
                                from,
                                to: Some(to),
                                insert: Some(insert),
                            } => buffer.replace_range(from..to, &insert),

                            // No op, ignore
                            StringOp {
                                to: None,
                                insert: None,
                                ..
                            } => {}
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
                let init = StringPatch {
                    version: version.load(Ordering::SeqCst),
                    ops: vec![StringOp::reset(content)],
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
                                ops.push(StringOp::insert(from, &new[op.new_range()]))
                            }
                            DiffTag::Delete => {
                                ops.push(StringOp::delete(from, from + op.old_range().len()))
                            }
                            DiffTag::Replace => ops.push(StringOp::replace(
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
                    let patch = StringPatch { version, ops };
                    if let Err(error) = patch_sender.send(patch).await {
                        tracing::error!("While sending string patch: {error}");
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
    use common_dev::pretty_assertions::assert_eq;
    use format::Format;
    use schema::shortcuts::{art, p, t};

    use crate::DocumentType;

    use super::*;

    /// Test receiving patches from a client
    #[tokio::test]
    async fn receive_patches() -> Result<()> {
        // Create a document and start syncing with Markdown buffer
        let document = Document::new(DocumentType::Article)?;
        let (patch_sender, patch_receiver) = channel::<StringPatch>(1);
        document
            .sync_string(
                Some(patch_receiver),
                None,
                Some(DecodeOptions {
                    format: Some(Format::Markdown),
                    ..Default::default()
                }),
                Some(EncodeOptions {
                    format: Some(Format::Markdown),
                    strip_props: vec!["id".to_string()],
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
                        strip_props: vec!["id".to_string()],
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
            .send(StringPatch {
                version: 0,
                ops: vec![StringOp::insert(0, "Hello world")],
            })
            .await?;
        watch.changed().await.ok();
        assert_eq!(md().await?, "Hello world");

        // Test delete operation
        patch_sender
            .send(StringPatch {
                version: 1,
                ops: vec![StringOp::delete(6, 9)],
            })
            .await?;
        watch.changed().await.ok();
        assert_eq!(md().await?, "Hello ld");

        // Test replace operation
        patch_sender
            .send(StringPatch {
                version: 2,
                ops: vec![StringOp::replace(6, 7, "frien")],
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
        let (patch_sender, mut patch_receiver) = channel::<StringPatch>(4);
        document
            .sync_string(
                None,
                Some(patch_sender),
                Some(DecodeOptions {
                    format: Some(Format::Markdown),
                    ..Default::default()
                }),
                Some(EncodeOptions {
                    format: Some(Format::Markdown),
                    strip_props: vec!["id".to_string()],
                    ..Default::default()
                }),
            )
            .await?;

        // First patch should be a reset with empty content
        assert_eq!(
            patch_receiver.recv().await.unwrap(),
            StringPatch {
                version: 0,
                ops: vec![StringOp::reset("")]
            }
        );

        // Test inserting content
        document
            .update_sender
            .send(art([p([t("Hello world")])]))
            .await?;
        assert_eq!(
            patch_receiver.recv().await.unwrap(),
            StringPatch {
                version: 1,
                ops: vec![StringOp::insert(0, "Hello world")]
            }
        );

        // Test deleting content
        document
            .update_sender
            .send(art([p([t("Hello ld")])]))
            .await?;
        assert_eq!(
            patch_receiver.recv().await.unwrap(),
            StringPatch {
                version: 2,
                ops: vec![StringOp::delete(6, 9)]
            }
        );

        // Test replacing content
        document
            .update_sender
            .send(art([p([t("Hello friend")])]))
            .await?;
        assert_eq!(
            patch_receiver.recv().await.unwrap(),
            StringPatch {
                version: 3,
                ops: vec![StringOp::replace(6, 7, "frien")]
            }
        );

        Ok(())
    }
}
