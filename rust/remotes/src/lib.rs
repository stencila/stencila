//! Resolve, track, and describe Stencila document remotes.
//!
//! A remote is an external endpoint linked to a local document path that Stencila
//! can push to, pull from, or both. In many cases that endpoint is a remote
//! document hosted by another service and in another format, such as Google Docs
//! or Microsoft 365. In other cases it is a document exchange or review surface,
//! such as a GitHub pull request, where pushing a document can also produce or
//! update review artifacts like comments and suggestions.
//!
//! In that sense, remotes play a similar role for documents that Git plays for
//! source code and DVC plays for data: they connect a local document in the
//! workspace to a corresponding remote representation or exchange endpoint in
//! another system.
//!
//! This crate is the coordination layer for that model. It does not implement the
//! service-specific push and pull logic itself; instead it:
//!
//! - identifies which remote service a URL belongs to using [`RemoteService`]
//! - resolves which remotes apply to a local path
//! - merges shared configuration with local tracking state into [`RemoteInfo`]
//! - records pull and push timestamps for future status checks
//! - calculates synchronization status for display in the CLI and LSP
//! - manages optional watch metadata for cloud-automated synchronization
//!
//! # Shared configuration vs local state
//!
//! The remote model is intentionally split across two data sources:
//!
//! 1. `stencila.toml` stores the declarative, team-shareable mapping from local
//!    workspace paths to remote targets, plus any persisted watch identifiers.
//! 2. `.stencila/remotes.json` stores local operational state such as pull/push
//!    timestamps and spread-variant argument bindings.
//!
//! These are merged at runtime into [`RemoteInfo`].
//!
//! ```text
//! stencila.toml             .stencila/remotes.json
//! (shared intent)           (local tracking state)
//!       \                         /
//!        \                       /
//!         \                     /
//!          v                   v
//!               RemoteInfo
//! ```
//!
//! # Scope of this crate
//!
//! This crate is about document remotes. Stencila Sites are managed separately
//! and are not part of this crate's main conceptual model.
//!
//! Not all remotes support the same capabilities:
//!
//! - some are bidirectional
//! - some are pull-only
//! - some are push-only
//! - some support watch-based automation
//!
//! The important invariant is not symmetry, but that a remote is a stable
//! external target associated with a local document path.
//!
//! # Key types and entry points
//!
//! - [`RemoteService`]: classification and capability dispatch for supported services
//! - [`RemoteInfo`]: merged view of configured and tracked data for one remote
//! - [`RemoteEntries`]: all tracked/configured remotes indexed by local path and URL
//! - [`get_remotes_for_path`]: resolve remotes for one local path
//! - [`get_all_remote_entries`]: enumerate all known remotes in a workspace
//! - [`update_remote_timestamp`]: record successful pull/push activity
//! - [`calculate_remote_statuses`]: compare local and remote state to derive status

use std::{
    collections::{BTreeMap, HashSet},
    path::{Path, PathBuf},
};

use clap::ValueEnum;
use eyre::{Result, bail};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum::{Display, EnumString};
use tokio::fs::{read_to_string, write};
use url::Url;

use stencila_config::Config;
use stencila_dirs::{closest_stencila_dir, closest_workspace_dir};

mod service;
pub use service::RemoteService;

/// Remote entries indexed by local path and remote URL.
///
/// The outer map is keyed by workspace-relative file paths. The inner map is
/// keyed by remote URLs. Each value is a [`RemoteInfo`] describing the merged
/// configured and tracked state for that local-path/remote pair.
///
/// In persisted JSON, URLs are serialized as strings even though Rust uses [`Url`].
pub type RemoteEntries = BTreeMap<PathBuf, IndexMap<Url, RemoteInfo>>;

/// The merged view of one configured or tracked remote for a local path.
///
/// [`RemoteInfo`] combines:
///
/// - declarative configuration from `stencila.toml`
/// - local tracking state from `.stencila/remotes.json`
/// - optional watch metadata associated with the remote mapping
///
/// It is the primary type used by the CLI and LSP when listing remotes,
/// choosing targets, or calculating status.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteInfo {
    /// The remote endpoint URL.
    ///
    /// This may identify a remote document directly, or another remote exchange
    /// surface such as a review-oriented target.
    #[serde(skip, default = "RemoteInfo::default_url")]
    pub url: Url,

    /// The configured workspace-relative path pattern that matched this remote.
    ///
    /// This usually comes from `stencila.toml` and may be a file path or a
    /// directory-like path that matched the local file being resolved.
    #[serde(skip, default = "RemoteInfo::default_path")]
    pub path: String,

    /// The last successful pull time recorded for this local-path/remote pair.
    ///
    /// Stored as a Unix timestamp in seconds in `.stencila/remotes.json`.
    pub pulled_at: Option<u64>,

    /// The last successful push time recorded for this local-path/remote pair.
    ///
    /// Stored as a Unix timestamp in seconds in `.stencila/remotes.json`.
    pub pushed_at: Option<u64>,

    /// The cloud watch identifier, if watch automation is enabled for this remote.
    pub watch_id: Option<String>,

    /// The watch direction, if known.
    ///
    /// This may come from configuration or from the cloud API when watch details
    /// are enriched elsewhere.
    pub watch_direction: Option<WatchDirection>,

    /// Bound arguments for a spread-generated remote variant, if any.
    ///
    /// When a document is pushed as multiple spread variants, each resulting
    /// remote target can be tracked with the concrete argument values that
    /// produced it. This allows later pushes to match a local run to the same
    /// remote variant.
    pub arguments: Option<std::collections::HashMap<String, String>>,
}

