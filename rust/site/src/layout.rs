//! Site layout rendering
//!
//! This module handles rendering the overall site layout structure including
//! regions (header, footer, sidebars) and dispatching to component renderers.

use std::{collections::HashSet, path::Path};

use stencila_config::{
    ColorModeStyle, ComponentConfig, ComponentSpec, CopyMarkdownStyle, CustomSocialLink,
    EditOnService, EditSourceStyle, LayoutConfig, LogoConfig, NavItem, PrevNextStyle, RegionSpec,
    ReviewsConfig, ReviewsPosition, RowConfig, SiteConfig, SiteFormat, SocialLinksStyle,
};

use crate::{
    RouteEntry, logo,
    nav_common::{normalize_icon_name, segment_to_label},
    nav_groups::{self, NavGroupsContext},
    nav_menu::{self, NavMenuContext},
    nav_tree::{self, NavTreeContext},
};

pub(crate) struct RenderContext<'a> {
    pub site_config: &'a SiteConfig,
    pub route: &'a str,
    pub routes: &'a [RouteEntry],
    pub routes_set: &'a HashSet<String>,
    pub nav_items: &'a Vec<NavItem>,
    pub resolved_logo: Option<&'a LogoConfig>,
    pub workspace_id: Option<&'a str>,
    pub git_repo_root: Option<&'a Path>,
    pub git_origin: Option<&'a str>,
    pub git_branch: Option<&'a str>,
}

/// Render a Stencila site layout for a specific route
///
/// # Arguments
/// * `site_config` - Site configuration
/// * `route` - Current route being rendered
/// * `routes` - All document routes for prev/next navigation etc
/// * `routes_set` - Set of valid routes (precomputed for O(1) lookups)
/// * `nav_items` - Nav items (either from site.nav config or auto-generated once)
/// * `resolved_logo` - Pre-resolved logo config (computed once per render)
/// * `workspace_id` - Optional workspace ID from config
/// * `git_repo_root` - Optional git repository root (for edit-source/edit-on)
/// * `git_origin` - Optional git remote origin URL (for edit-source)
/// * `git_branch` - Optional current git branch name (for edit-source)
#[allow(clippy::too_many_arguments)]
pub(crate) fn render_layout(
    site_config: &SiteConfig,
    route: &str,
    routes: &[RouteEntry],
    routes_set: &HashSet<String>,
    nav_items: &Vec<NavItem>,
    resolved_logo: Option<&LogoConfig>,
    workspace_id: Option<&str>,
    git_repo_root: Option<&Path>,
    git_origin: Option<&str>,
    git_branch: Option<&str>,
) -> String {
    let context = RenderContext {
        site_config,
        route,
        routes,
        routes_set,
        nav_items,
        resolved_logo,
        workspace_id,
        git_repo_root,
        git_origin,
        git_branch,
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

    // Render site review component if enabled
    let site_review = render_site_review_if_enabled(&context);

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
</stencila-layout>
{site_review}"##
    )
}

/// Render site review component if enabled and route matches include/exclude patterns
fn render_site_review_if_enabled(context: &RenderContext) -> String {
    // Check if reviews are configured and enabled
    let Some(reviews_spec) = &context.site_config.reviews else {
        return String::new();
    };

    if !reviews_spec.is_enabled() {
        return String::new();
    }

    let config = reviews_spec.to_config();

    // Check include/exclude patterns
    if !should_show_reviews_for_route(context.route, &config) {
        return String::new();
    }

    render_site_review(&config, context.workspace_id)
}

/// Check if reviews should be shown for a given route based on include/exclude patterns
pub fn should_show_reviews_for_route(route: &str, config: &ReviewsConfig) -> bool {
    // Normalize route for matching (remove leading/trailing slashes)
    let normalized_route = route.trim_matches('/');

    // Check exclude patterns first (they take precedence)
    if let Some(exclude) = &config.exclude {
        for pattern in exclude {
            if matches_glob_pattern(normalized_route, pattern) {
                return false;
            }
        }
    }

    // If include patterns are specified, route must match at least one
    if let Some(include) = &config.include {
        if include.is_empty() {
            return true; // Empty include list means include all
        }
        for pattern in include {
            if matches_glob_pattern(normalized_route, pattern) {
                return true;
            }
        }
        return false; // No include pattern matched
    }

    // No include patterns specified, show on all routes (except excluded)
    true
}

