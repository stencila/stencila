//! Navigation tree generation for Stencila Sites
//!
//! Builds hierarchical navigation structures from routes and configuration.

use std::collections::HashMap;

use stencila_config::{LeftSidebarConfig, NavConfig, NavItem};

use crate::list::{RouteEntry, RouteType};

// Re-export NavTreeItem from codec-dom
pub use stencila_codec_dom::NavTreeItem;

/// Build a navigation tree from routes
///
/// # Arguments
/// * `routes` - All discovered routes for the site
/// * `current_route` - The route being rendered (for active state)
/// * `config` - Sidebar configuration
/// * `named_navs` - Named nav configs from `site.layout.navs`
pub fn build_nav_tree(
    routes: &[RouteEntry],
    current_route: &str,
    config: &LeftSidebarConfig,
    named_navs: &HashMap<String, NavConfig>,
) -> Vec<NavTreeItem> {
    let nav_source = config.nav.as_deref().unwrap_or("auto");
    let max_depth = config.depth.unwrap_or(5) as usize;
    let expanded_depth = config.expanded;

    let mut tree = if nav_source == "auto" {
        build_auto_nav(routes, max_depth)
    } else if let Some(named) = named_navs.get(nav_source) {
        build_config_nav(&named.items, routes)
    } else {
        // Fallback to auto if named nav not found
        build_auto_nav(routes, max_depth)
    };

    // Expand items based on configured depth (None = expand all)
    expand_to_depth(&mut tree, expanded_depth, 1);

    // Mark active items and expand their ancestors
    mark_active_items(&mut tree, current_route);

    tree
}

/// Build navigation tree from file structure
fn build_auto_nav(routes: &[RouteEntry], max_depth: usize) -> Vec<NavTreeItem> {
    // Filter to document routes only (not static, redirect, etc.)
    let doc_routes: Vec<&RouteEntry> = routes
        .iter()
        .filter(|r| {
            matches!(
                r.route_type,
                RouteType::File | RouteType::Implied | RouteType::Spread
            )
        })
        .collect();

    // Build tree structure from routes
    let mut root_items: Vec<NavTreeItem> = Vec::new();

    // Group routes by their path segments
    for entry in doc_routes {
        let route = entry.route.trim_matches('/');
        if route.is_empty() {
            // Root index page - add as first item
            root_items.insert(
                0,
                NavTreeItem {
                    label: "Home".to_string(),
                    href: Some("/".to_string()),
                    icon: None,
                    active: false,
                    expanded: false,
                    children: None,
                },
            );
            continue;
        }

        let segments: Vec<&str> = route.split('/').collect();
        if segments.len() > max_depth {
            continue;
        }

        insert_route_into_tree(&mut root_items, &segments, &entry.route);
    }

    // Sort items alphabetically at each level
    sort_nav_items(&mut root_items);

    root_items
}

/// Insert a route into the navigation tree at the appropriate location
fn insert_route_into_tree(items: &mut Vec<NavTreeItem>, segments: &[&str], full_route: &str) {
    if segments.is_empty() {
        return;
    }

    let first = segments[0];
    let rest = &segments[1..];

    // Look for existing item with this segment
    let existing_idx = items.iter().position(|item| {
        // Match by label (titlecased segment)
        item.label == label_from_segment(first)
    });

    if let Some(idx) = existing_idx {
        if rest.is_empty() {
            // This is the final segment - update href if not set
            if items[idx].href.is_none() {
                items[idx].href = Some(full_route.to_string());
            }
        } else {
            // Continue down the tree
            let children = items[idx].children.get_or_insert_with(Vec::new);
            insert_route_into_tree(children, rest, full_route);
        }
    } else {
        // Create new item
        if rest.is_empty() {
            // Leaf node
            items.push(NavTreeItem {
                label: label_from_segment(first),
                href: Some(full_route.to_string()),
                icon: None,
                active: false,
                expanded: false,
                children: None,
            });
        } else {
            // Group node - create with children
            let mut children = Vec::new();
            insert_route_into_tree(&mut children, rest, full_route);

            items.push(NavTreeItem {
                label: label_from_segment(first),
                href: None, // Group header, no link
                icon: None,
                active: false,
                expanded: false,
                children: Some(children),
            });
        }
    }
}

/// Sort navigation items recursively
fn sort_nav_items(items: &mut [NavTreeItem]) {
    items.sort_by(|a, b| {
        // Keep "Home" at the top
        if a.label == "Home" {
            return std::cmp::Ordering::Less;
        }
        if b.label == "Home" {
            return std::cmp::Ordering::Greater;
        }
        a.label.to_lowercase().cmp(&b.label.to_lowercase())
    });

    for item in items.iter_mut() {
        if let Some(ref mut children) = item.children {
            sort_nav_items(children);
        }
    }
}

/// Build navigation tree from explicit configuration
fn build_config_nav(items: &[NavItem], routes: &[RouteEntry]) -> Vec<NavTreeItem> {
    items
        .iter()
        .map(|item| nav_item_to_tree(item, routes))
        .collect()
}

