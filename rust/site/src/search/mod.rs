//! Search index generation for static sites
//!
//! This module generates search indices during site rendering that can be
//! queried client-side without any backend services.
//!
//! The search system is schema-aware, indexing at the Stencila node level
//! (headings, paragraphs, datatables) rather than at the page level.

mod entry;
mod extract;
mod generate;
mod index;
mod manifest;
mod shard;
mod tokenize;

pub use entry::{DatatableMetadata, SearchEntry, weights};
pub use extract::{extract_entries, extract_entries_with_config};
pub use generate::{generate_search_index, generate_search_index_from_nodes};
pub use index::{SearchIndexBuilder, SearchIndexStats};
pub use manifest::{SCHEMA_VERSION, SearchManifest, ShardInfo};
pub use tokenize::{token_prefix, tokenize};
