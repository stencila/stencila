//! Search index generation
//!
//! Generates search index from rendered site documents.

use std::path::Path;

use eyre::Result;

use stencila_codec::stencila_schema::Node;
use stencila_config::{AccessLevel, NavItem, SearchConfig, SiteAccessConfig};

use super::breadcrumbs::{build_breadcrumbs_map, get_breadcrumbs};
use super::extract::extract_entries_with_config;
use super::index::{SearchIndexBuilder, SearchIndexStats};
use crate::auto_index::{
    find_nav_group_children, generate_auto_index_article, get_child_pages_from_nav,
};
use crate::{RouteType, list};

/// Normalize a route for access config lookup.
///
/// Ensures routes that look like directories (no file extension) end with "/".
/// Uses a heuristic to distinguish files from directories with dots in their names:
/// - If extension is purely numeric (e.g., "2" in "/docs/v1.2"), treat as directory
/// - Otherwise treat as file (e.g., "/data/export.zip", "/data/file.parquet")
///
/// This works because real file extensions are almost never purely numeric,
/// while version numbers in directory names typically are (v1.2, 2.0.0, etc.).
fn normalize_route_for_access(route: &str) -> std::borrow::Cow<'_, str> {
    if route.ends_with('/') {
        return std::borrow::Cow::Borrowed(route);
    }

    // Check if route has an extension in the last segment
    if let Some(last_segment) = route.rsplit('/').next()
        && let Some(ext_pos) = last_segment.rfind('.')
    {
        let ext = &last_segment[ext_pos + 1..];
        // If extension is non-empty and NOT purely numeric, treat as file
        // Purely numeric extensions (like "2" in "v1.2") indicate version directories
        if !ext.is_empty() && !ext.chars().all(|c| c.is_ascii_digit()) {
            return std::borrow::Cow::Borrowed(route);
        }
    }

    // No extension or numeric extension - treat as directory
    std::borrow::Cow::Owned(format!("{route}/"))
}

/// Get access level for a route
fn get_route_access_level(route: &str, access_config: Option<&SiteAccessConfig>) -> AccessLevel {
    match access_config {
        Some(config) => {
            let normalized = normalize_route_for_access(route);
            config.get_access_level(&normalized)
        }
        None => AccessLevel::Public,
    }
}

/// Generate search index for a site
///
/// This function walks through site routes, decodes documents, and generates
/// a search index. It should be called after rendering is complete.
///
/// When access_config is provided, entries are tagged with their access level
/// and the index is sharded by access level (separate directories per level).
///
/// # Arguments
/// * `output_dir` - The output directory where the site was rendered
/// * `config` - Search configuration
/// * `nav_items` - Navigation items for breadcrumb resolution
/// * `access_config` - Optional access configuration for access-level sharding
/// * `decode_fn` - Function to decode a document path into a Node
pub async fn generate_search_index<F, Fut>(
    output_dir: &Path,
    config: &SearchConfig,
    nav_items: &[NavItem],
    access_config: Option<&SiteAccessConfig>,
    decode_fn: F,
) -> Result<SearchIndexStats>
where
    F: Fn(std::path::PathBuf) -> Fut + Send + Sync,
    Fut: std::future::Future<Output = Result<Node>> + Send,
{
    let mut builder = SearchIndexBuilder::new().with_fuzzy(config.is_fuzzy_enabled());

    // Build breadcrumbs map from nav items
    let breadcrumbs_map = build_breadcrumbs_map(nav_items);

    // List all routes
    let routes = list(true, false, None, None, None).await?;

    // Process each route
    for entry in routes {
        // Check if route should be excluded
        if config.is_route_excluded(&entry.route) {
            continue;
        }

        // Get the node for this route (different handling for auto-index vs document routes)
        let node = match entry.route_type {
            RouteType::File | RouteType::Implied | RouteType::Spread => {
                // Document route - decode from source file
                let Some(source_path) = entry.source_path else {
                    continue;
                };
                match decode_fn(source_path.clone()).await {
                    Ok(node) => node,
                    Err(e) => {
                        tracing::warn!("Failed to decode {}: {}", source_path.display(), e);
                        continue;
                    }
                }
            }
            RouteType::AutoIndex => {
                // Auto-index route - generate Article node from nav children
                let children =
                    if let Some(child_items) = find_nav_group_children(nav_items, &entry.route) {
                        get_child_pages_from_nav(child_items)
                    } else {
                        Vec::new()
                    };
                generate_auto_index_article(&entry.route, children)
            }
            RouteType::Static | RouteType::Redirect => {
                // Static files and redirects are not indexed
                continue;
            }
        };

        // Get breadcrumbs for this route
        let breadcrumbs = get_breadcrumbs(&entry.route, &breadcrumbs_map);

        // Get access level for this route
        let access_level = get_route_access_level(&entry.route, access_config);

        // Extract entries with config and tag with access level
        let entries = extract_entries_with_config(&node, &entry.route, breadcrumbs, config)
            .into_iter()
            .map(|e| e.with_access_level(access_level))
            .collect::<Vec<_>>();
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
/// * `nav_items` - Navigation items for breadcrumb resolution
/// * `access_config` - Optional access configuration for access-level sharding
/// * `nodes` - Iterator of (route, node) pairs
pub async fn generate_search_index_from_nodes<'a, I>(
    output_dir: &Path,
    config: &SearchConfig,
    nav_items: &[NavItem],
    access_config: Option<&SiteAccessConfig>,
    nodes: I,
) -> Result<SearchIndexStats>
where
    I: IntoIterator<Item = (&'a str, &'a Node)>,
{
    let mut builder = SearchIndexBuilder::new().with_fuzzy(config.is_fuzzy_enabled());

    // Build breadcrumbs map from nav items
    let breadcrumbs_map = build_breadcrumbs_map(nav_items);

    for (route, node) in nodes {
        // Check if route should be excluded
        if config.is_route_excluded(route) {
            continue;
        }

        // Get breadcrumbs for this route
        let breadcrumbs = get_breadcrumbs(route, &breadcrumbs_map);

        // Get access level for this route
        let access_level = get_route_access_level(route, access_config);

        // Extract entries with config and tag with access level
        let entries = extract_entries_with_config(node, route, breadcrumbs, config)
            .into_iter()
            .map(|e| e.with_access_level(access_level))
            .collect::<Vec<_>>();
        builder.add_entries(entries);
    }

    builder.write(output_dir).await
}
