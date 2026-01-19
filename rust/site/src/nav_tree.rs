//! Navigation tree component rendering
//!
//! Displays hierarchical site navigation from `site.nav` configuration.
//! If `site.nav` is not specified, auto-generates navigation from routes.

use std::path::Path;

use stencila_config::{NavItem, NavTreeExpanded, NavTreeIcons, SiteConfig};

use crate::{
    RouteEntry,
    nav_common::{apply_icons, auto_generate_nav, filter_nav_items, render_icon_span},
};

/// Context for rendering the nav tree
pub(crate) struct NavTreeContext<'a> {
    pub site_config: &'a SiteConfig,
    pub site_root: &'a Path,
    pub route: &'a str,
    pub routes: &'a [RouteEntry],
}

/// Render a navigation tree component
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_nav_tree(
    title: &Option<String>,
    depth: &Option<u8>,
    collapsible: &Option<bool>,
    expanded: &Option<NavTreeExpanded>,
    scroll_to_active: &Option<bool>,
    include: &Option<Vec<String>>,
    exclude: &Option<Vec<String>>,
    icons: &Option<NavTreeIcons>,
    context: &NavTreeContext,
) -> String {
    // Get config values with defaults
    let collapsible = collapsible.unwrap_or(true);
    let expanded = expanded.unwrap_or_default();
    let scroll_to_active = scroll_to_active.unwrap_or(true);
    let icons_mode = icons.unwrap_or_default();

    // Resolve nav items: site.nav or auto-generated
    let nav_items = context
        .site_config
        .nav
        .clone()
        .unwrap_or_else(|| auto_generate_nav(context.routes, depth, Some(context.site_root)));

    // Apply icons from site.icons if icons mode is Show
    let nav_items = if matches!(icons_mode, NavTreeIcons::Show) {
        apply_icons(nav_items, &context.site_config.icons)
    } else {
        nav_items
    };

    // Apply filtering
    let nav_items = filter_nav_items(nav_items, include, exclude);

    // If no items, render empty component
    if nav_items.is_empty() {
        return "<stencila-nav-tree></stencila-nav-tree>".to_string();
    }

    // Build title HTML
    let title_html = title
        .as_ref()
        .map(|t| format!(r#"<h2 class="title">{t}</h2>"#))
        .unwrap_or_default();

    // Render nav items recursively (empty string for root-level parent_id)
    let items_html = render_nav_items(
        &nav_items,
        context.route,
        1,
        depth,
        &expanded,
        collapsible,
        "",
        &icons_mode,
    );

    // Build attributes
    let attrs = format!(
        r#" collapsible="{collapsible}" expanded="{expanded}" scroll-to-active="{scroll_to_active}""#
    );

    format!(
        r#"<stencila-nav-tree{attrs}><nav aria-label="Site navigation">{title_html}<ul class="list" role="tree">{items_html}</ul></nav></stencila-nav-tree>"#
    )
}

/// Render navigation items recursively
#[allow(clippy::too_many_arguments)]
fn render_nav_items(
    items: &[NavItem],
    current_route: &str,
    level: u8,
    max_depth: &Option<u8>,
    expanded: &NavTreeExpanded,
    collapsible: bool,
    parent_id: &str,
    icons_mode: &NavTreeIcons,
) -> String {
    // Check depth limit
    if let Some(max) = max_depth
        && level > *max
    {
        return String::new();
    }

    let mut html = String::new();

    for (index, item) in items.iter().enumerate() {
        match item {
            NavItem::Route(route) => {
                let is_active = route == current_route;
                let label = crate::nav_common::route_to_label(route);
                html.push_str(&format!(
                    r#"<li class="item" data-type="link" data-active="{is_active}" data-level="{level}" role="treeitem"{}><a href="{route}">{label}</a></li>"#,
                    if is_active { r#" aria-current="page""# } else { "" }
                ));
            }
            NavItem::Link {
                label, route, icon, ..
            } => {
                let is_active = route == current_route;
                let icon_html = render_nav_tree_icon(icon, icons_mode);
                html.push_str(&format!(
                    r#"<li class="item" data-type="link" data-active="{is_active}" data-level="{level}" role="treeitem"{}><a href="{route}">{icon_html}{label}</a></li>"#,
                    if is_active { r#" aria-current="page""# } else { "" }
                ));
            }
            NavItem::Group {
                label,
                route,
                children,
                icon,
                ..
            } => {
                // Determine if group should be expanded
                let is_expanded =
                    should_expand_group(expanded, level, route, children, current_route);
                // Include parent_id to ensure unique IDs across the full tree
                let label_slug = label.to_lowercase().replace(' ', "-");
                let group_id = if parent_id.is_empty() {
                    format!("nav-{index}-{label_slug}")
                } else {
                    format!("{parent_id}-{index}-{label_slug}")
                };

                // Check if the group header itself is active (if it has a route)
                let header_active = route.as_ref().is_some_and(|r| r == current_route);

                // Build icon HTML
                let icon_html = render_nav_tree_icon(icon, icons_mode);

                // Build the group header based on collapsible setting
                let header_html = if collapsible {
                    // Collapsible mode: include toggle button
                    if let Some(group_route) = route {
                        // Group has a route - render as clickable link with separate toggle
                        format!(
                            r#"<div class="group-header"><a href="{group_route}" class="group-link"{}>{icon_html}{label}</a><button class="toggle" aria-controls="{group_id}" aria-expanded="{is_expanded}"><span class="chevron"></span></button></div>"#,
                            if header_active {
                                r#" aria-current="page""#
                            } else {
                                ""
                            }
                        )
                    } else {
                        // Group has no route - header is just a toggle button
                        format!(
                            r#"<button class="toggle" aria-controls="{group_id}" aria-expanded="{is_expanded}"><span class="chevron"></span><span class="label">{icon_html}{label}</span></button>"#
                        )
                    }
                } else {
                    // Non-collapsible mode: no toggle button, always expanded
                    if let Some(group_route) = route {
                        format!(
                            r#"<a href="{group_route}" class="group-link"{}>{icon_html}{label}</a>"#,
                            if header_active {
                                r#" aria-current="page""#
                            } else {
                                ""
                            }
                        )
                    } else {
                        format!(r#"<span class="group-label">{icon_html}{label}</span>"#)
                    }
                };

                // When not collapsible, groups are always expanded
                let display_expanded = if collapsible { is_expanded } else { true };

                // Render children, passing this group's ID as parent for nested groups
                let children_html = render_nav_items(
                    children,
                    current_route,
                    level + 1,
                    max_depth,
                    expanded,
                    collapsible,
                    &group_id,
                    icons_mode,
                );

                html.push_str(&format!(
                    r#"<li class="item" data-type="group" data-expanded="{display_expanded}" data-active="{header_active}" data-level="{level}" role="treeitem"{}>{header_html}<ul id="{group_id}" class="children" role="group">{children_html}</ul></li>"#,
                    if collapsible {
                        format!(r#" aria-expanded="{display_expanded}""#)
                    } else {
                        String::new()
                    }
                ));
            }
        }
    }

    html
}

/// Render an icon for the nav tree (respects icons mode)
fn render_nav_tree_icon(icon: &Option<String>, mode: &NavTreeIcons) -> String {
    let Some(icon_name) = icon else {
        return String::new();
    };

    if matches!(mode, NavTreeIcons::Show) {
        render_icon_span(icon_name)
    } else {
        String::new()
    }
}

/// Determine if a navigation group should be expanded based on the expansion mode
fn should_expand_group(
    expanded: &NavTreeExpanded,
    level: u8,
    group_route: &Option<String>,
    children: &[NavItem],
    current_route: &str,
) -> bool {
    match expanded {
        NavTreeExpanded::All => true,
        NavTreeExpanded::None => false,
        NavTreeExpanded::FirstLevel => level == 1,
        NavTreeExpanded::CurrentPath => {
            // Expand if the group's own route is active
            if let Some(r) = group_route
                && r == current_route
            {
                return true;
            }
            // Or if any child (recursively) is the current route
            contains_route(children, current_route)
        }
    }
}

/// Check if any nav item (recursively) contains the given route
fn contains_route(items: &[NavItem], route: &str) -> bool {
    for item in items {
        match item {
            NavItem::Route(r) if r == route => return true,
            NavItem::Link { route: r, .. } if r == route => return true,
            NavItem::Group { route: Some(r), .. } if r == route => return true,
            NavItem::Group { children, .. } => {
                if contains_route(children, route) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}
