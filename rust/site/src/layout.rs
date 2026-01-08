use std::{collections::HashMap, path::Path};

use stencila_config::{
    ColorModeStyle, ComponentConfig, ComponentSpec, FeaturedContent, LayoutConfig, LogoConfig,
    NavItem, NavMenuDropdownStyle, NavMenuGroups, NavMenuIcons, NavMenuTrigger, NavTreeExpanded,
    NavTreeIcons, PrevNextStyle, RegionSpec, SiteConfig,
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
            "nav-menu" => render_nav_menu(
                &None, &None, &None, &None, &None, &None, &None, &None, &None, &None, &None,
                context,
            ),
            "nav-tree" => render_nav_tree(
                &None, &None, &None, &None, &None, &None, &None, &None, context,
            ),
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
        ComponentConfig::NavMenu {
            include,
            exclude,
            depth,
            groups,
            icons,
            descriptions,
            trigger,
            dropdown_style,
            hover_delay,
            close_delay,
            mobile_breakpoint,
        } => render_nav_menu(
            include,
            exclude,
            depth,
            groups,
            icons,
            descriptions,
            trigger,
            dropdown_style,
            hover_delay,
            close_delay,
            mobile_breakpoint,
            context,
        ),
        ComponentConfig::NavTree {
            title,
            depth,
            collapsible,
            expanded,
            scroll_to_active,
            include,
            exclude,
            icons,
        } => render_nav_tree(
            title,
            depth,
            collapsible,
            expanded,
            scroll_to_active,
            include,
            exclude,
            icons,
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
#[allow(clippy::too_many_arguments)]
fn render_nav_tree(
    title: &Option<String>,
    depth: &Option<u8>,
    collapsible: &Option<bool>,
    expanded: &Option<NavTreeExpanded>,
    scroll_to_active: &Option<bool>,
    include: &Option<Vec<String>>,
    exclude: &Option<Vec<String>>,
    icons: &Option<NavTreeIcons>,
    context: &RenderContext,
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
        .unwrap_or_else(|| auto_generate_nav(context.routes, depth));

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

/// Render a navigation menu component
///
/// Displays horizontal navigation with mega-dropdown panels on desktop
/// and accordion-style menu on mobile.
#[allow(clippy::too_many_arguments)]
fn render_nav_menu(
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
    context: &RenderContext,
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
fn limit_nav_depth(items: Vec<NavItem>, current_level: u8, max_depth: &Option<u8>) -> Vec<NavItem> {
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
fn remove_empty_groups(items: Vec<NavItem>) -> Vec<NavItem> {
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
        Some(icon_name) => format!(r#"<span class="icon i-{icon_name}"></span>"#),
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

/// Auto-generate navigation structure from routes
///
/// Groups routes by their path prefixes to create a hierarchical structure.
/// For example:
/// - `/` → Home
/// - `/docs/getting-started/` → under "Docs" group
/// - `/docs/configuration/` → under "Docs" group
/// - `/about/` → About
fn auto_generate_nav(routes: &[RouteEntry], max_depth: &Option<u8>) -> Vec<NavItem> {
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
    // Include root route ("/") as "Home" if it exists
    let mut nav_items = Vec::new();

    if root.route.is_some() {
        nav_items.push(NavItem::Route("/".to_string()));
    }

    nav_items.extend(build_nav_items(&root, 1, max_depth));

    nav_items
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
                let label = route_to_label(route);
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
        format!(r#"<span class="icon i-{icon_name}"></span>"#)
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

// =============================================================================
// Nav Item Filtering
// =============================================================================

/// Pattern types for nav item matching
enum FilterPattern<'a> {
    /// Route pattern (starts with "/"), supports glob syntax
    Route(&'a str),
    /// ID pattern (starts with "#"), matches by item's id field
    Id(&'a str),
    /// Label pattern (everything else), matches by item's label
    Label(&'a str),
}

/// Parse a filter pattern string into a FilterPattern
fn parse_filter_pattern(pattern: &str) -> FilterPattern<'_> {
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
fn filter_nav_items(
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
fn apply_icons(items: Vec<NavItem>, icons: &Option<HashMap<String, String>>) -> Vec<NavItem> {
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
    if let Some(r) = route {
        let normalized = r.trim_end_matches('/');
        if let Some(icon) = icons.get(r) {
            return Some(icon.clone());
        }
        if let Some(icon) = icons.get(normalized) {
            return Some(icon.clone());
        }
        let with_slash = format!("{normalized}/");
        if let Some(icon) = icons.get(&with_slash) {
            return Some(icon.clone());
        }
    }

    // Try label
    icons.get(label).cloned()
}
