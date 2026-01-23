//! Sharding logic for search indices
//!
//! Entries are sharded by the 2-character prefix of their tokens.
//! This allows the client to load only the shards needed for a query.

use std::collections::{BTreeMap, HashMap, HashSet};

use serde::{Deserialize, Serialize};

use super::entry::{SearchEntry, TokenDef};
use super::manifest::ShardInfo;
use super::tokenize::{token_prefix, tokenize};

/// Shard file structure with deduplicated tokens
///
/// Contains token definitions (deduplicated) and entries with compact token references.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShardData {
    /// Deduplicated token definitions for fuzzy matching
    pub token_defs: Vec<TokenDef>,

    /// Search entries with compact token references
    pub entries: Vec<SearchEntry>,
}

/// Result of sharding: shards and their info
pub struct ShardResult {
    /// The shards, keyed by prefix (e.g., "ab" -> ShardData)
    pub shards: BTreeMap<String, ShardData>,

    /// Shard info for the manifest, keyed by prefix
    pub shard_infos: BTreeMap<String, ShardInfo>,
}

/// Shard entries by token prefix
///
/// Each entry is placed in shards for all unique prefixes of its tokens.
/// This means an entry may appear in multiple shards if its text contains
/// tokens with different prefixes.
///
/// Token definitions are deduplicated at the shard level to reduce file size.
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

    // Convert to shards (BTreeMap gives sorted, deterministic output)
    let mut shards = BTreeMap::new();
    let mut shard_infos = BTreeMap::new();

    for (prefix, mut entries) in prefix_to_entries {
        let entry_count = entries.len();

        if entry_count == 0 {
            continue;
        }

        // Deduplicate tokens and convert to compact format
        let token_defs = deduplicate_tokens(&mut entries);

        let shard_info = ShardInfo::new(entry_count);
        let shard_data = ShardData {
            token_defs,
            entries,
        };

        shards.insert(prefix.clone(), shard_data);
        shard_infos.insert(prefix, shard_info);
    }

    ShardResult {
        shards,
        shard_infos,
    }
}

/// Deduplicate token definitions across entries in a shard
///
/// Converts each entry's `token_trigrams` to compact `tokens` references,
/// returning the deduplicated token definitions.
///
/// Uses token string as the deduplication key since trigrams are deterministically
/// derived from the token (same token always produces same trigrams).
fn deduplicate_tokens(entries: &mut [SearchEntry]) -> Vec<TokenDef> {
    // Key by token string only - avoids hashing/cloning Vec<String> trigrams
    let mut token_to_index: HashMap<String, u32> = HashMap::new();
    let mut token_defs: Vec<TokenDef> = Vec::new();

    for entry in entries.iter_mut() {
        if let Some(token_trigrams) = entry.token_trigrams.take() {
            let mut refs = Vec::with_capacity(token_trigrams.len());

            for tt in token_trigrams {
                let idx = *token_to_index.entry(tt.token.clone()).or_insert_with(|| {
                    let idx = token_defs.len() as u32;
                    token_defs.push(TokenDef::new(tt.token, tt.trigrams));
                    idx
                });

                refs.push([idx, tt.start, tt.end]);
            }

            entry.tokens = Some(refs);
        }
    }

    token_defs
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::search::entry::TokenTrigrams;

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
        assert!(result.shards.contains_key("he"));
        assert!(result.shards.contains_key("wo"));
        assert!(result.shards.contains_key("th"));
        assert!(result.shards.contains_key("pe"));

        // "hello" appears in two entries, so "he" shard should have 2 entries
        let he_shard = result.shards.get("he").expect("he shard should exist");
        assert_eq!(he_shard.entries.len(), 2);

        // "world" appears in two entries
        let wo_shard = result.shards.get("wo").expect("wo shard should exist");
        assert_eq!(wo_shard.entries.len(), 2);

        // "there" and "peace" appear in one entry each
        let th_shard = result.shards.get("th").expect("th shard should exist");
        assert_eq!(th_shard.entries.len(), 1);

        let pe_shard = result.shards.get("pe").expect("pe shard should exist");
        assert_eq!(pe_shard.entries.len(), 1);
    }

    #[test]
    fn test_empty_entries() {
        let result = shard_entries(vec![]);
        assert!(result.shards.is_empty());
        assert!(result.shard_infos.is_empty());
    }

    #[test]
    fn test_token_deduplication() {
        // Create entries with token_trigrams to test deduplication
        let mut entry1 = SearchEntry::new(
            "hed_1".to_string(),
            "Heading",
            "/test/".to_string(),
            "Hello World".to_string(),
            8,
            1,
        );
        entry1.token_trigrams = Some(vec![
            TokenTrigrams::new(
                "hello".to_string(),
                vec!["hel".to_string(), "ell".to_string(), "llo".to_string()],
                0,
                5,
            ),
            TokenTrigrams::new(
                "world".to_string(),
                vec!["wor".to_string(), "orl".to_string(), "rld".to_string()],
                6,
                11,
            ),
        ]);

        let mut entry2 = SearchEntry::new(
            "pgh_1".to_string(),
            "Paragraph",
            "/test/".to_string(),
            "Hello there".to_string(),
            2,
            2,
        );
        entry2.token_trigrams = Some(vec![
            TokenTrigrams::new(
                "hello".to_string(),
                vec!["hel".to_string(), "ell".to_string(), "llo".to_string()],
                0,
                5,
            ),
            TokenTrigrams::new(
                "there".to_string(),
                vec!["the".to_string(), "her".to_string(), "ere".to_string()],
                6,
                11,
            ),
        ]);

        let result = shard_entries(vec![entry1, entry2]);

        // Check the "he" shard (contains entries with "hello")
        let he_shard = result.shards.get("he").expect("he shard should exist");

        // Should have deduplicated "hello" - only one TokenDef for it
        let hello_count = he_shard
            .token_defs
            .iter()
            .filter(|td| td.token == "hello")
            .count();
        assert_eq!(hello_count, 1, "hello should be deduplicated");

        // Both entries should have tokens referencing the same index for "hello"
        assert_eq!(he_shard.entries.len(), 2);
        for entry in &he_shard.entries {
            assert!(entry.tokens.is_some(), "entries should have compact tokens");
            assert!(
                entry.token_trigrams.is_none(),
                "token_trigrams should be cleared"
            );
        }

        // Verify the compact format: tokens should reference token_defs indices
        let entry1_tokens = he_shard.entries[0]
            .tokens
            .as_ref()
            .expect("entry1 should have tokens");
        let entry2_tokens = he_shard.entries[1]
            .tokens
            .as_ref()
            .expect("entry2 should have tokens");

        // Find the "hello" token def index
        let hello_idx = he_shard
            .token_defs
            .iter()
            .position(|td| td.token == "hello")
            .expect("hello should exist in token_defs") as u32;

        // Both entries should reference the same index for "hello"
        assert!(entry1_tokens.iter().any(|t| t[0] == hello_idx));
        assert!(entry2_tokens.iter().any(|t| t[0] == hello_idx));
    }
}
