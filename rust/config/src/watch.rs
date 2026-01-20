use std::{
    path::Path,
    time::{Duration, Instant},
};

use eyre::Result;
use notify::{Event, RecursiveMode, Watcher, event::EventKind};
use tokio::sync::mpsc;

use crate::{CONFIG_FILENAME, CONFIG_LOCAL_FILENAME, Config, find_config_file, singleton};

/// Watch for config file changes and receive updates through a channel
///
/// Watches the nearest `stencila.toml` and `stencila.local.toml` files
/// for changes and returns updated configuration when either changes.
///
/// # Arguments
/// * `base_path` - The base path to search for config files from
///
/// # Returns
/// A receiver that will receive config updates when the files change.
/// Returns `Ok(None)` if no config file is found to watch.
/// The watcher stops when the receiver is dropped.
pub async fn watch(base_path: &Path) -> Result<Option<mpsc::Receiver<Result<Config>>>> {
    // Find the config file
    let config_path = match find_config_file(base_path, CONFIG_FILENAME) {
        Some(path) => path,
        None => return Ok(None), // No config file to watch
    };

    let watched_dir = match config_path.parent() {
        Some(parent) => parent.to_path_buf(),
        None => return Ok(None),
    };

    let (sender, receiver) = mpsc::channel(100);
    let base_path_owned = base_path.to_path_buf();

    // Spawn a background task to watch the files
    tokio::task::spawn(async move {
        let (file_sender, mut file_receiver) = mpsc::channel::<()>(100);

        let mut watcher = match notify::recommended_watcher(
            move |res: std::result::Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    // Check if event affects our config files
                    let affects_config = event.paths.iter().any(|path| {
                        path.file_name()
                            .and_then(|n| n.to_str())
                            .map(|n| n == CONFIG_FILENAME || n == CONFIG_LOCAL_FILENAME)
                            .unwrap_or(false)
                    });

                    if affects_config
                        && matches!(
                            event.kind,
                            EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
                        )
                    {
                        let _ = file_sender.blocking_send(());
                    }
                }
            },
        ) {
            Ok(w) => w,
            Err(error) => {
                let _ = sender
                    .send(Err(eyre::eyre!("Failed to create watcher: {error}")))
                    .await;
                return;
            }
        };

        if let Err(error) = watcher.watch(&watched_dir, RecursiveMode::NonRecursive) {
            let _ = sender
                .send(Err(eyre::eyre!("Failed to watch config: {error}")))
                .await;
            return;
        }

        // Debounce events - accumulate changes during debounce window
        const DEBOUNCE_MS: u64 = 500;
        let mut pending_change = false;
        let mut debounce_deadline: Option<Instant> = None;

        loop {
            let timeout = debounce_deadline
                .map(|d| d.saturating_duration_since(Instant::now()))
                .unwrap_or(Duration::from_secs(3600));

            tokio::select! {
                Some(()) = file_receiver.recv() => {
                    pending_change = true;
                    debounce_deadline = Some(Instant::now() + Duration::from_millis(DEBOUNCE_MS));
                }
                _ = tokio::time::sleep(timeout), if debounce_deadline.is_some() => {
                    if pending_change {
                        match singleton::load_and_validate(&base_path_owned) {
                            Ok(cfg) => {
                                if sender.send(Ok(cfg)).await.is_err() {
                                    break; // Receiver dropped
                                }
                            }
                            Err(e) => {
                                let _ = sender.send(Err(e)).await;
                            }
                        }
                        pending_change = false;
                    }
                    debounce_deadline = None;
                }
                else => break,
            }
        }
    });

    Ok(Some(receiver))
}
