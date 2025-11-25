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
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum::{Display, EnumString};
use tokio::fs::{read_to_string, write};
use url::Url;

use stencila_config::{Config, ConfigRelativePath};
use stencila_dirs::{closest_stencila_dir, closest_workspace_dir};

mod service;
pub use service::RemoteService;

/// Type alias for remote tracking entries in .stencila/remotes.json
///
/// Maps file paths to a map of remote URLs to tracking data.
/// URLs are serialized as strings in JSON but use Url type in Rust.
pub type RemoteEntries = BTreeMap<PathBuf, IndexMap<Url, RemoteInfo>>;

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

    /// File path from stencila.toml (workspace-relative)
    #[serde(skip, default = "RemoteInfo::default_path")]
    pub path: ConfigRelativePath,

    /// Last time pulled from this remote (Unix timestamp in seconds)
    pub pulled_at: Option<u64>,

    /// Last time pushed to this remote (Unix timestamp in seconds)
    pub pushed_at: Option<u64>,

    /// The watch ID from Stencila Cloud if watching is enabled
    pub watch_id: Option<String>,

    /// The watch direction (from stencila.toml or Cloud API)
    pub watch_direction: Option<WatchDirection>,

    /// Arguments used when pushing to this remote (for spread variants)
    ///
    /// When a document is pushed with spread parameters, each variant is tracked
    /// with its specific argument values. This allows matching variants on
    /// subsequent pushes.
    pub arguments: Option<std::collections::HashMap<String, String>>,
}

impl RemoteInfo {
    /// Default URL for deserialization (will be replaced from map key)
    fn default_url() -> Url {
        Url::parse("http://placeholder").expect("valid placeholder URL")
    }

