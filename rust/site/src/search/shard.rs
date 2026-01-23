//! Sharding logic for search indices
//!
//! Entries are sharded by the 2-character prefix of their tokens.
//! This allows the client to load only the shards needed for a query.

use std::collections::{HashMap, HashSet};

use super::entry::SearchEntry;
use super::manifest::ShardInfo;
use super::tokenize::{token_prefix, tokenize};

/// Result of sharding: shards and their info
pub struct ShardResult {
    /// The shards, keyed by filename (e.g., "shards/ab.json" -> entries)
    pub shards: HashMap<String, Vec<SearchEntry>>,

    /// Shard info for the manifest
    pub shard_infos: Vec<ShardInfo>,
}

/// Shard entries by token prefix
///
/// Each entry is placed in shards for all unique prefixes of its tokens.
/// This means an entry may appear in multiple shards if its text contains
/// tokens with different prefixes.
pub fn shard_entries(entries: Vec<SearchEntry>) -> ShardResult {
    // Group entries by prefix
    let mut prefix_to_entries: HashMap<String, Vec<SearchEntry>> = HashMap::new();

    for entry in entries {
        // Get unique prefixes for this entry's tokens
        let tokens = tokenize(&entry.text);
        let prefixes: HashSet<String> = tokens.iter().map(|t| token_prefix(t)).collect();

        for prefix in prefixes {
            prefix_to_entries
                .entry(prefix)
                .or_default()
                .push(entry.clone());
        }
    }

    // Convert to shards
    let mut shards = HashMap::new();
    let mut shard_infos = Vec::new();

    // Sort prefixes for deterministic output
    let mut prefixes: Vec<String> = prefix_to_entries.keys().cloned().collect();
    prefixes.sort();

    for prefix in prefixes {
        let entries = prefix_to_entries.remove(&prefix).unwrap_or_default();
        let entry_count = entries.len();

        if entry_count == 0 {
            continue;
        }

        // Use shards/aa.json format for cleaner organization
        let filename = format!("shards/{}.json", prefix);

        let shard_info = ShardInfo::new(filename.clone(), entry_count);

        shards.insert(filename, entries);
        shard_infos.push(shard_info);
    }

    ShardResult {
        shards,
        shard_infos,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shard_entries() {
        let entries = vec![
            SearchEntry::new(
                "hed_1".to_string(),
                "Heading",
                "/test/".to_string(),
                "Hello World".to_string(),
                8,
                1,
            ),
            SearchEntry::new(
                "pgh_1".to_string(),
                "Paragraph",
                "/test/".to_string(),
                "Hello there".to_string(),
                2,
                2,
            ),
            SearchEntry::new(
                "pgh_2".to_string(),
                "Paragraph",
                "/test/".to_string(),
                "World peace".to_string(),
                2,
                2,
            ),
        ];

        let result = shard_entries(entries);

        // Should have shards for: "he" (hello), "wo" (world), "th" (there), "pe" (peace)
        assert!(result.shards.contains_key("shards/he.json"));
        assert!(result.shards.contains_key("shards/wo.json"));
        assert!(result.shards.contains_key("shards/th.json"));
        assert!(result.shards.contains_key("shards/pe.json"));

        // "hello" appears in two entries, so "he" shard should have 2 entries
        let he_shard = result
            .shards
            .get("shards/he.json")
            .expect("he shard should exist");
        assert_eq!(he_shard.len(), 2);

        // "world" appears in two entries
        let wo_shard = result
            .shards
            .get("shards/wo.json")
            .expect("wo shard should exist");
        assert_eq!(wo_shard.len(), 2);

        // "there" and "peace" appear in one entry each
        let th_shard = result
            .shards
            .get("shards/th.json")
            .expect("th shard should exist");
        assert_eq!(th_shard.len(), 1);

        let pe_shard = result
            .shards
            .get("shards/pe.json")
            .expect("pe shard should exist");
        assert_eq!(pe_shard.len(), 1);
    }

    #[test]
    fn test_empty_entries() {
        let result = shard_entries(vec![]);
        assert!(result.shards.is_empty());
        assert!(result.shard_infos.is_empty());
    }
}
