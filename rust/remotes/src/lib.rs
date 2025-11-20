//! # Remote Tracking Module
//!
//! This module manages remote synchronization for Stencila documents.
//!
//! ## Core Types
//!
//! - [`RemoteInfo`]: Unified type containing all remote data (config, timestamps, watch info)
//! - [`RemoteEntries`]: Maps file paths → remote URLs → RemoteInfo
//!
//! ## Data Sources
//!
//! Remote information is combined from two sources:
//!
//! 1. **`stencila.toml`**: Configuration (paths, URLs, watch IDs)
//! 2. **`.stencila/remotes.json`**: Tracking timestamps (pulled_at, pushed_at)
//!
//! ```text
//! stencila.toml          .stencila/remotes.json
//! (config + watch)       (timestamps)
//!       \                      /
//!        \                    /
//!         \                  /
//!          v                v
//!          RemoteInfo
//!     (unified type)
//! ```
//!
//! ## Key Functions
//!
//! - [`get_remotes_for_path()`]: Get all remotes configured for a specific file
//! - [`get_all_remote_entries()`]: Get all files with remotes
//! - [`update_remote_timestamp()`]: Update pull/push timestamps
//! - [`calculate_remote_statuses()`]: Calculate sync status for remotes

use std::{
    collections::{BTreeMap, HashSet},
    path::{Path, PathBuf},
};

use clap::ValueEnum;
use eyre::Result;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum::{Display, EnumString};
use tokio::fs::{read_to_string, write};
use url::Url;

use stencila_config::{Config, ConfigRelativePath, RemoteConfig};
use stencila_dirs::{closest_stencila_dir, closest_workspace_dir};

mod service;
pub use service::RemoteService;

/// Type alias for remote tracking entries in .stencila/remotes.json
///
/// Maps file paths to a map of remote URLs to tracking data.
/// URLs are serialized as strings in JSON but use Url type in Rust.
pub type RemoteEntries = BTreeMap<PathBuf, BTreeMap<Url, RemoteInfo>>;

/// Remote information combining config and tracking data
///
/// This unified type contains all information about a remote:
/// - Configuration from stencila.toml (url, path, etc.)
/// - Tracking timestamps from .stencila/remotes.json
/// - Watch configuration from stencila.toml
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteInfo {
    /// Remote URL from config
    #[serde(skip, default = "RemoteInfo::default_url")]
    pub url: Url,

    /// Remote configuration from stencila.toml
    #[serde(skip, default = "RemoteInfo::default_config")]
    pub config: RemoteConfig,

    /// Last time pulled from this remote (Unix timestamp in seconds)
    pub pulled_at: Option<u64>,

    /// Last time pushed to this remote (Unix timestamp in seconds)
    pub pushed_at: Option<u64>,

    /// The watch ID from Stencila Cloud if watching is enabled
    pub watch_id: Option<String>,

    /// The watch direction (from stencila.toml or Cloud API)
    pub watch_direction: Option<WatchDirection>,
}

impl RemoteInfo {
    /// Default URL for deserialization (will be replaced from map key)
    fn default_url() -> Url {
        Url::parse("http://placeholder").expect("valid placeholder URL")
    }

    /// Default RemoteConfig for deserialization (will be replaced from stencila.toml)
    fn default_config() -> RemoteConfig {
        RemoteConfig {
            path: ConfigRelativePath(".".to_string()),
            url: "http://placeholder".to_string(),
            watch: None,
        }
    }

    /// Check if this remote is being watched
    pub fn is_watched(&self) -> bool {
        self.watch_id.is_some()
    }
}

/// The direction of synchronization for a watched remote
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    ValueEnum,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum WatchDirection {
    /// Bi-directional sync: changes from remote create PRs, changes to repo push to remote
    #[default]
    Bi,

    /// One-way sync from remote: only remote changes create PRs
    FromRemote,

    /// One-way sync to remote: only repo changes push to remote
    ToRemote,
}

