use std::path::Path;

use stencila_config::{
    ColorModeStyle, ComponentConfig, ComponentSpec, LayoutConfig, LogoConfig, RegionSpec,
    SiteConfig,
};

use crate::logo;

struct RenderContext<'a> {
    site_config: &'a SiteConfig,
    site_root: &'a Path,
    route: &'a str,
}

/// Render a Stencila site layout for a specific route
pub(crate) fn render_layout(site_config: &SiteConfig, site_root: &Path, route: &str) -> String {
    let context = RenderContext {
        site_config,
        site_root,
        route,
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

    format!(r#"<stencila-{name}>{subregions}</stencila-{name}>"#)
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
            "logo" => render_logo(None, context),
            "breadcrumbs" => render_breadcrumbs(context),
            _ => format!("<stencila-{name}></stencila-{name}>"),
        },
        ComponentSpec::Config(config) => render_component_config(config, context),
    }
}

/// Render a component config
fn render_component_config(component: &ComponentConfig, context: &RenderContext) -> String {
    match component {
        ComponentConfig::Logo(config) => render_logo(Some(config), context),
        ComponentConfig::Breadcrumbs => render_breadcrumbs(context),
        ComponentConfig::ColorMode { style } => render_color_mode(style),
        _ => String::new(),
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
