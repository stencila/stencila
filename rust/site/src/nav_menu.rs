//! Navigation menu component rendering
//!
//! Displays horizontal navigation with mega-dropdown panels on desktop
//! and accordion-style menu on mobile.

use std::collections::HashMap;

use stencila_config::{
    FeaturedContent, NavItem, NavMenuDropdownStyle, NavMenuGroups, NavMenuIcons, NavMenuTrigger,
    SiteConfig,
};

use crate::{
    RouteEntry,
    nav_common::{
        apply_icons, auto_generate_nav, filter_nav_items, render_icon_span, route_to_label,
    },
};

/// Context for rendering navigation components
pub(crate) struct NavMenuContext<'a> {
    pub site_config: &'a SiteConfig,
    pub route: &'a str,
    pub routes: &'a [RouteEntry],
}

/// Render a navigation menu component
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_nav_menu(
    include: &Option<Vec<String>>,
    exclude: &Option<Vec<String>>,
    depth: &Option<u8>,
    groups: &Option<NavMenuGroups>,
    icons: &Option<NavMenuIcons>,
    descriptions: &Option<bool>,
    trigger: &Option<NavMenuTrigger>,
    dropdown_style: &Option<NavMenuDropdownStyle>,
    hover_delay: &Option<u16>,
    close_delay: &Option<u16>,
    mobile_breakpoint: &Option<u16>,
    context: &NavMenuContext,
) -> String {
    // Get config values with defaults
    let groups = groups.unwrap_or_default();
    let icons_mode = icons.unwrap_or_default();
    let descriptions = descriptions.unwrap_or(true);
    let trigger = trigger.unwrap_or_default();
    let dropdown_style = dropdown_style.unwrap_or_default();
    let hover_delay = hover_delay.unwrap_or(150);
    let close_delay = close_delay.unwrap_or(300);
    let mobile_breakpoint = mobile_breakpoint.unwrap_or(1024);

    // Resolve nav items: site.nav or auto-generated
    let nav_items = context
        .site_config
        .nav
        .clone()
        .unwrap_or_else(|| auto_generate_nav(context.routes, depth));

    // Apply icons from site.icons
    let nav_items = apply_icons(nav_items, &context.site_config.icons);

    // Apply filtering
    let nav_items = filter_nav_items(nav_items, include, exclude);

    // Apply depth limit
    let nav_items = if depth.is_some() {
        limit_nav_depth(nav_items, 1, depth)
    } else {
        nav_items
    };

    // Remove groups with no route and no children (can occur after depth limiting)
    let nav_items = remove_empty_groups(nav_items);

    // If no items, render empty component
    if nav_items.is_empty() {
        return "<stencila-nav-menu></stencila-nav-menu>".to_string();
    }

    // Render nav items for the menu
    let items_html = render_nav_menu_items(
        &nav_items,
        context.route,
        &groups,
        &icons_mode,
        descriptions,
        &context.site_config.featured,
    );

    // Build attributes
    let attrs = format!(
        r#" groups="{groups}" icons="{icons_mode}" descriptions="{descriptions}" trigger="{trigger}" dropdown-style="{dropdown_style}" hover-delay="{hover_delay}" close-delay="{close_delay}" mobile-breakpoint="{mobile_breakpoint}""#
    );

    format!(
        r#"<stencila-nav-menu{attrs}><nav aria-label="Main navigation"><ul class="list">{items_html}</ul></nav></stencila-nav-menu>"#
    )
}

/// Limit nav items to a maximum depth
pub(crate) fn limit_nav_depth(
    items: Vec<NavItem>,
    current_level: u8,
    max_depth: &Option<u8>,
) -> Vec<NavItem> {
    let Some(max) = max_depth else {
        return items;
    };

    if current_level > *max {
        return Vec::new();
    }

    items
        .into_iter()
        .map(|item| match item {
            NavItem::Group {
                id,
                label,
                route,
                children,
                icon,
                section_title,
            } => {
                let limited_children = if current_level < *max {
                    limit_nav_depth(children, current_level + 1, max_depth)
                } else {
                    Vec::new()
                };
                NavItem::Group {
                    id,
                    label,
                    route,
                    children: limited_children,
                    icon,
                    section_title,
                }
            }
            other => other,
        })
        .collect()
}

