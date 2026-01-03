//! Layout resolution for Stencila Sites
//!
//! Resolves layout configuration for a specific route, including
//! nav tree generation and preparing data for rendering.

use glob::Pattern;
use stencila_codec::stencila_schema::Node;
use stencila_config::{
    LayoutFooter, LayoutFooterOverride, LayoutHeader, LayoutHeaderOverride, LayoutLeftSidebar,
    LayoutOverride, LayoutRightSidebar, LeftSidebarConfig, RightSidebarConfig, SiteLayout,
};

use crate::{
    headings::extract_headings_from_node,
    list::{RouteEntry, RouteType},
    nav::{build_nav_tree, label_from_route},
};

// Re-export layout types from codec-dom
pub use stencila_codec_dom::{
    BreadcrumbItem, HeadingItem, NavTreeItem, PageLink, PageNavLinks, ResolvedFooter,
    ResolvedFooterGroup, ResolvedHeader, ResolvedIconLink, ResolvedLayout, ResolvedLeftSidebar,
    ResolvedNavLink, ResolvedRightSidebar,
};

/// Find the first matching override for a route
///
/// Evaluates overrides in array order using glob pattern matching.
/// Returns the first override whose `routes` patterns match the given route.
fn find_matching_override<'a>(
    route: &str,
    overrides: &'a [LayoutOverride],
) -> Option<&'a LayoutOverride> {
    for override_config in overrides {
        for pattern in &override_config.routes {
            // Compile glob pattern - if invalid, skip silently
            if let Ok(glob) = Pattern::new(pattern)
                && glob.matches(route)
            {
                return Some(override_config);
            }
        }
    }
    None
}

/// Resolved effective layout after applying overrides
///
/// This struct holds the effective values after merging base layout
/// with any matching override.
struct EffectiveLayout<'a> {
    header: Option<&'a LayoutHeader>,
    header_disabled: bool,
    /// Left sidebar explicit setting: Some(true)=enabled, Some(false)=disabled, None=use smart default
    left_sidebar_explicit: Option<bool>,
    left_sidebar_config: Option<LeftSidebarConfig>,
    /// Right sidebar explicit setting: Some(true)=enabled, Some(false)=disabled, None=use smart default
    right_sidebar_explicit: Option<bool>,
    right_sidebar_config: Option<RightSidebarConfig>,
    footer: Option<&'a LayoutFooter>,
    footer_disabled: bool,
    /// None means "use smart default based on left sidebar visibility"
    page_nav: Option<bool>,
}

/// Compute the effective layout by merging base with override
fn compute_effective_layout<'a>(
    layout: &'a SiteLayout,
    override_config: Option<&'a LayoutOverride>,
) -> EffectiveLayout<'a> {
    // Start with base layout values
    // Sidebar explicit settings use Option<bool>: Some(true)=enabled, Some(false)=disabled, None=smart default
    let mut header = layout.header_config();
    let mut header_disabled = false;
    let mut left_sidebar_explicit = layout.left_sidebar_explicit();
    let mut left_sidebar_config = layout.left_sidebar_config();
    let mut right_sidebar_explicit = layout.right_sidebar_explicit();
    let mut right_sidebar_config = layout.right_sidebar_config();
    let mut footer = layout.footer_config();
    let mut footer_disabled = false;
    let mut page_nav = layout.has_page_nav();

    // Apply override if present
    if let Some(ov) = override_config {
        // Header override
        if let Some(ref header_ov) = ov.header {
            match header_ov {
                LayoutHeaderOverride::Enabled(false) => {
                    header = None;
                    header_disabled = true;
                }
                LayoutHeaderOverride::Enabled(true) => {
                    // Keep base header
                }
                LayoutHeaderOverride::Config(h) => {
                    header = Some(h);
                }
            }
        }

        // Left sidebar override
        if let Some(ref sidebar_ov) = ov.left_sidebar {
            left_sidebar_explicit = Some(sidebar_ov.is_enabled());
            left_sidebar_config = match sidebar_ov {
                LayoutLeftSidebar::Enabled(false) => None,
                LayoutLeftSidebar::Enabled(true) => Some(LeftSidebarConfig::default()),
                LayoutLeftSidebar::Config(c) => Some(c.clone()),
            };
        }

        // Right sidebar override
        if let Some(ref right_ov) = ov.right_sidebar {
            right_sidebar_explicit = Some(right_ov.is_enabled());
            right_sidebar_config = match right_ov {
                LayoutRightSidebar::Enabled(false) => None,
                LayoutRightSidebar::Enabled(true) => Some(RightSidebarConfig::default()),
                LayoutRightSidebar::Config(c) => Some(c.clone()),
            };
        }

        // Footer override
        if let Some(ref footer_ov) = ov.footer {
            match footer_ov {
                LayoutFooterOverride::Enabled(false) => {
                    footer = None;
                    footer_disabled = true;
                }
                LayoutFooterOverride::Enabled(true) => {
                    // Keep base footer
                }
                LayoutFooterOverride::Config(f) => {
                    footer = Some(f);
                }
            }
        }

        // Page nav override
        if let Some(nav_ov) = ov.page_nav {
            page_nav = Some(nav_ov);
        }
    }

    EffectiveLayout {
        header,
        header_disabled,
        left_sidebar_explicit,
        left_sidebar_config,
        right_sidebar_explicit,
        right_sidebar_config,
        footer,
        footer_disabled,
        page_nav,
    }
}