impl RemoteInfo {
    /// Default URL for deserialization (will be replaced from map key)
    fn default_url() -> Url {
        Url::parse("http://placeholder").expect("valid placeholder URL")
    }

    /// Default path for deserialization (will be replaced from stencila.toml)
    fn default_path() -> String {
        ".".to_string()
    }

    /// Whether this remote currently has an associated watch identifier.
    pub fn is_watched(&self) -> bool {
        self.watch_id.is_some()
    }
}

/// Direction of synchronization for a watched remote.
///
/// A watch is optional cloud-managed automation attached to a remote mapping.
/// The direction determines which side initiates synchronized change handling.
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
    /// Bidirectional automation.
    ///
    /// Remote changes can create pull requests, and repository changes can be
    /// pushed back to the remote.
    #[default]
    Bi,

    /// One-way automation from remote to repository.
    FromRemote,

    /// One-way automation from repository to remote.
    ToRemote,
}

/// Pull request mode used by watch automation.
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
    /// Create pull requests as drafts.
    #[default]
    Draft,

    /// Create pull requests ready for review.
    Ready,
}

/// Create cloud watch automation for a remote and persist its watch ID.
///
/// This helper is shared by the push, pull, and watch commands. It creates the
/// watch through Stencila Cloud and stores the returned watch ID in
/// `stencila.toml` so the watch becomes part of the shared remote mapping.
///
/// Preconditions, such as validating that a file is eligible to participate in
/// watch automation, are the responsibility of the caller.
///
/// # Arguments
///
/// * `path` - Path to the local file
/// * `url` - The remote URL to watch
/// * `direction` - Sync direction (bi, from-remote, to-remote)
/// * `pr_mode` - PR mode (draft, ready)
/// * `debounce_seconds` - Debounce time in seconds
///
/// # Returns
///
/// The watch ID on success
pub async fn create_and_save_watch(
    path: &Path,
    url: &Url,
    direction: Option<WatchDirection>,
    pr_mode: Option<WatchPrMode>,
    debounce_seconds: Option<u64>,
) -> Result<String> {
    // Get git file info (path relative to repo root)
    let git_info = stencila_codec_utils::git_file_info(path)?;

    // Ensure workspace exists
    let (workspace_id, _) = stencila_cloud::ensure_workspace(path).await?;

    // Get file path relative to repo
    let file_path = git_info.path.unwrap_or_else(|| {
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string()
    });

    // Create watch via Cloud API
    let request = stencila_cloud::WatchRequest {
        remote_url: url.to_string(),
        file_path,
        direction: direction.map(|d| d.to_string()),
        pr_mode: pr_mode.map(|m| m.to_string()),
        debounce_seconds,
    };
    let response = stencila_cloud::create_watch(&workspace_id, request).await?;

    // Save watch ID to stencila.toml
    stencila_config::config_update_remote_watch(path, url.as_ref(), Some(response.id.clone()))?;

    Ok(response.id)
}

/// Synchronization status for a local-path/remote pair.
///
/// The meaning of each status depends on the remote's capabilities. For example,
/// write-only remotes do not meaningfully support all status transitions that a
/// bidirectional document remote does.
#[derive(Debug, Default, Display, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RemoteStatus {
    /// The relationship could not be determined.
    #[default]
    Unknown,

    /// The local path no longer exists in the workspace.
    Deleted,

    /// The remote appears newer than the local file and may need to be pulled.
    Ahead,

    /// The local file appears newer than the remote and may need to be pushed.
    Behind,

    /// Both local and remote appear to have changed since the last known sync point.
    Diverged,

    /// Local and remote appear synchronized within the configured tolerances.
    Synced,
}

