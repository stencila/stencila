//! Navigation groups component rendering
//!
//! Displays footer-style navigation with flat, grouped links under headings.
//! Top-level nav items become group headings, their children become links.
//! Uses CSS grid for responsive auto-columns.

use std::path::Path;

use stencila_config::{NavGroupsIcons, NavItem, SiteConfig};

use crate::{
    RouteEntry,
    nav_common::{apply_icons, auto_generate_nav, filter_nav_items, render_icon_span},
};

/// Context for rendering nav groups
pub(crate) struct NavGroupsContext<'a> {
    pub site_config: &'a SiteConfig,
    pub site_root: &'a Path,
    pub route: &'a str,
    pub routes: &'a [RouteEntry],
}

/// Render a navigation groups component
pub(crate) fn render_nav_groups(
    include: &Option<Vec<String>>,
    exclude: &Option<Vec<String>>,
    depth: &Option<u8>,
    icons: &Option<NavGroupsIcons>,
    context: &NavGroupsContext,
) -> String {
    // Get config values with defaults
    let max_depth = depth.unwrap_or(2);
    let icons_mode = icons.unwrap_or_default();

    // Resolve nav items: site.nav or auto-generated
    let nav_items = context.site_config.nav.clone().unwrap_or_else(|| {
        auto_generate_nav(context.routes, &Some(max_depth), Some(context.site_root))
    });

    // Apply icons from site.icons if icons mode is Show
    let nav_items = if matches!(icons_mode, NavGroupsIcons::Show) {
        apply_icons(nav_items, &context.site_config.icons)
    } else {
        nav_items
    };

    // Apply filtering
    let nav_items = filter_nav_items(nav_items, include, exclude);

    // If no items, render empty component
    if nav_items.is_empty() {
        return "<stencila-nav-groups></stencila-nav-groups>".to_string();
    }

    // Render groups
    let groups_html = render_groups(&nav_items, context.route, max_depth, &icons_mode);

    format!(
        r#"<stencila-nav-groups><nav aria-label="Footer navigation"><div class="groups">{groups_html}</div></nav></stencila-nav-groups>"#
    )
}

/// Render navigation groups
fn render_groups(
    items: &[NavItem],
    current_route: &str,
    max_depth: u8,
    icons_mode: &NavGroupsIcons,
) -> String {
    let mut html = String::new();

    for item in items {
        match item {
            // A bare route at top level becomes a single-item group
            NavItem::Route(route) => {
                let label = crate::nav_common::route_to_label(route);
                let is_active = route == current_route;
                html.push_str(&format!(
                    r#"<div class="group"><ul class="group-links"><li class="link" data-active="{is_active}"><a href="{route}"{}><span class="label">{label}</span></a></li></ul></div>"#,
                    if is_active { r#" aria-current="page""# } else { "" }
                ));
            }
            // A link at top level becomes a single-item group
            NavItem::Link {
                label, route, icon, ..
            } => {
                let is_active = route == current_route;
                let icon_html = render_icon(icon, icons_mode);
                html.push_str(&format!(
                    r#"<div class="group"><ul class="group-links"><li class="link" data-active="{is_active}"><a href="{route}"{}>{icon_html}<span class="label">{label}</span></a></li></ul></div>"#,
                    if is_active { r#" aria-current="page""# } else { "" }
                ));
            }
            // A group becomes a heading with links
            NavItem::Group {
                label,
                route,
                children,
                ..
            } => {
                // Render heading (optionally as a link)
                let heading_html = if let Some(group_route) = route {
                    let is_active = group_route == current_route;
                    format!(
                        r#"<h3 class="group-heading"><a href="{group_route}"{}>{label}</a></h3>"#,
                        if is_active {
                            r#" aria-current="page""#
                        } else {
                            ""
                        }
                    )
                } else {
                    format!(r#"<h3 class="group-heading">{label}</h3>"#)
                };

                // Render children as links (respecting depth limit)
                let links_html = if max_depth > 1 {
                    render_links(children, current_route, 2, max_depth, icons_mode)
                } else {
                    String::new()
                };

                html.push_str(&format!(
                    r#"<div class="group">{heading_html}<ul class="group-links">{links_html}</ul></div>"#
                ));
            }
        }
    }

    html
}

/// Render navigation links (children of a group)
fn render_links(
    items: &[NavItem],
    current_route: &str,
    level: u8,
    max_depth: u8,
    icons_mode: &NavGroupsIcons,
) -> String {
    // Check depth limit
    if level > max_depth {
        return String::new();
    }

    let mut html = String::new();

    for item in items {
        match item {
            NavItem::Route(route) => {
                let label = crate::nav_common::route_to_label(route);
                let is_active = route == current_route;
                html.push_str(&format!(
                    r#"<li class="link" data-active="{is_active}"><a href="{route}"{}><span class="label">{label}</span></a></li>"#,
                    if is_active { r#" aria-current="page""# } else { "" }
                ));
            }
            NavItem::Link {
                label, route, icon, ..
            } => {
                let is_active = route == current_route;
                let icon_html = render_icon(icon, icons_mode);
                html.push_str(&format!(
                    r#"<li class="link" data-active="{is_active}"><a href="{route}"{}>{icon_html}<span class="label">{label}</span></a></li>"#,
                    if is_active { r#" aria-current="page""# } else { "" }
                ));
            }
            // Nested groups: render as a link if they have a route, otherwise skip
            NavItem::Group { label, route, .. } => {
                if let Some(group_route) = route {
                    let is_active = group_route == current_route;
                    html.push_str(&format!(
                        r#"<li class="link" data-active="{is_active}"><a href="{group_route}"{}><span class="label">{label}</span></a></li>"#,
                        if is_active { r#" aria-current="page""# } else { "" }
                    ));
                }
                // Note: We don't recurse into nested groups' children for nav-groups
                // as this is meant to be a flat display
            }
        }
    }

    html
}

/// Render an icon (respects icons mode)
fn render_icon(icon: &Option<String>, mode: &NavGroupsIcons) -> String {
    let Some(icon_name) = icon else {
        return String::new();
    };

    if matches!(mode, NavGroupsIcons::Show) {
        render_icon_span(icon_name)
    } else {
        String::new()
    }
}