/// Simple glob pattern matching supporting * and ** wildcards
fn matches_glob_pattern(path: &str, pattern: &str) -> bool {
    let pattern = pattern.trim_matches('/');

    // Simple exact match
    if pattern == path {
        return true;
    }

    // Handle ** (match any path segments)
    if pattern.contains("**") {
        let parts: Vec<&str> = pattern.split("**").collect();
        if parts.len() == 2 {
            let prefix = parts[0].trim_end_matches('/');
            let suffix = parts[1].trim_start_matches('/');

            let prefix_matches = prefix.is_empty() || path.starts_with(prefix);
            let suffix_matches = suffix.is_empty() || path.ends_with(suffix);

            return prefix_matches && suffix_matches;
        }
    }

    // Handle glob patterns with metacharacters (*, ?, [])
    if (pattern.contains('*') || pattern.contains('?') || pattern.contains('['))
        && !pattern.contains("**")
    {
        // Use glob crate for pattern matching
        if let Ok(glob_pattern) = glob::Pattern::new(pattern) {
            return glob_pattern.matches(path);
        }
    }

    false
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

    // Render subregions (or rows if specified)
    let content = if let Some(config) = spec.config() {
        // Check if rows are specified; if so, render each row
        if let Some(rows) = &config.rows {
            render_rows(rows, context)
        } else {
            // Single-row mode: render start/middle/end directly
            let start = render_subregion(&config.start, context);
            let middle = render_subregion(&config.middle, context);
            let end = render_subregion(&config.end, context);
            format!(
                r#"<div data-subregion="start">{start}</div><div data-subregion="middle">{middle}</div><div data-subregion="end">{end}</div>"#
            )
        }
    } else {
        String::new()
    };

    // Sidebars get an inner wrapper for sticky positioning
    // (outer element stretches for background, inner element is sticky)
    if name == "left-sidebar" || name == "right-sidebar" {
        format!(
            r#"<stencila-{name}><div class="sidebar-content">{content}</div></stencila-{name}>"#
        )
    } else {
        format!(r#"<stencila-{name}>{content}</stencila-{name}>"#)
    }
}

/// Render multiple rows within a region
///
/// Each row is wrapped in a `<div data-row="N">` container with its own
/// start/middle/end sub-regions. This enables multi-row layouts like:
/// - Row 0: prev-next navigation in the middle
/// - Row 1: edit-source on the left, last-edited on the right
fn render_rows(rows: &[RowConfig], context: &RenderContext) -> String {
    rows.iter()
        .enumerate()
        .map(|(idx, row)| {
            let start = render_subregion(&row.start, context);
            let middle = render_subregion(&row.middle, context);
            let end = render_subregion(&row.end, context);
            format!(
                r#"<div data-row="{idx}"><div data-subregion="start">{start}</div><div data-subregion="middle">{middle}</div><div data-subregion="end">{end}</div></div>"#
            )
        })
        .collect()
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
            "nav-menu" => {
                let menu_context = NavMenuContext {
                    site_config: context.site_config,
                    route: context.route,
                    nav_items: context.nav_items,
                };
                nav_menu::render_nav_menu(
                    &None,
                    &None,
                    &None,
                    &None,
                    &None,
                    &None,
                    &None,
                    &None,
                    &menu_context,
                )
            }
            "nav-tree" => {
                let tree_context = NavTreeContext {
                    site_config: context.site_config,
                    route: context.route,
                    nav_items: context.nav_items,
                };
                nav_tree::render_nav_tree(
                    &None,
                    &None,
                    &None,
                    &None,
                    &None,
                    &None,
                    &None,
                    &None,
                    &None,
                    &tree_context,
                )
            }
            "nav-groups" => {
                let groups_context = NavGroupsContext {
                    site_config: context.site_config,
                    route: context.route,
                    nav_items: context.nav_items,
                };
                nav_groups::render_nav_groups(&None, &None, &None, &None, &groups_context)
            }
            "prev-next" => render_prev_next(&None, &None, &None, &None, context),
            "social-links" => render_social_links(&None, &None, &None, &None, &None, context),
            "title" => render_title(&None, context),
            "toc-tree" => render_toc_tree(&None, &None),
            "edit-source" => render_edit_source(&None, &None, &None, &None, &None, context),
            "copy-markdown" => render_copy_markdown(&None, &None, context),
            "edit-on:gdocs" => render_edit_on(&EditOnService::GDocs, &None, &None, context),
            "edit-on:m365" => render_edit_on(&EditOnService::M365, &None, &None, context),
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
        } => {
            let menu_context = NavMenuContext {
                site_config: context.site_config,
                route: context.route,
                nav_items: context.nav_items,
            };
            nav_menu::render_nav_menu(
                include,
                exclude,
                depth,
                groups,
                icons,
                descriptions,
                trigger,
                dropdown_style,
                &menu_context,
            )
        }
        ComponentConfig::NavTree {
            title,
            depth,
            collapsible,
            expand_depth,
            expand_current,
            scroll_to_active,
            include,
            exclude,
            icons,
        } => {
            let tree_context = NavTreeContext {
                site_config: context.site_config,
                route: context.route,
                nav_items: context.nav_items,
            };
            nav_tree::render_nav_tree(
                title,
                depth,
                collapsible,
                expand_depth,
                expand_current,
                scroll_to_active,
                include,
                exclude,
                icons,
                &tree_context,
            )
        }
        ComponentConfig::PrevNext {
            style,
            prev_text,
            next_text,
            separator,
        } => render_prev_next(style, prev_text, next_text, separator, context),
        ComponentConfig::Title { text } => render_title(text, context),
        ComponentConfig::TocTree { title, depth } => render_toc_tree(title, depth),
        ComponentConfig::EditSource {
            text,
            style,
            base_url,
            branch,
            path_prefix,
        } => render_edit_source(text, style, base_url, branch, path_prefix, context),
        ComponentConfig::CopyMarkdown { text, style } => render_copy_markdown(text, style, context),
        ComponentConfig::SocialLinks {
            style,
            new_tab,
            include,
            exclude,
            custom,
        } => render_social_links(style, new_tab, include, exclude, custom, context),
        ComponentConfig::NavGroups {
            include,
            exclude,
            depth,
            icons,
        } => {
            let groups_context = NavGroupsContext {
                site_config: context.site_config,
                route: context.route,
                nav_items: context.nav_items,
            };
            nav_groups::render_nav_groups(include, exclude, depth, icons, &groups_context)
        }
        ComponentConfig::EditOn {
            service,
            text,
            style,
        } => render_edit_on(service, text, style, context),
    }
}

