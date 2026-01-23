//! Search index generation
//!
//! Generates search index from rendered site documents.

use std::path::Path;

use eyre::Result;

use stencila_codec::stencila_schema::Node;
use stencila_config::SearchConfig;

use super::extract::extract_entries_with_config;
use super::index::{SearchIndexBuilder, SearchIndexStats};
use crate::{RouteType, list};

/// Generate search index for a site
///
/// This function walks through site routes, decodes documents, and generates
/// a search index. It should be called after rendering is complete.
///
/// # Arguments
/// * `output_dir` - The output directory where the site was rendered
/// * `config` - Search configuration
/// * `decode_fn` - Function to decode a document path into a Node
pub async fn generate_search_index<F, Fut>(
    output_dir: &Path,
    config: &SearchConfig,
    decode_fn: F,
) -> Result<SearchIndexStats>
where
    F: Fn(std::path::PathBuf) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<Node>> + Send,
{
    let mut builder = SearchIndexBuilder::new().with_fuzzy(config.is_fuzzy_enabled());

    // List all routes
    let routes = list(true, false, None, None, None).await?;

    // Process each document route
    for entry in routes {
        if !matches!(
            entry.route_type,
            RouteType::File | RouteType::Implied | RouteType::Spread
        ) {
            continue;
        }

        // Check if route should be excluded
        if config.is_route_excluded(&entry.route) {
            continue;
        }

        let Some(source_path) = entry.source_path else {
            continue;
        };

        // Decode the document
        let node = match decode_fn(source_path.clone()).await {
            Ok(node) => node,
            Err(e) => {
                tracing::warn!("Failed to decode {}: {}", source_path.display(), e);
                continue;
            }
        };

        // Extract entries with config
        let entries = extract_entries_with_config(&node, &entry.route, config);
        builder.add_entries(entries);
    }

    // Write the index
    builder.write(output_dir).await
}

/// Generate search index from pre-decoded nodes
///
/// This is more efficient when nodes are already available (e.g., during rendering).
///
/// # Arguments
/// * `output_dir` - The output directory for the search index
/// * `config` - Search configuration
/// * `nodes` - Iterator of (route, node) pairs
pub async fn generate_search_index_from_nodes<'a, I>(
    output_dir: &Path,
    config: &SearchConfig,
    nodes: I,
) -> Result<SearchIndexStats>
where
    I: IntoIterator<Item = (&'a str, &'a Node)>,
{
    let mut builder = SearchIndexBuilder::new().with_fuzzy(config.is_fuzzy_enabled());

    for (route, node) in nodes {
        // Check if route should be excluded
        if config.is_route_excluded(route) {
            continue;
        }

        let entries = extract_entries_with_config(node, route, config);
        builder.add_entries(entries);
    }

    builder.write(output_dir).await
}