/// Resolve layout configuration for a specific route
///
/// Takes the site layout configuration and list of all routes,
/// and produces a fully resolved layout with navigation tree
/// and active/expanded states computed for the current route.
///
/// Applies route-specific overrides using first-match-wins semantics:
/// overrides are evaluated in array order, and the first matching
/// override's settings replace the corresponding base layout sections.
///
/// # Arguments
/// * `route` - The current route being rendered
/// * `routes` - All discovered routes for the site
/// * `layout` - The site layout configuration
/// * `node` - Optional document node for extracting headings
///
/// # Returns
/// A `ResolvedLayout` with all data needed for rendering
pub fn resolve_layout(
    route: &str,
    routes: &[RouteEntry],
    layout: &SiteLayout,
    node: Option<&Node>,
) -> ResolvedLayout {
    // Find first matching override (if any)
    let override_config = find_matching_override(route, &layout.overrides);

    // Compute effective layout after merging base with override
    let effective = compute_effective_layout(layout, override_config);

    // Resolve header if configured and not disabled
    let header = if effective.header_disabled {
        None
    } else {
        effective.header.map(|h| resolve_header(h, route))
    };

    // Count document routes to determine if this is a multi-page site
    let document_route_count = routes
        .iter()
        .filter(|r| {
            matches!(
                r.route_type,
                RouteType::File | RouteType::Implied | RouteType::Spread
            )
        })
        .count();
    let is_multi_page = document_route_count > 1;

    // Apply smart defaults for left sidebar:
    // - Some(true): explicitly enabled → show
    // - Some(false): explicitly disabled → hide
    // - None: use smart default (multi-page sites)
    let left_sidebar_enabled = effective.left_sidebar_explicit.unwrap_or(is_multi_page);

    // Resolve footer if configured and not disabled
    let footer = if effective.footer_disabled {
        None
    } else {
        effective.footer.map(resolve_footer)
    };

    // Build left sidebar if enabled
    let left_sidebar = if left_sidebar_enabled {
        let config = effective.left_sidebar_config.unwrap_or_default();
        let collapsible = config.collapsible.unwrap_or(true);
        let expanded_depth = config.expanded;
        let nav_tree = build_nav_tree(routes, route, &config, &layout.navs);
        Some(ResolvedLeftSidebar {
            nav_tree,
            collapsible,
            expanded_depth,
        })
    } else {
        None
    };

    // Extract headings from document if provided
    let headings =
        node.and_then(|n| extract_headings_from_node(n, effective.right_sidebar_config.as_ref()));

    // Resolve right sidebar:
    // - Some(true): explicitly enabled → show if headings exist
    // - Some(false): explicitly disabled → never show
    // - None: use smart default (auto-enable if headings exist)
    let right_sidebar = match effective.right_sidebar_explicit {
        Some(false) => None, // Explicitly disabled
        Some(true) => {
            // Explicitly enabled - show if headings exist
            headings.map(|h| {
                let title = effective
                    .right_sidebar_config
                    .as_ref()
                    .and_then(|c| c.title.clone())
                    .unwrap_or_else(|| "On this page".to_string());
                ResolvedRightSidebar { title, headings: h }
            })
        }
        None => {
            // Smart default - auto-enable if headings exist
            headings.map(|h| {
                let title = effective
                    .right_sidebar_config
                    .as_ref()
                    .and_then(|c| c.title.clone())
                    .unwrap_or_else(|| "On this page".to_string());
                ResolvedRightSidebar { title, headings: h }
            })
        }
    };

    // Compute breadcrumbs for current route
    let breadcrumbs = compute_breadcrumbs(route, routes);

    // Compute page navigation:
    // - If explicitly enabled/disabled: use that value
    // - If not specified: derive from left sidebar visibility
    let page_nav_enabled = effective.page_nav.unwrap_or(left_sidebar_enabled);
    let page_nav = if page_nav_enabled {
        compute_page_nav(route, routes)
    } else {
        None
    };

    ResolvedLayout {
        header,
        left_sidebar,
        right_sidebar,
        footer,
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
    use stencila_config::{
        IconLink, LayoutFooter, LayoutHeader, LayoutLeftSidebar, LayoutOverride, TextLink,
    };

    use super::*;
    use crate::list::RouteType;

    #[test]
    fn test_resolve_layout_no_sidebar() {
        // Explicitly disable left sidebar (it defaults to true)
        let layout = SiteLayout {
            left_sidebar: Some(LayoutLeftSidebar::Enabled(false)),
            ..Default::default()
        };
        let routes: Vec<RouteEntry> = vec![];

        let resolved = resolve_layout("/", &routes, &layout, None);

        assert!(resolved.header.is_none());
        assert!(resolved.left_sidebar.is_none());
        assert!(resolved.right_sidebar.is_none());
    }

    #[test]
    fn test_resolve_layout_with_sidebar() {
        let layout = SiteLayout {
            left_sidebar: Some(LayoutLeftSidebar::Enabled(true)),
            right_sidebar: Some(LayoutRightSidebar::Enabled(true)),
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

        let resolved = resolve_layout("/about/", &routes, &layout, None);

        assert!(resolved.header.is_none());
        // Left sidebar should be enabled (no headings for right sidebar without Node)
        let left = resolved
            .left_sidebar
            .expect("left sidebar should be present");
        assert!(left.collapsible);
        // Right sidebar is None because no Node was provided to extract headings
        assert!(resolved.right_sidebar.is_none());
        assert_eq!(resolved.current_route, "/about/");

        // Check that About is marked active
        let about = left
            .nav_tree
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
            left_sidebar: Some(LayoutLeftSidebar::Enabled(false)),
            ..Default::default()
        };

        let resolved = resolve_layout("/docs/install/", &[], &layout, None);

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

    #[test]
    fn test_find_matching_override() {
        let overrides = vec![
            LayoutOverride {
                routes: vec!["/blog/**".to_string()],
                left_sidebar: Some(LayoutLeftSidebar::Enabled(false)),
                ..Default::default()
            },
            LayoutOverride {
                routes: vec!["/docs/api/**".to_string()],
                right_sidebar: Some(LayoutRightSidebar::Enabled(true)),
                ..Default::default()
            },
            LayoutOverride {
                routes: vec!["/docs/**".to_string()],
                page_nav: Some(false),
                ..Default::default()
            },
        ];

        // Should match /blog/**
        let result = find_matching_override("/blog/post-1/", &overrides);
        assert!(result.is_some());
        assert!(matches!(
            result.unwrap().left_sidebar,
            Some(LayoutLeftSidebar::Enabled(false))
        ));

        // Should match /docs/api/** (first match wins)
        let result = find_matching_override("/docs/api/v2/", &overrides);
        assert!(result.is_some());
        assert!(matches!(
            result.unwrap().right_sidebar,
            Some(LayoutRightSidebar::Enabled(true))
        ));

        // Should match /docs/** (not /docs/api/**)
        let result = find_matching_override("/docs/getting-started/", &overrides);
        assert!(result.is_some());
        assert_eq!(result.unwrap().page_nav, Some(false));

        // Should not match any
        let result = find_matching_override("/about/", &overrides);
        assert!(result.is_none());
    }

    #[test]
    fn test_resolve_layout_with_override() {
        // Base layout has header and left sidebar enabled
        let layout = SiteLayout {
            header: Some(LayoutHeader {
                logo: Some("logo.svg".to_string()),
                title: Some("My Site".to_string()),
                ..Default::default()
            }),
            left_sidebar: Some(LayoutLeftSidebar::Enabled(true)),
            footer: Some(LayoutFooter {
                copyright: Some("© 2024".to_string()),
                ..Default::default()
            }),
            overrides: vec![
                // Blog pages: no sidebar, no page nav
                LayoutOverride {
                    routes: vec!["/blog/**".to_string()],
                    left_sidebar: Some(LayoutLeftSidebar::Enabled(false)),
                    page_nav: Some(false),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        // Non-matching route should use base layout
        let resolved = resolve_layout("/docs/", &[], &layout, None);
        assert!(resolved.left_sidebar.is_some());
        assert!(resolved.header.is_some());
        assert!(resolved.footer.is_some());

        // Blog route should use override (no sidebar, no page nav)
        let resolved = resolve_layout("/blog/post-1/", &[], &layout, None);
        assert!(resolved.left_sidebar.is_none());
        assert!(resolved.page_nav.is_none());
        // Header and footer should still be present (not overridden)
        assert!(resolved.header.is_some());
        assert!(resolved.footer.is_some());
    }

    #[test]
    fn test_resolve_layout_override_disables_header() {
        let layout = SiteLayout {
            header: Some(LayoutHeader {
                title: Some("My Site".to_string()),
                ..Default::default()
            }),
            overrides: vec![LayoutOverride {
                routes: vec!["/landing/**".to_string()],
                header: Some(LayoutHeaderOverride::Enabled(false)),
                ..Default::default()
            }],
            ..Default::default()
        };

        // Regular route has header
        let resolved = resolve_layout("/docs/", &[], &layout, None);
        assert!(resolved.header.is_some());

        // Landing page has no header
        let resolved = resolve_layout("/landing/page1/", &[], &layout, None);
        assert!(resolved.header.is_none());
    }

    #[test]
    fn test_resolve_layout_override_replaces_header() {
        let layout = SiteLayout {
            header: Some(LayoutHeader {
                title: Some("Main Site".to_string()),
                logo: Some("main-logo.svg".to_string()),
                ..Default::default()
            }),
            overrides: vec![LayoutOverride {
                routes: vec!["/blog/**".to_string()],
                header: Some(LayoutHeaderOverride::Config(LayoutHeader {
                    title: Some("Blog".to_string()),
                    logo: Some("blog-logo.svg".to_string()),
                    ..Default::default()
                })),
                ..Default::default()
            }],
            ..Default::default()
        };

        // Regular route has main header
        let resolved = resolve_layout("/docs/", &[], &layout, None);
        let header = resolved.header.expect("header");
        assert_eq!(header.title, Some("Main Site".to_string()));
        assert_eq!(header.logo, Some("/main-logo.svg".to_string()));

        // Blog route has blog header
        let resolved = resolve_layout("/blog/post/", &[], &layout, None);
        let header = resolved.header.expect("header");
        assert_eq!(header.title, Some("Blog".to_string()));
        assert_eq!(header.logo, Some("/blog-logo.svg".to_string()));
    }

    #[test]
    fn test_override_first_match_wins() {
        let layout = SiteLayout {
            overrides: vec![
                // More specific pattern first
                LayoutOverride {
                    routes: vec!["/docs/api/**".to_string()],
                    right_sidebar: Some(LayoutRightSidebar::Enabled(true)),
                    ..Default::default()
                },
                // Less specific pattern second
                LayoutOverride {
                    routes: vec!["/docs/**".to_string()],
                    right_sidebar: Some(LayoutRightSidebar::Enabled(false)),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        // /docs/api/test/ should match first override (right_sidebar = true config)
        // but without a Node to extract headings, right_sidebar will be None
        let resolved = resolve_layout("/docs/api/test/", &[], &layout, None);
        assert!(resolved.right_sidebar.is_none());

        // /docs/guide/ should match second override (right_sidebar = false)
        let resolved = resolve_layout("/docs/guide/", &[], &layout, None);
        assert!(resolved.right_sidebar.is_none());
    }

    #[test]
    fn test_page_nav_follows_left_sidebar_override() {
        // Base layout: left sidebar enabled (default), page_nav not set
        // So page_nav defaults to true
        let layout = SiteLayout {
            overrides: vec![LayoutOverride {
                routes: vec!["/blog/**".to_string()],
                // Disable left sidebar but DON'T explicitly set page_nav
                left_sidebar: Some(LayoutLeftSidebar::Enabled(false)),
                ..Default::default()
            }],
            ..Default::default()
        };

        // Create routes so page_nav can actually be computed
        let routes = vec![
            RouteEntry {
                route: "/docs/".to_string(),
                route_type: RouteType::Implied,
                target: "docs.md".to_string(),
                source_path: None,
                spread_count: None,
                spread_arguments: None,
            },
            RouteEntry {
                route: "/docs/guide/".to_string(),
                route_type: RouteType::Implied,
                target: "docs/guide.md".to_string(),
                source_path: None,
                spread_count: None,
                spread_arguments: None,
            },
            RouteEntry {
                route: "/blog/post1/".to_string(),
                route_type: RouteType::Implied,
                target: "blog/post1.md".to_string(),
                source_path: None,
                spread_count: None,
                spread_arguments: None,
            },
            RouteEntry {
                route: "/blog/post2/".to_string(),
                route_type: RouteType::Implied,
                target: "blog/post2.md".to_string(),
                source_path: None,
                spread_count: None,
                spread_arguments: None,
            },
        ];

        // Non-matching route: left sidebar enabled, page nav should be computed
        let resolved = resolve_layout("/docs/", &routes, &layout, None);
        assert!(resolved.left_sidebar.is_some());
        assert!(
            resolved.page_nav.is_some(),
            "page_nav should be Some for docs route"
        );

        // Blog route: left sidebar disabled, page nav should ALSO be disabled
        // (because page_nav wasn't explicitly set, it should follow left_sidebar)
        let resolved = resolve_layout("/blog/post1/", &routes, &layout, None);
        assert!(resolved.left_sidebar.is_none());
        assert!(
            resolved.page_nav.is_none(),
            "page_nav should be None when left_sidebar is disabled and page_nav not explicitly set"
        );
    }

    #[test]
    fn test_page_nav_explicit_override_independent() {
        // Test that explicit page_nav override is independent of left_sidebar
        let layout = SiteLayout {
            overrides: vec![LayoutOverride {
                routes: vec!["/blog/**".to_string()],
                // Disable left sidebar but EXPLICITLY enable page nav
                left_sidebar: Some(LayoutLeftSidebar::Enabled(false)),
                page_nav: Some(true),
                ..Default::default()
            }],
            ..Default::default()
        };

        // Create routes so page_nav can actually be computed
        let routes = vec![
            RouteEntry {
                route: "/blog/post1/".to_string(),
                route_type: RouteType::Implied,
                target: "blog/post1.md".to_string(),
                source_path: None,
                spread_count: None,
                spread_arguments: None,
            },
            RouteEntry {
                route: "/blog/post2/".to_string(),
                route_type: RouteType::Implied,
                target: "blog/post2.md".to_string(),
                source_path: None,
                spread_count: None,
                spread_arguments: None,
            },
        ];

        // Blog route: left sidebar disabled, but page nav explicitly enabled
        let resolved = resolve_layout("/blog/post1/", &routes, &layout, None);
        assert!(resolved.left_sidebar.is_none());
        assert!(
            resolved.page_nav.is_some(),
            "page_nav should be Some when explicitly set to true"
        );
    }
}
