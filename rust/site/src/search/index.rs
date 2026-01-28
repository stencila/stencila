//! Search index builder
//!
//! Collects search entries during rendering and writes the final index.

use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use eyre::Result;
use stencila_config::AccessLevel;
use tokio::fs;

use super::entry::{SearchEntry, TokenTrigrams};
use super::manifest::{AccessLevelInfo, SearchManifest, SearchRootManifest};
use super::shard::shard_entries;
use super::tokenize::{generate_trigrams, tokenize_with_positions};

/// Builder for search index
///
/// Collects entries during rendering and writes the final index.
#[derive(Debug, Default)]
pub struct SearchIndexBuilder {
    /// Collected entries
    entries: Vec<SearchEntry>,
    /// Set of routes that have been indexed
    routes: HashSet<String>,
    /// Whether to generate token trigrams for fuzzy search
    enable_fuzzy: bool,
}

impl SearchIndexBuilder {
    /// Create a new index builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable fuzzy search by generating token trigrams
    ///
    /// When enabled, each entry will have pre-computed trigrams for its tokens,
    /// enabling fuzzy matching in the search client.
    pub fn with_fuzzy(mut self, enable: bool) -> Self {
        self.enable_fuzzy = enable;
        self
    }

    /// Add entries to the index
    pub fn add_entries(&mut self, entries: Vec<SearchEntry>) {
        for mut entry in entries {
            self.routes.insert(entry.route.clone());

            // Generate token trigrams if fuzzy search is enabled
            if self.enable_fuzzy {
                entry = self.add_trigrams_to_entry(entry);
            }

            self.entries.push(entry);
        }
    }

    /// Add token trigrams to an entry for fuzzy search
    fn add_trigrams_to_entry(&self, entry: SearchEntry) -> SearchEntry {
        let tokens_with_positions = tokenize_with_positions(&entry.text);

        let token_trigrams: Vec<TokenTrigrams> = tokens_with_positions
            .into_iter()
            .filter_map(|twp| {
                let trigrams = generate_trigrams(&twp.token);
                // Only include tokens with trigrams (>= 3 chars)
                if trigrams.is_empty() {
                    None
                } else {
                    Some(TokenTrigrams::new(twp.token, trigrams, twp.start, twp.end))
                }
            })
            .collect();

        if token_trigrams.is_empty() {
            entry
        } else {
            entry.with_token_trigrams(token_trigrams)
        }
    }

    /// Get the number of entries collected
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Get the number of routes indexed
    pub fn route_count(&self) -> usize {
        self.routes.len()
    }

    /// Write the search index to the output directory
    ///
    /// Creates access-level sharded structure:
    /// - `_search/manifest.json` - Root manifest listing access levels
    /// - `_search/{level}/manifest.json` - Per-level manifest
    /// - `_search/{level}/shards/{prefix}.json` - Per-level shard files
    ///
    /// Each access level directory contains only entries at that level (non-cumulative).
    /// Client loads shards from all accessible levels and merges results.
    pub async fn write(self, output_dir: &Path) -> Result<SearchIndexStats> {
        let search_dir = output_dir.join("_search");
        fs::create_dir_all(&search_dir).await?;

        let total_entries = self.entries.len();
        let total_routes = self.routes.len();

        // Group entries by access level
        let mut entries_by_level: BTreeMap<AccessLevel, Vec<SearchEntry>> = BTreeMap::new();
        let mut routes_by_level: BTreeMap<AccessLevel, HashSet<String>> = BTreeMap::new();

        for entry in self.entries {
            let level = entry.access_level;
            routes_by_level
                .entry(level)
                .or_default()
                .insert(entry.route.clone());
            entries_by_level.entry(level).or_default().push(entry);
        }

        // Track per-level info for root manifest
        let mut level_infos: BTreeMap<String, AccessLevelInfo> = BTreeMap::new();
        let mut total_shard_count = 0;

        // Write each access level's index
        for (level, entries) in entries_by_level {
            let level_name = access_level_to_string(level);
            let level_dir = search_dir.join(&level_name);
            let shards_dir = level_dir.join("shards");
            fs::create_dir_all(&shards_dir).await?;

            let level_entry_count = entries.len();
            let level_route_count = routes_by_level
                .get(&level)
                .map(|r| r.len())
                .unwrap_or_default();

            // Shard the entries for this level
            let shard_result = shard_entries(entries);
            let level_shard_count = shard_result.shards.len();

            // Write shard files
            for (prefix, shard_data) in &shard_result.shards {
                let shard_path = shards_dir.join(format!("{prefix}.json"));
                let json = serde_json::to_string(shard_data)?;
                fs::write(&shard_path, &json).await?;
            }

            // Create per-level manifest with file sizes
            let mut shard_infos = shard_result.shard_infos;
            for (prefix, info) in &mut shard_infos {
                let shard_path = shards_dir.join(format!("{prefix}.json"));
                if let Ok(metadata) = fs::metadata(&shard_path).await {
                    info.size_bytes = Some(metadata.len() as usize);
                }
            }

            let level_manifest =
                SearchManifest::new(level_entry_count, level_route_count, shard_infos);

            // Write per-level manifest
            let manifest_path = level_dir.join("manifest.json");
            let manifest_json = serde_json::to_string_pretty(&level_manifest)?;
            fs::write(&manifest_path, &manifest_json).await?;

            // Track for root manifest
            level_infos.insert(
                level_name,
                AccessLevelInfo::new(level_entry_count, level_shard_count),
            );
            total_shard_count += level_shard_count;
        }

        // Write root manifest
        let root_manifest = SearchRootManifest::new(total_entries, total_routes, level_infos);
        let root_manifest_path = search_dir.join("manifest.json");
        let root_manifest_json = serde_json::to_string_pretty(&root_manifest)?;
        fs::write(&root_manifest_path, &root_manifest_json).await?;

        Ok(SearchIndexStats {
            total_entries,
            total_routes,
            shard_count: total_shard_count,
        })
    }
}

/// Convert an access level to its string representation for directory names
fn access_level_to_string(level: AccessLevel) -> String {
    match level {
        AccessLevel::Public => "public".to_string(),
        AccessLevel::Subscriber => "subscriber".to_string(),
        AccessLevel::Password => "password".to_string(),
        AccessLevel::Team => "team".to_string(),
    }
}

/// Statistics about the generated search index
#[derive(Debug, Clone)]
pub struct SearchIndexStats {
    /// Total number of entries
    pub total_entries: usize,
    /// Total number of routes indexed
    pub total_routes: usize,
    /// Number of shards created
    pub shard_count: usize,
}