    /// Default path for deserialization (will be replaced from stencila.toml)
    fn default_path() -> ConfigRelativePath {
        ConfigRelativePath(".".to_string())
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
    remotes: &IndexMap<Url, RemoteInfo>,
    local_status: RemoteStatus,
    local_modified_at: Option<u64>,
) -> IndexMap<Url, (Option<u64>, RemoteStatus)> {
    use futures::future::join_all;

    if remotes.is_empty() {
        return IndexMap::new();
    }

    // Create futures to fetch metadata for each remote
    let futures = remotes.iter().map(|(url, remote)| async move {
        // Check if this is a write-only remote (like Stencila Sites)
        let service = RemoteService::from_url(url);
        let is_write_only = service.map(|s| s.is_write_only()).unwrap_or(false);

        // Fetch metadata from remote service
        let remote_modified_at = async {
            match service {
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
            // For write-only remotes, deleted local files just need to be untracked
            // For other remotes, mark as Ahead if remote exists (suggesting pull)
            if is_write_only {
                RemoteStatus::Unknown
            } else if remote_modified_at.is_some() {
                RemoteStatus::Ahead
            } else {
                // Remote metadata unavailable - could be deleted, network issue, etc.
                RemoteStatus::Unknown
            }
        } else if is_write_only {
            // For write-only remotes (e.g., Stencila Sites), only show Behind or Synced
            // - Never "Diverged" (can't pull to reconcile)
            // - Never "Ahead" (remote changes don't matter, can't pull them)
            match (local_modified_at, remote.pushed_at) {
                (Some(local), Some(pushed)) => {
                    if local > pushed.saturating_add(LOCAL_TOLERANCE_SECS) {
                        RemoteStatus::Behind // Local changed, need to push
                    } else {
                        RemoteStatus::Synced
                    }
                }
                (Some(_), None) => RemoteStatus::Behind, // Never pushed, need to push
                _ => RemoteStatus::Unknown,
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

    // Execute all futures in parallel and collect results into a IndexMap
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

/// Remove remote tracking entries for a specific Stencila Site
///
/// This function removes all entries from `.stencila/remotes.json` that are
/// associated with the given site ID. This should be called when a site is
/// deleted to clean up implicit remote tracking.
///
/// # Arguments
///
/// * `stencila_dir` - The `.stencila` directory path
/// * `site_id` - The site ID to remove entries for
///
/// # Returns
///
/// The number of entries removed
pub async fn remove_site_remotes(stencila_dir: &Path, site_id: &str) -> Result<usize> {
    let mut entries = read_remote_entries(stencila_dir).await?;
    let mut removed_count = 0;

    // Remove entries for this site
    entries.retain(|_file_path, url_map| {
        url_map.retain(|url, _info| {
            let url_str = url.as_str();
            let is_this_site = url_str.contains(site_id) && url_str.contains(".stencila.site");

            if is_this_site {
                removed_count += 1;
            }

            !is_this_site
        });
        !url_map.is_empty()
    });

    if removed_count > 0 {
        write_remote_entries(stencila_dir, &entries).await?;
        tracing::debug!("Removed {removed_count} implicit remote entries for site {site_id}");
    }

    Ok(removed_count)
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
///
/// Returns tuples of (path_key, url, watch_id)
fn find_remotes_for_path(
    file_path: &Path,
    config: &Config,
    workspace_dir: &Path,
) -> Result<Vec<(String, Url, Option<String>)>> {
    let Some(remotes) = &config.remotes else {
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

    for (path_key, value) in remotes {
        let config_path = ConfigRelativePath(path_key.clone()).resolve(workspace_dir);
        let config_relative = config_path
            .strip_prefix(workspace_dir)
            .unwrap_or(&config_path);
        let config_relative = normalize_path(config_relative);

        // Check if file matches this remote's path
        if path_matches(&config_relative, &relative_path, workspace_dir)? {
            // Add all targets for this path (skip Spread targets which don't have URLs)
            for target in value.to_vec() {
                if let Some(url) = target.url_owned() {
                    matches.push((path_key.clone(), url, target.watch().map(|s| s.to_string())));
                }
            }
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
    for (path_key, url, watch_id) in matched_configs {
        // Look up existing tracking data
        let existing = tracking_entries
            .get(&relative_path)
            .and_then(|remotes| remotes.get(&url));

        results.push(RemoteInfo {
            url: url.clone(),
            path: ConfigRelativePath(path_key),
            pulled_at: existing.and_then(|e| e.pulled_at),
            pushed_at: existing.and_then(|e| e.pushed_at),
            watch_id,
            watch_direction: None, // Will be filled from Cloud API later if needed
            arguments: existing.and_then(|e| e.arguments.clone()),
        });
    }

    Ok(results)
}

/// Get all tracked remotes for a path from the tracking file
///
/// Unlike `get_remotes_for_path`, this function reads directly from `.stencila/remotes.json`
/// and returns all tracked remotes for the given path, including spread variants that may
/// not have corresponding entries in `stencila.toml`.
///
/// This is useful for spread push operations where variants are tracked in the remotes file
/// but not configured in the TOML config.
pub async fn get_tracked_remotes_for_path(path: &Path) -> Result<Vec<RemoteInfo>> {
    let workspace = closest_workspace_dir(path, false).await?;
    let stencila_dir = match closest_stencila_dir(path, false).await {
        Ok(dir) => dir,
        Err(_) => return Ok(Vec::new()), // No .stencila dir means no tracking
    };

    // Canonicalize path to be relative to workspace
    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let relative_path = absolute_path
        .strip_prefix(&workspace)
        .unwrap_or(&absolute_path)
        .to_path_buf();

    // Load tracking data
    let tracking_entries = read_remote_entries(&stencila_dir).await?;

    // Get all remotes for this path from tracking
    let Some(remotes_for_path) = tracking_entries.get(&relative_path) else {
        return Ok(Vec::new());
    };

    // Convert to Vec<RemoteInfo>, populating url from map key
    // (url field has #[serde(skip)] so needs to be set from key)
    let results: Vec<RemoteInfo> = remotes_for_path
        .iter()
        .map(|(url, info)| {
            let mut info = info.clone();
            info.url = url.clone();
            info
        })
        .collect();

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

    // Canonicalize path to be relative to workspace
    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let relative_path = absolute_path
        .strip_prefix(&workspace)
        .unwrap_or(&absolute_path)
        .to_path_buf();

    // Load existing tracking
    let mut tracking_entries = read_remote_entries(&stencila_dir).await?;

    // Load config to find path and watch info
    let config = stencila_config::config(path)?;
    let (config_path, watch_id) = if let Some(remotes) = &config.remotes {
        // Find the remote config that matches this URL
        let mut found_path = None;
        let mut found_watch = None;
        for (path_key, value) in remotes {
            for target in value.to_vec() {
                if target.url() == Some(url) {
                    found_path = Some(path_key.clone());
                    found_watch = target.watch().map(|s| s.to_string());
                    break;
                }
            }
            if found_path.is_some() {
                break;
            }
        }
        (
            found_path.unwrap_or_else(|| path.to_string_lossy().to_string()),
            found_watch,
        )
    } else {
        (path.to_string_lossy().to_string(), None)
    };

    // Update or create entry
    let file_entry = tracking_entries.entry(relative_path).or_default();

    // Get or create the remote entry
    let remote_entry = file_entry
        .entry(parsed_url.clone())
        .or_insert_with(|| RemoteInfo {
            url: parsed_url.clone(),
            path: ConfigRelativePath(config_path),
            pulled_at: None,
            pushed_at: None,
            watch_id,
            watch_direction: None,
            arguments: None,
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

/// Update remote tracking with spread arguments after push
///
/// This is used when pushing spread variants to track each variant's URL
/// along with its specific argument values.
pub async fn update_spread_remote_timestamp(
    path: &Path,
    url: &str,
    arguments: &std::collections::HashMap<String, String>,
    pushed_at: u64,
) -> Result<()> {
    let workspace = closest_workspace_dir(path, false).await?;
    let stencila_dir = closest_stencila_dir(path, true).await?;

    // Parse URL
    let parsed_url = Url::parse(url)?;

    // Canonicalize path to be relative to workspace
    let absolute_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let relative_path = absolute_path
        .strip_prefix(&workspace)
        .unwrap_or(&absolute_path)
        .to_path_buf();

    // Load existing tracking
    let mut tracking_entries = read_remote_entries(&stencila_dir).await?;

    // Load config to find path and watch info
    let config = stencila_config::config(path)?;
    let config_path = if let Some(remotes) = &config.remotes {
        // Find the remote config that matches this URL
        let mut found_path = None;
        for (path_key, value) in remotes {
            for target in value.to_vec() {
                if target.url() == Some(url) {
                    found_path = Some(path_key.clone());
                    break;
                }
            }
            if found_path.is_some() {
                break;
            }
        }
        found_path.unwrap_or_else(|| path.to_string_lossy().to_string())
    } else {
        path.to_string_lossy().to_string()
    };

    // Update or create entry
    let file_entry = tracking_entries.entry(relative_path).or_default();

    // Get or create the remote entry with arguments
    let remote_entry = file_entry
        .entry(parsed_url.clone())
        .or_insert_with(|| RemoteInfo {
            url: parsed_url.clone(),
            path: ConfigRelativePath(config_path),
            pulled_at: None,
            pushed_at: None,
            watch_id: None,
            watch_direction: None,
            arguments: None,
        });

    remote_entry.pushed_at = Some(pushed_at);
    remote_entry.arguments = Some(arguments.clone());

    // Write back
    write_remote_entries(&stencila_dir, &tracking_entries).await?;

    Ok(())
}

/// Find a remote matching the given arguments
///
/// Searches through a list of remotes to find one with matching argument values.
/// Returns the first remote that has arguments matching exactly.
pub fn find_remote_for_arguments<'a>(
    remotes: &'a [RemoteInfo],
    arguments: &std::collections::HashMap<String, String>,
) -> Option<&'a RemoteInfo> {
    remotes.iter().find(|remote| {
        if let Some(remote_args) = &remote.arguments {
            remote_args == arguments
        } else {
            false
        }
    })
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

    // Load .stencila/remotes.json for pull/push timestamps
    let stencila_dir = closest_stencila_dir(workspace_dir, false).await?;
    let remotes_tracking = read_remote_entries(&stencila_dir).await?;

    // Collect all files from config
    let mut result: RemoteEntries = BTreeMap::new();

    // Process explicit remotes from [remotes] section if it exists
    if let Some(remotes) = &config.remotes {
        for (path_key, value) in remotes {
            let config_path = ConfigRelativePath(path_key.clone()).resolve(workspace_dir);

            // Expand to actual files, or use the config path itself if it doesn't exist yet
            let files = if config_path.exists() {
                expand_path_to_files(&config_path)?
            } else {
                // Path doesn't exist yet (remote-first workflow) - include it anyway
                vec![config_path]
            };

            for target in value.to_vec() {
                // Skip Spread targets which don't have URLs
                let Some(remote_url) = target.url_owned() else {
                    continue;
                };
                let watch_id = target.watch().map(|s| s.to_string());

                for file in &files {
                    // Get relative path for tracking lookup
                    let relative_path = match file.strip_prefix(workspace_dir) {
                        Ok(rel) => rel.to_path_buf(),
                        Err(_) => file.clone(),
                    };

                    // Get tracking data from remotes.json (look up by Url, not String)
                    let existing = remotes_tracking
                        .get(&relative_path)
                        .and_then(|url_map| url_map.get(&remote_url));

                    // Create RemoteInfo
                    let file_remote_info = RemoteInfo {
                        url: remote_url.clone(),
                        path: ConfigRelativePath(path_key.clone()),
                        pulled_at: existing.and_then(|t| t.pulled_at),
                        pushed_at: existing.and_then(|t| t.pushed_at),
                        watch_id: watch_id.clone(),
                        watch_direction: None, // Will be filled from Cloud API later if needed
                        arguments: existing.and_then(|t| t.arguments.clone()),
                    };

                    // Add remote to tracking
                    result
                        .entry(relative_path.clone())
                        .or_default()
                        .insert(remote_url.clone(), file_remote_info);
                }
            }
        }
    }

    // Process implicit site remotes
    // If site.id is configured, check for tracked files under site.root
    if let Some(site_config) = &config.site
        && site_config.id.is_some()
    {
        // Check tracking data for files under site root
        for (tracked_path, url_map) in &remotes_tracking {
            // Resolve the tracked path relative to workspace
            let absolute_tracked_path = workspace_dir.join(tracked_path);

            // Check if this file is under site root (or is the site root itself)
            if !config.path_is_in_site_root(&absolute_tracked_path, workspace_dir) {
                continue;
            }

            // Process each remote URL for this file
            for (remote_url, tracking_info) in url_map {
                // Check if this is a Stencila Sites URL
                if !matches!(
                    RemoteService::from_url(remote_url),
                    Some(RemoteService::StencilaSites)
                ) {
                    continue;
                }

                // Skip if we already have this remote from explicit config
                if result.contains_key(tracked_path)
                    && result[tracked_path].contains_key(remote_url)
                {
                    continue;
                }

                // Create implicit remote info
                // Site watches are stored in [site].watch, not [remotes]
                let file_remote_info = RemoteInfo {
                    url: remote_url.clone(),
                    path: ConfigRelativePath(tracked_path.to_string_lossy().to_string()),
                    pulled_at: tracking_info.pulled_at,
                    pushed_at: tracking_info.pushed_at,
                    watch_id: site_config.watch.clone(),
                    watch_direction: site_config.watch.as_ref().map(|_| WatchDirection::ToRemote),
                    arguments: tracking_info.arguments.clone(),
                };

                // Add implicit remote
                result
                    .entry(tracked_path.clone())
                    .or_default()
                    .insert(remote_url.clone(), file_remote_info);
            }
        }
    }

    // Process any tracked remotes that aren't already in the result
    // These may be created via spread push or other operations without explicit [remotes] entries
    for (tracked_path, url_map) in &remotes_tracking {
        for (remote_url, tracking_info) in url_map {
            // Skip if we already have this remote from explicit config or implicit site remotes
            if result.contains_key(tracked_path) && result[tracked_path].contains_key(remote_url) {
                continue;
            }

            // Create implicit remote info
            let file_remote_info = RemoteInfo {
                url: remote_url.clone(),
                path: ConfigRelativePath(tracked_path.to_string_lossy().to_string()),
                pulled_at: tracking_info.pulled_at,
                pushed_at: tracking_info.pushed_at,
                watch_id: None, // Implicit remotes don't have watches
                watch_direction: None,
                arguments: tracking_info.arguments.clone(),
            };

            // Add implicit remote
            result
                .entry(tracked_path.clone())
                .or_default()
                .insert(remote_url.clone(), file_remote_info);
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
    valid_watch_ids: &HashSet<String>,
) -> Result<Vec<(PathBuf, Url, String)>> {
    let mut deleted_watches = Vec::new();

    // Load the config to find remotes with watch IDs
    let config = stencila_config::config(path)?;
    let Some(remotes) = &config.remotes else {
        return Ok(deleted_watches);
    };

    let workspace_dir = closest_workspace_dir(path, false).await?;

    // Check each remote for watch IDs that need to be removed
    for (path_key, value) in remotes {
        for target in value.to_vec() {
            if let Some(watch_id) = target.watch()
                && !valid_watch_ids.contains(watch_id)
                && let Some(url_str) = target.url()
            {
                // Watch no longer exists, remove it from config
                let remote_path = ConfigRelativePath(path_key.clone()).resolve(&workspace_dir);

                // Remove the watch ID from the config
                if stencila_config::config_update_remote_watch(
                    &remote_path,
                    url_str,
                    None, // Remove watch ID
                )
                .is_ok()
                {
                    deleted_watches.push((remote_path, Url::parse(url_str)?, watch_id.to_string()));
                }
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