/// Render a logo component
///
/// Uses pre-resolved logo from context (computed once per render).
/// If component_config is provided, merges it with the pre-resolved config.
/// A logo must have at least one image field to render; link/alt alone are not sufficient.
fn render_logo(logo_config: Option<&LogoConfig>, context: &RenderContext) -> String {
    // Use pre-resolved logo from context (avoids per-document filesystem scanning)
    let config = match (logo_config, context.resolved_logo) {
        // Component override: merge with pre-resolved config
        (Some(component), Some(base)) => {
            let mut merged = base.clone();
            logo::merge_logo_config(&mut merged, component);
            Some(merged)
        }
        // Component override only (no site-level logo)
        (Some(component), None) => Some(component.clone()),
        // Use pre-resolved config as-is
        (None, Some(base)) => Some(base.clone()),
        // No logo configured
        (None, None) => None,
    };

    // Guard: logo must have at least one image field (matches resolve_logo's behavior)
    let config = config.filter(logo::has_any_image);

    if let Some(config) = config {
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
///
/// Intermediate segments are only rendered as links if the route exists.
/// Non-existent routes are rendered as non-clickable text (similar to nav-tree groups).
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

    // Add intermediate segments
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
            // Intermediate segment: check if route exists (O(1) HashSet lookup)
            let route_with_slash = format!("{current_path}/");
            let route_exists = context.routes_set.contains(&route_with_slash)
                || context.routes_set.contains(&current_path);

            if route_exists {
                // Route exists: render as clickable link
                items.push_str(&format!(
                    r#"<li><a href="{current_path}/">{label}</a></li>"#
                ));
            } else {
                // Route doesn't exist: render as non-clickable text
                items.push_str(&format!(r#"<li><span>{label}</span></li>"#));
            }
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

/// Supported platforms for edit page links
#[derive(Debug, Clone, Copy)]
enum EditPlatform {
    GitHub,
    GitLab,
    Bitbucket,
}

impl EditPlatform {
    /// Detect platform from normalized origin URL
    ///
    /// Only matches exact hosts (github.com, gitlab.com, bitbucket.org).
    /// Self-hosted instances require `base-url` override.
    fn from_origin(origin: &str) -> Option<Self> {
        if origin.contains("://github.com/") || origin.starts_with("https://github.com") {
            Some(Self::GitHub)
        } else if origin.contains("://gitlab.com/") || origin.starts_with("https://gitlab.com") {
            Some(Self::GitLab)
        } else if origin.contains("://bitbucket.org/")
            || origin.starts_with("https://bitbucket.org")
        {
            Some(Self::Bitbucket)
        } else {
            None
        }
    }

    /// Platform name for display
    fn name(&self) -> &'static str {
        match self {
            Self::GitHub => "GitHub",
            Self::GitLab => "GitLab",
            Self::Bitbucket => "Bitbucket",
        }
    }

    /// Icon class for this platform (using simple-icons via UnoCSS)
    fn icon_class(&self) -> &'static str {
        match self {
            Self::GitHub => "i-simple-icons:github",
            Self::GitLab => "i-simple-icons:gitlab",
            Self::Bitbucket => "i-simple-icons:bitbucket",
        }
    }

    /// Construct edit URL for this platform
    fn edit_url(&self, origin: &str, branch: &str, path: &str) -> String {
        // URL-encode the path for special characters
        let encoded_path = percent_encode_path(path);

        match self {
            Self::GitHub => {
                format!("{origin}/edit/{branch}/{encoded_path}")
            }
            Self::GitLab => {
                format!("{origin}/-/edit/{branch}/{encoded_path}")
            }
            Self::Bitbucket => {
                format!("{origin}/src/{branch}/{encoded_path}?mode=edit")
            }
        }
    }
}

/// Render an edit source link component
///
/// Displays a link to edit the current page on GitHub/GitLab/Bitbucket.
/// Auto-detects the repository from git origin for supported hosts.
/// For self-hosted instances, use the `base-url` option.
///
/// The component hides itself when:
/// - No source file path is available for the current route
/// - Not in a git repository
/// - No git origin is configured and no `base-url` is provided
/// - Origin host is not supported and no `base-url` is provided
fn render_edit_source(
    text: &Option<String>,
    style: &Option<EditSourceStyle>,
    base_url: &Option<String>,
    branch: &Option<String>,
    path_prefix: &Option<String>,
    context: &RenderContext,
) -> String {
    // Get source_path from routes by matching current route
    let source_path = context
        .routes
        .iter()
        .find(|r| r.route == context.route)
        .and_then(|r| r.source_path.as_ref());

    let Some(source_path) = source_path else {
        // Hide: no source path for this route
        return String::new();
    };

    // Get the repo-relative file path using cached git repo root
    let Some(repo_root) = context.git_repo_root else {
        // Hide: not in a git repo
        return String::new();
    };
    let relative_path = source_path
        .strip_prefix(repo_root)
        .ok()
        .and_then(|p| p.to_str())
        .map(|s| s.to_string());
    let Some(relative_path) = relative_path else {
        // Hide: couldn't determine relative path
        return String::new();
    };

    // Construct the file path with optional prefix
    let file_path = construct_edit_path(path_prefix, &relative_path);

    // Determine edit URL and platform
    let (edit_url, platform) = if let Some(base) = base_url {
        // User-provided base URL - just append path
        let url = format!(
            "{}/{}",
            base.trim_end_matches('/'),
            percent_encode_path(&file_path)
        );
        (url, None)
    } else {
        // Auto-detect from origin using cached git origin
        let Some(origin) = context.git_origin else {
            // Hide: no git origin
            return String::new();
        };

        let Some(platform) = EditPlatform::from_origin(origin) else {
            // Hide: unsupported host
            return String::new();
        };

        // Determine branch: config > cached git branch > "main"
        let branch_name = branch
            .clone()
            .or_else(|| context.git_branch.map(String::from))
            .unwrap_or_else(|| "main".to_string());

        let url = platform.edit_url(origin, &branch_name, &file_path);
        (url, Some(platform))
    };

    // Determine link text and title
    let link_text = if let Some(custom_text) = text {
        custom_text.clone()
    } else if let Some(p) = platform {
        format!("Edit on {}", p.name())
    } else {
        "Edit source".to_string()
    };

    // Get style with default
    let style = style.unwrap_or_default();

    // Get icon class - use platform icon if available, otherwise fallback to pencil
    let icon_class = platform
        .map(|p| p.icon_class())
        .unwrap_or("i-lucide:square-pen");

    // Build inner HTML based on style
    let inner_html = match style {
        EditSourceStyle::Icon => format!(r#"<span class="icon {icon_class}"></span>"#),
        EditSourceStyle::Text => format!(r#"<span class="text">{link_text}</span>"#),
        EditSourceStyle::Both => {
            format!(
                r#"<span class="icon {icon_class}"></span><span class="text">{link_text}</span>"#
            )
        }
    };

    format!(
        r#"<stencila-edit-source><a href="{edit_url}" target="_blank" rel="noopener noreferrer">{inner_html}</a></stencila-edit-source>"#
    )
}

/// Render a copy markdown button component
///
/// Displays a button that copies the page's markdown to clipboard.
/// The web component handles fetching the markdown file and clipboard operations.
/// Returns empty string if markdown format is not enabled in site.formats.
fn render_copy_markdown(
    text: &Option<String>,
    style: &Option<CopyMarkdownStyle>,
    context: &RenderContext,
) -> String {
    // Hide if markdown format is not enabled
    if !context.site_config.is_format_enabled(SiteFormat::Md) {
        return String::new();
    }

    // Get style with default
    let style = style.unwrap_or_default();

    // Get button text
    let button_text = text
        .clone()
        .unwrap_or_else(|| "Copy as Markdown".to_string());

    // Use relative URL so it works with any base path deployment
    let md_url = "page.md";

    // Icon class - using lucide clipboard icon
    let icon_class = "i-lucide:clipboard-copy";

    // Build inner HTML based on style
    let inner_html = match style {
        CopyMarkdownStyle::Icon => format!(r#"<span class="icon {icon_class}"></span>"#),
        CopyMarkdownStyle::Text => format!(r#"<span class="text">{button_text}</span>"#),
        CopyMarkdownStyle::Both => {
            format!(
                r#"<span class="icon {icon_class}"></span><span class="text">{button_text}</span>"#
            )
        }
    };

    format!(
        r#"<stencila-copy-markdown data-url="{md_url}"><button type="button" aria-label="Copy page as Markdown">{inner_html}</button></stencila-copy-markdown>"#
    )
}

/// Render an edit-on cloud service component
///
/// Displays a link to edit the current page on Google Docs or Microsoft 365
/// via Stencila Cloud. Only renders if `workspace.id` is configured.
fn render_edit_on(
    service: &EditOnService,
    text: &Option<String>,
    style: &Option<EditSourceStyle>,
    context: &RenderContext,
) -> String {
    // Only render if workspace_id is configured
    let Some(workspace_id) = context.workspace_id else {
        return String::new();
    };

    // Get source file path for current route
    let source_path = context
        .routes
        .iter()
        .find(|r| r.route == context.route)
        .and_then(|r| r.source_path.as_ref());

    let Some(source_path) = source_path else {
        return String::new();
    };

    // Get repo-relative path using cached git repo root
    let Some(repo_root) = context.git_repo_root else {
        return String::new();
    };
    let relative_path = source_path
        .strip_prefix(repo_root)
        .ok()
        .and_then(|p| p.to_str())
        .map(|s| s.to_string());
    let Some(relative_path) = relative_path else {
        return String::new();
    };

    // Build URL
    let service_param = match service {
        EditOnService::GDocs => "gdocs",
        EditOnService::M365 => "m365",
    };
    let file_path = percent_encode_path(&relative_path);
    let url = format!(
        "https://stencila.cloud/workspaces/{workspace_id}/edit?file={file_path}&service={service_param}"
    );

    // Determine link text
    let link_text = text.clone().unwrap_or_else(|| match service {
        EditOnService::GDocs => "Edit on Google Docs".to_string(),
        EditOnService::M365 => "Edit on Microsoft 365".to_string(),
    });

    // Get icon class
    let icon_class = match service {
        EditOnService::GDocs => "i-bi:google",
        EditOnService::M365 => "i-bi:microsoft",
    };

    // Build HTML based on style (similar to edit-source)
    let style = style.unwrap_or_default();
    let inner_html = match style {
        EditSourceStyle::Icon => format!(r#"<span class="icon {icon_class}"></span>"#),
        EditSourceStyle::Text => format!(r#"<span class="text">{link_text}</span>"#),
        EditSourceStyle::Both => {
            format!(
                r#"<span class="icon {icon_class}"></span><span class="text">{link_text}</span>"#
            )
        }
    };

    format!(
        r#"<stencila-edit-on service="{service_param}"><a href="{url}" target="_blank" rel="noopener noreferrer">{inner_html}</a></stencila-edit-on>"#
    )
}

/// Render a site review component
///
/// Enables users to select text on rendered pages and submit comments/suggestions.
/// Only renders if reviews are enabled and `workspace.id` is configured.
/// Returns empty string if reviews are not enabled or workspace.id is not set.
fn render_site_review(config: &ReviewsConfig, workspace_id: Option<&str>) -> String {
    // Require workspace.id for reviews to function
    let Some(id) = workspace_id else {
        tracing::warn!(
            "Reviews are enabled but no workspace.id is configured - review widget will not render. \
            Run `stencila site push` to configure a workspace."
        );
        return String::new();
    };

    let mut attrs = format!(r#"workspace-id="{id}""#);

    // Position attribute
    let position = match config.position() {
        ReviewsPosition::BottomRight => "bottom-right",
        ReviewsPosition::BottomLeft => "bottom-left",
        ReviewsPosition::TopRight => "top-right",
        ReviewsPosition::TopLeft => "top-left",
    };
    attrs.push_str(&format!(r#" position="{position}""#));

    // Types attribute (comma-separated)
    if config.allows_comments() && config.allows_suggestions() {
        attrs.push_str(r#" types="comment,suggestion""#);
    } else if config.allows_comments() {
        attrs.push_str(r#" types="comment""#);
    } else if config.allows_suggestions() {
        attrs.push_str(r#" types="suggestion""#);
    }

    // Selection limits
    attrs.push_str(&format!(r#" min-selection="{}""#, config.min_selection()));
    attrs.push_str(&format!(r#" max-selection="{}""#, config.max_selection()));

    // Shortcuts
    if config.shortcuts_enabled() {
        attrs.push_str(r#" shortcuts="true""#);
    }

    // Public/anon settings (inform UI, enforced server-side)
    if config.is_public() {
        attrs.push_str(r#" public="true""#);
    }
    if config.is_anon() {
        attrs.push_str(r#" anon="true""#);
    }

    format!("<stencila-site-review {attrs}></stencila-site-review>")
}

/// Construct the file path for edit URL
///
/// Combines optional path prefix with the repo-relative file path,
/// normalizing slashes to avoid double slashes or missing separators.
fn construct_edit_path(path_prefix: &Option<String>, relative_path: &str) -> String {
    let prefix = path_prefix
        .as_ref()
        .map(|p| p.trim_matches('/'))
        .unwrap_or("");
    let file_path = relative_path.trim_start_matches('/');

    if prefix.is_empty() {
        file_path.to_string()
    } else {
        format!("{prefix}/{file_path}")
    }
}

/// Percent-encode a file path for use in URLs
///
/// Encodes characters that need escaping in URL paths while preserving
/// forward slashes (which are valid path separators). This handles common
/// special characters like spaces, hash signs, and percent signs.
fn percent_encode_path(path: &str) -> String {
    let mut result = String::with_capacity(path.len() * 2);
    for ch in path.chars() {
        match ch {
            // Safe characters (unreserved + slash for path separators)
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' | '/' => {
                result.push(ch);
            }
            // Encode everything else
            _ => {
                for byte in ch.to_string().as_bytes() {
                    result.push_str(&format!("%{byte:02X}"));
                }
            }
        }
    }
    result
}

/// Render a social links component
///
/// Displays links to social media and external platforms with automatic icons.
/// Uses `site.socials` as the primary data source, with optional filtering
/// and custom links from component config.
#[allow(clippy::too_many_arguments)]
fn render_social_links(
    style: &Option<SocialLinksStyle>,
    new_tab: &Option<bool>,
    include: &Option<Vec<String>>,
    exclude: &Option<Vec<String>>,
    custom: &Option<Vec<CustomSocialLink>>,
    context: &RenderContext,
) -> String {
    // Collect social links from site.socials and custom
    let mut social_links: Vec<(String, String, Option<String>)> = Vec::new();

    // Add links from site.socials
    // If `include` is specified, use it to control order; otherwise iterate over socials
    if let Some(socials) = &context.site_config.socials {
        // Check if x is present (for twitter/x deduplication)
        let has_x = socials.keys().any(|k| k.eq_ignore_ascii_case("x"));

        if let Some(inc) = include {
            // Iterate over include list to preserve order
            for platform in inc {
                // Check if excluded
                if let Some(exc) = exclude
                    && exc.iter().any(|p| p.eq_ignore_ascii_case(platform))
                {
                    continue;
                }
                // Skip twitter if x is present (x takes precedence)
                if platform.eq_ignore_ascii_case("twitter") && has_x {
                    continue;
                }
                // Case-insensitive lookup in socials
                if let Some((key, url)) = socials
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(platform))
                {
                    let final_url = expand_social_url(key, url);
                    social_links.push((key.clone(), final_url, None));
                }
            }
        } else {
            // No include filter - iterate over socials in defined order
            for (platform, url) in socials {
                // Apply exclude filter
                if let Some(exc) = exclude
                    && exc.iter().any(|p| p.eq_ignore_ascii_case(platform))
                {
                    continue;
                }
                // Skip twitter if x is present (x takes precedence)
                if platform.eq_ignore_ascii_case("twitter") && has_x {
                    continue;
                }
                let final_url = expand_social_url(platform, url);
                social_links.push((platform.clone(), final_url, None));
            }
        }
    }

    // Add custom links (always appended after site.socials)
    if let Some(custom_links) = custom {
        // Check if "custom" is excluded
        let custom_excluded = exclude
            .as_ref()
            .is_some_and(|e| e.iter().any(|p| p.eq_ignore_ascii_case("custom")));

        if !custom_excluded {
            for link in custom_links {
                social_links.push((link.name.clone(), link.url.clone(), link.icon.clone()));
            }
        }
    }

    // If no links, return empty
    if social_links.is_empty() {
        return String::new();
    }

    // Get style settings with defaults
    let style = style.unwrap_or_default();
    let open_in_new_tab = new_tab.unwrap_or(true);

    // Build link elements
    let link_elements: Vec<String> = social_links
        .iter()
        .map(|(name, url, custom_icon)| {
            let icon_class = custom_icon
                .as_ref()
                .map(|i| ["i-", &normalize_icon_name(i)].concat())
                .unwrap_or_else(|| get_social_icon_class(name));

            let label = get_platform_label(name);

            let inner_html = match style {
                SocialLinksStyle::Icon => format!(r#"<span class="icon {icon_class}"></span>"#),
                SocialLinksStyle::Text => format!(r#"<span class="text">{label}</span>"#),
                SocialLinksStyle::Both => {
                    format!(r#"<span class="icon {icon_class}"></span><span class="text">{label}</span>"#)
                }
            };

            let target_attrs = if open_in_new_tab {
                r#" target="_blank" rel="noopener noreferrer""#
            } else {
                ""
            };

            format!(
                r#"<a href="{url}" aria-label="{label}"{target_attrs}>{inner_html}</a>"#
            )
        })
        .collect();

    format!(
        r#"<stencila-social-links>{}</stencila-social-links>"#,
        link_elements.join("")
    )
}

/// Get the icon class for a known social platform
fn get_social_icon_class(platform: &str) -> String {
    // Use simple-icons for brand icons via UnoCSS
    match platform.to_lowercase().as_str() {
        "bluesky" => "i-simple-icons:bluesky".to_string(),
        "discord" => "i-simple-icons:discord".to_string(),
        "facebook" => "i-simple-icons:facebook".to_string(),
        "github" => "i-simple-icons:github".to_string(),
        "gitlab" => "i-simple-icons:gitlab".to_string(),
        "instagram" => "i-simple-icons:instagram".to_string(),
        "linkedin" => "i-simple-icons:linkedin".to_string(),
        "mastodon" => "i-simple-icons:mastodon".to_string(),
        "reddit" => "i-simple-icons:reddit".to_string(),
        "twitch" => "i-simple-icons:twitch".to_string(),
        "twitter" | "x" => "i-simple-icons:x".to_string(),
        "youtube" => "i-simple-icons:youtube".to_string(),
        // Fallback to lucide link icon for unknown platforms
        _ => "i-lucide:link".to_string(),
    }
}

/// Get the display label for a platform
fn get_platform_label(platform: &str) -> String {
    match platform.to_lowercase().as_str() {
        "bluesky" => "Bluesky".to_string(),
        "discord" => "Discord".to_string(),
        "facebook" => "Facebook".to_string(),
        "github" => "GitHub".to_string(),
        "gitlab" => "GitLab".to_string(),
        "instagram" => "Instagram".to_string(),
        "linkedin" => "LinkedIn".to_string(),
        "mastodon" => "Mastodon".to_string(),
        "reddit" => "Reddit".to_string(),
        "twitch" => "Twitch".to_string(),
        "twitter" => "Twitter".to_string(),
        "x" => "X".to_string(),
        "youtube" => "YouTube".to_string(),
        // For custom links, preserve original casing
        _ => platform.to_string(),
    }
}

/// Expand a social link shortcut to a full URL
///
/// If the value already looks like a URL (starts with http:// or https://),
/// returns it unchanged. Otherwise, expands platform-specific shortcuts:
///
/// - `bluesky = "handle.bsky.social"` → `https://bsky.app/profile/handle.bsky.social`
/// - `discord = "invite"` → `https://discord.gg/invite`
/// - `facebook = "page"` → `https://facebook.com/page`
/// - `github = "org"` or `"org/repo"` → `https://github.com/org` or `.../org/repo`
/// - `gitlab = "org"` or `"org/repo"` → `https://gitlab.com/org` or `.../org/repo`
/// - `instagram = "handle"` → `https://instagram.com/handle`
/// - `linkedin = "company/name"` → `https://linkedin.com/company/name`
/// - `linkedin = "in/name"` → `https://linkedin.com/in/name`
/// - `mastodon` - returned as-is (requires full URL due to federated instances)
/// - `reddit = "r/sub"` → `https://reddit.com/r/sub`
/// - `twitch = "channel"` → `https://twitch.tv/channel`
/// - `x = "handle"` or `twitter = "handle"` → `https://x.com/handle`
/// - `youtube = "@channel"` → `https://youtube.com/@channel`
fn expand_social_url(platform: &str, value: &str) -> String {
    // If already a URL, return as-is
    if value.starts_with("http://") || value.starts_with("https://") {
        return value.to_string();
    }

    match platform.to_lowercase().as_str() {
        "bluesky" => {
            let handle = value.trim_start_matches('@');
            format!("https://bsky.app/profile/{handle}")
        }
        "discord" => format!("https://discord.gg/{value}"),
        "facebook" => format!("https://facebook.com/{value}"),
        "github" => format!("https://github.com/{value}"),
        "gitlab" => format!("https://gitlab.com/{value}"),
        "instagram" => {
            let handle = value.trim_start_matches('@');
            format!("https://instagram.com/{handle}")
        }
        "linkedin" => {
            // Support both "in/name" and "company/name" formats
            if value.starts_with("in/") || value.starts_with("company/") {
                format!("https://linkedin.com/{value}")
            } else {
                // Default to personal profile
                format!("https://linkedin.com/in/{value}")
            }
        }
        "reddit" => {
            // Support r/subreddit and u/user formats
            if value.starts_with("r/") || value.starts_with("u/") {
                format!("https://reddit.com/{value}")
            } else {
                // Default to subreddit
                format!("https://reddit.com/r/{value}")
            }
        }
        "twitch" => format!("https://twitch.tv/{value}"),
        "twitter" | "x" => {
            let handle = value.trim_start_matches('@');
            format!("https://x.com/{handle}")
        }
        "youtube" => {
            // Support @channel, channel/ID, and c/custom formats
            if value.starts_with('@') || value.starts_with("channel/") || value.starts_with("c/") {
                format!("https://youtube.com/{value}")
            } else {
                format!("https://youtube.com/@{value}")
            }
        }
        // Mastodon and unknown platforms: return as-is (may be invalid, but we don't know the instance)
        _ => value.to_string(),
    }
}

/// Get navigation order for prev/next links
///
/// Uses the nav_items from context (which may be from site.nav config or
/// auto-generated from routes, respecting `_nav.yaml` overrides).
fn get_nav_order(context: &RenderContext) -> Vec<String> {
    flatten_nav_routes(context.nav_items)
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