/// Convert a NavItem to a NavTreeItem
fn nav_item_to_tree(item: &NavItem, routes: &[RouteEntry]) -> NavTreeItem {
    match item {
        NavItem::Route(route) => {
            // Look up label from routes or derive from path
            let label = routes
                .iter()
                .find(|r| &r.route == route)
                .map(|_| label_from_route(route))
                .unwrap_or_else(|| label_from_route(route));

            NavTreeItem {
                label,
                href: Some(route.clone()),
                icon: None,
                active: false,
                expanded: false,
                children: None,
            }
        }
        NavItem::Link {
            label,
            target: href,
            icon,
        } => NavTreeItem {
            label: label.clone(),
            href: Some(href.clone()),
            icon: icon.clone(),
            active: false,
            expanded: false,
            children: None,
        },
        NavItem::Group { group, children } => {
            let tree_children: Vec<NavTreeItem> = children
                .iter()
                .map(|c| nav_item_to_tree(c, routes))
                .collect();

            NavTreeItem {
                label: group.clone(),
                href: None,
                icon: None,
                active: false,
                expanded: false,
                children: Some(tree_children),
            }
        }
    }
}

/// Expand items up to a specified depth
///
/// - `None` = expand all items
/// - `Some(0)` = expand none (all collapsed)
/// - `Some(1)` = expand top-level only
/// - etc.
fn expand_to_depth(items: &mut [NavTreeItem], max_depth: Option<u8>, current_depth: u8) {
    for item in items.iter_mut() {
        if let Some(ref mut children) = item.children {
            // Expand if no max_depth specified, or current depth is within limit
            item.expanded = max_depth.is_none() || current_depth <= max_depth.unwrap_or(0);

            // Recursively process children
            expand_to_depth(children, max_depth, current_depth + 1);
        }
    }
}

/// Mark active items and expand their ancestors
fn mark_active_items(items: &mut [NavTreeItem], current_route: &str) -> bool {
    let mut has_active = false;

    for item in items.iter_mut() {
        // Check if this item is active
        if let Some(ref href) = item.href
            && href == current_route
        {
            item.active = true;
            has_active = true;
        }

        // Recursively check children
        if let Some(ref mut children) = item.children {
            let child_has_active = mark_active_items(children, current_route);
            if child_has_active {
                item.expanded = true;
                has_active = true;
            }
        }
    }

    has_active
}

/// Convert a route path to a display label
///
/// Examples:
/// - "/getting-started/" -> "Getting Started"
/// - "/api/v2/" -> "Api V2"
/// - "/docs/intro/" -> "Intro"
pub fn label_from_route(route: &str) -> String {
    let route = route.trim_matches('/');

    // Get the last segment
    let segment = route.split('/').next_back().unwrap_or(route);

    label_from_segment(segment)
}

/// Convert a URL segment to a display label
///
/// Handles:
/// - Hyphen separation: "getting-started" -> "Getting Started"
/// - Underscore separation: "getting_started" -> "Getting Started"
/// - Index pages: "index" -> parent folder name would be used
fn label_from_segment(segment: &str) -> String {
    // Handle index-like names
    if segment == "index" || segment == "main" || segment == "readme" {
        return "Overview".to_string();
    }

    // Split on hyphens and underscores, titlecase each word
    segment
        .split(['-', '_'])
        .filter(|s| !s.is_empty())
        .map(titlecase_word)
        .collect::<Vec<_>>()
        .join(" ")
}

