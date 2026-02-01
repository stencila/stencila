//! Auto-generated index pages for directories without content
//!
//! This module generates Article nodes for directories that appear in navigation
//! but lack content files (main.md, README.md, index.md, etc.). The generated
//! pages display a listing of child pages with links.

use std::collections::HashSet;

use stencila_codec::stencila_schema::{
    Article, Block, Cord, Inline, Link, List, ListItem, ListOrder, Node, Paragraph, Text,
};

use stencila_config::{AutoIndexConfig, NavItem};

use crate::nav_common::segment_to_label;
use crate::{RouteEntry, RouteType};

/// Normalize a route for consistent comparison
///
/// Ensures routes have a trailing slash (except root "/").
/// This is public so it can be used when building existing_routes sets.
pub fn normalize_route(route: &str) -> std::borrow::Cow<'_, str> {
    if route == "/" || route.ends_with('/') {
        std::borrow::Cow::Borrowed(route)
    } else {
        std::borrow::Cow::Owned(format!("{route}/"))
    }
}

/// Check if existing_routes contains a route (with normalization)
fn existing_routes_contains(existing_routes: &HashSet<String>, route: &str) -> bool {
    let normalized = normalize_route(route);
    existing_routes.contains(normalized.as_ref())
}

/// Information about a child page for auto-index generation
#[derive(Debug, Clone)]
pub struct ChildPage {
    /// The route to the child page
    pub route: String,
    /// Display label for the child page
    pub label: String,
}

/// Generate an Article node for an auto-index page
///
/// Creates a simple article with:
/// - A heading derived from the route segment
/// - An unordered list of links to child pages
///
/// # Arguments
/// * `route` - The route for this auto-index page
/// * `children` - Information about child pages to list
pub fn generate_auto_index_article(route: &str, children: Vec<ChildPage>) -> Node {
    let mut content: Vec<Block> = Vec::new();

    // Generate title from route segment
    let title = route_to_title(route);

    // Generate list of child page links
    if !children.is_empty() {
        let list_items: Vec<ListItem> = children
            .into_iter()
            .map(|child| {
                let link = Link::new(
                    vec![Inline::Text(Text {
                        value: Cord::from(child.label),
                        ..Default::default()
                    })],
                    child.route,
                );

                let para = Paragraph {
                    content: vec![Inline::Link(link)],
                    ..Default::default()
                };

                ListItem {
                    content: vec![Block::Paragraph(para)],
                    ..Default::default()
                }
            })
            .collect();

        let list = List {
            items: list_items,
            order: ListOrder::Unordered,
            ..Default::default()
        };
        content.push(Block::List(list));
    }

    // Create article with title in metadata
    let article = Article {
        title: Some(vec![Inline::Text(Text {
            value: Cord::from(title),
            ..Default::default()
        })]),
        content,
        ..Default::default()
    };

    Node::Article(article)
}

/// Convert a route to a human-readable title
///
/// Extracts the last segment of the route and converts it from kebab-case
/// to Title Case using `segment_to_label` from `nav_common`.
fn route_to_title(route: &str) -> String {
    let trimmed = route.trim_matches('/');
    if trimmed.is_empty() {
        return "Home".to_string();
    }

    let segment = trimmed.rsplit('/').next().unwrap_or(trimmed);
    segment_to_label(segment)
}

/// Extract child pages from navigation items
///
/// Gets immediate children of a nav group for display on an auto-index page.
pub fn get_child_pages_from_nav(nav_items: &[NavItem]) -> Vec<ChildPage> {
    nav_items
        .iter()
        .filter_map(|item| match item {
            NavItem::Route(route) => Some(ChildPage {
                route: route.clone(),
                label: route_to_label(route),
            }),
            NavItem::Link { route, label, .. } => Some(ChildPage {
                route: route.clone(),
                label: label.clone(),
            }),
            NavItem::Group {
                route: Some(r),
                label,
                ..
            } => Some(ChildPage {
                route: r.clone(),
                label: label.clone(),
            }),
            NavItem::Group { route: None, .. } => {
                // Groups without routes don't get listed as children
                None
            }
        })
        .collect()
}

/// Convert a route to a display label
///
/// Extracts the last segment and converts to title case.
fn route_to_label(route: &str) -> String {
    route_to_title(route)
}

