//! Shared navigation utilities for nav-tree and nav-menu components
//!
//! This module contains:
//! - Auto-generation of navigation from routes
//! - Navigation override loading from `_nav.yaml`/`_nav.toml`/`_nav.json` files
//! - Filtering of navigation items (include/exclude)
//! - Icon resolution from site.icons
//! - Route-to-label conversion utilities

use std::{collections::HashMap, path::Path};

use serde::Deserialize;
use stencila_config::{AccessLevel, NavItem, SiteAccessConfig};
use tracing::warn;

use crate::RouteEntry;

// =============================================================================
// Navigation Override Files (_nav.yaml/_nav.toml/_nav.json)
// =============================================================================

/// Navigation override from a `_nav.*` file
///
/// These files can be placed in any directory to control the order and grouping
/// of navigation items for that directory, overriding alphabetical sorting.
#[derive(Debug, Deserialize)]
pub struct NavOverride {
    pub items: Vec<NavOverrideItem>,
}

/// A navigation override item
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum NavOverrideItem {
    /// Simple route reference (just a command/page name)
    Route(String),
    /// Group with label and children
    Group {
        label: String,
        children: Vec<NavOverrideChild>,
        #[serde(default)]
        icon: Option<String>,
    },
}

/// A child item in a navigation override group
///
/// Can be either a simple string (route name) or an object with nested children.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum NavOverrideChild {
    /// Simple route reference (just a command/page name)
    Route(String),
    /// Route with nested children (for preserving order of subcommands)
    Nested {
        name: String,
        children: Vec<NavOverrideChild>,
    },
}

