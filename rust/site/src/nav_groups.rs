//! Navigation groups component rendering
//!
//! Displays footer-style navigation with flat, grouped links under headings.
//! Top-level nav items become group headings, their children become links.
//! Uses CSS grid for responsive auto-columns.

use stencila_config::{AccessLevel, NavGroupsIcons, NavItem, SiteAccessConfig, SiteConfig};

use crate::nav_common::{
    apply_depth_limit, apply_icons, filter_nav_items, render_access_attr_if_more_restrictive,
    render_group_access_attr, render_icon_span,
};

/// Context for rendering nav groups
pub(crate) struct NavGroupsContext<'a> {
    pub site_config: &'a SiteConfig,
    pub route: &'a str,
    pub nav_items: &'a Vec<NavItem>,
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

    // Clone nav items from context (already resolved from site.nav or auto-generated)
    let nav_items = context.nav_items.clone();

    // Apply icons from site.icons if icons mode is Show
    let nav_items = if matches!(icons_mode, NavGroupsIcons::Show) {
        apply_icons(nav_items, &context.site_config.icons)
    } else {
        nav_items
    };

    // Apply filtering
    let nav_items = filter_nav_items(nav_items, include, exclude);

    // Apply depth limit (converts empty groups to links or removes them)
    let nav_items = apply_depth_limit(nav_items, Some(max_depth));

    // If no items, render empty component
    if nav_items.is_empty() {
        return "<stencila-nav-groups></stencila-nav-groups>".to_string();
    }

    // Render groups
    // Start with Public as inherited level - badges show when a route is MORE
    // restrictive than what's been indicated on ancestors (nothing at root)
    let groups_html = render_groups(
        &nav_items,
        context.route,
        &icons_mode,
        context.site_config.access.as_ref(),
        AccessLevel::Public,
    );

    format!(
        r#"<stencila-nav-groups><nav aria-label="Footer navigation"><div class="groups">{groups_html}</div></nav></stencila-nav-groups>"#
    )
}

/// Render navigation groups
fn render_groups(
    items: &[NavItem],
    current_route: &str,
    icons_mode: &NavGroupsIcons,
    access_config: Option<&SiteAccessConfig>,
    inherited_access: AccessLevel,
) -> String {
    let mut html = String::new();

    for item in items {
        match item {
            // A bare route at top level becomes a single-item group
            NavItem::Route(route) => {
                let label = crate::nav_common::route_to_label(route);
                let is_active = route == current_route;
                let (access_attr, _) =
                    render_access_attr_if_more_restrictive(route, inherited_access, access_config);
                html.push_str(&format!(
                    r#"<div class="group"><ul class="group-links"><li class="link" data-active="{is_active}"{access_attr}><a href="{route}"{}><span class="label">{label}</span></a></li></ul></div>"#,
                    if is_active { r#" aria-current="page""# } else { "" }
                ));
            }
            // A link at top level becomes a single-item group
            NavItem::Link {
                label, route, icon, ..
            } => {
                let is_active = route == current_route;
                let icon_html = render_icon(icon, icons_mode);
                let (access_attr, _) =
                    render_access_attr_if_more_restrictive(route, inherited_access, access_config);
                html.push_str(&format!(
                    r#"<div class="group"><ul class="group-links"><li class="link" data-active="{is_active}"{access_attr}><a href="{route}"{}>{icon_html}<span class="label">{label}</span></a></li></ul></div>"#,
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
                // Get access attr and inherited level for children
                // (derives route from label if not present)
                let (heading_access_attr, children_inherited) =
                    render_group_access_attr(route, label, inherited_access, access_config);

                // Render heading (optionally as a link)
                let heading_html = if let Some(group_route) = route {
                    let is_active = group_route == current_route;
                    format!(
                        r#"<h3 class="group-heading"{heading_access_attr}><a href="{group_route}"{}>{label}</a></h3>"#,
                        if is_active {
                            r#" aria-current="page""#
                        } else {
                            ""
                        }
                    )
                } else {
                    // Group without route - still add access attr to heading if restricted
                    format!(r#"<h3 class="group-heading"{heading_access_attr}>{label}</h3>"#)
                };

                // Render children as links (depth already applied)
                let links_html = render_links(
                    children,
                    current_route,
                    icons_mode,
                    access_config,
                    children_inherited,
                );

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
    icons_mode: &NavGroupsIcons,
    access_config: Option<&SiteAccessConfig>,
    inherited_access: AccessLevel,
) -> String {
    let mut html = String::new();

    for item in items {
        match item {
            NavItem::Route(route) => {
                let label = crate::nav_common::route_to_label(route);
                let is_active = route == current_route;
                let (access_attr, _) =
                    render_access_attr_if_more_restrictive(route, inherited_access, access_config);
                html.push_str(&format!(
                    r#"<li class="link" data-active="{is_active}"{access_attr}><a href="{route}"{}><span class="label">{label}</span></a></li>"#,
                    if is_active { r#" aria-current="page""# } else { "" }
                ));
            }
            NavItem::Link {
                label, route, icon, ..
            } => {
                let is_active = route == current_route;
                let icon_html = render_icon(icon, icons_mode);
                let (access_attr, _) =
                    render_access_attr_if_more_restrictive(route, inherited_access, access_config);
                html.push_str(&format!(
                    r#"<li class="link" data-active="{is_active}"{access_attr}><a href="{route}"{}>{icon_html}<span class="label">{label}</span></a></li>"#,
                    if is_active { r#" aria-current="page""# } else { "" }
                ));
            }
            // Nested groups: render as a link if they have a route, otherwise skip
            // Note: After apply_depth_limit, groups with empty children are already converted to links
            NavItem::Group { label, route, .. } => {
                if let Some(group_route) = route {
                    let is_active = group_route == current_route;
                    let (access_attr, _) = render_access_attr_if_more_restrictive(
                        group_route,
                        inherited_access,
                        access_config,
                    );
                    html.push_str(&format!(
                        r#"<li class="link" data-active="{is_active}"{access_attr}><a href="{group_route}"{}><span class="label">{label}</span></a></li>"#,
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