/// Titlecase a single word
fn titlecase_word(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use stencila_config::{LeftSidebarConfig, NavConfig, NavItem};

    use super::*;

    #[test]
    fn test_label_from_route() {
        assert_eq!(label_from_route("/getting-started/"), "Getting Started");
        assert_eq!(label_from_route("/api/v2/"), "V2");
        assert_eq!(label_from_route("/docs/"), "Docs");
        assert_eq!(label_from_route("/hello_world/"), "Hello World");
    }

    #[test]
    fn test_label_from_segment() {
        assert_eq!(label_from_segment("getting-started"), "Getting Started");
        assert_eq!(label_from_segment("api"), "Api");
        assert_eq!(label_from_segment("hello_world"), "Hello World");
        assert_eq!(label_from_segment("index"), "Overview");
    }

    #[test]
    fn test_build_auto_nav_empty() {
        let routes: Vec<RouteEntry> = vec![];
        let tree = build_auto_nav(&routes, 3);
        assert!(tree.is_empty());
    }

    #[test]
    fn test_build_auto_nav_simple() {
        let routes = vec![
            RouteEntry {
                route: "/".to_string(),
                route_type: RouteType::Implied,
                target: "index.md".to_string(),
                source_path: None,
                spread_count: None,
                spread_arguments: None,
            },
            RouteEntry {
                route: "/about/".to_string(),
                route_type: RouteType::Implied,
                target: "about.md".to_string(),
                source_path: None,
                spread_count: None,
                spread_arguments: None,
            },
            RouteEntry {
                route: "/docs/intro/".to_string(),
                route_type: RouteType::Implied,
                target: "docs/intro.md".to_string(),
                source_path: None,
                spread_count: None,
                spread_arguments: None,
            },
        ];

        let tree = build_auto_nav(&routes, 3);

        assert_eq!(tree.len(), 3); // Home, About, Docs
        assert_eq!(tree[0].label, "Home");
        assert_eq!(tree[0].href, Some("/".to_string()));

        // Find the Docs item
        let docs = tree.iter().find(|i| i.label == "Docs").expect("Docs item");
        assert!(docs.children.is_some());
        assert_eq!(docs.children.as_ref().unwrap().len(), 1);
        assert_eq!(docs.children.as_ref().unwrap()[0].label, "Intro");
    }

    #[test]
    fn test_mark_active_items() {
        let mut tree = vec![
            NavTreeItem {
                label: "Home".to_string(),
                href: Some("/".to_string()),
                icon: None,
                active: false,
                expanded: false,
                children: None,
            },
            NavTreeItem {
                label: "Docs".to_string(),
                href: None,
                icon: None,
                active: false,
                expanded: false,
                children: Some(vec![NavTreeItem {
                    label: "Intro".to_string(),
                    href: Some("/docs/intro/".to_string()),
                    icon: None,
                    active: false,
                    expanded: false,
                    children: None,
                }]),
            },
        ];

        mark_active_items(&mut tree, "/docs/intro/");

        assert!(!tree[0].active); // Home not active
        assert!(tree[1].expanded); // Docs expanded (has active child)
        assert!(tree[1].children.as_ref().unwrap()[0].active); // Intro active
    }

    #[test]
    fn test_named_nav_config() {
        // Create a named nav config
        let mut named_navs = HashMap::new();
        named_navs.insert(
            "api".to_string(),
            NavConfig {
                items: vec![
                    NavItem::Link {
                        label: "Getting Started".to_string(),
                        target: "/api/getting-started/".to_string(),
                        icon: None,
                    },
                    NavItem::Group {
                        group: "Endpoints".to_string(),
                        children: vec![
                            NavItem::Route("/api/documents/".to_string()),
                            NavItem::Route("/api/nodes/".to_string()),
                        ],
                    },
                ],
            },
        );

        let routes = vec![
            RouteEntry {
                route: "/api/getting-started/".to_string(),
                route_type: RouteType::Implied,
                target: "api/getting-started.md".to_string(),
                source_path: None,
                spread_count: None,
                spread_arguments: None,
            },
            RouteEntry {
                route: "/api/documents/".to_string(),
                route_type: RouteType::Implied,
                target: "api/documents.md".to_string(),
                source_path: None,
                spread_count: None,
                spread_arguments: None,
            },
            RouteEntry {
                route: "/api/nodes/".to_string(),
                route_type: RouteType::Implied,
                target: "api/nodes.md".to_string(),
                source_path: None,
                spread_count: None,
                spread_arguments: None,
            },
        ];

        let config = LeftSidebarConfig {
            nav: Some("api".to_string()),
            ..Default::default()
        };

        let tree = build_nav_tree(&routes, "/api/documents/", &config, &named_navs);

        // Should have 2 items: "Getting Started" link and "Endpoints" group
        assert_eq!(tree.len(), 2);
        assert_eq!(tree[0].label, "Getting Started");
        assert_eq!(tree[0].href, Some("/api/getting-started/".to_string()));

        // Check the group
        assert_eq!(tree[1].label, "Endpoints");
        assert!(tree[1].children.is_some());
        let children = tree[1].children.as_ref().expect("Endpoints children");
        assert_eq!(children.len(), 2);
        assert_eq!(children[0].label, "Documents");
        assert!(children[0].active); // Current route should be active
        assert_eq!(children[1].label, "Nodes");
        assert!(!children[1].active);

        // The group should be expanded because it contains the active item
        assert!(tree[1].expanded);
    }

    #[test]
    fn test_named_nav_fallback_to_auto() {
        // When a named nav doesn't exist, it should fall back to auto nav
        let named_navs = HashMap::new(); // Empty - no named navs

        let routes = vec![
            RouteEntry {
                route: "/".to_string(),
                route_type: RouteType::Implied,
                target: "index.md".to_string(),
                source_path: None,
                spread_count: None,
                spread_arguments: None,
            },
            RouteEntry {
                route: "/about/".to_string(),
                route_type: RouteType::Implied,
                target: "about.md".to_string(),
                source_path: None,
                spread_count: None,
                spread_arguments: None,
            },
        ];

        let config = LeftSidebarConfig {
            nav: Some("nonexistent".to_string()),
            ..Default::default()
        };

        let tree = build_nav_tree(&routes, "/about/", &config, &named_navs);

        // Should fall back to auto nav
        assert_eq!(tree.len(), 2); // Home and About
        assert_eq!(tree[0].label, "Home");
    }
}
