use std::path::Path;

use stencila_config::{
    ColorModeStyle, ComponentConfig, ComponentSpec, LayoutConfig, LogoConfig, NavItem,
    NavTreeExpanded, PrevNextStyle, RegionSpec, SiteConfig,
};

use crate::{RouteEntry, logo};

struct RenderContext<'a> {
    site_config: &'a SiteConfig,
    site_root: &'a Path,
    route: &'a str,
    routes: &'a [RouteEntry],
}

/// Render a Stencila site layout for a specific route
///
/// # Arguments
/// * `site_config` - Site configuration
/// * `site_root` - Path to the site root directory
/// * `route` - Current route being rendered
/// * `routes` - All document routes for prev/next navigation etc
pub(crate) fn render_layout(
    site_config: &SiteConfig,
    site_root: &Path,
    route: &str,
    routes: &[RouteEntry],
) -> String {
    let context = RenderContext {
        site_config,
        site_root,
        route,
        routes,
    };

    // Resolve the config for the route
    let resolved = site_config
        .layout
        .as_ref()
        .map(|layout| layout.resolve_for_route(route))
        .unwrap_or_default();

    // Render regions
    let mut regions_enabled = String::new();
    let header = render_region("header", &resolved.header, &mut regions_enabled, &context);
    let left_sidebar = render_region(
        "left-sidebar",
        &resolved.left_sidebar,
        &mut regions_enabled,
        &context,
    );
    let top = render_region("top", &resolved.top, &mut regions_enabled, &context);
    let bottom = render_region("bottom", &resolved.bottom, &mut regions_enabled, &context);
    let right_sidebar = render_region(
        "right-sidebar",
        &resolved.right_sidebar,
        &mut regions_enabled,
        &context,
    );
    let footer = render_region("footer", &resolved.footer, &mut regions_enabled, &context);

    // Build responsive configuration attributes
    let responsive_attrs = build_responsive_attributes(&resolved);

    format!(
        r##"<stencila-layout{regions_enabled}{responsive_attrs}>
  <a href="#main-content" class="skip-link">Skip to content</a>
  {header}
  <div class="layout-body">
    {left_sidebar}
    <div class="layout-main">
      {top}
      <main id="main-content" tabindex="-1"><!--MAIN CONTENT--></main>
      {bottom}
    </div>
    {right_sidebar}
  </div>
  {footer}
</stencila-layout>"##
    )
}

/// Build responsive configuration attributes for the layout element
///
/// Returns attributes for:
/// - `collapse-breakpoint`: Custom breakpoint (only if not default 1024)
/// - `left-sidebar-collapsible="false"`: When left sidebar should not collapse
/// - `right-sidebar-collapsible="false"`: When right sidebar should not collapse
fn build_responsive_attributes(config: &LayoutConfig) -> String {
    let mut attrs = String::new();

    // Get global responsive settings
    let global_breakpoint = config
        .responsive
        .as_ref()
        .and_then(|r| r.breakpoint)
        .unwrap_or(1024);

    // Add breakpoint attribute if not default
    if global_breakpoint != 1024 {
        attrs.push_str(&format!(" collapse-breakpoint=\"{global_breakpoint}\""));
    }

    // Check left sidebar collapsible setting
    if let Some(RegionSpec::Config(left_config)) = &config.left_sidebar
        && let Some(responsive) = &left_config.responsive
        && responsive.collapsible == Some(false)
    {
        attrs.push_str(" left-sidebar-collapsible=\"false\"");
    }

    // Check right sidebar collapsible setting
    if let Some(RegionSpec::Config(right_config)) = &config.right_sidebar
        && let Some(responsive) = &right_config.responsive
        && responsive.collapsible == Some(false)
    {
        attrs.push_str(" right-sidebar-collapsible=\"false\"");
    }

    attrs
}

