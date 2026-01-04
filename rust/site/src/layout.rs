use stencila_config::{LayoutConfig, RegionSpec};

/// Render a Stencila site layout for a specific route
pub(crate) fn render_layout(config: &LayoutConfig) -> String {
    // Render regions
    let mut regions_enabled = String::new();
    let header = render_region("header", &config.header, &mut regions_enabled);
    let left_sidebar = render_region("left-sidebar", &config.left_sidebar, &mut regions_enabled);
    let top = render_region("top", &config.top, &mut regions_enabled);
    let bottom = render_region("bottom", &config.bottom, &mut regions_enabled);
    let right_sidebar = render_region("right-sidebar", &config.right_sidebar, &mut regions_enabled);
    let footer = render_region("footer", &config.footer, &mut regions_enabled);

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
    // Do not render empty disabled regions
    if !spec
        .as_ref()
        .map(|spec| spec.is_enabled())
        .unwrap_or_default()
    {
        return String::new();
    }

    // Record as an enabled region
    regions_enabled.push(' ');
    regions_enabled.push_str(&name);

    // TODO: render region components
    format!(r#"<stencila-{name}></stencila-{name}>"#)
}