/// The pull request mode for a watched remote
#[derive(
    Debug,
    Default,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    ValueEnum,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum WatchPrMode {
    /// Create PRs as drafts (default)
    #[default]
    Draft,

    /// Create PRs ready for review
    Ready,
}

/// Remote tracking status indicating synchronization state
///
/// This enum represents the synchronization status between a local file
/// and its remote counterpart (e.g., Google Docs, Microsoft 365, etc.)
#[derive(Debug, Default, Display, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RemoteStatus {
    /// Status is unknown (no modification times available)
    #[default]
    Unknown,

    /// There is an entry for the file but that path no longer exists in the workspace directory
    Deleted,

    /// Remote has changes that need to be pulled
    Ahead,

    /// Local file has changes that need to be pushed
    Behind,

    /// Both local and remote have changes since last sync
    Diverged,

    /// Local and remote are in sync
    Synced,
}

/// Tolerance in seconds for local file modification time comparisons
pub const LOCAL_TOLERANCE_SECS: u64 = 5;

/// Tolerance in seconds for remote modification time comparisons
/// (accounts for cloud service processing delays)
pub const REMOTE_TOLERANCE_SECS: u64 = 30;

/// Calculate synchronization status for all remotes
///
/// This function:
/// - Fetches remote modification timestamps from each remote service in parallel
/// - Compares remote modification times with local modification times
/// - Considers `pushed_at` and `pulled_at` timestamps to detect divergence
/// - Applies tolerance values to account for timing variances
///
/// Returns a map of URL to (remote_modified_at, status) tuples
pub async fn calculate_remote_statuses(
    remotes: &BTreeMap<Url, RemoteInfo>,
    local_status: RemoteStatus,
    local_modified_at: Option<u64>,
) -> BTreeMap<Url, (Option<u64>, RemoteStatus)> {
    use futures::future::join_all;

    if remotes.is_empty() {
        return BTreeMap::new();
    }

    // Create futures to fetch metadata for each remote
    let futures = remotes.iter().map(|(url, remote)| async move {
        // Fetch metadata from remote service
        let remote_modified_at = async {
            match RemoteService::from_url(url) {
                Some(RemoteService::GoogleDocs) => stencila_codec_gdoc::modified_at(url).await,
                Some(RemoteService::Microsoft365) => stencila_codec_m365::modified_at(url).await,
                Some(RemoteService::StencilaSites) => stencila_codec_site::modified_at(url).await,
                None => eyre::bail!("Unsupported remote service: {url}"),
            }
        }
        .await
        .ok();

        tracing::debug!("Remote {url} modified at {remote_modified_at:?}");

        // Calculate status by comparing local/remote modified times with pushed_at/pulled_at
        let status = if local_status == RemoteStatus::Deleted {
            // Only mark as Ahead if we can confirm the remote exists
            if remote_modified_at.is_some() {
                RemoteStatus::Ahead
            } else {
                // Remote metadata unavailable - could be deleted, network issue, etc.
                RemoteStatus::Unknown
            }
        } else {
            match (
                local_modified_at,
                remote_modified_at,
                remote.pushed_at,
                remote.pulled_at,
            ) {
                (Some(local), Some(remote_mod), Some(pushed), Some(pulled)) => {
                    // Use the most recent sync time (push or pull) as our reference point
                    let last_synced = pushed.max(pulled);

                    // Check if local or remote have changed since last sync
                    let local_changed = local > last_synced.saturating_add(LOCAL_TOLERANCE_SECS);
                    let remote_changed =
                        remote_mod > last_synced.saturating_add(REMOTE_TOLERANCE_SECS);

                    if local_changed && remote_changed {
                        RemoteStatus::Diverged
                    } else if local_changed {
                        RemoteStatus::Behind
                    } else if remote_changed {
                        RemoteStatus::Ahead
                    } else {
                        RemoteStatus::Synced
                    }
                }
                (Some(local), Some(remote_mod), Some(pushed), None) => {
                    // Only pushed_at exists, use it as reference
                    let local_changed = local > pushed.saturating_add(LOCAL_TOLERANCE_SECS);
                    let remote_changed = remote_mod > pushed.saturating_add(REMOTE_TOLERANCE_SECS);

                    if local_changed && remote_changed {
                        RemoteStatus::Diverged
                    } else if local_changed {
                        RemoteStatus::Behind
                    } else if remote_changed {
                        RemoteStatus::Ahead
                    } else {
                        RemoteStatus::Synced
                    }
                }
                (Some(local), Some(remote_mod), None, Some(pulled)) => {
                    // Only pulled_at exists, use it as reference
                    let local_changed = local > pulled.saturating_add(LOCAL_TOLERANCE_SECS);
                    let remote_changed = remote_mod > pulled.saturating_add(REMOTE_TOLERANCE_SECS);

                    if local_changed && remote_changed {
                        RemoteStatus::Diverged
                    } else if local_changed {
                        RemoteStatus::Behind
                    } else if remote_changed {
                        RemoteStatus::Ahead
                    } else {
                        RemoteStatus::Synced
                    }
                }
                (Some(local), Some(remote_mod), _, _) => {
                    // Fallback: if we don't have pushed_at/pulled_at, just compare modified times
                    if local > remote_mod.saturating_add(REMOTE_TOLERANCE_SECS) {
                        RemoteStatus::Behind
                    } else if remote_mod > local.saturating_add(REMOTE_TOLERANCE_SECS) {
                        RemoteStatus::Ahead
                    } else {
                        RemoteStatus::Synced
                    }
                }
                _ => RemoteStatus::Unknown,
            }
        };

        (url.clone(), (remote_modified_at, status))
    });

    // Execute all futures in parallel and collect results into a BTreeMap
    join_all(futures).await.into_iter().collect()
}