/// Render a layout region (returns empty string if not enabled)
fn render_region(
    name: &str,
    spec: &Option<RegionSpec>,
    regions_enabled: &mut String,
    context: &RenderContext,
) -> String {
    let Some(spec) = spec else {
        return String::new();
    };
    if !spec.is_enabled() {
        return String::new();
    }

    // Record as an enabled region
    regions_enabled.push(' ');
    regions_enabled.push_str(name);

    // Render subregions
    let subregions = if let Some(config) = spec.config() {
        let start = render_subregion(&config.start, context);
        let middle = render_subregion(&config.middle, context);
        let end = render_subregion(&config.end, context);
        format!(
            r#"<div data-subregion="start">{start}</div><div data-subregion="middle">{middle}</div><div data-subregion="end">{end}</div>"#
        )
    } else {
        String::new()
    };

    // Sidebars get an inner wrapper for sticky positioning
    // (outer element stretches for background, inner element is sticky)
    if name == "left-sidebar" || name == "right-sidebar" {
        format!(
            r#"<stencila-{name}><div class="sidebar-content">{subregions}</div></stencila-{name}>"#
        )
    } else {
        format!(r#"<stencila-{name}>{subregions}</stencila-{name}>"#)
    }
}

/// Render a layout subregion (returns empty string if not enabled or no components)
fn render_subregion(components: &Option<Vec<ComponentSpec>>, context: &RenderContext) -> String {
    let Some(components) = components else {
        return String::new();
    };

    let mut html = String::new();
    for component in components {
        html.push_str(&render_component_spec(component, context));
    }

    html
}

/// Render a component spec
fn render_component_spec(component: &ComponentSpec, context: &RenderContext) -> String {
    match component {
        ComponentSpec::Name(name) => match name.as_str() {
            "breadcrumbs" => render_breadcrumbs(context),
            "copyright" => render_copyright(&None, &None, &None, &None, context),
            "logo" => render_logo(None, context),
            "nav-tree" => render_nav_tree(&None, &None, &None, &None, &None, context),
            "prev-next" => render_prev_next(&None, &None, &None, &None, context),
            "title" => render_title(&None, context),
            "toc-tree" => render_toc_tree(&None, &None),
            _ => format!("<stencila-{name}></stencila-{name}>"),
        },
        ComponentSpec::Config(config) => render_component_config(config, context),
    }
}

/// Render a component config
fn render_component_config(component: &ComponentConfig, context: &RenderContext) -> String {
    match component {
        ComponentConfig::Breadcrumbs => render_breadcrumbs(context),
        ComponentConfig::ColorMode { style } => render_color_mode(style),
        ComponentConfig::Copyright {
            text,
            holder,
            start_year,
            link,
        } => render_copyright(text, holder, start_year, link, context),
        ComponentConfig::Logo(config) => render_logo(Some(config), context),
        ComponentConfig::NavTree {
            title,
            depth,
            collapsible,
            expanded,
            scroll_to_active,
        } => render_nav_tree(
            title,
            depth,
            collapsible,
            expanded,
            scroll_to_active,
            context,
        ),
        ComponentConfig::PrevNext {
            style,
            prev_text,
            next_text,
            separator,
        } => render_prev_next(style, prev_text, next_text, separator, context),
        ComponentConfig::Title { text } => render_title(text, context),
        ComponentConfig::TocTree { title, depth } => render_toc_tree(title, depth),
    }
}

/// Render a logo component
///
/// When component_config is None, uses site-level logo config.
/// When provided, merges component config with site config.
fn render_logo(logo_config: Option<&LogoConfig>, context: &RenderContext) -> String {
    if let Some(config) = logo::resolve_logo(
        logo_config,
        context.site_config.logo.as_ref(),
        Some(context.site_root),
    ) {
        logo::render_logo(&config)
    } else {
        // Empty logo placeholder when no config available
        "<stencila-logo></stencila-logo>".to_string()
    }
}

/// Render a breadcrumbs component
///
/// Generates semantic HTML for breadcrumb navigation based on the route path.
/// For example, `/docs/guide/getting-started/` generates:
///   Home > Docs > Guide > Getting Started
fn render_breadcrumbs(context: &RenderContext) -> String {
    // Parse the route into segments
    let segments: Vec<&str> = context
        .route
        .trim_matches('/')
        .split('/')
        .filter(|s| !s.is_empty())
        .collect();

    // Build the breadcrumb list items
    let mut items = String::new();
    let mut current_path = String::new();

    // Home link (always first)
    items.push_str(r#"<li><a href="/">Home</a></li>"#);

    // Add intermediate segments as links
    let segment_count = segments.len();
    for (index, segment) in segments.iter().enumerate() {
        current_path.push('/');
        current_path.push_str(segment);

        // Convert segment to title case (e.g., "getting-started" -> "Getting Started")
        let label = segment_to_label(segment);

        if index == segment_count - 1 {
            // Last segment: current page (no link)
            items.push_str(&format!(r#"<li aria-current="page">{label}</li>"#));
        } else {
            // Intermediate segment: link to parent
            items.push_str(&format!(
                r#"<li><a href="{current_path}/">{label}</a></li>"#
            ));
        }
    }

    format!(
        r#"<stencila-breadcrumbs><nav aria-label="Breadcrumb"><ol>{items}</ol></nav></stencila-breadcrumbs>"#
    )
}

/// Render a color mode component
fn render_color_mode(style: &Option<ColorModeStyle>) -> String {
    format!(
        "<stencila-color-mode{}></stencila-color-mode>",
        match style {
            Some(style) => format!(" style={style}"),
            None => String::new(),
        },
    )
}

/// Render a copyright component
///
/// Generates a copyright notice with auto-updating year support.
/// When `text` is provided, uses it verbatim.
/// Otherwise, generates: © {start_year?}-{current_year} {holder}
fn render_copyright(
    text: &Option<String>,
    holder: &Option<String>,
    start_year: &Option<u16>,
    link: &Option<String>,
    context: &RenderContext,
) -> String {
    // If custom text is provided, use it verbatim (no auto-year)
    if let Some(custom_text) = text {
        return format!(
            r#"<stencila-copyright><span class="text">{custom_text}</span></stencila-copyright>"#
        );
    }

    // Get current year
    let current_year = chrono::Utc::now().format("%Y").to_string();

    // Build year string with data attributes for client-side updates
    let year_html = if let Some(start) = start_year {
        format!(
            r#"<span class="year" data-start="{start}" data-end="{current_year}">{start}-{current_year}</span>"#
        )
    } else {
        format!(r#"<span class="year" data-end="{current_year}">{current_year}</span>"#)
    };

    // Resolve holder: component holder > site.author > empty
    let holder_name = holder
        .clone()
        .or_else(|| context.site_config.author.as_ref().map(|a| a.name()))
        .unwrap_or_default();

    // Build holder HTML (with optional link)
    let holder_html = if holder_name.is_empty() {
        String::new()
    } else if let Some(url) = link {
        format!(r#" <a class="holder" href="{url}">{holder_name}</a>"#)
    } else {
        format!(r#" <span class="holder">{holder_name}</span>"#)
    };

    format!(
        r#"<stencila-copyright><span class="symbol">©</span> {year_html}{holder_html}</stencila-copyright>"#
    )
}

/// Render a prev/next navigation component
///
/// Generates navigation links to previous and next pages based on the document routes.
/// When `site.nav` is configured, follows that order for consistency with nav-tree.
/// Otherwise, uses the default route order.
/// The style controls what information is shown (icons, labels, titles, position).
fn render_prev_next(
    style: &Option<PrevNextStyle>,
    prev_text: &Option<String>,
    next_text: &Option<String>,
    separator: &Option<String>,
    context: &RenderContext,
) -> String {
    // Get navigation order: use site.nav if available, else default route order
    let nav_routes = get_nav_order(context);

    // Find current route index in the navigation order
    let current_idx = nav_routes.iter().position(|r| *r == context.route);
    let Some(idx) = current_idx else {
        return "<stencila-prev-next></stencila-prev-next>".to_string();
    };

    // Find prev/next routes and look up their RouteEntry for title
    let prev_route = if idx > 0 {
        Some(&nav_routes[idx - 1])
    } else {
        None
    };
    let next_route = if idx < nav_routes.len() - 1 {
        Some(&nav_routes[idx + 1])
    } else {
        None
    };

    // Look up RouteEntry for each (to get title)
    let prev = prev_route.and_then(|r| context.routes.iter().find(|re| re.route == *r));
    let next = next_route.and_then(|r| context.routes.iter().find(|re| re.route == *r));

    // If neither prev nor next exists, render empty component
    if prev.is_none() && next.is_none() {
        return "<stencila-prev-next></stencila-prev-next>".to_string();
    }

    // Get config values with defaults
    let style = style.unwrap_or_default();
    let prev_label = prev_text.clone().unwrap_or_else(|| "Previous".to_string());
    let next_label = next_text.clone().unwrap_or_else(|| "Next".to_string());
    // Use nav_routes length for correct page position when site.nav is configured
    let total_pages = nav_routes.len();
    let current_page = idx + 1;

    // Derive what to show from style
    let (show_labels, show_titles, show_position) = match style {
        PrevNextStyle::Minimal => (false, false, false),
        PrevNextStyle::Compact => (true, false, false),
        PrevNextStyle::Standard => (true, true, false),
        PrevNextStyle::Detailed => (true, true, true),
    };

    let link_content = |label: &str, title: &str| -> String {
        match (show_labels, show_titles) {
            (true, true) => format!(
                r#"<span class="content"><span class="label">{label}</span><span class="title">{title}</span></span>"#
            ),
            (true, false) => {
                format!(r#"<span class="content"><span class="label">{label}</span></span>"#)
            }
            (false, true) => {
                format!(r#"<span class="content"><span class="title">{title}</span></span>"#)
            }
            (false, false) => String::new(),
        }
    };

    // Build prev link HTML
    let prev_html = if let Some(prev_route) = prev {
        let content = link_content(&prev_label, &prev_route.title());
        format!(
            r#"<a href="{}" rel="prev" class="link prev"><span class="icon"></span>{content}</a>"#,
            prev_route.route
        )
    } else {
        String::new()
    };

    // Build next link HTML
    let next_html = if let Some(next_route) = next {
        let content = link_content(&next_label, &next_route.title());
        format!(
            r#"<a href="{}" rel="next" class="link next">{content}<span class="icon"></span></a>"#,
            next_route.route
        )
    } else {
        String::new()
    };

    // Build center element (separator or position indicator)
    let center_html = if show_position {
        format!(r#"<span class="position">Page {current_page} of {total_pages}</span>"#)
    } else if let Some(sep) = separator
        && prev.is_some()
        && next.is_some()
    {
        format!(r#"<span class="separator">{sep}</span>"#)
    } else {
        String::new()
    };

    // Build the complete component
    format!(
        r#"<stencila-prev-next style="{style}"><nav aria-label="Page navigation">{prev_html}{center_html}{next_html}</nav></stencila-prev-next>"#
    )
}

/// Render a title component
///
/// Displays the site title. When `text` is None, uses `site.title`.
fn render_title(text: &Option<String>, context: &RenderContext) -> String {
    let title_text = text
        .clone()
        .or_else(|| context.site_config.title.clone())
        .unwrap_or_default();

    if title_text.is_empty() {
        "<stencila-title></stencila-title>".to_string()
    } else {
        format!("<stencila-title>{title_text}</stencila-title>")
    }
}

/// Render a table of contents tree component
///
/// Renders an empty shell with config attributes. The web component will
/// extract headings from the DOM and build the TOC client-side.
fn render_toc_tree(title: &Option<String>, depth: &Option<u8>) -> String {
    let title_attr = title
        .as_ref()
        .map(|t| format!(r#" title="{t}""#))
        .unwrap_or_default();

    let depth_attr = depth
        .map(|d| format!(r#" depth="{d}""#))
        .unwrap_or_default();

    format!("<stencila-toc-tree{title_attr}{depth_attr}></stencila-toc-tree>")
}

/// Render a navigation tree component
///
/// Displays hierarchical site navigation from `site.nav` configuration.
/// If `site.nav` is not specified, auto-generates navigation from routes.
fn render_nav_tree(
    title: &Option<String>,
    depth: &Option<u8>,
    collapsible: &Option<bool>,
    expanded: &Option<NavTreeExpanded>,
    scroll_to_active: &Option<bool>,
    context: &RenderContext,
) -> String {
    // Get config values with defaults
    let collapsible = collapsible.unwrap_or(true);
    let expanded = expanded.unwrap_or_default();
    let scroll_to_active = scroll_to_active.unwrap_or(true);

    // Resolve nav items: site.nav or auto-generated
    let nav_items = context
        .site_config
        .nav
        .clone()
        .unwrap_or_else(|| auto_generate_nav(context.routes, depth));

    // If no items, render empty component
    if nav_items.is_empty() {
        return "<stencila-nav-tree></stencila-nav-tree>".to_string();
    }

    // Build title HTML
    let title_html = title
        .as_ref()
        .map(|t| format!(r#"<h2 class="nav-tree-title">{t}</h2>"#))
        .unwrap_or_default();

    // Render nav items recursively (empty string for root-level parent_id)
    let items_html =
        render_nav_items(&nav_items, context.route, 1, depth, &expanded, collapsible, "");

    // Build attributes
    let attrs = format!(
        r#" collapsible="{collapsible}" expanded="{expanded}" scroll-to-active="{scroll_to_active}""#
    );

    format!(
        r#"<stencila-nav-tree{attrs}><nav aria-label="Site navigation">{title_html}<ul class="nav-tree-list" role="tree">{items_html}</ul></nav></stencila-nav-tree>"#
    )
}

/// Auto-generate navigation structure from routes
///
/// Groups routes by their path prefixes to create a hierarchical structure.
/// For example:
/// - `/` → Home
/// - `/docs/getting-started/` → under "Docs" group
/// - `/docs/configuration/` → under "Docs" group
/// - `/about/` → About
fn auto_generate_nav(routes: &[RouteEntry], max_depth: &Option<u8>) -> Vec<NavItem> {
    // Simple auto-generation: create flat list limited by depth
    // For now, just list all routes at top level
    // A more sophisticated implementation could group by path prefix

    routes
        .iter()
        .filter(|r| {
            // If max_depth is set, filter by path depth
            if let Some(max) = max_depth {
                let segments: Vec<&str> = r
                    .route
                    .trim_matches('/')
                    .split('/')
                    .filter(|s| !s.is_empty())
                    .collect();
                segments.len() <= *max as usize || r.route == "/"
            } else {
                // No depth limit - include all routes
                true
            }
        })
        .map(|r| NavItem::Route(r.route.clone()))
        .collect()
}

/// Render navigation items recursively
fn render_nav_items(
    items: &[NavItem],
    current_route: &str,
    level: u8,
    max_depth: &Option<u8>,
    expanded: &NavTreeExpanded,
    collapsible: bool,
    parent_id: &str,
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
                let label = route_to_label(route);
                html.push_str(&format!(
                    r#"<li class="nav-tree-item" data-type="link" data-active="{is_active}" data-level="{level}" role="treeitem"{}><a href="{route}">{label}</a></li>"#,
                    if is_active { r#" aria-current="page""# } else { "" }
                ));
            }
            NavItem::Link { label, route } => {
                let is_active = route == current_route;
                html.push_str(&format!(
                    r#"<li class="nav-tree-item" data-type="link" data-active="{is_active}" data-level="{level}" role="treeitem"{}><a href="{route}">{label}</a></li>"#,
                    if is_active { r#" aria-current="page""# } else { "" }
                ));
            }
            NavItem::Group {
                label,
                route,
                children,
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

                // Build the group header based on collapsible setting
                let header_html = if collapsible {
                    // Collapsible mode: include toggle button
                    if let Some(group_route) = route {
                        // Group has a route - render as clickable link with separate toggle
                        format!(
                            r#"<div class="nav-tree-group-header"><a href="{group_route}" class="nav-tree-group-link"{}>{label}</a><button class="nav-tree-toggle" aria-controls="{group_id}" aria-expanded="{is_expanded}"><span class="chevron"></span></button></div>"#,
                            if header_active {
                                r#" aria-current="page""#
                            } else {
                                ""
                            }
                        )
                    } else {
                        // Group has no route - header is just a toggle button
                        format!(
                            r#"<button class="nav-tree-toggle" aria-controls="{group_id}" aria-expanded="{is_expanded}"><span class="chevron"></span><span class="label">{label}</span></button>"#
                        )
                    }
                } else {
                    // Non-collapsible mode: no toggle button, always expanded
                    if let Some(group_route) = route {
                        format!(
                            r#"<a href="{group_route}" class="nav-tree-group-link"{}>{label}</a>"#,
                            if header_active {
                                r#" aria-current="page""#
                            } else {
                                ""
                            }
                        )
                    } else {
                        format!(r#"<span class="nav-tree-group-label">{label}</span>"#)
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
                );

                html.push_str(&format!(
                    r#"<li class="nav-tree-item" data-type="group" data-expanded="{display_expanded}" data-active="{header_active}" data-level="{level}" role="treeitem"{}>{header_html}<ul id="{group_id}" class="nav-tree-children" role="group">{children_html}</ul></li>"#,
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

/// Convert a route path to a human-readable label
///
/// Takes the last segment of the path and converts it to title case.
/// - `/docs/getting-started/` → "Getting Started"
/// - `/` → "Home"
fn route_to_label(route: &str) -> String {
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
fn segment_to_label(segment: &str) -> String {
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

/// Get navigation order for prev/next links
///
/// If `site.nav` is configured, flattens it to get the navigation order.
/// Otherwise, returns the default route order from context.routes.
fn get_nav_order(context: &RenderContext) -> Vec<String> {
    if let Some(nav) = &context.site_config.nav {
        flatten_nav_routes(nav)
    } else {
        context.routes.iter().map(|r| r.route.clone()).collect()
    }
}

/// Flatten nav items into an ordered list of routes
///
/// Traverses the nav tree depth-first, collecting all routes in order.
/// Groups with routes have their route included before children.
fn flatten_nav_routes(items: &[NavItem]) -> Vec<String> {
    let mut routes = Vec::new();

    for item in items {
        match item {
            NavItem::Route(route) => {
                routes.push(route.clone());
            }
            NavItem::Link { route, .. } => {
                routes.push(route.clone());
            }
            NavItem::Group {
                route, children, ..
            } => {
                // If group has a route, include it first
                if let Some(r) = route {
                    routes.push(r.clone());
                }
                // Then include children's routes
                routes.extend(flatten_nav_routes(children));
            }
        }
    }

    routes
}
