use stencila_config::{ColorModeStyle, ComponentConfig, ComponentSpec, LayoutConfig, RegionSpec};

/// Render a Stencila site layout for a specific route
pub(crate) fn render_layout(config: &LayoutConfig, route: &str) -> String {
    // Resolve the config for the route
    let resolved = config.resolve_for_route(route);

    // Render regions
    let mut regions_enabled = String::new();
    let header = render_region("header", &resolved.header, &mut regions_enabled, route);
    let left_sidebar = render_region(
        "left-sidebar",
        &resolved.left_sidebar,
        &mut regions_enabled,
        route,
    );
    let top = render_region("top", &resolved.top, &mut regions_enabled, route);
    let bottom = render_region("bottom", &resolved.bottom, &mut regions_enabled, route);
    let right_sidebar = render_region(
        "right-sidebar",
        &resolved.right_sidebar,
        &mut regions_enabled,
        route,
    );
    let footer = render_region("footer", &resolved.footer, &mut regions_enabled, route);

    format!(
        r##"<stencila-layout{regions_enabled}>
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

/// Render a layout region (returns empty string if not enabled)
fn render_region(
    name: &str,
    spec: &Option<RegionSpec>,
    regions_enabled: &mut String,
    route: &str,
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
        let start = render_subregion(&config.start, route);
        let middle = render_subregion(&config.middle, route);
        let end = render_subregion(&config.end, route);
        format!(
            r#"<div data-subregion="start">{start}</div><div data-subregion="middle">{middle}</div><div data-subregion="end">{end}</div>"#
        )
    } else {
        String::new()
    };

    format!(r#"<stencila-{name}>{subregions}</stencila-{name}>"#)
}

/// Render a layout subregion (returns empty string if not enabled or no components)
fn render_subregion(components: &Option<Vec<ComponentSpec>>, route: &str) -> String {
    let Some(components) = components else {
        return String::new();
    };

    let mut html = String::new();
    for component in components {
        html.push_str(&render_component_spec(component, route));
    }

    html
}

/// Render a component spec
fn render_component_spec(component: &ComponentSpec, route: &str) -> String {
    match component {
        ComponentSpec::Name(name) => match name.as_str() {
            "breadcrumbs" => render_breadcrumbs(route),
            _ => format!("<stencila-{name}></stencila-{name}>"),
        },
        ComponentSpec::Config(config) => render_component_config(config, route),
    }
}

/// Render a component config
fn render_component_config(component: &ComponentConfig, route: &str) -> String {
    match component {
        ComponentConfig::Breadcrumbs => render_breadcrumbs(route),
        ComponentConfig::ColorMode { style } => render_color_mode(style),
        _ => String::new(),
    }
}

/// Render a breadcrumbs component
///
/// Generates semantic HTML for breadcrumb navigation based on the route path.
/// For example, `/docs/guide/getting-started/` generates:
///   Home > Docs > Guide > Getting Started
fn render_breadcrumbs(route: &str) -> String {
    // Parse the route into segments
    let segments: Vec<&str> = route
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
