//! Search index manifest data structures

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Current search index schema version
pub const SCHEMA_VERSION: u32 = 1;

/// Top-level manifest file for access-level sharded search index
///
/// Lists available access levels and aggregate statistics.
/// Client loads this first, then loads per-level manifests based on user's access.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchRootManifest {
    /// Schema version for forward compatibility
    pub version: u32,

    /// Total number of indexed entries across all access levels
    pub total_entries: usize,

    /// Total number of indexed documents/routes
    pub total_routes: usize,

    /// Available access levels with entry counts
    ///
    /// Keys are access level names (e.g., "public", "password", "team").
    /// Values contain entry count for that level's index.
    pub levels: BTreeMap<String, AccessLevelInfo>,
}

/// Information about a single access level's index
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessLevelInfo {
    /// Number of entries at this access level
    pub entry_count: usize,

    /// Number of shards in this level's directory
    pub shard_count: usize,
}

impl SearchRootManifest {
    /// Create a new root manifest
    pub fn new(
        total_entries: usize,
        total_routes: usize,
        levels: BTreeMap<String, AccessLevelInfo>,
    ) -> Self {
        Self {
            version: SCHEMA_VERSION,
            total_entries,
            total_routes,
            levels,
        }
    }
}

impl AccessLevelInfo {
    /// Create a new access level info
    pub fn new(entry_count: usize, shard_count: usize) -> Self {
        Self {
            entry_count,
            shard_count,
        }
    }
}

/// Manifest file describing the search index for a single access level
///
/// Located at `_search/{level}/manifest.json`.
/// Client loads this to discover shards for a specific access level.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchManifest {
    /// Schema version for forward compatibility
    pub version: u32,

    /// Total number of indexed entries across all shards
    pub total_entries: usize,

    /// Total number of indexed documents/routes at this level
    pub total_routes: usize,

    /// Information about each shard, keyed by prefix (e.g., "ab", "co")
    ///
    /// The shard file can be derived as `shards/{prefix}.json`.
    pub shards: BTreeMap<String, ShardInfo>,
}

/// Information about a single shard file
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShardInfo {
    /// Number of entries in this shard
    pub entry_count: usize,

    /// File size in bytes (after any compression)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<usize>,
}

impl SearchManifest {
    /// Create a new manifest
    pub fn new(
        total_entries: usize,
        total_routes: usize,
        shards: BTreeMap<String, ShardInfo>,
    ) -> Self {
        Self {
            version: SCHEMA_VERSION,
            total_entries,
            total_routes,
            shards,
        }
    }
}

impl ShardInfo {
    /// Create a new shard info
    pub fn new(entry_count: usize) -> Self {
        Self {
            entry_count,
            size_bytes: None,
        }
    }
}
