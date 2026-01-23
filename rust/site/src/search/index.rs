//! Search index builder
//!
//! Collects search entries during rendering and writes the final index.

use std::collections::HashSet;
use std::path::Path;

use eyre::Result;
use tokio::fs;

use super::entry::SearchEntry;
use super::manifest::SearchManifest;
use super::shard::shard_entries;

/// Builder for search index
///
/// Collects entries during rendering and writes the final index.
#[derive(Debug, Default)]
pub struct SearchIndexBuilder {
    /// Collected entries
    entries: Vec<SearchEntry>,
    /// Set of routes that have been indexed
    routes: HashSet<String>,
}

impl SearchIndexBuilder {
    /// Create a new index builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Add entries to the index
    pub fn add_entries(&mut self, entries: Vec<SearchEntry>) {
        for entry in entries {
            self.routes.insert(entry.route.clone());
            self.entries.push(entry);
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
    /// Creates:
    /// - `_search/manifest.json` - Index manifest
    /// - `_search/shards/{prefix}.json` - Shard files
    pub async fn write(self, output_dir: &Path) -> Result<SearchIndexStats> {
        let search_dir = output_dir.join("_search");
        let shards_dir = search_dir.join("shards");
        fs::create_dir_all(&shards_dir).await?;

        let total_entries = self.entries.len();
        let total_routes = self.routes.len();

        // Shard the entries
        let shard_result = shard_entries(self.entries);

        // Write shard files
        let mut shard_count = 0;
        for (filename, entries) in &shard_result.shards {
            let shard_path = search_dir.join(filename);
            let json = serde_json::to_string(entries)?;
            fs::write(&shard_path, &json).await?;
            shard_count += 1;
        }

        // Create manifest with file sizes
        let mut shard_infos = shard_result.shard_infos;
        for info in &mut shard_infos {
            let shard_path = search_dir.join(&info.file);
            if let Ok(metadata) = fs::metadata(&shard_path).await {
                info.size_bytes = Some(metadata.len() as usize);
            }
        }

        let manifest = SearchManifest::new(total_entries, total_routes, shard_infos);

        // Write manifest
        let manifest_path = search_dir.join("manifest.json");
        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        fs::write(&manifest_path, &manifest_json).await?;

        Ok(SearchIndexStats {
            total_entries,
            total_routes,
            shard_count,
        })
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
