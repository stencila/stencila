use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use eyre::Result;
use notify::{Event, RecursiveMode, Watcher, event::EventKind};
use tokio::sync::mpsc;

/// Event emitted when site files change
#[derive(Debug, Clone)]
pub struct SiteChangeEvent {
    /// Paths that changed
    pub paths: Vec<PathBuf>,
}

/// Watch site root directory for file changes
///
/// Watches the specified directory recursively for file changes and returns
/// events through a channel. Changes are debounced to avoid excessive events
/// from rapid saves.
///
/// # Arguments
/// * `site_root` - The root directory of the site to watch
/// * `exclude_dir` - Optional directory to exclude from watching (e.g., output directory)
///
/// # Returns
/// A receiver that will receive `SiteChangeEvent` when files change.
/// The watcher stops when the receiver is dropped.
pub async fn watch(
    site_root: &Path,
    exclude_dir: Option<&Path>,
) -> Result<mpsc::Receiver<SiteChangeEvent>> {
    let (sender, receiver) = mpsc::channel(100);
    let site_root = site_root.to_path_buf();
    let exclude_dir = exclude_dir.map(|p| p.to_path_buf());

    tokio::task::spawn(async move {
        let (file_sender, mut file_receiver) = mpsc::channel::<Vec<PathBuf>>(100);
        let exclude_dir_clone = exclude_dir.clone();

        let mut watcher = match notify::recommended_watcher(
            move |res: std::result::Result<Event, notify::Error>| {
                if let Ok(event) = res
                    && matches!(
                        event.kind,
                        EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_)
                    )
                {
                    // Filter out hidden files/directories and excluded paths
                    let paths: Vec<PathBuf> = event
                        .paths
                        .into_iter()
                        .filter(|path| {
                            // Skip hidden files/directories
                            let is_hidden = path.components().any(|c| {
                                c.as_os_str()
                                    .to_str()
                                    .map(|s| s.starts_with('.'))
                                    .unwrap_or(false)
                            });
                            if is_hidden {
                                return false;
                            }

                            // Skip excluded directory
                            if let Some(ref exclude) = exclude_dir_clone
                                && path.starts_with(exclude)
                            {
                                return false;
                            }

                            true
                        })
                        .collect();

                    if !paths.is_empty() {
                        let _ = file_sender.blocking_send(paths);
                    }
                }
            },
        ) {
            Ok(w) => w,
            Err(error) => {
                tracing::error!("Failed to create site watcher: {error}");
                return;
            }
        };

        if let Err(error) = watcher.watch(&site_root, RecursiveMode::Recursive) {
            tracing::error!("Failed to watch site root: {error}");
            return;
        }

        // Debounce events - accumulate paths during debounce window
        const DEBOUNCE_MS: u64 = 500;
        let mut pending_paths: Vec<PathBuf> = Vec::new();
        let mut debounce_deadline: Option<Instant> = None;

        loop {
            let timeout = debounce_deadline
                .map(|d| d.saturating_duration_since(Instant::now()))
                .unwrap_or(Duration::from_secs(3600));

            tokio::select! {
                Some(paths) = file_receiver.recv() => {
                    // Deduplicate paths (file systems often emit multiple events per save)
                    for path in paths {
                        if !pending_paths.contains(&path) {
                            pending_paths.push(path);
                        }
                    }
                    debounce_deadline = Some(Instant::now() + Duration::from_millis(DEBOUNCE_MS));
                }
                _ = tokio::time::sleep(timeout), if debounce_deadline.is_some() => {
                    if !pending_paths.is_empty() {
                        let paths = std::mem::take(&mut pending_paths);
                        if sender.send(SiteChangeEvent { paths }).await.is_err() {
                            break;
                        }
                    }
                    debounce_deadline = None;
                }
                else => break,
            }
        }
    });

    Ok(receiver)
}