/// Read remote entries from `.stencila/remotes.json`
pub async fn read_remote_entries(stencila_dir: &Path) -> Result<RemoteEntries> {
    let remotes_file = stencila_dir.join("remotes.json");

    if !remotes_file.exists() {
        return Ok(BTreeMap::new());
    }

    let json = read_to_string(&remotes_file).await?;
    let entries = serde_json::from_str(&json)?;

    Ok(entries)
}

/// Write remote entries to `.stencila/remotes.json`
pub async fn write_remote_entries(stencila_dir: &Path, entries: &RemoteEntries) -> Result<()> {
    let remotes_file = stencila_dir.join("remotes.json");

    let json = serde_json::to_string_pretty(entries)?;
    write(&remotes_file, json).await?;

    Ok(())
}

/// Normalize a path by removing leading "./" and normalizing separators
///
/// This ensures paths like "./docs/file.md" match config entries like "docs/file.md"
fn normalize_path(path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy();

    // Strip leading "./"
    let normalized_str = if let Some(stripped) = path_str.strip_prefix("./") {
        stripped
    } else {
        path_str.as_ref()
    };

    // Convert to PathBuf to normalize separators
    PathBuf::from(normalized_str)
}

/// Find all remotes that match a given file path from config
///
/// Matches based on:
/// - Exact file match
/// - Directory match (implicitly recursive)
/// - Optional: glob pattern match (future enhancement)
fn find_remotes_for_path(
    file_path: &Path,
    config: &Config,
    workspace_dir: &Path,
) -> Result<Vec<RemoteConfig>> {
    let Some(remotes_config) = &config.remotes else {
        return Ok(Vec::new());
    };

    // Get relative path from workspace and normalize it
    let relative_path = if file_path.is_absolute() {
        file_path.strip_prefix(workspace_dir).unwrap_or(file_path)
    } else {
        file_path
    };
    let relative_path = normalize_path(relative_path);

    let mut matches = Vec::new();

    for remote_config in remotes_config {
        let config_path = remote_config.path.resolve(workspace_dir);
        let config_relative = config_path
            .strip_prefix(workspace_dir)
            .unwrap_or(&config_path);
        let config_relative = normalize_path(config_relative);

        // Check if file matches this remote's path
        if path_matches(&config_relative, &relative_path, workspace_dir)? {
            matches.push(remote_config.clone());
        }
    }

    Ok(matches)
}

