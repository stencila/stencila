//! Navigation menu component rendering
//!
//! Displays horizontal navigation with mega-dropdown panels on desktop
//! and accordion-style menu on mobile.

use std::{collections::HashMap, path::Path};

use stencila_config::{
    FeaturedContent, NavItem, NavMenuDropdownStyle, NavMenuGroups, NavMenuIcons, NavMenuTrigger,
    SiteConfig,
};

use crate::{
    RouteEntry,
    nav_common::{
        apply_descriptions, apply_icons, auto_generate_nav, filter_nav_items, label_to_segment,
        normalize_icon_name, render_icon_span, route_to_label,
    },
};

/// Context for rendering navigation components
pub(crate) struct NavMenuContext<'a> {
    pub site_config: &'a SiteConfig,
    pub site_root: &'a Path,
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
    context: &NavMenuContext,
) -> String {
    // Get config values with defaults
    let groups = groups.unwrap_or_default();
    let icons_mode = icons.unwrap_or_default();
    let descriptions = descriptions.unwrap_or(true);
    let trigger = trigger.unwrap_or_default();
    let dropdown_style = dropdown_style.unwrap_or_default();

    // Resolve nav items: site.nav or auto-generated
    let nav_items = context
        .site_config
        .nav
        .clone()
        .unwrap_or_else(|| auto_generate_nav(context.routes, depth, Some(context.site_root)));

    // Apply icons from site.icons
    let nav_items = apply_icons(nav_items, &context.site_config.icons);

    // Apply descriptions from site.descriptions
    let nav_items = apply_descriptions(nav_items, &context.site_config.descriptions);

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
        r#" groups="{groups}" icons="{icons_mode}" descriptions="{descriptions}" trigger="{trigger}" dropdown-style="{dropdown_style}""#
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
                let icon_html = render_menu_icon(icon, icons_mode, false, false);
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
                    let icon_html = render_menu_icon(icon, icons_mode, false, false);

                    // Build dropdown panel content
                    let dropdown_html = render_nav_menu_dropdown(
                        children,
                        current_route,
                        icons_mode,
                        descriptions,
                        label,
                        route,
                        featured,
                    );

                    html.push_str(&format!(
                        r#"<li class="item" data-type="dropdown"><button class="trigger" aria-expanded="false" aria-controls="{dropdown_id}">{icon_html}{label}<span class="chevron"></span></button><div id="{dropdown_id}" class="dropdown">{dropdown_html}</div></li>"#
                    ));
                } else if let Some(group_route) = route {
                    // Render as link
                    let is_active = group_route == current_route;
                    let icon_html = render_menu_icon(icon, icons_mode, false, false);
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

/// Check if any items in a list have icons
fn any_items_have_icons(items: &[NavItem]) -> bool {
    items.iter().any(|item| match item {
        NavItem::Link { icon, .. } | NavItem::Group { icon, .. } => icon.is_some(),
        NavItem::Route(_) => false,
    })
}

/// Render an icon for the menu (respects icons mode)
/// In dropdowns, returns a placeholder span when no icon to maintain alignment,
/// but only if other items in the group have icons (group_has_icons=true)
fn render_menu_icon(
    icon: &Option<String>,
    mode: &NavMenuIcons,
    in_dropdown: bool,
    group_has_icons: bool,
) -> String {
    let should_show = match mode {
        NavMenuIcons::Show => true,
        NavMenuIcons::Hide => false,
        NavMenuIcons::Dropdowns => in_dropdown,
    };

    if !should_show {
        return String::new();
    }

    match icon {
        Some(icon_name) => render_icon_span(icon_name),
        // In dropdowns, render placeholder for alignment only when other items have icons
        None if in_dropdown && group_has_icons => r#"<span class="icon"></span>"#.to_string(),
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
    group_route: &Option<String>,
    featured: &Option<HashMap<String, FeaturedContent>>,
) -> String {
    let mut main_html = String::new();

    // Check if any children have icons (for alignment purposes)
    let has_icons = any_items_have_icons(children);

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
                // Route items have no icon, but need placeholder for alignment if others have icons
                let icon_html = render_menu_icon(&None, icons_mode, true, has_icons);
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
                let icon_html = render_menu_icon(icon, icons_mode, true, has_icons);
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
                    let nested_items = render_nested_dropdown_items(
                        nested_children,
                        current_route,
                        icons_mode,
                        descriptions,
                    );
                    // Only render if nested_items produced output
                    if !nested_items.is_empty() {
                        section_items.push_str(&format!(
                            r#"<li class="dropdown-group"><span class="section-title">{label}</span><ul class="nested-list">{nested_items}</ul></li>"#
                        ));
                    } else if let Some(group_route) = route {
                        // Children produced no output but group has route - render as link
                        let is_active = group_route == current_route;
                        let icon_html = render_menu_icon(icon, icons_mode, true, has_icons);
                        section_items.push_str(&format!(
                            r#"<li class="dropdown-item"><a href="{group_route}"{}>{icon_html}<span class="label">{label}</span></a></li>"#,
                            if is_active { r#" aria-current="page""# } else { "" }
                        ));
                    }
                    // Else: no nested output and no route - skip entirely
                } else if let Some(group_route) = route {
                    // Group with route but no children - render as link
                    let is_active = group_route == current_route;
                    let icon_html = render_menu_icon(icon, icons_mode, true, has_icons);
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
    let featured_html = render_featured_content(group_route, group_label, featured);

    format!("{main_content}{featured_html}")
}

/// Render nested items in a dropdown (simplified, no sections/featured)
fn render_nested_dropdown_items(
    items: &[NavItem],
    current_route: &str,
    icons_mode: &NavMenuIcons,
    descriptions: bool,
) -> String {
    let mut html = String::new();

    // Check if any items have icons (for alignment purposes)
    let has_icons = any_items_have_icons(items);

    for item in items {
        match item {
            NavItem::Route(route) => {
                let is_active = route == current_route;
                let label = route_to_label(route);
                // Route items have no icon, but need placeholder for alignment if others have icons
                let icon_html = render_menu_icon(&None, icons_mode, true, has_icons);
                html.push_str(&format!(
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
                let icon_html = render_menu_icon(icon, icons_mode, true, has_icons);
                let desc_html = if descriptions {
                    description
                        .as_ref()
                        .map(|d| format!(r#"<span class="description">{d}</span>"#))
                        .unwrap_or_default()
                } else {
                    String::new()
                };
                html.push_str(&format!(
                    r#"<li class="dropdown-item"><a href="{route}"{}>{icon_html}<span class="content"><span class="label">{label}</span>{desc_html}</span></a></li>"#,
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
                    let icon_html = render_menu_icon(icon, icons_mode, true, has_icons);
                    html.push_str(&format!(
                        r#"<li class="dropdown-item"><a href="{group_route}"{}>{icon_html}<span class="content"><span class="label">{label}</span></span></a></li>"#,
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
///
/// Lookup priority (same as icons/descriptions):
/// 1. Route (with slash normalization)
/// 2. Label
/// 3. Derived route from label (e.g., "Getting Started" -> "/getting-started/")
fn render_featured_content(
    group_route: &Option<String>,
    group_label: &str,
    featured: &Option<HashMap<String, FeaturedContent>>,
) -> String {
    let Some(featured_map) = featured else {
        return String::new();
    };

    // Try to find featured content using the same lookup pattern as icons/descriptions
    let content = lookup_featured(group_route, group_label, featured_map);

    let Some(content) = content else {
        return String::new();
    };

    // Badge text (e.g., "Featured", "New", "Spotlight")
    let badge_html = content
        .badge
        .as_ref()
        .map(|b| format!(r#"<span class="featured-badge">{b}</span>"#))
        .unwrap_or_default();

    // Icon with accent background
    let icon_html = content
        .icon
        .as_ref()
        .map(|icon| {
            let icon_class = format!("i-{}", normalize_icon_name(icon));
            format!(r#"<span class="featured-icon {icon_class}"></span>"#)
        })
        .unwrap_or_default();

    let image_html = content
        .image
        .as_ref()
        .map(|src| {
            // Ensure path is absolute so it resolves from site root (unless external URL)
            let src = if src.starts_with('/')
                || src.starts_with("http://")
                || src.starts_with("https://")
            {
                src.to_string()
            } else {
                format!("/{src}")
            };
            format!(r#"<img src="{src}" alt="" class="featured-image">"#)
        })
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
        r#"<aside class="featured">{badge_html}{icon_html}{image_html}<h4 class="featured-title">{}</h4>{desc_html}{cta_html}</aside>"#,
        content.title
    )
}

/// Look up featured content from the map, trying route first, then label
fn lookup_featured<'a>(
    route: &Option<String>,
    label: &str,
    featured: &'a HashMap<String, FeaturedContent>,
) -> Option<&'a FeaturedContent> {
    // Try route first (with and without trailing slash normalization)
    if let Some(r) = route
        && let Some(content) = lookup_featured_by_route(r, featured)
    {
        return Some(content);
    }

    // Try label as-is
    if let Some(content) = featured.get(label) {
        return Some(content);
    }

    // Try deriving a route from the label (for groups without explicit routes)
    // Convert "Getting Started" -> "/getting-started/" and try that
    let derived_route = format!("/{}/", label_to_segment(label));
    lookup_featured_by_route(&derived_route, featured)
}

/// Try to find featured content by route, with various normalizations
///
/// Tries multiple key formats to find a match:
/// - Exact route, without/with trailing slash
/// - Without leading slash (e.g., "docs/config" for "/docs/config/")
/// - Bare segment (e.g., "docs" for "/docs/")
fn lookup_featured_by_route<'a>(
    route: &str,
    featured: &'a HashMap<String, FeaturedContent>,
) -> Option<&'a FeaturedContent> {
    // Try exact route
    if let Some(content) = featured.get(route) {
        return Some(content);
    }

    // Normalize: trim trailing slash
    let normalized = route.trim_end_matches('/');
    if let Some(content) = featured.get(normalized) {
        return Some(content);
    }

    // Try with trailing slash
    let with_trailing = format!("{normalized}/");
    if let Some(content) = featured.get(&with_trailing) {
        return Some(content);
    }

    // Try without leading slash (e.g., "/docs/config/" -> "docs/config/", "docs/config")
    let without_leading = normalized.trim_start_matches('/');
    if without_leading != normalized {
        if let Some(content) = featured.get(without_leading) {
            return Some(content);
        }
        let with_trailing = format!("{without_leading}/");
        if let Some(content) = featured.get(&with_trailing) {
            return Some(content);
        }
    }

    // Try bare segment (e.g., "/docs/" -> "docs", "/docs/config/" -> "config")
    let segment = without_leading.rsplit('/').next().unwrap_or("");
    if !segment.is_empty()
        && let Some(content) = featured.get(segment)
    {
        return Some(content);
    }

    None
}
