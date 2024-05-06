use std::{
    path::Path,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use notify::{EventKind, RecursiveMode, Watcher};

use codecs::{DecodeOptions, EncodeOptions};
use common::{
    eyre::Result,
    tokio::{self, time},
    tracing,
};
use schema::Node;

use crate::{Document, SyncDirection};

impl Document {
    /// Synchronize the document with a file (e.g. an `Article` root node)
    /// or directory (a `Directory` root node)
    ///
    /// This function spawns a task to synchronize the document with the file system:
    /// changes in the content of the path result in an update being sent to
    /// the document, and changes to the document result in an update to the
    /// file (depending on the `direction` argument).
    #[tracing::instrument(skip(self))]
    pub async fn sync_file(
        &self,
        path: &Path,
        direction: SyncDirection,
        decode_options: Option<DecodeOptions>,
        encode_options: Option<EncodeOptions>,
    ) -> Result<()> {
        tracing::trace!("Syncing file");

        // Before starting watches import and export as necessary.
        let node = match direction {
            SyncDirection::In => {
                let node = codecs::from_path(path, decode_options.clone()).await?;
                *self.root.write().await = node.clone();
                node
            }
            SyncDirection::Out => {
                let node = self.root.read().await;
                codecs::to_path(&node, path, encode_options.clone()).await?;
                node.clone()
            }
            SyncDirection::InOut => {
                if path.exists() {
                    let node = codecs::from_path(path, decode_options.clone()).await?;
                    *self.root.write().await = node.clone();
                    node
                } else {
                    let node = self.root.read().await;
                    codecs::to_path(&node, path, encode_options.clone()).await?;
                    node.clone()
                }
            }
        };

        // Record when file last written to
        let last_write = Arc::new(AtomicU64::default());

        // Spawn a task to read the file or directory when it changes
        if matches!(direction, SyncDirection::In | SyncDirection::InOut) {
            // A channel to send file change events from the sync file watcher thread to the
            // async node updating thread
            let (sender, mut receiver) = tokio::sync::mpsc::unbounded_channel();

            // A std thread to watch for file changes
            let path_buf = path.to_path_buf();
            let last_write = last_write.clone();
            std::thread::spawn(move || {
                let (watch_sender, watch_receiver) = std::sync::mpsc::channel();
                let mut watcher = match notify::recommended_watcher(watch_sender) {
                    Ok(watcher) => watcher,
                    Err(error) => {
                        tracing::error!("While instantiating watcher: {}", error);
                        return;
                    }
                };

                if let Err(error) = watcher.watch(&path_buf, RecursiveMode::Recursive) {
                    tracing::error!("While watching file `{}`: {error}", path_buf.display());
                }

                tracing::trace!("File watch thread for `{}` started", path_buf.display());

                loop {
                    let event = watch_receiver.recv();
                    match event {
                        Ok(Ok(event)) => {
                            if matches!(
                                event.kind,
                                EventKind::Create(..)
                                    | EventKind::Modify(..)
                                    | EventKind::Remove(..)
                            ) {
                                if let Err(error) = sender.send(()) {
                                    tracing::error!(
                                        "While forwarding file watching event: {}",
                                        error
                                    );
                                }
                            }
                        }
                        Ok(Err(error)) => {
                            tracing::error!("While watching file: {}", error);
                        }
                        Err(error) => {
                            tracing::error!(
                                "While receiving file watching events: {}",
                                error.to_string()
                            );
                            break;
                        }
                    }
                }

                tracing::trace!("File watch thread for `{}` stopped", path_buf.display());
            });

            // An async task to handle the file watcher events by reading the file
            let path_buf = path.to_path_buf();
            let update_sender = self.update_sender.clone();
            tokio::spawn(async move {
                const DEBOUNCE_DELAY_MILLIS: u64 = 100;
                const WRITE_IGNORE_MILLIS: u64 = 200;

                let mut event = false;
                loop {
                    match time::timeout(
                        Duration::from_millis(DEBOUNCE_DELAY_MILLIS),
                        receiver.recv(),
                    )
                    .await
                    {
                        Ok(None) => {
                            break;
                        }
                        Ok(Some(..)) => {
                            event = true;
                            continue;
                        }
                        Err(..) => {
                            if !event {
                                continue;
                            }
                            event = false;
                        }
                    }

                    // Ignore event if we wrote to the file very recently
                    if now() - last_write.load(Ordering::SeqCst) < WRITE_IGNORE_MILLIS {
                        continue;
                    }

                    tracing::trace!(
                        "File `{}` changed, importing to root node",
                        path_buf.display()
                    );

                    match codecs::from_path(&path_buf, decode_options.clone()).await {
                        Ok(node) => {
                            if let Err(error) = update_sender.send(node).await {
                                tracing::error!("While sending node update: {error}");
                            }
                        }
                        Err(error) => {
                            tracing::error!(
                                "While importing from `{}`: {error}",
                                path_buf.display()
                            );
                        }
                    }
                }
            });
        }

        // Spawn a task to write non-`Directory` nodes to the file when the node changes
        if !matches!(node, Node::Directory(..))
            && matches!(direction, SyncDirection::Out | SyncDirection::InOut)
        {
            let mut receiver = self.watch_receiver.clone();
            let path_buf = path.to_path_buf();
            tokio::spawn(async move {
                while receiver.changed().await.is_ok() {
                    tracing::trace!("Root node changed, exporting to `{}`", path_buf.display());

                    let node = receiver.borrow_and_update().clone();

                    if let Err(error) =
                        codecs::to_path(&node, &path_buf, encode_options.clone()).await
                    {
                        tracing::error!("While exporting node to `{}`: {error}", path_buf.display())
                    }

                    last_write.store(now(), Ordering::SeqCst);
                }
            });
        }

        // Get current time as milliseconds
        fn now() -> u64 {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards!")
                .as_millis() as u64
        }

        Ok(())
    }
}
