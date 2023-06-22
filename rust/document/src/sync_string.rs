use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::{Duration},
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
/// Uses the the data model as a CodeMirror change (see https://codemirror.net/examples/change/)
/// which allows a `StringChange` to be serialized to/from a browser based code editor.
#[derive(Debug, Default, Serialize, Deserialize)]
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

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct StringPatch {
    version: u32,

    ops: Vec<StringOp>,
}

impl Document {
    /// Synchronize the document with a string buffer
    ///
    /// This function spawns a task to synchronize a document with an internal
    /// string buffer. It accepts a
    #[tracing::instrument(skip(self))]
    pub async fn sync_string(
        &self,
        patch_receiver: Option<Receiver<StringPatch>>,
        patch_sender: Sender<StringPatch>,
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

        // Send initial patch of initial content
        let init = StringPatch {
            version: 0,
            ops: vec![StringOp {
                from: 0,
                insert: Some(content),
                ..Default::default()
            }],
        };
        if let Err(error) = patch_sender.send(init).await {
            tracing::error!("While sending string change: {error}");
        }

        // Start task to receive any incoming changes
        if let Some(mut patch_receiver) = patch_receiver {
            let buffer_clone = buffer.clone();
            let version_clone = version.clone();
            let update_sender = self.update_sender.clone();
            tokio::spawn(async move {
                while let Some(patch) = patch_receiver.recv().await {
                    tracing::trace!("Received patch");

                    // If the patch is not for the current version then send an
                    // initialization patch
                    if patch.version != version_clone.load(Ordering::SeqCst) {
                        continue;
                    }

                    let buffer = buffer_clone.lock().await;

                    //change.apply(&mut buffer);

                    if let Ok(node) = codecs::from_str(&buffer, decode_options.clone()).await {
                        if let Err(error) = update_sender.send(node).await {
                            tracing::error!("While sending node update: {error}");
                        }
                    }
                }
            });
        }

        // Start task to send outgoing changes
        let mut node_receiver = self.watch_receiver.clone();
        tokio::spawn(async move {
            while node_receiver.changed().await.is_ok() {
                tracing::trace!("Root node changed, updating buffer");

                let node = node_receiver.borrow_and_update().clone();

                // Encode to string
                let content = match codecs::to_string(&node, encode_options.clone()).await {
                    Ok(content) => content,
                    Err(error) => {
                        tracing::error!("While encoding node to buffer: {error}");
                        continue;
                    }
                };

                // Calculate a diff to the string
                let mut buffer = buffer.lock().await;
                let old = buffer.clone();
                let diff = TextDiffConfig::default()
                    .algorithm(Algorithm::Patience)
                    .timeout(Duration::from_secs(5))
                    .diff_chars(&old, &content);

                // Update buffer
                *buffer = content.clone();
                drop(buffer);

                // Increment version (getting previous version for patch)
                let version = version.fetch_add(1, Ordering::SeqCst);

                // Send diff as a patch
                let mut ops = Vec::new();
                let mut from = 0usize;
                for op in diff.ops() {
                    match op.tag() {
                        DiffTag::Insert => {
                            let insert = Some(content[op.new_range()].to_string());
                            ops.push(StringOp {
                                from,
                                insert,
                                ..Default::default()
                            })
                        }
                        DiffTag::Delete => {
                            let to = Some(from + op.old_range().len());
                            ops.push(StringOp {
                                from,
                                to,
                                ..Default::default()
                            })
                        }
                        DiffTag::Replace => {
                            let to = Some(from + op.old_range().len());
                            let insert = Some(content[op.new_range()].to_string());
                            ops.push(StringOp { from, to, insert })
                        }
                        DiffTag::Equal => {}
                    };

                    from += op.new_range().len();
                }

                let patch = StringPatch { version, ops };
                if let Err(error) = patch_sender.send(patch).await {
                    tracing::error!("While sending string change: {error}");
                }
            }
        });

        Ok(())
    }
}