/// Find a nav group matching a specific route in the navigation tree
///
/// Recursively searches the navigation tree for a group whose derived route
/// matches the given route.
pub fn find_nav_group_children<'a>(
    nav_items: &'a [NavItem],
    target_route: &str,
) -> Option<&'a [NavItem]> {
    for item in nav_items {
        if let NavItem::Group {
            label,
            route,
            children,
            ..
        } = item
        {
            // Check if this group's route matches (or derived route matches)
            // Normalize routes for comparison to handle /docs vs /docs/
            let normalized_target = normalize_route(target_route);
            if let Some(group_route) = route {
                if normalize_route(group_route) == normalized_target {
                    return Some(children);
                }
            } else if !children.is_empty() {
                // Try label-based derivation first (for logical groups like "Markup")
                // then common-parent derivation (for directory-based groups)
                let derived = derive_label_based_route(label, children)
                    .or_else(|| derive_group_route(children));
                if let Some(derived_route) = derived
                    && derived_route == normalized_target.as_ref()
                {
                    return Some(children);
                }
            }

            // Recurse into children
            if let Some(found) = find_nav_group_children(children, target_route) {
                return Some(found);
            }
        }
    }
    None
}

/// Derive a route for a group from its children's common prefix
///
/// For example, if children have routes `/docs/getting-started/` and `/docs/config/`,
/// the derived route is `/docs/`.
///
/// Returns None if children don't share a common parent path.
pub fn derive_group_route(children: &[NavItem]) -> Option<String> {
    get_children_common_parent(children)
}

/// Get the common parent path of all children
///
/// Returns None if children don't all share the same parent path.
fn get_children_common_parent(children: &[NavItem]) -> Option<String> {
    let child_routes: Vec<String> = children
        .iter()
        .filter_map(|item| match item {
            NavItem::Route(route) => Some(route.clone()),
            NavItem::Link { route, .. } => Some(route.clone()),
            NavItem::Group { route, .. } => route.clone(),
        })
        .collect();

    if child_routes.is_empty() {
        return None;
    }

    // Get parent path of first child
    let first_route = &child_routes[0];
    let trimmed = first_route.trim_matches('/');
    let segments: Vec<&str> = trimmed.split('/').collect();

    if segments.len() <= 1 {
        return None;
    }

    let parent_path = segments[..segments.len() - 1].join("/");
    let expected_parent = format!("/{}/", parent_path);

    // Verify all children share this parent path
    for route in &child_routes[1..] {
        let route_trimmed = route.trim_start_matches('/');
        if !route_trimmed.starts_with(&format!("{}/", parent_path)) {
            // Children don't share common parent - can't derive route
            return None;
        }
    }

    Some(expected_parent)
}

/// Derive a label-based route for a logical group
///
/// For groups defined in _nav.yaml that logically group items (like "Markup"
/// grouping DOM, HTML, JATS formats), derives a route from the label and
/// children's common parent path.
///
/// For example: group "Markup" with children `/docs/formats/dom/`, `/docs/formats/html/`
/// derives route `/docs/formats/markup/`.
///
/// Returns None if:
/// - Children don't share a common parent
/// - The label slug matches the last segment of the parent path (e.g., "Docs" group
///   with children under `/docs/` - use `derive_group_route` instead to get `/docs/`)
pub fn derive_label_based_route(label: &str, children: &[NavItem]) -> Option<String> {
    let parent_path = get_children_common_parent(children)?;
    let slug = crate::nav_common::label_to_segment(label);

    // Check if slug matches the last segment of parent_path
    // For directory-based groups like "Docs" with children under /docs/,
    // the slug "docs" matches the last segment, so we should return None
    // and let derive_group_route handle it (returning /docs/ not /docs/docs/)
    let parent_trimmed = parent_path.trim_matches('/');
    if let Some(last_segment) = parent_trimmed.rsplit('/').next()
        && last_segment == slug
    {
        return None;
    }

    Some(format!("{}{}/", parent_path, slug))
}

// =============================================================================
// Auto-index route identification
// =============================================================================

/// Collect auto-index routes and their route set
///
/// This is a convenience wrapper that returns both the list of auto-index route entries
/// and a HashSet of the route strings for efficient lookups.
///
/// # Arguments
/// * `existing_routes` - Set of routes that already exist (have content)
/// * `nav_items` - Navigation tree built from routes
/// * `config` - Auto-index configuration
///
/// # Returns
/// Tuple of (auto_index_routes, auto_index_route_set)
#[allow(dead_code)]
fn collect_auto_index_routes(
    existing_routes: &HashSet<String>,
    nav_items: &[NavItem],
    config: &AutoIndexConfig,
) -> (Vec<RouteEntry>, HashSet<String>) {
    if !config.is_enabled() {
        return (Vec::new(), HashSet::new());
    }

    let routes = identify_auto_index_routes(existing_routes, nav_items, config);
    let route_set: HashSet<String> = routes.iter().map(|e| e.route.clone()).collect();
    (routes, route_set)
}

/// Identify routes that need auto-generated index pages
///
/// These are routes that:
/// 1. Exist as nav group paths (directories with children)
/// 2. Don't have any corresponding content file
/// 3. Are not excluded by config patterns
///
/// # Arguments
/// * `existing_routes` - Set of routes that already exist (have content)
/// * `nav_items` - Navigation tree built from routes
/// * `config` - Auto-index configuration
///
/// # Returns
/// Vector of route entries for auto-index pages
pub fn identify_auto_index_routes(
    existing_routes: &HashSet<String>,
    nav_items: &[NavItem],
    config: &AutoIndexConfig,
) -> Vec<RouteEntry> {
    let mut auto_routes = Vec::new();
    collect_auto_index_routes_recursive(nav_items, existing_routes, config, &mut auto_routes);
    auto_routes
}