/// Check if a file path matches a config path
///
/// Supports:
/// - Exact file match
/// - Directory match (implicitly recursive)
fn path_matches(config_path: &Path, file_path: &Path, workspace_dir: &Path) -> Result<bool> {
    // Exact file match
    if config_path == file_path {
        return Ok(true);
    }

    // Check if config_path is a directory and file_path starts with it
    let full_config_path = workspace_dir.join(config_path);
    if full_config_path.is_dir() && file_path.starts_with(config_path) {
        return Ok(true);
    }

    // Future: Add glob pattern matching here
    // For now, we'll also check if config_path without checking is_dir matches
    // in case the directory doesn't exist yet
    if file_path.starts_with(config_path) {
        return Ok(true);
    }

    Ok(false)
}

/// Get all remotes for a file path, combining config and tracking data
///
/// This is the main function to use when resolving remotes for push/pull operations.
/// It loads the config, finds matching remotes, and merges with tracking timestamps.
pub async fn get_remotes_for_path(
    path: &Path,
    workspace_dir_opt: Option<&Path>,
) -> Result<Vec<RemoteInfo>> {
    // Resolve workspace directory
    let workspace = if let Some(dir) = workspace_dir_opt {
        dir.to_path_buf()
    } else {
        closest_workspace_dir(path, false).await?
    };

    // Load config
    let config = stencila_config::config(path)?;

    // Find matching remote configs
    let matched_configs = find_remotes_for_path(path, &config, &workspace)?;

    if matched_configs.is_empty() {
        return Ok(Vec::new());
    }

    // Load tracking data
    let stencila_dir = closest_stencila_dir(path, false).await?;
    let tracking_entries = read_remote_entries(&stencila_dir).await?;

    // Get relative path for tracking lookup
    let relative_path = if path.is_absolute() {
        path.strip_prefix(&workspace).unwrap_or(path).to_path_buf()
    } else {
        path.to_path_buf()
    };

    // Combine config + tracking
    let mut results = Vec::new();
    for remote_config in matched_configs {
        let url = Url::parse(&remote_config.url)?;

        // Look up existing tracking data
        let existing = tracking_entries
            .get(&relative_path)
            .and_then(|remotes| remotes.get(&url));

        results.push(RemoteInfo {
            url: url.clone(),
            config: remote_config.clone(),
            pulled_at: existing.and_then(|e| e.pulled_at),
            pushed_at: existing.and_then(|e| e.pushed_at),
            watch_id: remote_config.watch.clone(),
            watch_direction: None, // Will be filled from Cloud API later if needed
        });
    }

    Ok(results)
}