/// Load a navigation override file from a directory if one exists
///
/// Checks for files in this order: `_nav.yaml`, `_nav.yml`, `_nav.toml`, `_nav.json`
fn load_nav_override(dir: &Path) -> Option<NavOverride> {
    for ext in ["yaml", "yml", "toml", "json"] {
        let nav_file = dir.join(format!("_nav.{ext}"));
        if let Ok(content) = std::fs::read_to_string(&nav_file) {
            let result: Result<NavOverride, String> = match ext {
                "yaml" | "yml" => serde_yaml::from_str(&content).map_err(|e| e.to_string()),
                "toml" => toml::from_str(&content).map_err(|e| e.to_string()),
                "json" => serde_json::from_str(&content).map_err(|e| e.to_string()),
                _ => Err("unsupported format".to_string()),
            };
            match result {
                Ok(nav_override) => return Some(nav_override),
                Err(err) => {
                    warn!(
                        "Failed to parse navigation override file {}: {}",
                        nav_file.display(),
                        err
                    );
                    // Continue to try other formats or fall back to alphabetical
                }
            }
        }
    }
    None
}

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
///
/// If a `_nav.yaml`, `_nav.toml`, or `_nav.json` file exists in a directory,
/// its ordering and grouping will be used instead of alphabetical sorting.
pub(crate) fn auto_generate_nav(
    routes: &[RouteEntry],
    max_depth: &Option<u8>,
    site_root: Option<&Path>,
) -> Vec<NavItem> {
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
    fn build_nav_items(
        node: &NavNode,
        segment_path: &str,
        site_root: Option<&Path>,
        depth: u8,
        max_depth: &Option<u8>,
    ) -> Vec<NavItem> {
        // Check depth limit
        if let Some(max) = max_depth
            && depth > *max
        {
            return Vec::new();
        }

        // Check for _nav.* override file
        if let Some(root) = site_root {
            let dir = if segment_path.is_empty() {
                root.to_path_buf()
            } else {
                root.join(segment_path)
            };
            if let Some(nav_override) = load_nav_override(&dir) {
                return build_from_override(
                    nav_override,
                    node,
                    segment_path,
                    site_root,
                    depth,
                    max_depth,
                );
            }
        }

        let mut items = Vec::new();

        // BTreeMap keeps children sorted alphabetically by segment name
        let sorted_children: Vec<_> = node.children.iter().collect();

        for (segment, child_node) in sorted_children {
            let label = segment_to_label(segment);
            let child_path = if segment_path.is_empty() {
                segment.clone()
            } else {
                format!("{segment_path}/{segment}")
            };

            if child_node.children.is_empty() {
                // Leaf node - create a simple link
                if let Some(route) = &child_node.route {
                    items.push(NavItem::Route(route.clone()));
                }
            } else {
                // Has children - create a group
                let children =
                    build_nav_items(child_node, &child_path, site_root, depth + 1, max_depth);

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

    /// Build navigation items from a _nav.* override file
    fn build_from_override(
        config: NavOverride,
        node: &NavNode,
        segment_path: &str,
        site_root: Option<&Path>,
        depth: u8,
        max_depth: &Option<u8>,
    ) -> Vec<NavItem> {
        // Check depth limit
        if let Some(max) = max_depth
            && depth > *max
        {
            return Vec::new();
        }

        let mut items = Vec::new();

        for override_item in config.items {
            match override_item {
                NavOverrideItem::Route(name) => {
                    // Find this route in node.children
                    if let Some(child_node) = node.children.get(&name) {
                        let child_path = if segment_path.is_empty() {
                            name.clone()
                        } else {
                            format!("{segment_path}/{name}")
                        };

                        if child_node.children.is_empty() {
                            // Leaf node
                            if let Some(route) = &child_node.route {
                                items.push(NavItem::Route(route.clone()));
                            }
                        } else {
                            // Has subcommands - recursively build (uses alphabetical order)
                            let children = build_nav_items(
                                child_node,
                                &child_path,
                                site_root,
                                depth + 1,
                                max_depth,
                            );
                            if !children.is_empty() || child_node.route.is_some() {
                                items.push(NavItem::Group {
                                    id: None,
                                    label: segment_to_label(&name),
                                    route: child_node.route.clone(),
                                    children,
                                    icon: None,
                                    section_title: None,
                                });
                            } else if let Some(route) = &child_node.route {
                                items.push(NavItem::Route(route.clone()));
                            }
                        }
                    }
                }
                NavOverrideItem::Group {
                    label,
                    children,
                    icon,
                } => {
                    // Build a group with the specified children in order
                    let group_children = build_children_from_override(
                        &children,
                        node,
                        segment_path,
                        site_root,
                        depth + 1,
                        max_depth,
                    );
                    if !group_children.is_empty() {
                        items.push(NavItem::Group {
                            id: None,
                            label,
                            route: None,
                            children: group_children,
                            icon,
                            section_title: None,
                        });
                    }
                }
            }
        }

        items
    }

    /// Build nav items from override children, preserving order
    fn build_children_from_override(
        override_children: &[NavOverrideChild],
        node: &NavNode,
        segment_path: &str,
        site_root: Option<&Path>,
        depth: u8,
        max_depth: &Option<u8>,
    ) -> Vec<NavItem> {
        // Check depth limit
        if let Some(max) = max_depth
            && depth > *max
        {
            return Vec::new();
        }

        let mut items = Vec::new();

        for child in override_children {
            match child {
                NavOverrideChild::Route(name) => {
                    if let Some(child_node) = node.children.get(name) {
                        let child_path = if segment_path.is_empty() {
                            name.clone()
                        } else {
                            format!("{segment_path}/{name}")
                        };

                        if child_node.children.is_empty() {
                            // Leaf node
                            if let Some(route) = &child_node.route {
                                items.push(NavItem::Route(route.clone()));
                            }
                        } else {
                            // Has nested children - recursively build (uses alphabetical order)
                            let nested = build_nav_items(
                                child_node,
                                &child_path,
                                site_root,
                                depth + 1,
                                max_depth,
                            );
                            if !nested.is_empty() || child_node.route.is_some() {
                                items.push(NavItem::Group {
                                    id: None,
                                    label: segment_to_label(name),
                                    route: child_node.route.clone(),
                                    children: nested,
                                    icon: None,
                                    section_title: None,
                                });
                            } else if let Some(route) = &child_node.route {
                                items.push(NavItem::Route(route.clone()));
                            }
                        }
                    }
                }
                NavOverrideChild::Nested {
                    name,
                    children: nested_children,
                } => {
                    if let Some(child_node) = node.children.get(name) {
                        let child_path = if segment_path.is_empty() {
                            name.clone()
                        } else {
                            format!("{segment_path}/{name}")
                        };

                        // Build children using the override order
                        let nested = build_children_from_override(
                            nested_children,
                            child_node,
                            &child_path,
                            site_root,
                            depth + 1,
                            max_depth,
                        );
                        if !nested.is_empty() || child_node.route.is_some() {
                            items.push(NavItem::Group {
                                id: None,
                                label: segment_to_label(name),
                                route: child_node.route.clone(),
                                children: nested,
                                icon: None,
                                section_title: None,
                            });
                        } else if let Some(route) = &child_node.route {
                            items.push(NavItem::Route(route.clone()));
                        }
                    }
                }
            }
        }

        items
    }

    // Start building from root's children (depth 1)
    // Note: the root route ("/") is not included since the logo links to home
    build_nav_items(&root, "", site_root, 1, max_depth)
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
// Depth Limiting
// =============================================================================

/// Apply depth limit to nav items, converting groups with no visible children to links
///
/// This function:
/// - Limits the tree to the specified depth
/// - Converts groups with no children (after depth limiting) to links if they have routes
/// - Omits groups with no children and no route
///
/// This maintains behavioral equivalence with the previous auto-generation path which
/// applied depth limiting during generation rather than at render time.
pub(crate) fn apply_depth_limit(items: Vec<NavItem>, max_depth: Option<u8>) -> Vec<NavItem> {
    let Some(max_depth) = max_depth else {
        return items;
    };
    apply_depth_limit_recursive(items, 1, max_depth)
}

fn apply_depth_limit_recursive(items: Vec<NavItem>, level: u8, max_depth: u8) -> Vec<NavItem> {
    // If we're beyond max depth, return empty
    if level > max_depth {
        return Vec::new();
    }

    items
        .into_iter()
        .filter_map(|item| match item {
            // Routes and Links pass through unchanged (they're leaf nodes)
            NavItem::Route(_) | NavItem::Link { .. } => Some(item),
            NavItem::Group {
                id,
                label,
                route,
                children,
                icon,
                section_title,
            } => {
                // Process children at next level
                let filtered_children = if level >= max_depth {
                    // At max depth, don't include children
                    Vec::new()
                } else {
                    apply_depth_limit_recursive(children, level + 1, max_depth)
                };

                if filtered_children.is_empty() {
                    // No children after depth limiting - convert to link if has route
                    route.map(|r| NavItem::Link {
                        id,
                        label,
                        route: r,
                        icon,
                        description: None,
                    })
                } else {
                    // Has children, keep as group
                    Some(NavItem::Group {
                        id,
                        label,
                        route,
                        children: filtered_children,
                        icon,
                        section_title,
                    })
                }
            }
        })
        .collect()
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

/// Try to find an icon by route, with various normalizations
///
/// Tries multiple key formats to find a match:
/// - Exact route, without/with trailing slash
/// - Without leading slash (e.g., "docs/config" for "/docs/config/")
/// - Bare segment (e.g., "docs" for "/docs/")
fn lookup_icon_by_route(route: &str, icons: &HashMap<String, String>) -> Option<String> {
    lookup_by_route(route, icons, |v| v.clone())
}

/// Generic route lookup with various normalizations
fn lookup_by_route<T, F>(route: &str, map: &HashMap<String, T>, clone_fn: F) -> Option<T>
where
    F: Fn(&T) -> T,
{
    // Try exact route
    if let Some(value) = map.get(route) {
        return Some(clone_fn(value));
    }

    // Normalize: trim trailing slash
    let normalized = route.trim_end_matches('/');
    if let Some(value) = map.get(normalized) {
        return Some(clone_fn(value));
    }

    // Try with trailing slash
    let with_trailing = format!("{normalized}/");
    if let Some(value) = map.get(&with_trailing) {
        return Some(clone_fn(value));
    }

    // Try without leading slash (e.g., "/docs/config/" -> "docs/config/", "docs/config")
    let without_leading = normalized.trim_start_matches('/');
    if without_leading != normalized {
        if let Some(value) = map.get(without_leading) {
            return Some(clone_fn(value));
        }
        let with_trailing = format!("{without_leading}/");
        if let Some(value) = map.get(&with_trailing) {
            return Some(clone_fn(value));
        }
    }

    // Try bare segment (e.g., "/docs/" -> "docs", "/docs/config/" -> "config")
    let segment = without_leading.rsplit('/').next().unwrap_or("");
    if !segment.is_empty()
        && let Some(value) = map.get(segment)
    {
        return Some(clone_fn(value));
    }

    None
}

/// Convert a label back to a URL segment
/// "Getting Started" -> "getting-started"
pub(crate) fn label_to_segment(label: &str) -> String {
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

// =============================================================================
// Description Resolution
// =============================================================================

/// Apply descriptions from site.descriptions to nav items
///
/// For each Link item, if item.description is already set, keep it (explicit takes precedence).
/// Otherwise, try matching item.route in descriptions map, then item.label.
pub(crate) fn apply_descriptions(
    items: Vec<NavItem>,
    descriptions: &Option<HashMap<String, String>>,
) -> Vec<NavItem> {
    let Some(descriptions) = descriptions else {
        return items;
    };

    items
        .into_iter()
        .map(|item| apply_description_to_item(item, descriptions))
        .collect()
}

fn apply_description_to_item(item: NavItem, descriptions: &HashMap<String, String>) -> NavItem {
    match item {
        NavItem::Route(route) => {
            // Route shorthand - check if route has a description, convert to Link if so
            if let Some(description) =
                lookup_description(&Some(route.clone()), &route_to_label(&route), descriptions)
            {
                NavItem::Link {
                    id: None,
                    label: route_to_label(&route),
                    route,
                    icon: None,
                    description: Some(description),
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
            let resolved_description = description
                .or_else(|| lookup_description(&Some(route.clone()), &label, descriptions));
            NavItem::Link {
                id,
                label,
                route,
                icon,
                description: resolved_description,
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
            // Groups don't have descriptions, but recurse into children
            let resolved_children = children
                .into_iter()
                .map(|c| apply_description_to_item(c, descriptions))
                .collect();
            NavItem::Group {
                id,
                label,
                route,
                children: resolved_children,
                icon,
                section_title,
            }
        }
    }
}

/// Look up a description from the descriptions map, trying route first, then label
fn lookup_description(
    route: &Option<String>,
    label: &str,
    descriptions: &HashMap<String, String>,
) -> Option<String> {
    // Try route first (with and without trailing slash normalization)
    if let Some(r) = route
        && let Some(description) = lookup_description_by_route(r, descriptions)
    {
        return Some(description);
    }

    // Try label as-is
    if let Some(description) = descriptions.get(label) {
        return Some(description.clone());
    }

    // Try deriving a route from the label (for items without explicit routes)
    // Convert "Getting Started" -> "/getting-started/" and try that
    let derived_route = format!("/{}/", label_to_segment(label));
    lookup_description_by_route(&derived_route, descriptions)
}

/// Try to find a description by route, with various normalizations
fn lookup_description_by_route(
    route: &str,
    descriptions: &HashMap<String, String>,
) -> Option<String> {
    lookup_by_route(route, descriptions, |v| v.clone())
}

// =============================================================================
// Access Level Rendering
// =============================================================================

/// Convert AccessLevel to string for data attribute
fn access_level_to_str(level: AccessLevel) -> &'static str {
    match level {
        AccessLevel::Public => "public",
        AccessLevel::Subscriber => "subscriber",
        AccessLevel::Password => "password",
        AccessLevel::Team => "team",
    }
}

/// Normalize a route for access config lookup.
///
/// Ensures routes that look like directories (no file extension) end with "/".
/// This is required because access config keys must end with "/" for prefix matching.
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

/// Render the data-access attribute only if the route's access level is more
/// restrictive than the inherited level.
///
/// This prevents badges from appearing on every item when a parent already
/// indicates the restriction. Only the "top-most" restricted item in a
/// hierarchy will have the attribute.
///
/// Routes are normalized to ensure directory-style paths have trailing slashes,
/// which is required for matching against access config keys.
///
/// Returns the new inherited access level for children (the more restrictive
/// of the current inherited level and this route's level).
pub(crate) fn render_access_attr_if_more_restrictive(
    route: &str,
    inherited_level: AccessLevel,
    access_config: Option<&SiteAccessConfig>,
) -> (String, AccessLevel) {
    let Some(config) = access_config else {
        return (String::new(), inherited_level);
    };

    // Normalize route for access lookup (ensure trailing slash for directories)
    let normalized_route = normalize_route_for_access(route);
    let level = config.get_access_level(&normalized_route);

    // The new inherited level for children is the more restrictive of the two
    let new_inherited = if level > inherited_level {
        level
    } else {
        inherited_level
    };

    // Only add attribute if this route is MORE restrictive than inherited
    if level > inherited_level {
        (
            format!(r#" data-access="{}""#, access_level_to_str(level)),
            new_inherited,
        )
    } else {
        (String::new(), new_inherited)
    }
}

/// Render access attribute for a group, deriving route from label if not present.
///
/// Groups without explicit routes (e.g., a "Docs" group with no index.html)
/// still need access badges if their derived path (e.g., "/docs/") is restricted.
/// This function derives the route from the label and checks access.
pub(crate) fn render_group_access_attr(
    route: &Option<String>,
    label: &str,
    inherited_level: AccessLevel,
    access_config: Option<&SiteAccessConfig>,
) -> (String, AccessLevel) {
    // If group has an explicit route, use it
    if let Some(r) = route {
        return render_access_attr_if_more_restrictive(r, inherited_level, access_config);
    }

    // Otherwise, derive route from label (e.g., "Docs" -> "/docs/")
    let derived_route = format!("/{}/", label_to_segment(label));
    render_access_attr_if_more_restrictive(&derived_route, inherited_level, access_config)
}
