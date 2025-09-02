use std::{
    ops::{DerefMut, Range},
    sync::{
        Arc,
        atomic::{AtomicU32, Ordering},
    },
    time::Duration,
};

use eyre::Result;
use json_patch::{PatchOperation as MappingOperation, ReplaceOperation};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use similar::{Algorithm, DiffTag, TextDiffConfig};
use tokio::{
    self,
    sync::{
        Mutex,
        mpsc::{Receiver, Sender},
    },
};

use stencila_codecs::{DecodeOptions, EncodeInfo, EncodeOptions, Mapping};

use crate::{Document, Update};

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
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
#[serde(default)]
pub struct FormatPatch {
    /// The version of the patch
    version: u32,

    /// The operations in the patch
    ops: Vec<FormatOperation>,
}

/// An operation on either the content or mapping
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum FormatOperation {
    Content(ContentOperation),
    Mapping(MappingOperation),
}

impl FormatOperation {
    /// Create a content reset operation
    fn reset_content<S>(value: S) -> Self
    where
        S: Into<String>,
    {
        Self::Content(ContentOperation {
            r#type: ContentOperationType::Reset,
            insert: Some(value.into()),
            ..Default::default()
        })
    }

    /// Create a content insert operation
    fn insert_content<S>(from: usize, value: S) -> Self
    where
        S: Into<String>,
    {
        Self::Content(ContentOperation {
            r#type: ContentOperationType::Insert,
            from: Some(from),
            insert: Some(value.into()),
            ..Default::default()
        })
    }

    /// Create a content delete operation
    fn delete_content(from: usize, to: usize) -> Self {
        Self::Content(ContentOperation {
            r#type: ContentOperationType::Delete,
            from: Some(from),
            to: Some(to),
            ..Default::default()
        })
    }

    /// Create a content replace operation
    fn replace_content<S>(from: usize, to: usize, value: S) -> Self
    where
        S: Into<String>,
    {
        Self::Content(ContentOperation {
            r#type: ContentOperationType::Replace,
            from: Some(from),
            to: Some(to),
            insert: Some(value.into()),
        })
    }

    /// Create a mapping reset operation
    fn reset_mapping(mapping: &Mapping) -> Self {
        let value = match serde_json::to_value(mapping) {
            Ok(value) => value,
            Err(error) => {
                tracing::error!("While serializing format mapping: {error}");
                serde_json::Value::Array(vec![])
            }
        };

        Self::Mapping(MappingOperation::Replace(ReplaceOperation {
            path: String::new(),
            value,
        }))
    }

    /// Difference two mappings to create a set of mapping operations
    fn diff_mappings(old: &Mapping, new: &Mapping) -> Vec<Self> {
        json_patch::diff(
            &serde_json::to_value(old).unwrap_or_default(),
            &serde_json::to_value(new).unwrap_or_default(),
        )
        .0
        .into_iter()
        .map(Self::Mapping)
        .collect()
    }
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
pub struct ContentOperation {
    /// The type of operation
    r#type: ContentOperationType,

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

/// The type of an operation
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
enum ContentOperationType {
    /// Reset content
    #[default]
    Reset,

    /// Insert characters (sent by clients and server)
    Insert,

    /// Delete characters (sent by clients and server)
    Delete,

    /// Replace characters (sent by clients and server)
    Replace,

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
        let node = self.root.read().await;
        let (
            initial_content,
            EncodeInfo {
                mapping: initial_mapping,
                ..
            },
        ) = stencila_codecs::to_string_with_info(&node, encode_options.clone()).await?;

        // Create the mutex for the current content and mapping and initialize the version
        let current = Arc::new(Mutex::new((
            initial_content.clone(),
            initial_mapping.clone(),
        )));
        let version = Arc::new(AtomicU32::new(1));