/// Additional remote-side activity information for a local-path/remote pair.
///
/// This complements [`RemoteStatus`] for remotes where remote activity is useful
/// to surface separately from push/pull synchronization semantics. It is
/// especially useful for write-only review-oriented remotes such as GitHub pull
/// requests, where the remote may have new review activity even though there is
/// no pull workflow.
#[derive(Debug, Default, Display, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RemoteActivity {
    /// Remote activity could not be determined.
    #[default]
    Unknown,

    /// No remote activity is known since the last relevant sync point.
    Unchanged,

    /// The remote has changed since the last relevant sync point.
    Changed,
}

/// Derived status information for one remote.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteStatusInfo {
    /// The remote's last known modification or activity time, if available.
    pub remote_modified_at: Option<u64>,

    /// The derived synchronization status for this local-path/remote pair.
    pub status: RemoteStatus,

    /// Additional remote-side activity information, if available.
    pub activity: Option<RemoteActivity>,
}

/// Tolerance in seconds for local file modification time comparisons
pub const LOCAL_TOLERANCE_SECS: u64 = 5;

/// Tolerance in seconds for remote modification time comparisons
/// (accounts for cloud service processing delays)
pub const REMOTE_TOLERANCE_SECS: u64 = 30;

/// Calculate synchronization status for a set of remotes for one local path.
///
/// For each remote this function:
///
/// - fetches remote modification metadata in parallel where supported
/// - compares remote and local modification times
/// - uses recorded `pushed_at` and `pulled_at` timestamps as the last known sync point
/// - applies tolerance windows to account for filesystem and service timing skew
/// - adapts status interpretation for capability-limited remotes such as write-only ones
///
/// Returns a map from remote URL to [`RemoteStatusInfo`].
pub async fn calculate_remote_statuses(
    remotes: &IndexMap<Url, RemoteInfo>,
    local_status: RemoteStatus,
    local_modified_at: Option<u64>,
) -> IndexMap<Url, RemoteStatusInfo> {
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
                Some(RemoteService::GitHubIssues) => {
                    stencila_codec_github::issues::modified_at(url).await
                }
                Some(RemoteService::GitHubPullRequests) => {
                    bail!("GitHub pull request remotes do not support remote status checks")
                }
                Some(RemoteService::StencilaEmail) => stencila_cloud::email::modified_at(url).await,
                None => bail!("Unsupported remote service: {url}"),
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

        let activity = if is_write_only {
            match (remote_modified_at, remote.pushed_at) {
                (Some(remote_mod), Some(pushed)) => {
                    if remote_mod > pushed.saturating_add(REMOTE_TOLERANCE_SECS) {
                        Some(RemoteActivity::Changed)
                    } else {
                        Some(RemoteActivity::Unchanged)
                    }
                }
                _ => Some(RemoteActivity::Unknown),
            }
        } else {
            None
        };

        (
            url.clone(),
            RemoteStatusInfo {
                remote_modified_at,
                status,
                activity,
            },
        )
    });

    // Execute all futures in parallel and collect results into a IndexMap
    join_all(futures).await.into_iter().collect()
}

/// Read remote tracking state from `.stencila/remotes.json`.
///
/// This file stores local operational state and may legitimately be absent.
pub async fn read_remote_entries(stencila_dir: &Path) -> Result<RemoteEntries> {
    let remotes_file = stencila_dir.join("remotes.json");

    if !remotes_file.exists() {
        return Ok(BTreeMap::new());
    }

    let json = read_to_string(&remotes_file).await?;
    let entries = serde_json::from_str(&json)?;

    Ok(entries)
}

/// Write remote tracking state to `.stencila/remotes.json`.
pub async fn write_remote_entries(stencila_dir: &Path, entries: &RemoteEntries) -> Result<()> {
    let remotes_file = stencila_dir.join("remotes.json");

    let json = serde_json::to_string_pretty(entries)?;
    write(&remotes_file, json).await?;

    Ok(())
}

