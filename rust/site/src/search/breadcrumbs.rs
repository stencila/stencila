//! Breadcrumb resolution for search entries
//!
//! This module builds a mapping from routes to breadcrumb labels,
//! using the navigation structure to resolve labels.

use std::collections::HashMap;

use stencila_config::NavItem;

use crate::nav_common::segment_to_label;

/// Build a route-to-breadcrumbs lookup map from nav items
///
/// Recursively traverses the nav tree and builds a map from routes
/// to their breadcrumb label paths.
pub fn build_breadcrumbs_map(nav_items: &[NavItem]) -> HashMap<String, Vec<String>> {
    let mut map = HashMap::new();

    // Add root route
    map.insert("/".to_string(), vec!["Home".to_string()]);

    // Build breadcrumbs for all nav items
    build_breadcrumbs_recursive(nav_items, &["Home".to_string()], &mut map);

    map
}

/// Recursively build breadcrumbs for nav items
fn build_breadcrumbs_recursive(
    items: &[NavItem],
    parent_breadcrumbs: &[String],
    map: &mut HashMap<String, Vec<String>>,
) {
    for item in items {
        match item {
            NavItem::Route(route) => {
                let normalized = normalize_route(route);
                // Skip root route as it's already added with just ["Home"]
                if normalized == "/" {
                    continue;
                }
                // For route shorthand, derive label from route
                let label = route_to_label(route);
                let mut breadcrumbs = parent_breadcrumbs.to_vec();
                breadcrumbs.push(label);
                map.insert(normalized, breadcrumbs);
            }
            NavItem::Link { label, route, .. } => {
                let normalized = normalize_route(route);
                // Skip root route as it's already added with just ["Home"]
                if normalized == "/" {
                    continue;
                }
                let mut breadcrumbs = parent_breadcrumbs.to_vec();
                breadcrumbs.push(label.clone());
                map.insert(normalized, breadcrumbs);
            }
            NavItem::Group {
                label,
                route,
                children,
                ..
            } => {
                let mut breadcrumbs = parent_breadcrumbs.to_vec();
                breadcrumbs.push(label.clone());

                // If group has a route, add it to the map (skip root)
                if let Some(route) = route {
                    let normalized = normalize_route(route);
                    if normalized != "/" {
                        map.insert(normalized, breadcrumbs.clone());
                    }
                }

                // Recurse into children
                build_breadcrumbs_recursive(children, &breadcrumbs, map);
            }
        }
    }
}

/// Get breadcrumbs for a route
///
/// Looks up the route in the breadcrumbs map. If not found, falls back
/// to deriving breadcrumbs from the route segments using title-case conversion.
pub fn get_breadcrumbs(route: &str, breadcrumbs_map: &HashMap<String, Vec<String>>) -> Vec<String> {
    let normalized = normalize_route(route);

    // Try exact match first
    if let Some(breadcrumbs) = breadcrumbs_map.get(&normalized) {
        return breadcrumbs.clone();
    }

    // Fall back to deriving from route segments
    derive_breadcrumbs_from_route(route)
}

/// Normalize a route for consistent lookup
///
/// Ensures routes have a trailing slash for consistent matching.
fn normalize_route(route: &str) -> String {
    let trimmed = route.trim_matches('/');
    if trimmed.is_empty() {
        "/".to_string()
    } else {
        format!("/{trimmed}/")
    }
}

/// Derive the last segment label from a route
fn route_to_label(route: &str) -> String {
    let trimmed = route.trim_matches('/');
    if trimmed.is_empty() {
        return "Home".to_string();
    }

    // Get the last segment
    let segment = trimmed.rsplit('/').next().unwrap_or(trimmed);
    segment_to_label(segment)
}