        // Start task to receive incoming patches from the client, apply them
        // to the buffer, and update the document's root node
        if let Some(mut patch_receiver) = patch_receiver {
            let current_clone = current.clone();
            let version_clone = version.clone();
            let patch_sender_clone = patch_sender.clone();
            let update_sender = self.update_sender.clone();
            tokio::spawn(async move {
                while let Some(patch) = patch_receiver.recv().await {
                    tracing::trace!("Received string patch");

                    let mut current = current_clone.lock().await;
                    let (current_content, current_mapping) = current.deref_mut();

                    // If the patch is not for the current version then send a reset patch
                    // (if there is a patch sender) and ignore the patch
                    let current_version = version_clone.load(Ordering::SeqCst);
                    if patch.version != current_version {
                        if let Some(patch_sender) = &patch_sender_clone {
                            let reset = FormatPatch {
                                version: current_version,
                                ops: vec![
                                    FormatOperation::reset_content(&*current_content),
                                    FormatOperation::reset_mapping(current_mapping),
                                ],
                            };
                            if let Err(error) = patch_sender.send(reset).await {
                                tracing::error!("While sending content reset patch: {error}");
                            }
                        }
                        continue;
                    }

                    // Apply the patch to the current content
                    let mut updated = false;
                    for op in patch.ops {
                        match op {
                            FormatOperation::Content(ContentOperation {
                                r#type: ContentOperationType::Reset,
                                ..
                            }) => {
                                tracing::warn!("Client attempted to reset string")
                            }

                            FormatOperation::Content(ContentOperation {
                                r#type: ContentOperationType::Insert,
                                from: Some(from),
                                to: None,
                                insert: Some(insert),
                            }) => {
                                if current_content.is_empty() {
                                    current_content.push_str(&insert);
                                    updated = true;
                                } else if let Some((from, ..)) =
                                    current_content.char_indices().nth(from)
                                {
                                    current_content.insert_str(from, &insert);
                                    updated = true;
                                }
                            }

                            FormatOperation::Content(ContentOperation {
                                r#type: ContentOperationType::Delete,
                                from: Some(from),
                                to: Some(to),
                                insert: None,
                            }) => {
                                if let (Some((from, ..)), Some((to, ..))) = (
                                    current_content.char_indices().nth(from),
                                    current_content.char_indices().nth(to),
                                ) {
                                    current_content.replace_range(from..to, "");
                                    updated = true;
                                }
                            }

                            FormatOperation::Content(ContentOperation {
                                r#type: ContentOperationType::Replace,
                                from: Some(from),
                                to: Some(to),
                                insert: Some(insert),
                            }) => {
                                if let (Some((from, ..)), Some((to, ..))) = (
                                    current_content.char_indices().nth(from),
                                    current_content.char_indices().nth(to),
                                ) {
                                    current_content.replace_range(from..to, &insert);
                                    updated = true;
                                }
                            }

                            FormatOperation::Content(ContentOperation {
                                r#type: ContentOperationType::Viewport,
                                from: Some(from),
                                to: Some(to),
                                insert: None,
                            }) => {
                                // TODO
                                tracing::debug!("Viewport operation {from}-{to}")
                            }

                            FormatOperation::Content(ContentOperation {
                                r#type: ContentOperationType::Selection,
                                from: Some(from),
                                to: Some(to),
                                insert: None,
                            }) => {
                                // TODO
                                tracing::debug!("Selection operation {from}-{to}")
                            }

                            _ => tracing::warn!("Client sent invalid operation"),
                        }
                    }

                    if updated {
                        // Increment the version number
                        version_clone.fetch_add(1, Ordering::SeqCst);

                        // Update the root node
                        // TODO consider debouncing this since `from_str` and the update will be relatively expensive
                        if let Ok(node) =
                            stencila_codecs::from_str(current_content, decode_options.clone()).await
                        {
                            // TODO: update `format` should be based on the `path` & `decode_options`
                            // and `authors` should use the local user
                            if let Err(error) = update_sender.send((Update::new(node), None)).await
                            {
                                tracing::error!("While sending root update: {error}");
                            }
                        }
                    }
                }
            });
        }

        // Start task to listen for changes to the document's root node,
        // convert them to a patch and send to the client
        if let Some(patch_sender) = patch_sender {
            let mut node_receiver = self.watch_receiver.clone();
            tokio::spawn(async move {
                // Send initial patch to set initial content
                let init = FormatPatch {
                    version: version.load(Ordering::SeqCst),
                    ops: vec![
                        FormatOperation::reset_content(initial_content),
                        FormatOperation::reset_mapping(&initial_mapping),
                    ],
                };
                if let Err(error) = patch_sender.send(init).await {
                    tracing::error!("While sending initial string patch: {error}");
                }

                // TODO: consider debouncing this
                while node_receiver.changed().await.is_ok() {
                    tracing::trace!("Root node changed, updating string buffer");

                    let node = node_receiver.borrow_and_update().clone();

                    // Encode the node to a string in the format
                    let (
                        new_content,
                        EncodeInfo {
                            mapping: new_mapping,
                            ..
                        },
                    ) = match stencila_codecs::to_string_with_info(&node, encode_options.clone())
                        .await
                    {
                        Ok(string) => string,
                        Err(error) => {
                            tracing::error!("While encoding node to string: {error}");
                            continue;
                        }
                    };

                    let mut current = current.lock().await;
                    let (current_content, current_mapping) = current.deref_mut();

                    let mut ops = Vec::new();

                    if new_content != *current_content {
                        // Calculate a diff between old and new string contents
                        // Use Myers because it is faster than Patience.Limit to 1 second.
                        let diff = TextDiffConfig::default()
                            .algorithm(Algorithm::Myers)
                            .timeout(Duration::from_secs(1))
                            .diff_chars(current_content.as_str(), new_content.as_str());

                        // Convert the diff to a set of operations
                        let mut from = 0usize;
                        for op in diff.ops() {
                            match op.tag() {
                                DiffTag::Insert => ops.push(FormatOperation::insert_content(
                                    from,
                                    extract_chars(&new_content, op.new_range()),
                                )),
                                DiffTag::Delete => ops.push(FormatOperation::delete_content(
                                    from,
                                    from + op.old_range().len(),
                                )),
                                DiffTag::Replace => ops.push(FormatOperation::replace_content(
                                    from,
                                    from + op.old_range().len(),
                                    extract_chars(&new_content, op.new_range()),
                                )),
                                DiffTag::Equal => {}
                            };

                            from += op.new_range().len();
                        }

                        // Increment version
                        version.fetch_add(1, Ordering::SeqCst);

                        // Update current content
                        *current_content = new_content;
                    }

                    if new_mapping != *current_mapping {
                        // Calculate patch operations for the mapping
                        ops.append(&mut FormatOperation::diff_mappings(
                            current_mapping,
                            &new_mapping,
                        ));

                        // Update current mapping
                        *current_mapping = new_mapping;
                    }

                    if !ops.is_empty() {
                        // Create and send a patch for the content
                        let version = version.load(Ordering::SeqCst);
                        let patch = FormatPatch { version, ops };
                        if patch_sender.send(patch).await.is_err() {
                            // Most likely receiver has dropped so just finish this task
                            break;
                        }
                    }
                }
            });
        }

