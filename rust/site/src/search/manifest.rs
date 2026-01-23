//! Search index manifest data structures

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Current search index schema version
pub const SCHEMA_VERSION: u32 = 1;

/// Manifest file describing the search index
///
/// This file is loaded first by the client to discover available shards
/// and validate compatibility.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchManifest {
    /// Schema version for forward compatibility
    pub version: u32,

    /// Total number of indexed entries across all shards
    pub total_entries: usize,

    /// Total number of indexed documents/routes
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
