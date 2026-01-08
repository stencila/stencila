//! Site layout rendering
//!
//! This module handles rendering the overall site layout structure including
//! regions (header, footer, sidebars) and dispatching to component renderers.

use std::path::Path;

use stencila_codec_utils::{get_current_branch, git_info};
use stencila_config::{
    ColorModeStyle, ComponentConfig, ComponentSpec, EditPageStyle, LayoutConfig, LogoConfig,
    NavItem, PrevNextStyle, RegionSpec, SiteConfig,
};

use crate::{
    RouteEntry, logo,
    nav_common::segment_to_label,
    nav_menu::{self, NavMenuContext},
    nav_tree::{self, NavTreeContext},
};

pub(crate) struct RenderContext<'a> {
    pub site_config: &'a SiteConfig,
    pub site_root: &'a Path,
    pub route: &'a str,
    pub routes: &'a [RouteEntry],
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
            "nav-menu" => {
                let menu_context = NavMenuContext {
                    site_config: context.site_config,
                    route: context.route,
                    routes: context.routes,
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
                    routes: context.routes,
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
                    &tree_context,
                )
            }
            "prev-next" => render_prev_next(&None, &None, &None, &None, context),
            "title" => render_title(&None, context),
            "toc-tree" => render_toc_tree(&None, &None),
            "edit-page" => {
                render_edit_page(&None, &None, &None, &None, &None, &None, context)
            }
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
        } => {
            let menu_context = NavMenuContext {
                site_config: context.site_config,
                route: context.route,
                routes: context.routes,
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
                hover_delay,
                close_delay,
                mobile_breakpoint,
                &menu_context,
            )
        }
        ComponentConfig::NavTree {
            title,
            depth,
            collapsible,
            expanded,
            scroll_to_active,
            include,
            exclude,
            icons,
        } => {
            let tree_context = NavTreeContext {
                site_config: context.site_config,
                route: context.route,
                routes: context.routes,
            };
            nav_tree::render_nav_tree(
                title,
                depth,
                collapsible,
                expanded,
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
        ComponentConfig::EditPage {
            text,
            style,
            base_url,
            branch,
            path_prefix,
            show_platform,
        } => render_edit_page(text, style, base_url, branch, path_prefix, show_platform, context),
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
            // Intermediate segment: check if route exists
            let route_with_slash = format!("{current_path}/");
            let route_exists = context
                .routes
                .iter()
                .any(|r| r.route == route_with_slash || r.route == current_path);

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

/// Render an edit page link component
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
fn render_edit_page(
    text: &Option<String>,
    style: &Option<EditPageStyle>,
    base_url: &Option<String>,
    branch: &Option<String>,
    path_prefix: &Option<String>,
    show_platform: &Option<bool>,
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

    // Get git info (origin + repo-relative path)
    let Ok(info) = git_info(source_path) else {
        // Hide: not in a git repo or git error
        return String::new();
    };

    // Get the repo-relative file path
    let Some(relative_path) = info.path else {
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
        // Auto-detect from origin
        let Some(origin) = info.origin else {
            // Hide: no git origin
            return String::new();
        };

        let Some(platform) = EditPlatform::from_origin(&origin) else {
            // Hide: unsupported host
            return String::new();
        };

        // Determine branch: config > auto-detect > "main"
        let branch_name = branch
            .clone()
            .or_else(|| get_current_branch(Some(source_path)))
            .unwrap_or_else(|| "main".to_string());

        let url = platform.edit_url(&origin, &branch_name, &file_path);
        (url, Some(platform))
    };

    // Determine link text
    let show_platform_name = show_platform.unwrap_or(true);
    let link_text = if let Some(custom_text) = text {
        custom_text.clone()
    } else if show_platform_name {
        if let Some(p) = platform {
            format!("Edit on {}", p.name())
        } else {
            "Edit this page".to_string()
        }
    } else {
        "Edit this page".to_string()
    };

    // Get style with default
    let style = style.unwrap_or_default();

    // Build inner HTML based on style
    // Uses UnoCSS i-lucide:square-pen icon class
    let inner_html = match style {
        EditPageStyle::Icon => r#"<span class="icon i-lucide:square-pen"></span>"#.to_string(),
        EditPageStyle::Text => format!(r#"<span class="text">{link_text}</span>"#),
        EditPageStyle::Both => {
            format!(r#"<span class="icon i-lucide:square-pen"></span><span class="text">{link_text}</span>"#)
        }
    };

    format!(
        r#"<stencila-edit-page><a href="{edit_url}" target="_blank" rel="noopener noreferrer">{inner_html}</a></stencila-edit-page>"#
    )
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