/// Update remote tracking timestamps after push or pull
pub async fn update_remote_timestamp(
    path: &Path,
    url: &str,
    pulled_at: Option<u64>,
    pushed_at: Option<u64>,
) -> Result<()> {
    let workspace = closest_workspace_dir(path, false).await?;
    let stencila_dir = closest_stencila_dir(path, true).await?;

    // Parse URL
    let parsed_url = Url::parse(url)?;

    // Get relative path
    let relative_path = if path.is_absolute() {
        path.strip_prefix(&workspace).unwrap_or(path).to_path_buf()
    } else {
        path.to_path_buf()
    };

    // Load existing tracking
    let mut tracking_entries = read_remote_entries(&stencila_dir).await?;

    // Load config to get RemoteConfig
    let config = stencila_config::config(path)?;
    let remote_config = config
        .remotes
        .as_ref()
        .and_then(|remotes| remotes.iter().find(|r| r.url == url))
        .cloned();

    // Update or create entry
    let file_entry = tracking_entries.entry(relative_path).or_default();

    // Get or create the remote entry
    let remote_entry = file_entry
        .entry(parsed_url.clone())
        .or_insert_with(|| RemoteInfo {
            url: parsed_url.clone(),
            config: remote_config.unwrap_or_else(|| RemoteConfig {
                path: ConfigRelativePath(path.to_string_lossy().to_string()),
                url: url.to_string(),
                watch: None,
            }),
            pulled_at: None,
            pushed_at: None,
            watch_id: None,
            watch_direction: None,
        });

    if let Some(pulled) = pulled_at {
        remote_entry.pulled_at = Some(pulled);
    }
    if let Some(pushed) = pushed_at {
        remote_entry.pushed_at = Some(pushed);
    }

    // Write back
    write_remote_entries(&stencila_dir, &tracking_entries).await?;

    Ok(())
}

/// Expand a path into a list of files
///
/// If the path is a file, returns it as-is.
/// If the path is a directory, recursively finds all files within it.
pub fn expand_path_to_files(path: &Path) -> Result<Vec<PathBuf>> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }

    if !path.is_dir() {
        return Ok(Vec::new()); // Path doesn't exist yet or is neither file nor directory
    }

    let mut files = Vec::new();
    collect_files_recursive(path, &mut files)?;
    Ok(files)
}

/// Recursively collect all files in a directory
fn collect_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        // Skip hidden files and .stencila directory
        if let Some(name) = path.file_name().and_then(|n| n.to_str())
            && name.starts_with('.')
        {
            continue;
        }

        if path.is_file() {
            files.push(path);
        } else if path.is_dir() {
            collect_files_recursive(&path, files)?;
        }
    }
    Ok(())
}

/// Get all remote entries for files with configured remotes
///
/// This function enumerates all files that have remote configurations in stencila.toml
/// and combines tracking data from two sources:
/// 1. stencila.toml - Remote configurations and watch IDs
/// 2. .stencila/remotes.json - Pull/push timestamps
///
/// Returns `None` if no remotes are configured in stencila.toml.
/// Returns `Some(RemoteEntries)` with all files and their remote data.
pub async fn get_all_remote_entries(workspace_dir: &Path) -> Result<Option<RemoteEntries>> {
    // Load config to get remote configurations
    let config = stencila_config::config(workspace_dir)?;
    let Some(remotes_config) = config.remotes else {
        return Ok(None);
    };

    // Load .stencila/remotes.json for pull/push timestamps
    let stencila_dir = closest_stencila_dir(workspace_dir, false).await?;
    let remotes_tracking = read_remote_entries(&stencila_dir).await?;

    // Collect all files from config
    let mut result: RemoteEntries = BTreeMap::new();

    for remote_config in remotes_config {
        let config_path = remote_config.path.resolve(workspace_dir);

        // Expand to actual files, or use the config path itself if it doesn't exist yet
        let files = if config_path.exists() {
            expand_path_to_files(&config_path)?
        } else {
            // Path doesn't exist yet (remote-first workflow) - include it anyway
            vec![config_path]
        };

        for file in files {
            // Get relative path for tracking lookup
            let relative_path = match file.strip_prefix(workspace_dir) {
                Ok(rel) => rel.to_path_buf(),
                Err(_) => file.clone(),
            };

            // Parse remote URL
            let remote_url = match Url::parse(&remote_config.url) {
                Ok(url) => url,
                Err(e) => {
                    tracing::warn!("Skipping invalid URL '{}': {}", remote_config.url, e);
                    continue;
                }
            };

            // Get tracking data from remotes.json (look up by Url, not String)
            let existing = remotes_tracking
                .get(&relative_path)
                .and_then(|url_map| url_map.get(&remote_url));

            // Create RemoteInfo
            let file_remote_info = RemoteInfo {
                url: remote_url.clone(),
                config: remote_config.clone(),
                pulled_at: existing.and_then(|t| t.pulled_at),
                pushed_at: existing.and_then(|t| t.pushed_at),
                watch_id: remote_config.watch.clone(),
                watch_direction: None, // Will be filled from Cloud API later if needed
            };

            // Add remote to tracking
            result
                .entry(relative_path.clone())
                .or_default()
                .insert(remote_url, file_remote_info);
        }
    }

    if result.is_empty() {
        Ok(None)
    } else {
        Ok(Some(result))
    }
}

