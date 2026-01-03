//! Layout resolution for Stencila Sites
//!
//! Resolves layout configuration for a specific route, including
//! nav tree generation and preparing data for rendering.

use stencila_config::{LayoutFooter, LayoutHeader, SiteLayout};

use crate::{
    list::{RouteEntry, RouteType},
    nav::{build_nav_tree, label_from_route},
};

// Re-export layout types from codec-dom
pub use stencila_codec_dom::{
    BreadcrumbItem, PageLink, PageNavLinks, ResolvedFooter, ResolvedFooterGroup, ResolvedHeader,
    ResolvedIconLink, ResolvedLayout, ResolvedNavLink,
};

/// Resolve layout configuration for a specific route
///
/// Takes the site layout configuration and list of all routes,
/// and produces a fully resolved layout with navigation tree
/// and active/expanded states computed for the current route.
///
/// # Arguments
/// * `route` - The current route being rendered
/// * `routes` - All discovered routes for the site
/// * `layout` - The site layout configuration
///
/// # Returns
/// A `ResolvedLayout` with all data needed for rendering
pub fn resolve_layout(route: &str, routes: &[RouteEntry], layout: &SiteLayout) -> ResolvedLayout {
    // Resolve header if configured
    let header = layout.header_config().map(|h| resolve_header(h, route));

    let left_sidebar = layout.has_left_sidebar();
    let right_sidebar = layout.has_right_sidebar();

    // Resolve footer if configured
    let footer = layout.footer_config().map(resolve_footer);

    // Build nav tree if left sidebar is enabled
    let (nav_tree, collapsible, expanded_depth) = if left_sidebar {
        let config = layout.left_sidebar_config().unwrap_or_default();
        let collapsible = config.collapsible.unwrap_or(true);
        let expanded_depth = config.expanded;
        let tree = build_nav_tree(routes, route, &config, &layout.navs);
        (Some(tree), collapsible, expanded_depth)
    } else {
        (None, false, None)
    };

    // Compute breadcrumbs for current route
    let breadcrumbs = compute_breadcrumbs(route, routes);

    // Compute page navigation if enabled
    let page_nav = if layout.has_page_nav() {
        compute_page_nav(route, routes)
    } else {
        None
    };

    ResolvedLayout {
        header,
        left_sidebar,
        right_sidebar,
        footer,
        nav_tree,
        collapsible,
        expanded_depth,
        breadcrumbs,
        page_nav,
        current_route: route.to_string(),
    }
}

/// Resolve header configuration for the current route
///
/// Computes active state for tabs based on current route.
fn resolve_header(header: &LayoutHeader, current_route: &str) -> ResolvedHeader {
    // Resolve tabs with active state
    let tabs = header
        .links
        .iter()
        .map(|tab| ResolvedNavLink {
            label: tab.label.clone(),
            href: tab.target.clone(),
            // Tab is active if current route starts with tab href
            // e.g., route "/docs/install/" matches tab "/docs/"
            active: current_route.starts_with(&tab.target),
        })
        .collect();

    // Resolve icons (ensure label has a default)
    let icons = header
        .icons
        .iter()
        .map(|icon| ResolvedIconLink {
            icon: icon.icon.clone(),
            href: icon.target.clone(),
            label: icon.label.clone().unwrap_or_else(|| capitalize(&icon.icon)),
        })
        .collect();

    ResolvedHeader {
        logo: header.logo.as_ref().map(|path| make_absolute(path)),
        title: header.title.clone(),
        links: tabs,
        icons,
    }
}