/// Recursively collect auto-index routes from navigation tree
fn collect_auto_index_routes_recursive(
    items: &[NavItem],
    existing_routes: &HashSet<String>,
    config: &AutoIndexConfig,
    result: &mut Vec<RouteEntry>,
) {
    for item in items {
        match item {
            NavItem::Group {
                label,
                route,
                children,
                ..
            } => {
                // Determine the route for this group
                let group_route = if let Some(explicit_route) = route {
                    // Group has explicit route - use it if no content file exists
                    // Use normalized comparison for consistency (/docs vs /docs/)
                    if existing_routes_contains(existing_routes, explicit_route) {
                        None // Content file exists, no auto-index needed
                    } else {
                        // Normalize the explicit route for storage
                        Some(normalize_route(explicit_route).into_owned())
                    }
                } else if !children.is_empty() {
                    // No explicit route - try to derive one
                    // First try label-based route (for logical groups from _nav.yaml)
                    // Then fall back to children's common parent (for directory-based groups)
                    // These functions already return normalized routes with trailing slashes
                    derive_label_based_route(label, children)
                        .or_else(|| derive_group_route(children))
                } else {
                    None
                };

                // Add auto-index entry if we have a route that doesn't exist and isn't excluded
                if let Some(target_route) = group_route
                    && !existing_routes_contains(existing_routes, &target_route)
                    && !config.is_route_excluded(&target_route)
                {
                    result.push(RouteEntry {
                        route: target_route,
                        route_type: RouteType::AutoIndex,
                        target: format!("[auto-index: {}]", label),
                        source_path: None,
                        spread_count: None,
                        spread_arguments: None,
                    });
                }

                // Recurse into children
                collect_auto_index_routes_recursive(children, existing_routes, config, result);
            }
            NavItem::Route(_) | NavItem::Link { .. } => {
                // Simple routes don't have children, nothing to do
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_to_title() {
        assert_eq!(route_to_title("/"), "Home");
        assert_eq!(route_to_title("/docs/"), "Docs");
        assert_eq!(route_to_title("/getting-started/"), "Getting Started");
        assert_eq!(route_to_title("/docs/api-reference/"), "Api Reference");
        assert_eq!(route_to_title("/my_section/"), "My Section");
    }

    #[test]
    fn test_generate_auto_index_article() {
        let children = vec![
            ChildPage {
                route: "/docs/intro/".to_string(),
                label: "Introduction".to_string(),
            },
            ChildPage {
                route: "/docs/guide/".to_string(),
                label: "User Guide".to_string(),
            },
        ];

        let node = generate_auto_index_article("/docs/", children);

        // Verify it's an article
        if let Node::Article(article) = node {
            // Check title is in metadata
            assert!(article.title.is_some());

            // Check content has only the list (title is in metadata, not as heading)
            assert_eq!(article.content.len(), 1);
            assert!(matches!(article.content[0], Block::List(_)));

            // Check list has 2 items
            if let Block::List(list) = &article.content[0] {
                assert_eq!(list.items.len(), 2);
            }
        } else {
            panic!("Expected Article node");
        }
    }

    #[test]
    fn test_generate_auto_index_article_empty_children() {
        let node = generate_auto_index_article("/empty/", vec![]);

        if let Node::Article(article) = node {
            // Title should be in metadata
            assert!(article.title.is_some());
            // Content should be empty (no list when no children)
            assert!(article.content.is_empty());
        } else {
            panic!("Expected Article node");
        }
    }

    #[test]
    fn test_derive_label_based_route() {
        // Logical group: "Markup" with children under /docs/formats/
        // Should produce /docs/formats/markup/
        let children = vec![
            NavItem::Route("/docs/formats/dom/".to_string()),
            NavItem::Route("/docs/formats/html/".to_string()),
        ];
        assert_eq!(
            derive_label_based_route("Markup", &children),
            Some("/docs/formats/markup/".to_string())
        );

        // Directory group: "Docs" with children under /docs/
        // Should return None because slug "docs" matches last segment of parent "/docs/"
        let children = vec![
            NavItem::Route("/docs/intro/".to_string()),
            NavItem::Route("/docs/guide/".to_string()),
        ];
        assert_eq!(derive_label_based_route("Docs", &children), None);

        // Directory group with different casing: "DOCS" should still match "docs"
        // because label_to_segment lowercases
        assert_eq!(derive_label_based_route("DOCS", &children), None);

        // Empty children should return None
        assert_eq!(derive_label_based_route("Empty", &[]), None);
    }
}
