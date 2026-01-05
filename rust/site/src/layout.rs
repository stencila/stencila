use stencila_config::{ComponentConfig, ComponentSpec, LayoutConfig, RegionSpec};

/// Render a Stencila site layout for a specific route
pub(crate) fn render_layout(config: &LayoutConfig, route: &str) -> String {
    // Resolve the config for the route
    let resolved = config.resolve_for_route(route);

    // Render regions
    let mut regions_enabled = String::new();
    let header = render_region("header", &resolved.header, &mut regions_enabled);
    let left_sidebar = render_region("left-sidebar", &resolved.left_sidebar, &mut regions_enabled);
    let top = render_region("top", &resolved.top, &mut regions_enabled);
    let bottom = render_region("bottom", &resolved.bottom, &mut regions_enabled);
    let right_sidebar = render_region(
        "right-sidebar",
        &resolved.right_sidebar,
        &mut regions_enabled,
    );
    let footer = render_region("footer", &resolved.footer, &mut regions_enabled);

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
fn render_region(name: &str, spec: &Option<RegionSpec>, regions_enabled: &mut String) -> String {
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
        let start = render_subregion(&config.start);
        let middle = render_subregion(&config.middle);
        let end = render_subregion(&config.end);
        format!(
            r#"<div data-subregion="start">{start}</div><div data-subregion="middle">{middle}</div><div data-subregion="end">{end}</div>"#
        )
    } else {
        String::new()
    };

    format!(r#"<stencila-{name}>{subregions}</stencila-{name}>"#)
}

/// Render a layout subregion (returns empty string if not enabled or no components)
fn render_subregion(components: &Option<Vec<ComponentSpec>>) -> String {
    let Some(components) = components else {
        return String::new();
    };

    let mut html = String::new();
    for component in components {
        html.push_str(&render_component_spec(component));
    }

    html
}

/// Render a component spec
fn render_component_spec(component: &ComponentSpec) -> String {
    match component {
        ComponentSpec::Name(name) => format!("<stencila-{name}></stencila-{name}>"),
        ComponentSpec::Config(config) => render_component_config(config),
    }
}

/// Render a component config
fn render_component_config(component: &ComponentConfig) -> String {
    match component {
        ComponentConfig::ColorMode { style } => format!(
            "<stencila-color-mode{}></stencila-color-mode>",
            match style {
                Some(style) => format!(" style={style}"),
                None => String::new(),
            },
        ),
        _ => return String::new(),
    }
}