/// Derive breadcrumbs from route path using title-case conversion
///
/// Used as fallback when route is not in the nav structure.
fn derive_breadcrumbs_from_route(route: &str) -> Vec<String> {
    let trimmed = route.trim_matches('/');

    if trimmed.is_empty() {
        return vec!["Home".to_string()];
    }

    let mut breadcrumbs = vec!["Home".to_string()];
    for segment in trimmed.split('/') {
        breadcrumbs.push(segment_to_label(segment));
    }
    breadcrumbs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_route() {
        assert_eq!(normalize_route("/"), "/");
        assert_eq!(normalize_route("/docs/"), "/docs/");
        assert_eq!(normalize_route("/docs"), "/docs/");
        assert_eq!(normalize_route("docs/"), "/docs/");
        assert_eq!(normalize_route("docs"), "/docs/");
        assert_eq!(normalize_route("/docs/guide/"), "/docs/guide/");
    }

    #[test]
    fn test_derive_breadcrumbs_from_route() {
        assert_eq!(derive_breadcrumbs_from_route("/"), vec!["Home"]);
        assert_eq!(
            derive_breadcrumbs_from_route("/docs/"),
            vec!["Home", "Docs"]
        );
        assert_eq!(
            derive_breadcrumbs_from_route("/docs/getting-started/"),
            vec!["Home", "Docs", "Getting Started"]
        );
    }

    #[test]
    fn test_build_breadcrumbs_map_simple() {
        let nav_items = vec![
            NavItem::Route("/docs/".to_string()),
            NavItem::Link {
                id: None,
                label: "About Us".to_string(),
                route: "/about/".to_string(),
                icon: None,
                description: None,
            },
        ];

        let map = build_breadcrumbs_map(&nav_items);

        assert_eq!(map.get("/"), Some(&vec!["Home".to_string()]));
        assert_eq!(
            map.get("/docs/"),
            Some(&vec!["Home".to_string(), "Docs".to_string()])
        );
        assert_eq!(
            map.get("/about/"),
            Some(&vec!["Home".to_string(), "About Us".to_string()])
        );
    }

    #[test]
    fn test_build_breadcrumbs_map_nested() {
        let nav_items = vec![NavItem::Group {
            id: None,
            label: "Documentation".to_string(),
            route: Some("/docs/".to_string()),
            children: vec![
                NavItem::Route("/docs/getting-started/".to_string()),
                NavItem::Link {
                    id: None,
                    label: "Configuration Guide".to_string(),
                    route: "/docs/config/".to_string(),
                    icon: None,
                    description: None,
                },
            ],
            icon: None,
            section_title: None,
        }];

        let map = build_breadcrumbs_map(&nav_items);

        assert_eq!(
            map.get("/docs/"),
            Some(&vec!["Home".to_string(), "Documentation".to_string()])
        );
        assert_eq!(
            map.get("/docs/getting-started/"),
            Some(&vec![
                "Home".to_string(),
                "Documentation".to_string(),
                "Getting Started".to_string()
            ])
        );
        assert_eq!(
            map.get("/docs/config/"),
            Some(&vec![
                "Home".to_string(),
                "Documentation".to_string(),
                "Configuration Guide".to_string()
            ])
        );
    }

    #[test]
    fn test_get_breadcrumbs_from_map() {
        let nav_items = vec![NavItem::Link {
            id: None,
            label: "Documentation".to_string(),
            route: "/docs/".to_string(),
            icon: None,
            description: None,
        }];

        let map = build_breadcrumbs_map(&nav_items);

        // Route in map
        assert_eq!(
            get_breadcrumbs("/docs/", &map),
            vec!["Home", "Documentation"]
        );

        // Route not in map - falls back to segment conversion
        assert_eq!(
            get_breadcrumbs("/other/page/", &map),
            vec!["Home", "Other", "Page"]
        );
    }

    #[test]
    fn test_root_route_not_duplicated() {
        // If nav includes root route, it should not result in "Home > Home"
        let nav_items = vec![
            NavItem::Route("/".to_string()),
            NavItem::Link {
                id: None,
                label: "Home Page".to_string(),
                route: "/".to_string(),
                icon: None,
                description: None,
            },
            NavItem::Route("/docs/".to_string()),
        ];

        let map = build_breadcrumbs_map(&nav_items);

        // Root should always be just ["Home"], not ["Home", "Home"] or ["Home", "Home Page"]
        assert_eq!(map.get("/"), Some(&vec!["Home".to_string()]));

        // Other routes should work normally
        assert_eq!(
            map.get("/docs/"),
            Some(&vec!["Home".to_string(), "Docs".to_string()])
        );
    }
}
