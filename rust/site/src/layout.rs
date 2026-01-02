//! Layout resolution for Stencila Sites
//!
//! Resolves layout configuration for a specific route, including
//! nav tree generation and preparing data for rendering.

use stencila_config::SiteLayout;

use crate::{list::RouteEntry, nav::build_nav_tree};

// Re-export ResolvedLayout from codec-dom
pub use stencila_codec_dom::ResolvedLayout;

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
        left_sidebar,
        right_sidebar,
        nav_tree,
        collapsible,
        expanded_depth,
        current_route: route.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use stencila_config::LayoutSidebar;

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
}
