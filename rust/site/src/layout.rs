//! Layout resolution for Stencila Sites
//!
//! Resolves layout configuration for a specific route, including
//! nav tree generation and preparing data for rendering.

use stencila_config::{LayoutHeader, SiteLayout};

use crate::{list::RouteEntry, nav::build_nav_tree};

// Re-export layout types from codec-dom
pub use stencila_codec_dom::{ResolvedHeader, ResolvedIconLink, ResolvedLayout, ResolvedTab};

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

    ResolvedLayout {
        header,
        left_sidebar,
        right_sidebar,
        nav_tree,
        collapsible,
        expanded_depth,
        current_route: route.to_string(),
    }
}

/// Resolve header configuration for the current route
///
/// Computes active state for tabs based on current route.
fn resolve_header(header: &LayoutHeader, current_route: &str) -> ResolvedHeader {
    // Resolve tabs with active state
    let tabs = header
        .tabs
        .iter()
        .map(|tab| ResolvedTab {
            label: tab.label.clone(),
            href: tab.href.clone(),
            // Tab is active if current route starts with tab href
            // e.g., route "/docs/install/" matches tab "/docs/"
            active: current_route.starts_with(&tab.href),
        })
        .collect();

    // Resolve icons (ensure label has a default)
    let icons = header
        .icons
        .iter()
        .map(|icon| ResolvedIconLink {
            icon: icon.icon.clone(),
            href: icon.href.clone(),
            label: icon.label.clone().unwrap_or_else(|| capitalize(&icon.icon)),
        })
        .collect();

    ResolvedHeader {
        logo: header.logo.as_ref().map(|path| make_absolute(path)),
        title: header.title.clone(),
        tabs,
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

#[cfg(test)]
mod tests {
    use stencila_config::{IconLink, LayoutHeader, LayoutSidebar, TabLink};

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
                tabs: vec![
                    TabLink {
                        label: "Docs".to_string(),
                        href: "/docs/".to_string(),
                    },
                    TabLink {
                        label: "API".to_string(),
                        href: "/api/".to_string(),
                    },
                ],
                icons: vec![IconLink {
                    icon: "github".to_string(),
                    href: "https://github.com/example".to_string(),
                    label: None,
                }],
            }),
            left_sidebar: Some(LayoutSidebar::Enabled(false)),
            ..Default::default()
        };

        let resolved = resolve_layout("/docs/install/", &[], &layout);

        let header = resolved.header.expect("header should be present");
        assert_eq!(header.logo, Some("logo.svg".to_string()));
        assert_eq!(header.title, Some("My Site".to_string()));

        // Check active tab detection
        assert_eq!(header.tabs.len(), 2);
        assert!(header.tabs[0].active); // /docs/ matches /docs/install/
        assert!(!header.tabs[1].active); // /api/ does not match

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
}