/// Make a path absolute (relative to site root)
///
/// If the path is already absolute (starts with `/` or is a URL), return as-is.
/// Otherwise, prefix with `/` to make it relative to site root.
fn make_absolute(path: &str) -> String {
    if path.starts_with('/') || path.starts_with("http://") || path.starts_with("https://") {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

/// Capitalize the first letter of a string
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

/// Resolve footer configuration
///
/// Converts footer config to resolved footer with icon labels.
fn resolve_footer(footer: &LayoutFooter) -> ResolvedFooter {
    // Resolve groups
    let groups = footer
        .groups
        .iter()
        .map(|group| ResolvedFooterGroup {
            title: group.title.clone(),
            links: group
                .links
                .iter()
                .map(|link| ResolvedNavLink {
                    label: link.label.clone(),
                    href: link.target.clone(),
                    active: false, // Footer links don't have active state
                })
                .collect(),
        })
        .collect();

    // Resolve icons (ensure label has a default)
    let icons = footer
        .icons
        .iter()
        .map(|icon| ResolvedIconLink {
            icon: icon.icon.clone(),
            href: icon.target.clone(),
            label: icon.label.clone().unwrap_or_else(|| capitalize(&icon.icon)),
        })
        .collect();

    ResolvedFooter {
        groups,
        icons,
        copyright: footer.copyright.clone(),
    }
}

/// Compute breadcrumbs for a route
///
/// Generates a breadcrumb trail from the root to the current page.
/// For route "/docs/getting-started/install/", generates:
/// - Home → /
/// - Docs → /docs/ (if exists)
/// - Getting Started → /docs/getting-started/ (if exists)
/// - Install → current (no link)
fn compute_breadcrumbs(route: &str, routes: &[RouteEntry]) -> Vec<BreadcrumbItem> {
    let mut breadcrumbs = Vec::new();

    // Always start with Home
    breadcrumbs.push(BreadcrumbItem {
        label: "Home".to_string(),
        href: "/".to_string(),
        current: route == "/",
    });

    // If we're on the home page, we're done
    if route == "/" {
        return breadcrumbs;
    }

    // Split route into segments: "/docs/getting-started/install/" → ["docs", "getting-started", "install"]
    let segments: Vec<&str> = route.trim_matches('/').split('/').collect();

    // Build breadcrumb for each segment
    for (i, _segment) in segments.iter().enumerate() {
        let path = format!("/{}/", segments[..=i].join("/"));
        let is_current = path == route;

        // Try to find a matching route to get its label
        let label = routes
            .iter()
            .find(|r| r.route == path)
            .map(|r| label_from_route(&r.route))
            .unwrap_or_else(|| label_from_route(&path));

        breadcrumbs.push(BreadcrumbItem {
            label,
            href: path,
            current: is_current,
        });
    }

    breadcrumbs
}

/// Compute page navigation links
///
/// Finds the previous and next pages in the navigation order.
/// Only considers document routes (not static, redirect, etc.)
fn compute_page_nav(route: &str, routes: &[RouteEntry]) -> Option<PageNavLinks> {
    // Filter to document routes only and sort by route
    let mut doc_routes: Vec<&RouteEntry> = routes
        .iter()
        .filter(|r| {
            matches!(
                r.route_type,
                RouteType::File | RouteType::Implied | RouteType::Spread
            )
        })
        .collect();

    doc_routes.sort_by(|a, b| a.route.cmp(&b.route));

    // Find current route index
    let current_idx = doc_routes.iter().position(|r| r.route == route)?;

    // Get previous page
    let prev = if current_idx > 0 {
        let prev_route = doc_routes[current_idx - 1];
        Some(PageLink {
            label: label_from_route(&prev_route.route),
            href: prev_route.route.clone(),
        })
    } else {
        None
    };

    // Get next page
    let next = if current_idx < doc_routes.len() - 1 {
        let next_route = doc_routes[current_idx + 1];
        Some(PageLink {
            label: label_from_route(&next_route.route),
            href: next_route.route.clone(),
        })
    } else {
        None
    };

    // Return None if there are no links (orphan page)
    if prev.is_none() && next.is_none() {
        return None;
    }

    Some(PageNavLinks { prev, next })
}

#[cfg(test)]
mod tests {
    use stencila_config::{IconLink, LayoutHeader, LayoutSidebar, TextLink};

    use super::*;
    use crate::list::RouteType;

    #[test]
    fn test_resolve_layout_no_sidebar() {
        // Explicitly disable left sidebar (it defaults to true)
        let layout = SiteLayout {
            left_sidebar: Some(LayoutSidebar::Enabled(false)),
            ..Default::default()
        };
        let routes: Vec<RouteEntry> = vec![];

        let resolved = resolve_layout("/", &routes, &layout);

        assert!(resolved.header.is_none());
        assert!(!resolved.left_sidebar);
        assert!(!resolved.right_sidebar);
        assert!(resolved.nav_tree.is_none());
    }

    #[test]
    fn test_resolve_layout_with_sidebar() {
        let layout = SiteLayout {
            left_sidebar: Some(LayoutSidebar::Enabled(true)),
            right_sidebar: Some(true),
            ..Default::default()
        };

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

        let resolved = resolve_layout("/about/", &routes, &layout);

        assert!(resolved.header.is_none());
        assert!(resolved.left_sidebar);
        assert!(resolved.right_sidebar);
        assert!(resolved.nav_tree.is_some());
        assert!(resolved.collapsible);
        assert_eq!(resolved.current_route, "/about/");

        // Check that About is marked active
        let tree = resolved.nav_tree.unwrap();
        let about = tree
            .iter()
            .find(|i| i.label == "About")
            .expect("About item");
        assert!(about.active);
    }

    #[test]
    fn test_resolve_layout_with_header() {
        let layout = SiteLayout {
            header: Some(LayoutHeader {
                logo: Some("logo.svg".to_string()),
                title: Some("My Site".to_string()),
                links: vec![
                    TextLink {
                        label: "Docs".to_string(),
                        target: "/docs/".to_string(),
                    },
                    TextLink {
                        label: "API".to_string(),
                        target: "/api/".to_string(),
                    },
                ],
                icons: vec![IconLink {
                    icon: "github".to_string(),
                    target: "https://github.com/example".to_string(),
                    label: None,
                }],
            }),
            left_sidebar: Some(LayoutSidebar::Enabled(false)),
            ..Default::default()
        };

        let resolved = resolve_layout("/docs/install/", &[], &layout);

        let header = resolved.header.expect("header should be present");
        assert_eq!(header.logo, Some("/logo.svg".to_string())); // Relative paths get / prefix
        assert_eq!(header.title, Some("My Site".to_string()));

        // Check active tab detection
        assert_eq!(header.links.len(), 2);
        assert!(header.links[0].active); // /docs/ matches /docs/install/
        assert!(!header.links[1].active); // /api/ does not match

        // Check icon label default (capitalized icon name)
        assert_eq!(header.icons.len(), 1);
        assert_eq!(header.icons[0].label, "Github");
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("github"), "Github");
        assert_eq!(capitalize("discord"), "Discord");
        assert_eq!(capitalize(""), "");
        assert_eq!(capitalize("x"), "X");
    }

    #[test]
    fn test_compute_breadcrumbs_home() {
        let routes: Vec<RouteEntry> = vec![];
        let breadcrumbs = compute_breadcrumbs("/", &routes);

        assert_eq!(breadcrumbs.len(), 1);
        assert_eq!(breadcrumbs[0].label, "Home");
        assert_eq!(breadcrumbs[0].href, "/");
        assert!(breadcrumbs[0].current);
    }

    #[test]
    fn test_compute_breadcrumbs_nested() {
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
                route: "/docs/".to_string(),
                route_type: RouteType::Implied,
                target: "docs/index.md".to_string(),
                source_path: None,
                spread_count: None,
                spread_arguments: None,
            },
            RouteEntry {
                route: "/docs/getting-started/".to_string(),
                route_type: RouteType::Implied,
                target: "docs/getting-started.md".to_string(),
                source_path: None,
                spread_count: None,
                spread_arguments: None,
            },
        ];

        let breadcrumbs = compute_breadcrumbs("/docs/getting-started/", &routes);

        assert_eq!(breadcrumbs.len(), 3);

        // Home
        assert_eq!(breadcrumbs[0].label, "Home");
        assert_eq!(breadcrumbs[0].href, "/");
        assert!(!breadcrumbs[0].current);

        // Docs
        assert_eq!(breadcrumbs[1].label, "Docs");
        assert_eq!(breadcrumbs[1].href, "/docs/");
        assert!(!breadcrumbs[1].current);

        // Getting Started (current)
        assert_eq!(breadcrumbs[2].label, "Getting Started");
        assert_eq!(breadcrumbs[2].href, "/docs/getting-started/");
        assert!(breadcrumbs[2].current);
    }

    #[test]
    fn test_compute_page_nav() {
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
                route: "/docs/".to_string(),
                route_type: RouteType::Implied,
                target: "docs.md".to_string(),
                source_path: None,
                spread_count: None,
                spread_arguments: None,
            },
        ];

        // Test middle page has both prev and next
        let page_nav = compute_page_nav("/about/", &routes).expect("should have page nav");
        assert!(page_nav.prev.is_some());
        assert_eq!(page_nav.prev.as_ref().unwrap().href, "/");
        assert!(page_nav.next.is_some());
        assert_eq!(page_nav.next.as_ref().unwrap().href, "/docs/");

        // Test first page has only next
        let page_nav = compute_page_nav("/", &routes).expect("should have page nav");
        assert!(page_nav.prev.is_none());
        assert!(page_nav.next.is_some());

        // Test last page has only prev
        let page_nav = compute_page_nav("/docs/", &routes).expect("should have page nav");
        assert!(page_nav.prev.is_some());
        assert!(page_nav.next.is_none());
    }

    #[test]
    fn test_compute_page_nav_single_page() {
        let routes = vec![RouteEntry {
            route: "/".to_string(),
            route_type: RouteType::Implied,
            target: "index.md".to_string(),
            source_path: None,
            spread_count: None,
            spread_arguments: None,
        }];

        // Single page should have no navigation
        let page_nav = compute_page_nav("/", &routes);
        assert!(page_nav.is_none());
    }
}
