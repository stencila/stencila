//! Breadcrumb resolution for search entries
//!
//! This module builds a mapping from routes to breadcrumb labels,
//! using the navigation structure to resolve labels.

use std::collections::HashMap;

use stencila_config::NavItem;

use crate::nav_common::segment_to_label;

/// A breadcrumb entry with a label and optional route
///
/// When route is None, the breadcrumb represents a nav group without
/// a corresponding page and should be rendered as non-clickable text.
pub type Breadcrumb = (String, Option<String>);

/// Build a route-to-breadcrumbs lookup map from nav items
///
/// Recursively traverses the nav tree and builds a map from routes
/// to their breadcrumb paths, where each breadcrumb includes its label
/// and optional route (for groups without routes).
pub fn build_breadcrumbs_map(nav_items: &[NavItem]) -> HashMap<String, Vec<Breadcrumb>> {
    let mut map = HashMap::new();

    // Add root route
    map.insert(
        "/".to_string(),
        vec![("Home".to_string(), Some("/".to_string()))],
    );

    // Build breadcrumbs for all nav items
    let home_breadcrumb = vec![("Home".to_string(), Some("/".to_string()))];
    build_breadcrumbs_recursive(nav_items, &home_breadcrumb, &mut map);

    map
}

/// Recursively build breadcrumbs for nav items
fn build_breadcrumbs_recursive(
    items: &[NavItem],
    parent_breadcrumbs: &[Breadcrumb],
    map: &mut HashMap<String, Vec<Breadcrumb>>,
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
                breadcrumbs.push((label, Some(normalized.clone())));
                map.insert(normalized, breadcrumbs);
            }
            NavItem::Link { label, route, .. } => {
                let normalized = normalize_route(route);
                // Skip root route as it's already added with just ["Home"]
                if normalized == "/" {
                    continue;
                }
                let mut breadcrumbs = parent_breadcrumbs.to_vec();
                breadcrumbs.push((label.clone(), Some(normalized.clone())));
                map.insert(normalized, breadcrumbs);
            }
            NavItem::Group {
                label,
                route,
                children,
                ..
            } => {
                let mut breadcrumbs = parent_breadcrumbs.to_vec();

                // Add group to breadcrumbs with its route (or None if no route)
                let group_route = route.as_ref().map(|r| normalize_route(r));
                breadcrumbs.push((label.clone(), group_route.clone()));

                // If group has a route, add it to the map (skip root)
                if let Some(ref normalized) = group_route
                    && normalized != "/"
                {
                    map.insert(normalized.clone(), breadcrumbs.clone());
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
pub fn get_breadcrumbs(
    route: &str,
    breadcrumbs_map: &HashMap<String, Vec<Breadcrumb>>,
) -> Vec<Breadcrumb> {
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
/// All derived breadcrumbs have routes since they come from URL segments.
fn derive_breadcrumbs_from_route(route: &str) -> Vec<Breadcrumb> {
    let trimmed = route.trim_matches('/');

    if trimmed.is_empty() {
        return vec![("Home".to_string(), Some("/".to_string()))];
    }

    let mut breadcrumbs = vec![("Home".to_string(), Some("/".to_string()))];
    let mut current_path = String::new();

    for segment in trimmed.split('/') {
        current_path.push('/');
        current_path.push_str(segment);
        let route_with_slash = format!("{current_path}/");
        breadcrumbs.push((segment_to_label(segment), Some(route_with_slash)));
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
        assert_eq!(
            derive_breadcrumbs_from_route("/"),
            vec![("Home".to_string(), Some("/".to_string()))]
        );
        assert_eq!(
            derive_breadcrumbs_from_route("/docs/"),
            vec![
                ("Home".to_string(), Some("/".to_string())),
                ("Docs".to_string(), Some("/docs/".to_string()))
            ]
        );
        assert_eq!(
            derive_breadcrumbs_from_route("/docs/getting-started/"),
            vec![
                ("Home".to_string(), Some("/".to_string())),
                ("Docs".to_string(), Some("/docs/".to_string())),
                (
                    "Getting Started".to_string(),
                    Some("/docs/getting-started/".to_string())
                )
            ]
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

        assert_eq!(
            map.get("/"),
            Some(&vec![("Home".to_string(), Some("/".to_string()))])
        );
        assert_eq!(
            map.get("/docs/"),
            Some(&vec![
                ("Home".to_string(), Some("/".to_string())),
                ("Docs".to_string(), Some("/docs/".to_string()))
            ])
        );
        assert_eq!(
            map.get("/about/"),
            Some(&vec![
                ("Home".to_string(), Some("/".to_string())),
                ("About Us".to_string(), Some("/about/".to_string()))
            ])
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
            Some(&vec![
                ("Home".to_string(), Some("/".to_string())),
                ("Documentation".to_string(), Some("/docs/".to_string()))
            ])
        );
        assert_eq!(
            map.get("/docs/getting-started/"),
            Some(&vec![
                ("Home".to_string(), Some("/".to_string())),
                ("Documentation".to_string(), Some("/docs/".to_string())),
                (
                    "Getting Started".to_string(),
                    Some("/docs/getting-started/".to_string())
                )
            ])
        );
        assert_eq!(
            map.get("/docs/config/"),
            Some(&vec![
                ("Home".to_string(), Some("/".to_string())),
                ("Documentation".to_string(), Some("/docs/".to_string())),
                (
                    "Configuration Guide".to_string(),
                    Some("/docs/config/".to_string())
                )
            ])
        );
    }

    #[test]
    fn test_build_breadcrumbs_map_group_without_route() {
        // This tests the scenario where a nav group has no route (like "Markup" in formats)
        let nav_items = vec![NavItem::Group {
            id: None,
            label: "Formats".to_string(),
            route: Some("/formats/".to_string()),
            children: vec![NavItem::Group {
                id: None,
                label: "Markup".to_string(),
                route: None, // No route for this group!
                children: vec![NavItem::Link {
                    id: None,
                    label: "HTML".to_string(),
                    route: "/formats/html/".to_string(),
                    icon: None,
                    description: None,
                }],
                icon: None,
                section_title: None,
            }],
            icon: None,
            section_title: None,
        }];

        let map = build_breadcrumbs_map(&nav_items);

        // The HTML page should have Markup in breadcrumbs but with None route
        assert_eq!(
            map.get("/formats/html/"),
            Some(&vec![
                ("Home".to_string(), Some("/".to_string())),
                ("Formats".to_string(), Some("/formats/".to_string())),
                ("Markup".to_string(), None), // No route!
                ("HTML".to_string(), Some("/formats/html/".to_string()))
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
            vec![
                ("Home".to_string(), Some("/".to_string())),
                ("Documentation".to_string(), Some("/docs/".to_string()))
            ]
        );

        // Route not in map - falls back to segment conversion
        assert_eq!(
            get_breadcrumbs("/other/page/", &map),
            vec![
                ("Home".to_string(), Some("/".to_string())),
                ("Other".to_string(), Some("/other/".to_string())),
                ("Page".to_string(), Some("/other/page/".to_string()))
            ]
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
        assert_eq!(
            map.get("/"),
            Some(&vec![("Home".to_string(), Some("/".to_string()))])
        );

        // Other routes should work normally
        assert_eq!(
            map.get("/docs/"),
            Some(&vec![
                ("Home".to_string(), Some("/".to_string())),
                ("Docs".to_string(), Some("/docs/".to_string()))
            ])
        );
    }
}
