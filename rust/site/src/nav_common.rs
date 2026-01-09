//! Shared navigation utilities for nav-tree and nav-menu components
//!
//! This module contains:
//! - Auto-generation of navigation from routes
//! - Filtering of navigation items (include/exclude)
//! - Icon resolution from site.icons
//! - Route-to-label conversion utilities

use std::collections::HashMap;

use stencila_config::NavItem;

use crate::RouteEntry;

// =============================================================================
// Auto-generation
// =============================================================================

/// Auto-generate navigation structure from routes
///
/// Groups routes by their path prefixes to create a hierarchical structure.
/// For example:
/// - `/` → Home
/// - `/docs/getting-started/` → under "Docs" group
/// - `/docs/configuration/` → under "Docs" group
/// - `/about/` → About
pub(crate) fn auto_generate_nav(routes: &[RouteEntry], max_depth: &Option<u8>) -> Vec<NavItem> {
    use std::collections::BTreeMap;

    // Build a tree structure from routes
    // Each node can have a route (if a page exists at that path) and children
    #[derive(Default)]
    struct NavNode {
        /// The route for this node (if a page exists at this path)
        route: Option<String>,
        /// Child nodes keyed by path segment
        children: BTreeMap<String, NavNode>,
    }

    let mut root = NavNode::default();

    // Insert all routes into the tree
    for entry in routes {
        let trimmed = entry.route.trim_matches('/');
        if trimmed.is_empty() {
            // Root route
            root.route = Some(entry.route.clone());
        } else {
            // Split into segments and traverse/create nodes
            let segments: Vec<&str> = trimmed.split('/').collect();
            let mut current = &mut root;

            for (i, segment) in segments.iter().enumerate() {
                current = current.children.entry((*segment).to_string()).or_default();

                // If this is the last segment, set the route
                if i == segments.len() - 1 {
                    current.route = Some(entry.route.clone());
                }
            }
        }
    }

    // Convert tree to NavItems recursively
    fn build_nav_items(node: &NavNode, depth: u8, max_depth: &Option<u8>) -> Vec<NavItem> {
        // Check depth limit
        if let Some(max) = max_depth
            && depth > *max
        {
            return Vec::new();
        }

        let mut items = Vec::new();

        // BTreeMap keeps children sorted alphabetically by segment name
        let sorted_children: Vec<_> = node.children.iter().collect();

        for (segment, child_node) in sorted_children {
            let label = segment_to_label(segment);

            if child_node.children.is_empty() {
                // Leaf node - create a simple link
                if let Some(route) = &child_node.route {
                    items.push(NavItem::Route(route.clone()));
                }
            } else {
                // Has children - create a group
                let children = build_nav_items(child_node, depth + 1, max_depth);

                // Only create a group if there are children to show
                if children.is_empty() && child_node.route.is_none() {
                    continue;
                }

                if children.is_empty() {
                    // No children after filtering - just add as link
                    if let Some(route) = &child_node.route {
                        items.push(NavItem::Route(route.clone()));
                    }
                } else {
                    items.push(NavItem::Group {
                        id: None,
                        label,
                        route: child_node.route.clone(),
                        children,
                        icon: None,
                        section_title: None,
                    });
                }
            }
        }

        items
    }

    // Start building from root's children (depth 1)
    // Note: the root route ("/") is not included since the logo links to home
    build_nav_items(&root, 1, max_depth)
}

// =============================================================================
// Label Utilities
// =============================================================================

/// Convert a route path to a human-readable label
///
/// Takes the last segment of the path and converts it to title case.
/// - `/docs/getting-started/` → "Getting Started"
/// - `/` → "Home"
pub(crate) fn route_to_label(route: &str) -> String {
    let trimmed = route.trim_matches('/');
    if trimmed.is_empty() {
        return "Home".to_string();
    }

    // Get the last segment
    let segment = trimmed.rsplit('/').next().unwrap_or(trimmed);
    segment_to_label(segment)
}