/// Remove groups that have no route and no children
/// These can occur after depth limiting or filtering
pub(crate) fn remove_empty_groups(items: Vec<NavItem>) -> Vec<NavItem> {
    items
        .into_iter()
        .filter_map(|item| match item {
            NavItem::Group {
                id,
                label,
                route,
                children,
                icon,
                section_title,
            } => {
                // Recursively remove empty groups from children
                let filtered_children = remove_empty_groups(children);

                // Keep group only if it has a route OR has children
                if route.is_some() || !filtered_children.is_empty() {
                    Some(NavItem::Group {
                        id,
                        label,
                        route,
                        children: filtered_children,
                        icon,
                        section_title,
                    })
                } else {
                    None
                }
            }
            other => Some(other),
        })
        .collect()
}

/// Render nav menu items (top-level horizontal bar)
fn render_nav_menu_items(
    items: &[NavItem],
    current_route: &str,
    groups: &NavMenuGroups,
    icons_mode: &NavMenuIcons,
    descriptions: bool,
    featured: &Option<HashMap<String, FeaturedContent>>,
) -> String {
    let mut html = String::new();

    for (index, item) in items.iter().enumerate() {
        match item {
            NavItem::Route(route) => {
                let is_active = route == current_route;
                let label = route_to_label(route);
                html.push_str(&format!(
                    r#"<li class="item" data-type="link"><a href="{route}"{}>{label}</a></li>"#,
                    if is_active {
                        r#" aria-current="page""#
                    } else {
                        ""
                    }
                ));
            }
            NavItem::Link {
                label, route, icon, ..
            } => {
                let is_active = route == current_route;
                let icon_html = render_menu_icon(icon, icons_mode, false);
                html.push_str(&format!(
                    r#"<li class="item" data-type="link"><a href="{route}"{}>{icon_html}{label}</a></li>"#,
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
                // Determine if this group should render as dropdown or link
                let render_as_dropdown = match groups {
                    NavMenuGroups::Dropdowns => true,
                    NavMenuGroups::Links => false,
                    NavMenuGroups::Auto => !children.is_empty(),
                };

                if render_as_dropdown && !children.is_empty() {
                    // Render as dropdown
                    let dropdown_id = format!(
                        "dropdown-{index}-{}",
                        label.to_lowercase().replace(' ', "-")
                    );
                    let icon_html = render_menu_icon(icon, icons_mode, false);

                    // Build dropdown panel content
                    let dropdown_html = render_nav_menu_dropdown(
                        children,
                        current_route,
                        icons_mode,
                        descriptions,
                        label,
                        featured,
                    );

                    html.push_str(&format!(
                        r#"<li class="item" data-type="dropdown"><button class="trigger" aria-expanded="false" aria-controls="{dropdown_id}">{icon_html}{label}<span class="chevron"></span></button><div id="{dropdown_id}" class="dropdown">{dropdown_html}</div></li>"#
                    ));
                } else if let Some(group_route) = route {
                    // Render as link
                    let is_active = group_route == current_route;
                    let icon_html = render_menu_icon(icon, icons_mode, false);
                    html.push_str(&format!(
                        r#"<li class="item" data-type="link"><a href="{group_route}"{}>{icon_html}{label}</a></li>"#,
                        if is_active { r#" aria-current="page""# } else { "" }
                    ));
                }
                // If groups="links" and no route, skip (validation should catch this)
            }
        }
    }

    html
}

/// Render an icon for the menu (respects icons mode)
/// In dropdowns, returns a placeholder span when no icon to maintain alignment
fn render_menu_icon(icon: &Option<String>, mode: &NavMenuIcons, in_dropdown: bool) -> String {
    let should_show = match mode {
        NavMenuIcons::Show => true,
        NavMenuIcons::Hide => false,
        NavMenuIcons::DropdownsOnly => in_dropdown,
    };

    if !should_show {
        return String::new();
    }

    match icon {
        Some(icon_name) => render_icon_span(icon_name),
        // In dropdowns, render placeholder for alignment when no icon specified
        None if in_dropdown => r#"<span class="icon"></span>"#.to_string(),
        None => String::new(),
    }
}

/// Render a dropdown panel for a nav menu group
fn render_nav_menu_dropdown(
    children: &[NavItem],
    current_route: &str,
    icons_mode: &NavMenuIcons,
    descriptions: bool,
    group_label: &str,
    featured: &Option<HashMap<String, FeaturedContent>>,
) -> String {
    let mut main_html = String::new();

    // Group children by section_title if present
    let mut current_section: Option<String> = None;
    let mut section_items = String::new();

    for item in children {
        let item_section = match item {
            NavItem::Group { section_title, .. } => section_title.clone(),
            _ => None,
        };

        // If section changed, flush previous section
        if item_section != current_section && !section_items.is_empty() {
            main_html.push_str(&wrap_section(&current_section, &section_items));
            section_items.clear();
        }
        current_section = item_section;

        // Render the item
        match item {
            NavItem::Route(route) => {
                let is_active = route == current_route;
                let label = route_to_label(route);
                // Route items have no icon, but need placeholder for alignment
                let icon_html = render_menu_icon(&None, icons_mode, true);
                section_items.push_str(&format!(
                    r#"<li class="dropdown-item"><a href="{route}"{}>{icon_html}<span class="content"><span class="label">{label}</span></span></a></li>"#,
                    if is_active { r#" aria-current="page""# } else { "" }
                ));
            }
            NavItem::Link {
                label,
                route,
                icon,
                description,
                ..
            } => {
                let is_active = route == current_route;
                let icon_html = render_menu_icon(icon, icons_mode, true);
                let desc_html = if descriptions {
                    description
                        .as_ref()
                        .map(|d| format!(r#"<span class="description">{d}</span>"#))
                        .unwrap_or_default()
                } else {
                    String::new()
                };
                section_items.push_str(&format!(
                    r#"<li class="dropdown-item"><a href="{route}"{}>{icon_html}<span class="content"><span class="label">{label}</span>{desc_html}</span></a></li>"#,
                    if is_active { r#" aria-current="page""# } else { "" }
                ));
            }
            NavItem::Group {
                label,
                route,
                icon,
                children: nested_children,
                ..
            } => {
                // Nested groups in dropdown - render as nested list (not full dropdown)
                if !nested_children.is_empty() {
                    let nested_items =
                        render_nested_dropdown_items(nested_children, current_route, icons_mode);
                    // Only render if nested_items produced output
                    if !nested_items.is_empty() {
                        section_items.push_str(&format!(
                            r#"<li class="dropdown-group"><span class="section-title">{label}</span><ul class="nested-list">{nested_items}</ul></li>"#
                        ));
                    } else if let Some(group_route) = route {
                        // Children produced no output but group has route - render as link
                        let is_active = group_route == current_route;
                        let icon_html = render_menu_icon(icon, icons_mode, true);
                        section_items.push_str(&format!(
                            r#"<li class="dropdown-item"><a href="{group_route}"{}>{icon_html}<span class="label">{label}</span></a></li>"#,
                            if is_active { r#" aria-current="page""# } else { "" }
                        ));
                    }
                    // Else: no nested output and no route - skip entirely
                } else if let Some(group_route) = route {
                    // Group with route but no children - render as link
                    let is_active = group_route == current_route;
                    let icon_html = render_menu_icon(icon, icons_mode, true);
                    section_items.push_str(&format!(
                        r#"<li class="dropdown-item"><a href="{group_route}"{}>{icon_html}<span class="label">{label}</span></a></li>"#,
                        if is_active { r#" aria-current="page""# } else { "" }
                    ));
                }
                // Else: no children and no route - skip entirely
            }
        }
    }

    // Flush remaining section
    if !section_items.is_empty() {
        main_html.push_str(&wrap_section(&current_section, &section_items));
    }

    // Wrap main content - main_html contains section divs (each with their own ul)
    // or a single ul for unsectioned items
    let main_content = format!(r#"<div class="dropdown-main">{main_html}</div>"#);

    // Add featured content if available
    let featured_html = render_featured_content(group_label, featured);

    format!("{main_content}{featured_html}")
}

/// Render nested items in a dropdown (simplified, no sections/featured)
fn render_nested_dropdown_items(
    items: &[NavItem],
    current_route: &str,
    icons_mode: &NavMenuIcons,
) -> String {
    let mut html = String::new();

    for item in items {
        match item {
            NavItem::Route(route) => {
                let is_active = route == current_route;
                let label = route_to_label(route);
                // Route items have no icon, but need placeholder for alignment
                let icon_html = render_menu_icon(&None, icons_mode, true);
                html.push_str(&format!(
                    r#"<li class="dropdown-item"><a href="{route}"{}>{icon_html}<span class="label">{label}</span></a></li>"#,
                    if is_active { r#" aria-current="page""# } else { "" }
                ));
            }
            NavItem::Link {
                label, route, icon, ..
            } => {
                let is_active = route == current_route;
                let icon_html = render_menu_icon(icon, icons_mode, true);
                html.push_str(&format!(
                    r#"<li class="dropdown-item"><a href="{route}"{}>{icon_html}<span class="label">{label}</span></a></li>"#,
                    if is_active { r#" aria-current="page""# } else { "" }
                ));
            }
            NavItem::Group {
                label,
                route,
                icon,
                children,
                ..
            } => {
                if let Some(group_route) = route {
                    let is_active = group_route == current_route;
                    let icon_html = render_menu_icon(icon, icons_mode, true);
                    html.push_str(&format!(
                        r#"<li class="dropdown-item"><a href="{group_route}"{}>{icon_html}<span class="label">{label}</span></a></li>"#,
                        if is_active { r#" aria-current="page""# } else { "" }
                    ));
                }
                // Skip deeply nested children - only render the group as a link if it has a route
                let _ = children; // Explicitly ignore further nesting
            }
        }
    }

    html
}

/// Wrap items in a section div with optional title
/// Always wraps items in a ul for valid HTML structure
fn wrap_section(title: &Option<String>, items: &str) -> String {
    if let Some(t) = title {
        format!(
            r#"<div class="section"><h3 class="section-title">{t}</h3><ul class="section-list">{items}</ul></div>"#
        )
    } else {
        // No title - wrap in ul without section div
        format!(r#"<ul class="section-list">{items}</ul>"#)
    }
}

/// Render featured content for a dropdown (from site.featured)
fn render_featured_content(
    group_label: &str,
    featured: &Option<HashMap<String, FeaturedContent>>,
) -> String {
    let Some(featured_map) = featured else {
        return String::new();
    };

    // Try to find featured content by label
    let content = featured_map.get(group_label);

    let Some(content) = content else {
        return String::new();
    };

    let image_html = content
        .image
        .as_ref()
        .map(|src| format!(r#"<img src="{src}" alt="" class="featured-image">"#))
        .unwrap_or_default();

    let desc_html = content
        .description
        .as_ref()
        .map(|d| format!(r#"<p class="featured-description">{d}</p>"#))
        .unwrap_or_default();

    let cta_html = content
        .cta
        .as_ref()
        .map(|cta| {
            format!(
                r#"<a href="{}" class="featured-cta">{}</a>"#,
                cta.route, cta.label
            )
        })
        .unwrap_or_default();

    format!(
        r#"<aside class="featured">{image_html}<h4 class="featured-title">{}</h4>{desc_html}{cta_html}</aside>"#,
        content.title
    )
}
