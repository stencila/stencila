//! Search index builder
//!
//! Collects search entries during rendering and writes the final index.

use std::collections::HashSet;
use std::path::Path;

use eyre::Result;
use tokio::fs;

use super::entry::{SearchEntry, TokenTrigrams};
use super::manifest::SearchManifest;
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
        for (prefix, shard_data) in &shard_result.shards {
            let shard_path = shards_dir.join(format!("{prefix}.json"));
            let json = serde_json::to_string(shard_data)?;
            fs::write(&shard_path, &json).await?;
            shard_count += 1;
        }

        // Create manifest with file sizes
        let mut shard_infos = shard_result.shard_infos;
        for (prefix, info) in &mut shard_infos {
            let shard_path = shards_dir.join(format!("{prefix}.json"));
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