/// Convert a URL segment to a human-readable label
///
/// - Replaces hyphens and underscores with spaces
/// - Capitalizes each word
pub(crate) fn segment_to_label(segment: &str) -> String {
    segment
        .split(['-', '_'])
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect(),
                None => String::new(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}

// =============================================================================
// Nav Item Filtering
// =============================================================================

/// Pattern types for nav item matching
pub(crate) enum FilterPattern<'a> {
    /// Route pattern (starts with "/"), supports glob syntax
    Route(&'a str),
    /// ID pattern (starts with "#"), matches by item's id field
    Id(&'a str),
    /// Label pattern (everything else), matches by item's label
    Label(&'a str),
}

/// Parse a filter pattern string into a FilterPattern
pub(crate) fn parse_filter_pattern(pattern: &str) -> FilterPattern<'_> {
    if pattern.starts_with('/') {
        FilterPattern::Route(pattern)
    } else if let Some(id) = pattern.strip_prefix('#') {
        FilterPattern::Id(id)
    } else {
        FilterPattern::Label(pattern)
    }
}

/// Check if a pattern matches a string using glob-style matching
fn glob_matches(pattern: &str, value: &str) -> bool {
    // Normalize trailing slashes for route comparison
    let pattern = pattern.trim_end_matches('/');
    let value = value.trim_end_matches('/');

    if let Some(prefix) = pattern.strip_suffix("**") {
        // Match prefix and all descendants
        let prefix = prefix.trim_end_matches('/');
        value == prefix || value.starts_with(&format!("{prefix}/"))
    } else if let Some(prefix) = pattern.strip_suffix('*') {
        // Match prefix and immediate children only
        let prefix = prefix.trim_end_matches('/');
        if value == prefix {
            return true;
        }
        if !value.starts_with(&format!("{prefix}/")) {
            return false;
        }
        // Check it's an immediate child (no additional slashes after prefix)
        let suffix = &value[prefix.len() + 1..];
        !suffix.contains('/')
    } else if pattern.contains('*') {
        // Simple wildcard matching for IDs/labels
        let parts: Vec<&str> = pattern.split('*').collect();
        if parts.len() == 2 {
            value.starts_with(parts[0]) && value.ends_with(parts[1])
        } else {
            pattern == value
        }
    } else {
        pattern == value
    }
}

/// Check if a nav item matches a filter pattern
fn matches_nav_item(item: &NavItem, pattern: &FilterPattern) -> bool {
    match (item, pattern) {
        // Route shorthand - only matches route patterns
        (NavItem::Route(route), FilterPattern::Route(pat)) => glob_matches(pat, route),
        (NavItem::Route(_), _) => false,

        // Link - matches route, id, or label
        (NavItem::Link { route, .. }, FilterPattern::Route(pat)) => glob_matches(pat, route),
        (NavItem::Link { id: Some(id), .. }, FilterPattern::Id(pat)) => glob_matches(pat, id),
        (NavItem::Link { label, .. }, FilterPattern::Label(pat)) => glob_matches(pat, label),
        (NavItem::Link { .. }, FilterPattern::Id(_)) => false,

        // Group - matches route (if present), id, or label
        (NavItem::Group { route: Some(r), .. }, FilterPattern::Route(pat)) => glob_matches(pat, r),
        (NavItem::Group { route: None, .. }, FilterPattern::Route(_)) => false,
        (NavItem::Group { id: Some(id), .. }, FilterPattern::Id(pat)) => glob_matches(pat, id),
        (NavItem::Group { label, .. }, FilterPattern::Label(pat)) => glob_matches(pat, label),
        (NavItem::Group { .. }, FilterPattern::Id(_)) => false,
    }
}

/// Filter nav items based on include/exclude patterns
///
/// - If include is specified, only items matching at least one pattern are kept
/// - Exclude patterns are applied after include, removing matching items
/// - Filtering is recursive: excluding a group excludes all its children
/// - If a child matches include but parent doesn't, parent is auto-included as container
pub(crate) fn filter_nav_items(
    items: Vec<NavItem>,
    include: &Option<Vec<String>>,
    exclude: &Option<Vec<String>>,
) -> Vec<NavItem> {
    // If no filters, return unchanged
    if include.is_none() && exclude.is_none() {
        return items;
    }

    let include_patterns: Option<Vec<FilterPattern>> = include
        .as_ref()
        .map(|v| v.iter().map(|s| parse_filter_pattern(s)).collect());
    let exclude_patterns: Option<Vec<FilterPattern>> = exclude
        .as_ref()
        .map(|v| v.iter().map(|s| parse_filter_pattern(s)).collect());

    filter_nav_items_recursive(items, &include_patterns, &exclude_patterns)
}

fn filter_nav_items_recursive(
    items: Vec<NavItem>,
    include: &Option<Vec<FilterPattern>>,
    exclude: &Option<Vec<FilterPattern>>,
) -> Vec<NavItem> {
    let mut result = Vec::new();

    for item in items {
        // Check exclude first - if excluded, skip entirely
        if let Some(patterns) = exclude
            && patterns.iter().any(|p| matches_nav_item(&item, p))
        {
            continue;
        }

        // Check include
        let included = if let Some(patterns) = include {
            patterns.iter().any(|p| matches_nav_item(&item, p))
        } else {
            true // No include filter means include all
        };

        match item {
            NavItem::Route(_) | NavItem::Link { .. } => {
                if included {
                    result.push(item);
                }
            }
            NavItem::Group {
                id,
                label,
                route,
                children,
                icon,
                section_title,
            } => {
                // Recursively filter children
                let filtered_children = filter_nav_items_recursive(children, include, exclude);

                // Include group if:
                // 1. Group itself is included, OR
                // 2. Any children remain after filtering (auto-include as container)
                if included || !filtered_children.is_empty() {
                    result.push(NavItem::Group {
                        id,
                        label,
                        route,
                        children: filtered_children,
                        icon,
                        section_title,
                    });
                }
            }
        }
    }

    result
}

// =============================================================================
// Icon Resolution
// =============================================================================

/// Apply icons from site.icons to nav items
///
/// For each item, if item.icon is already set, keep it (explicit takes precedence).
/// Otherwise, try matching item.route in icons map, then item.label.
pub(crate) fn apply_icons(
    items: Vec<NavItem>,
    icons: &Option<HashMap<String, String>>,
) -> Vec<NavItem> {
    let Some(icons) = icons else {
        return items;
    };

    items
        .into_iter()
        .map(|item| apply_icon_to_item(item, icons))
        .collect()
}

fn apply_icon_to_item(item: NavItem, icons: &HashMap<String, String>) -> NavItem {
    match item {
        NavItem::Route(route) => {
            // Route shorthand - check if route has an icon, convert to Link if so
            if let Some(icon) = lookup_icon(&Some(route.clone()), &route_to_label(&route), icons) {
                NavItem::Link {
                    id: None,
                    label: route_to_label(&route),
                    route,
                    icon: Some(icon),
                    description: None,
                }
            } else {
                NavItem::Route(route)
            }
        }
        NavItem::Link {
            id,
            label,
            route,
            icon,
            description,
        } => {
            let resolved_icon = icon.or_else(|| lookup_icon(&Some(route.clone()), &label, icons));
            NavItem::Link {
                id,
                label,
                route,
                icon: resolved_icon,
                description,
            }
        }
        NavItem::Group {
            id,
            label,
            route,
            children,
            icon,
            section_title,
        } => {
            let resolved_icon = icon.or_else(|| lookup_icon(&route, &label, icons));
            let resolved_children = children
                .into_iter()
                .map(|c| apply_icon_to_item(c, icons))
                .collect();
            NavItem::Group {
                id,
                label,
                route,
                children: resolved_children,
                icon: resolved_icon,
                section_title,
            }
        }
    }
}

/// Look up an icon from the icons map, trying route first, then label
fn lookup_icon(
    route: &Option<String>,
    label: &str,
    icons: &HashMap<String, String>,
) -> Option<String> {
    // Try route first (with and without trailing slash normalization)
    if let Some(r) = route
        && let Some(icon) = lookup_icon_by_route(r, icons)
    {
        return Some(icon);
    }

    // Try label as-is
    if let Some(icon) = icons.get(label) {
        return Some(icon.clone());
    }

    // Try deriving a route from the label (for groups without explicit routes)
    // Convert "Getting Started" -> "/getting-started/" and try that
    let derived_route = format!("/{}/", label_to_segment(label));
    lookup_icon_by_route(&derived_route, icons)
}

/// Try to find an icon by route, with various slash normalizations
fn lookup_icon_by_route(route: &str, icons: &HashMap<String, String>) -> Option<String> {
    let normalized = route.trim_end_matches('/');
    if let Some(icon) = icons.get(route) {
        return Some(icon.clone());
    }
    if let Some(icon) = icons.get(normalized) {
        return Some(icon.clone());
    }
    let with_slash = format!("{normalized}/");
    if let Some(icon) = icons.get(&with_slash) {
        return Some(icon.clone());
    }
    None
}

/// Convert a label back to a URL segment
/// "Getting Started" -> "getting-started"
fn label_to_segment(label: &str) -> String {
    label.to_lowercase().replace(' ', "-")
}

/// Normalize an icon name by adding the default "lucide:" prefix if no icon set is specified
///
/// This allows users to use shorthand like "home" instead of "lucide:home" in configuration.
/// If the icon name already contains a ":" (e.g., "simple-icons:github"), it is returned as-is.
pub(crate) fn normalize_icon_name(icon: &str) -> String {
    if icon.contains(':') {
        icon.to_string()
    } else {
        ["lucide:", icon].concat()
    }
}

pub(crate) fn render_icon_span(icon: &str) -> String {
    [
        r#"<span class="icon i-"#,
        &normalize_icon_name(icon),
        r#""></span>"#,
    ]
    .concat()
}