/// Remove tracking entries associated with a deleted Stencila Site.
///
/// Sites are managed separately from document remotes, but older or implicit
/// tracking entries may still exist in `.stencila/remotes.json`. This helper
/// cleans up those local tracking records.
///
/// # Arguments
///
/// * `stencila_dir` - The `.stencila` directory path
/// * `workspace_id` - The site ID to remove entries for
///
/// # Returns
///
/// The number of entries removed
pub async fn remove_site_remotes(stencila_dir: &Path, workspace_id: &str) -> Result<usize> {
    let mut entries = read_remote_entries(stencila_dir).await?;
    let mut removed_count = 0;

    // Remove entries for this site
    entries.retain(|_file_path, url_map| {
        url_map.retain(|url, _info| {
            let url_str = url.as_str();
            let is_this_site = url_str.contains(workspace_id) && url_str.contains(".stencila.site");

            if is_this_site {
                removed_count += 1;
            }

            !is_this_site
        });
        !url_map.is_empty()
    });

    if removed_count > 0 {
        write_remote_entries(stencila_dir, &entries).await?;
        tracing::debug!("Removed {removed_count} implicit remote entries for site {workspace_id}");
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

fn resolve_workspace_path(path_key: &str, workspace_dir: &Path) -> PathBuf {
    let path = PathBuf::from(path_key);
    if path.is_absolute() {
        path
    } else {
        workspace_dir.join(path)
    }
}

/// Find configured remotes whose path mapping applies to a file.
///
/// Matching currently supports exact file matches and directory-style prefix
/// matches. Directory paths are treated as implicitly recursive.
///
/// Returns `(path_key, url, watch_id)` tuples derived from `stencila.toml`.
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
        let config_path = resolve_workspace_path(path_key, workspace_dir);
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

/// Resolve all configured remotes for a local path.
///
/// This is the main entry point for push, pull, open, and status operations that
/// start from a local file path. It:
///
/// - loads the shared remote mappings from `stencila.toml`
/// - finds mappings that apply to the supplied path
/// - merges those mappings with local tracking state from `.stencila/remotes.json`
///
/// It returns only remotes that are configured for the path. For tracked-but-not-
/// configured remotes, such as spread-generated variants, use
/// [`get_tracked_remotes_for_path`].
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
    let config = stencila_config::get()?;

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
            path: path_key,
            pulled_at: existing.and_then(|e| e.pulled_at),
            pushed_at: existing.and_then(|e| e.pushed_at),
            watch_id,
            watch_direction: None, // Will be filled from Cloud API later if needed
            arguments: existing.and_then(|e| e.arguments.clone()),
        });
    }

    Ok(results)
}

/// Get all tracked remotes for a local path from `.stencila/remotes.json`.
///
/// Unlike [`get_remotes_for_path`], this function does not require a matching
/// `stencila.toml` entry. It returns all tracked remotes for the path, including
/// spread-generated variants that may exist only in local operational state.
///
/// This is primarily useful for workflows that need to reconnect to previously
/// created remote variants even when there is no explicit shared configuration
/// entry for each variant.
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

/// Update local tracking timestamps after a successful push or pull.
///
/// This writes to `.stencila/remotes.json`, creating the local-path/remote pair
/// if necessary. It records operational state only; it does not create shared
/// remote configuration unless that has already been done elsewhere.
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
    let config = stencila_config::get()?;
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
            path: config_path,
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

/// Update local tracking state for a spread-generated remote variant.
///
/// In addition to recording `pushed_at`, this stores the concrete argument values
/// that produced the remote so later runs can match the same variant.
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
    let config = stencila_config::get()?;
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
            path: config_path,
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

/// Find the tracked remote variant whose bound arguments match exactly.
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

/// Expand a configured path into concrete files.
///
/// If the path is a file, returns it unchanged. If it is a directory, returns
/// all descendant files, excluding hidden entries.
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

/// Enumerate all known remotes in a workspace.
///
/// This function starts from `stencila.toml`, expands configured file and
/// directory mappings into concrete paths, then merges in local tracking data
/// from `.stencila/remotes.json`.
///
/// It also includes tracked remotes that are not currently represented by an
/// explicit config entry, such as spread-generated variants.
///
/// Returns `None` when the workspace has no known remotes.
pub async fn get_all_remote_entries(workspace_dir: &Path) -> Result<Option<RemoteEntries>> {
    // Load config to get remote configurations
    let config = stencila_config::get()?;

    // Load .stencila/remotes.json for pull/push timestamps
    let stencila_dir = closest_stencila_dir(workspace_dir, false).await?;
    let remotes_tracking = read_remote_entries(&stencila_dir).await?;

    // Collect all files from config
    let mut result: RemoteEntries = BTreeMap::new();

    // Process explicit remotes from [remotes] section if it exists
    if let Some(remotes) = &config.remotes {
        for (path_key, value) in remotes {
            let config_path = resolve_workspace_path(path_key, workspace_dir);

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
                        path: path_key.clone(),
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
                path: tracked_path.to_string_lossy().to_string(),
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

/// Update the persisted watch ID for a configured remote.
///
/// This modifies `stencila.toml`, not `.stencila/remotes.json`, because watch IDs
/// are part of the shared remote mapping rather than local operational state.
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

/// Remove persisted watch IDs that no longer exist in Stencila Cloud.
///
/// This reconciles locally configured watch IDs against a set of valid watch IDs
/// returned by the cloud API. Any missing IDs are removed from `stencila.toml`.
///
/// Returns details of removed watches for display to the user.
pub async fn remove_deleted_watches(
    path: &Path,
    valid_watch_ids: &HashSet<String>,
) -> Result<Vec<(PathBuf, Url, String)>> {
    let mut deleted_watches = Vec::new();

    // Load the config to find remotes with watch IDs
    let config = stencila_config::get()?;
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
                let remote_path = resolve_workspace_path(path_key, &workspace_dir);

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