/// Update the watch ID for a remote in stencila.toml configuration
///
/// This stores or removes a watch ID in the configuration file for a specific
/// remote URL. Watch IDs are assigned by Stencila Cloud when watching is enabled.
///
/// # Arguments
///
/// * `path` - Path to a file in the workspace (used to locate stencila.toml)
/// * `url` - The remote URL to update
/// * `watch_id` - The watch ID to set, or None to remove watching
pub async fn update_watch_id(path: &Path, url: &str, watch_id: Option<String>) -> Result<()> {
    stencila_config::config_update_remote_watch(path, url, watch_id)?;
    Ok(())
}

/// Remove watch_ids that no longer exist on Stencila Cloud
///
/// Takes a set of valid watch IDs and removes any watch_id from the stencila.toml
/// config that is not in the provided set. Returns a list of removed watches
/// for display purposes.
pub async fn remove_deleted_watches(
    path: &Path,
    valid_watch_ids: &HashSet<u64>,
) -> Result<Vec<(PathBuf, Url, u64)>> {
    let mut deleted_watches = Vec::new();

    // Load the config to find remotes with watch IDs
    let config = stencila_config::config(path)?;
    let Some(remotes) = config.remotes else {
        return Ok(deleted_watches);
    };

    // Check each remote for watch IDs that need to be removed
    for remote_config in remotes {
        if let Some(watch_id_str) = &remote_config.watch
            && let Ok(watch_id) = watch_id_str.parse::<u64>()
            && !valid_watch_ids.contains(&watch_id)
        {
            // Watch no longer exists, remove it from config
            let workspace_dir = closest_workspace_dir(path, false).await?;
            let remote_path = remote_config.path.resolve(&workspace_dir);

            // Remove the watch ID from the config
            if stencila_config::config_update_remote_watch(
                &remote_path,
                &remote_config.url,
                None, // Remove watch ID
            )
            .is_ok()
            {
                deleted_watches.push((remote_path, Url::parse(&remote_config.url)?, watch_id));
            }
        }
    }

    Ok(deleted_watches)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path_with_leading_dot_slash() {
        let path = Path::new("./docs/file.md");
        let normalized = normalize_path(path);
        assert_eq!(normalized, PathBuf::from("docs/file.md"));
    }

    #[test]
    fn test_normalize_path_without_leading_dot_slash() {
        let path = Path::new("docs/file.md");
        let normalized = normalize_path(path);
        assert_eq!(normalized, PathBuf::from("docs/file.md"));
    }

    #[test]
    fn test_normalize_path_multiple_components() {
        let path = Path::new("./site/blog/post.md");
        let normalized = normalize_path(path);
        assert_eq!(normalized, PathBuf::from("site/blog/post.md"));
    }

    #[test]
    fn test_normalize_path_single_file() {
        let path = Path::new("./file.md");
        let normalized = normalize_path(path);
        assert_eq!(normalized, PathBuf::from("file.md"));
    }

    #[test]
    fn test_normalize_path_already_normalized() {
        let path = Path::new("already/normalized/path.md");
        let normalized = normalize_path(path);
        assert_eq!(normalized, PathBuf::from("already/normalized/path.md"));
    }
}