        Ok(())
    }
}

/// Extract a character range from a string
fn extract_chars(content: &str, range: Range<usize>) -> &str {
    let start_byte = content
        .char_indices()
        .nth(range.start)
        .map_or_else(|| 0, |(start, ..)| start);
    let end_byte = content
        .char_indices()
        .nth(range.end)
        .map_or_else(|| content.len(), |(end, ..)| end);

    &content[start_byte..end_byte]
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use eyre::Report;
    use ntest::timeout;
    use pretty_assertions::assert_eq;
    use stencila_format::Format;
    use stencila_node_strip::StripScope;
    use stencila_schema::{
        NodeType,
        shortcuts::{art, p, t},
    };
    use tokio::sync::mpsc::channel;

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
        let document = Document::new(NodeType::Article).await?;

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
                .dump(
                    Format::Markdown,
                    Some(EncodeOptions {
                        strip_scopes: vec![StripScope::Authors, StripScope::Provenance],
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
                ops: vec![FormatOperation::insert_content(0, "Hello world")],
            })
            .await?;
        watch.changed().await.ok();
        assert_eq!(md().await?, "Hello world\n");

        // Test delete operation
        patch_sender
            .send(FormatPatch {
                version: 2,
                ops: vec![FormatOperation::delete_content(6, 9)],
            })
            .await?;
        watch.changed().await.ok();
        assert_eq!(md().await?, "Hello ld\n");

        // Test replace operation
        patch_sender
            .send(FormatPatch {
                version: 3,
                ops: vec![FormatOperation::replace_content(6, 7, "frien")],
            })
            .await?;
        watch.changed().await.ok();
        assert_eq!(md().await?, "Hello friend\n");

        Ok(())
    }

    /// Test sending patches to the client
    #[tokio::test]
    async fn send_patches() -> Result<()> {
        // Create a document and start syncing with Markdown buffer
        let document = Document::new(NodeType::Article).await?;

        let (patch_sender, mut patch_receiver) = channel::<FormatPatch>(4);
        document
            .sync_format(
                None,
                Some(patch_sender),
                Some(DecodeOptions {
                    format: Some(Format::Markdown),
                    strip_scopes: vec![StripScope::Authors, StripScope::Provenance],
                    ..Default::default()
                }),
                Some(EncodeOptions {
                    format: Some(Format::Markdown),
                    strip_scopes: vec![StripScope::Authors, StripScope::Provenance],
                    ..Default::default()
                }),
            )
            .await?;

        // First patch should be a reset with empty content
        let patch = patch_receiver.recv().await.unwrap();
        assert_eq!(patch.version, 1);
        assert_eq!(patch.ops[0], FormatOperation::reset_content(""));

        // Test inserting content
        document
            .update(Update::new(art([p([t("Hello world")])])))
            .await?;
        let patch = patch_receiver.recv().await.unwrap();
        assert_eq!(patch.version, 2);
        assert_eq!(
            patch.ops[0],
            FormatOperation::insert_content(0, "Hello world\n")
        );

        // Test deleting content
        document
            .update(Update::new(art([p([t("Hello ld")])])))
            .await?;
        let patch = patch_receiver.recv().await.unwrap();
        assert_eq!(patch.version, 3);
        assert_eq!(patch.ops[0], FormatOperation::delete_content(6, 9));

        // Test replacing content
        document
            .update(Update::new(art([p([t("Hello friend")])])))
            .await?;
        let patch = patch_receiver.recv().await.unwrap();
        assert_eq!(patch.version, 4);
        assert_eq!(
            patch.ops[0],
            FormatOperation::replace_content(6, 7, "frien")
        );

        Ok(())
    }
}
