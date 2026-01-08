//! Layout configuration
//!
//! This module contains the main `LayoutConfig` struct that orchestrates
//! layout configuration with presets, regions, components, and route overrides.

use std::collections::HashMap;

use glob::Pattern;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::components::{
    BUILTIN_COMPONENT_TYPES, ComponentConfig, ComponentSpec, is_builtin_component_type,
};
use super::overrides::LayoutOverride;
use super::presets::LayoutPreset;
use super::regions::{RegionSpec, ResponsiveConfig, merge_region, resolve_region};

/// Site layout configuration
///
/// Controls the layout structure of site pages using a region-based system.
/// Each region (header, sidebars, etc.) has sub-regions (start, middle, end)
/// where components can be placed.
///
/// Example:
/// ```toml
/// [site.layout]
/// preset = "docs"
///
/// [site.layout.header]
/// start = "logo"
/// end = ["icon-links", "color-mode"]
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct LayoutConfig {
    /// Named preset to use as base (docs, blog, landing, api)
    ///
    /// Presets provide sensible defaults that can be extended with explicit config.
    pub preset: Option<LayoutPreset>,

    /// Header region configuration
    ///
    /// Horizontal region at the top of the page.
    pub header: Option<RegionSpec>,

    /// Left sidebar region configuration
    ///
    /// Vertical region on the left side of the page.
    /// Auto-enabled for multi-page sites when not specified.
    pub left_sidebar: Option<RegionSpec>,

    /// Top region configuration
    ///
    /// Horizontal region above the main content area.
    pub top: Option<RegionSpec>,

    /// Bottom region configuration
    ///
    /// Horizontal region below the main content area.
    pub bottom: Option<RegionSpec>,

    /// Right sidebar region configuration
    ///
    /// Vertical region on the right side of the page.
    /// Auto-enabled when document has headings.
    pub right_sidebar: Option<RegionSpec>,

    /// Footer region configuration
    ///
    /// Horizontal region at the bottom of the page.
    pub footer: Option<RegionSpec>,

    /// Global responsive configuration for sidebar collapse
    ///
    /// These settings apply to both sidebars unless overridden per-sidebar.
    ///
    /// Example:
    /// ```toml
    /// [site.layout.responsive]
    /// breakpoint = 1024
    /// toggle-style = "fixed-edge"
    /// ```
    pub responsive: Option<ResponsiveConfig>,

    /// Named component definitions for reuse
    ///
    /// Define components once and reference them by name in regions.
    ///
    /// Example:
    /// ```toml
    /// [site.layout.components.main-nav]
    /// type = "nav-tree"
    /// collapsible = true
    /// depth = 3
    /// ```
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub components: HashMap<String, ComponentConfig>,

    /// Route-specific layout overrides
    ///
    /// First matching override wins (order matters).
    ///
    /// Example:
    /// ```toml
    /// [[site.layout.overrides]]
    /// routes = ["/blog/**"]
    /// left-sidebar = false
    /// ```
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overrides: Vec<LayoutOverride>,
}

impl LayoutConfig {
    /// Validate the layout configuration
    ///
    /// Checks that:
    /// - All component name references are either built-in types or defined in `components`
    /// - All layout overrides have valid route patterns
    pub fn validate(&self) -> eyre::Result<()> {
        // Validate component references in each region
        self.validate_region("header", &self.header)?;
        self.validate_region("left-sidebar", &self.left_sidebar)?;
        self.validate_region("top", &self.top)?;
        self.validate_region("bottom", &self.bottom)?;
        self.validate_region("right-sidebar", &self.right_sidebar)?;
        self.validate_region("footer", &self.footer)?;

        // Validate overrides
        for (index, override_config) in self.overrides.iter().enumerate() {
            override_config
                .validate()
                .map_err(|e| eyre::eyre!("Invalid layout override at index {}: {}", index, e))?;
        }
        Ok(())
    }

    /// Validate component references in a region
    fn validate_region(&self, region_name: &str, spec: &Option<RegionSpec>) -> eyre::Result<()> {
        let Some(spec) = spec else {
            return Ok(());
        };

        let RegionSpec::Config(config) = spec else {
            return Ok(());
        };

        self.validate_component_list(region_name, "start", &config.start)?;
        self.validate_component_list(region_name, "middle", &config.middle)?;
        self.validate_component_list(region_name, "end", &config.end)?;

        Ok(())
    }

    /// Validate a list of component specs
    fn validate_component_list(
        &self,
        region_name: &str,
        subregion_name: &str,
        specs: &Option<Vec<ComponentSpec>>,
    ) -> eyre::Result<()> {
        let Some(specs) = specs else {
            return Ok(());
        };

        for spec in specs {
            if let ComponentSpec::Name(name) = spec
                && !is_builtin_component_type(name)
                && !self.components.contains_key(name)
            {
                eyre::bail!(
                    "Unknown component '{}' in {}.{}. \
                         Must be a built-in type ({}) or defined in [site.layout.components.{}]",
                    name,
                    region_name,
                    subregion_name,
                    BUILTIN_COMPONENT_TYPES.join(", "),
                    name
                );
            }
        }

        Ok(())
    }

    /// Resolve the layout configuration by merging preset defaults with explicit config
    ///
    /// If a preset is specified, its defaults are used as the base and any explicit
    /// region configurations override those defaults. If no preset is specified,
    /// the explicit config is used as-is.
    ///
    /// This method also resolves named component references: any `ComponentSpec::Name`
    /// that matches a key in `components` is expanded to `ComponentSpec::Config` with
    /// the corresponding configuration.
    pub fn resolve(&self) -> Self {
        let base = match &self.preset {
            Some(preset) => preset.defaults(),
            None => Self::default(),
        };

        // Merge components from base and self
        let components = {
            let mut merged = base.components;
            merged.extend(self.components.clone());
            merged
        };

        // Merge regions and resolve named component references
        Self {
            preset: self.preset,
            components: components.clone(),
            header: resolve_region(merge_region(&base.header, &self.header), &components),
            left_sidebar: resolve_region(
                merge_region(&base.left_sidebar, &self.left_sidebar),
                &components,
            ),
            top: resolve_region(merge_region(&base.top, &self.top), &components),
            bottom: resolve_region(merge_region(&base.bottom, &self.bottom), &components),
            right_sidebar: resolve_region(
                merge_region(&base.right_sidebar, &self.right_sidebar),
                &components,
            ),
            footer: resolve_region(merge_region(&base.footer, &self.footer), &components),
            responsive: self.responsive.clone().or(base.responsive.clone()),
            overrides: self.overrides.clone(),
        }
    }

    /// Resolve the layout configuration for a specific route
    ///
    /// This method:
    /// 1. Calls `resolve()` to merge preset defaults with explicit config
    /// 2. Finds the first matching override for the route (if any)
    /// 3. If the override has a preset, uses that preset's defaults as the new base
    /// 4. Applies the override's explicit region configs on top
    ///
    /// Example:
    /// ```ignore
    /// let config = layout_config.resolve_for_route("/blog/my-post/");
    /// ```
    pub fn resolve_for_route(&self, route: &str) -> Self {
        let base = self.resolve();

        // Find first matching override
        let Some(override_config) = self.find_matching_override(route) else {
            return base;
        };

        // Build the region base with proper layering:
        // 1. If override has a preset, start with that preset's defaults
        // 2. Merge global explicit config on top (so global customizations persist)
        // 3. Merge override's explicit regions on top
        //
        // This ensures that global config like `header.end = ["search"]` applies
        // everywhere, even on routes that switch to a different preset.
        let region_base = match &override_config.preset {
            Some(preset) => {
                // Start with override's preset defaults
                let preset_defaults = preset.defaults();
                // Layer global explicit config on top of preset defaults
                LayoutConfig {
                    header: merge_region(&preset_defaults.header, &self.header),
                    left_sidebar: merge_region(&preset_defaults.left_sidebar, &self.left_sidebar),
                    top: merge_region(&preset_defaults.top, &self.top),
                    bottom: merge_region(&preset_defaults.bottom, &self.bottom),
                    right_sidebar: merge_region(
                        &preset_defaults.right_sidebar,
                        &self.right_sidebar,
                    ),
                    footer: merge_region(&preset_defaults.footer, &self.footer),
                    ..Default::default()
                }
            }
            None => base.clone(),
        };

        // Apply override's explicit regions on top of the region base,
        // then resolve named component references (consistent with resolve())
        let components = &base.components;
        Self {
            // Use override preset if specified, otherwise keep base preset
            preset: override_config.preset.or(base.preset),
            // Keep merged components from base
            components: base.components.clone(),
            header: resolve_region(
                merge_region(&region_base.header, &override_config.header),
                components,
            ),
            left_sidebar: resolve_region(
                merge_region(&region_base.left_sidebar, &override_config.left_sidebar),
                components,
            ),
            top: resolve_region(
                merge_region(&region_base.top, &override_config.top),
                components,
            ),
            bottom: resolve_region(
                merge_region(&region_base.bottom, &override_config.bottom),
                components,
            ),
            right_sidebar: resolve_region(
                merge_region(&region_base.right_sidebar, &override_config.right_sidebar),
                components,
            ),
            footer: resolve_region(
                merge_region(&region_base.footer, &override_config.footer),
                components,
            ),
            // Use override responsive if specified, otherwise keep base
            responsive: override_config
                .responsive
                .clone()
                .or(base.responsive.clone()),
            // Don't include overrides in resolved config - they've been applied
            overrides: vec![],
        }
    }

    /// Find the first override that matches a route
    ///
    /// Returns the first `LayoutOverride` where any of its route patterns
    /// match the given route. Patterns are glob patterns (e.g., "/blog/**").
    fn find_matching_override(&self, route: &str) -> Option<&LayoutOverride> {
        for override_config in &self.overrides {
            for pattern in &override_config.routes {
                if let Ok(glob) = Pattern::new(pattern)
                    && glob.matches(route)
                {
                    return Some(override_config);
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use eyre::Result;

    use super::*;

    #[test]
    fn test_region_bool() -> Result<()> {
        let toml = r#"left-sidebar = true"#;
        let layout: LayoutConfig = toml::from_str(toml)?;
        assert!(matches!(
            layout.left_sidebar,
            Some(RegionSpec::Enabled(true))
        ));

        let toml = r#"left-sidebar = false"#;
        let layout: LayoutConfig = toml::from_str(toml)?;
        assert!(matches!(
            layout.left_sidebar,
            Some(RegionSpec::Enabled(false))
        ));

        Ok(())
    }

    #[test]
    fn test_region_config_string() -> Result<()> {
        let toml = r#"
            [header]
            start = "logo"
        "#;
        let layout: LayoutConfig = toml::from_str(toml)?;

        let header = layout.header.expect("header should be present");
        let config = header.config().expect("should be config");
        assert_eq!(config.start.as_ref().map(|v| v.len()), Some(1));
        let start = config.start.as_ref().expect("start should be present");
        assert!(matches!(&start[0], ComponentSpec::Name(n) if n == "logo"));

        Ok(())
    }

    #[test]
    fn test_region_config_array() -> Result<()> {
        let toml = r#"
            [header]
            end = ["icon-links", "color-mode"]
        "#;
        let layout: LayoutConfig = toml::from_str(toml)?;

        let header = layout.header.expect("header should be present");
        let config = header.config().expect("should be config");
        let end = config.end.as_ref().expect("end should be present");
        assert_eq!(end.len(), 2);
        assert!(matches!(&end[0], ComponentSpec::Name(n) if n == "icon-links"));
        assert!(matches!(&end[1], ComponentSpec::Name(n) if n == "color-mode"));

        Ok(())
    }

    #[test]
    fn test_region_config_object() -> Result<()> {
        let toml = r#"
            [header]
            middle = { type = "logo" }
        "#;
        let layout: LayoutConfig = toml::from_str(toml)?;

        let header = layout.header.expect("header should be present");
        let config = header.config().expect("should be config");
        let middle = config.middle.as_ref().expect("middle should be present");
        assert_eq!(middle.len(), 1);
        assert!(matches!(
            &middle[0],
            ComponentSpec::Config(ComponentConfig::Logo { .. })
        ));

        Ok(())
    }

    #[test]
    fn test_named_components() -> Result<()> {
        let toml = r#"
            [components.main-nav]
            type = "nav-tree"
            collapsible = true
            depth = 3

            [left-sidebar]
            middle = "main-nav"
        "#;
        let layout: LayoutConfig = toml::from_str(toml)?;

        assert!(layout.components.contains_key("main-nav"));
        if let ComponentConfig::NavTree {
            collapsible, depth, ..
        } = &layout.components["main-nav"]
        {
            assert_eq!(*collapsible, Some(true));
            assert_eq!(*depth, Some(3));
        } else {
            panic!("Expected NavTree component");
        }

        Ok(())
    }

    #[test]
    fn test_override_basic() -> Result<()> {
        let toml = r#"
            [[overrides]]
            routes = ["/blog/**"]
            left-sidebar = false
            bottom = false
        "#;
        let layout: LayoutConfig = toml::from_str(toml)?;

        assert_eq!(layout.overrides.len(), 1);
        assert_eq!(layout.overrides[0].routes, vec!["/blog/**"]);
        assert!(matches!(
            layout.overrides[0].left_sidebar,
            Some(RegionSpec::Enabled(false))
        ));
        assert!(matches!(
            layout.overrides[0].bottom,
            Some(RegionSpec::Enabled(false))
        ));

        Ok(())
    }

    #[test]
    fn test_override_with_subregion() -> Result<()> {
        let toml = r#"
            [[overrides]]
            routes = ["/api/**"]
            right-sidebar = false
            left-sidebar.middle = "api-nav"
        "#;
        let layout: LayoutConfig = toml::from_str(toml)?;

        assert_eq!(layout.overrides.len(), 1);
        assert!(matches!(
            layout.overrides[0].right_sidebar,
            Some(RegionSpec::Enabled(false))
        ));

        if let Some(RegionSpec::Config(config)) = &layout.overrides[0].left_sidebar {
            let middle = config.middle.as_ref().expect("middle should be present");
            assert!(matches!(&middle[0], ComponentSpec::Name(n) if n == "api-nav"));
        } else {
            panic!("Expected RegionSpec::Config for left-sidebar");
        }

        Ok(())
    }

    #[test]
    fn test_full_example() -> Result<()> {
        let toml = r#"
            preset = "docs"

            [components.api-nav]
            type = "nav-tree"
            collapsible = false

            [header]
            start = "logo"
            middle = []
            end = ["color-mode"]

            [left-sidebar]
            middle = { type = "nav-tree", collapsible = true }

            [top]
            start = "breadcrumbs"

            [bottom]
            middle = "prev-next"

            [right-sidebar]
            start = { type = "toc-tree", title = "On this page", depth = 3 }

            [footer]
            start = { type = "copyright" }
            end = ["color-mode"]

            [[overrides]]
            routes = ["/blog/**"]
            left-sidebar = false
            bottom = false

            [[overrides]]
            routes = ["/api/**"]
            left-sidebar.middle = "api-nav"
            right-sidebar = false
        "#;
        let layout: LayoutConfig = toml::from_str(toml)?;

        assert_eq!(layout.preset, Some(LayoutPreset::Docs));
        assert!(layout.components.contains_key("api-nav"));
        assert!(layout.header.is_some());
        assert!(layout.left_sidebar.is_some());
        assert!(layout.top.is_some());
        assert!(layout.bottom.is_some());
        assert!(layout.right_sidebar.is_some());
        assert!(layout.footer.is_some());
        assert_eq!(layout.overrides.len(), 2);

        Ok(())
    }

    #[test]
    fn test_color_mode_style() -> Result<()> {
        use super::super::components::ColorModeStyle;

        let toml = r#"
            [header]
            end = [{ type = "color-mode", style = "both" }]
        "#;
        let layout: LayoutConfig = toml::from_str(toml)?;

        let header = layout.header.expect("header should be present");
        let config = header.config().expect("should be config");
        let end = config.end.as_ref().expect("end should be present");

        if let ComponentSpec::Config(ComponentConfig::ColorMode { style }) = &end[0] {
            assert_eq!(*style, Some(ColorModeStyle::Both));
        } else {
            panic!("Expected ComponentSpec::Config(ComponentConfig::ColorMode)");
        }

        Ok(())
    }

    #[test]
    fn test_prev_next_style() -> Result<()> {
        use super::super::components::PrevNextStyle;

        let toml = r#"
            [bottom]
            middle = { type = "prev-next", style = "minimal" }
        "#;
        let layout: LayoutConfig = toml::from_str(toml)?;
        let bottom = layout.bottom.expect("bottom should be present");
        let config = bottom.config().expect("should be config");
        let middle = config.middle.as_ref().expect("middle should be present");
        if let ComponentSpec::Config(ComponentConfig::PrevNext { style, .. }) = &middle[0] {
            assert_eq!(*style, Some(PrevNextStyle::Minimal));
        } else {
            panic!("Expected ComponentSpec::Config(ComponentConfig::PrevNext)");
        }

        Ok(())
    }

    #[test]
    fn test_validate_builtin_component_names() -> Result<()> {
        let toml = r#"
            [header]
            start = "logo"
            middle = ["title", "nav-menu"]
            end = ["breadcrumbs", "nav-tree", "toc-tree", "prev-next", "color-mode", "copyright"]
        "#;
        let config: LayoutConfig = toml::from_str(toml)?;
        config.validate()?;

        Ok(())
    }

    #[test]
    fn test_validate_named_component_reference() -> Result<()> {
        let toml = r#"
            [components.my-nav]
            type = "nav-tree"
            collapsible = false

            [left-sidebar]
            middle = "my-nav"
        "#;
        let config: LayoutConfig = toml::from_str(toml)?;
        config.validate()?;

        Ok(())
    }

    #[test]
    fn test_validate_unknown_component_fails() {
        let toml = r#"
            [header]
            start = "unknown-component"
        "#;
        let config: LayoutConfig = toml::from_str(toml).expect("should parse");

        let result = config.validate();
        assert!(result.is_err());
        let err_msg = result
            .expect_err("validation should fail for unknown component")
            .to_string();
        assert!(
            err_msg.contains("Unknown component 'unknown-component'"),
            "Error should mention unknown component: {err_msg}"
        );
        assert!(
            err_msg.contains("header.start"),
            "Error should mention location: {err_msg}"
        );
    }

    #[test]
    fn test_validate_undefined_named_component_fails() {
        let toml = r#"
            [left-sidebar]
            middle = "my-nav"
        "#;
        let config: LayoutConfig = toml::from_str(toml).expect("should parse");

        let result = config.validate();
        assert!(result.is_err());
        let err_msg = result
            .expect_err("validation should fail for undefined component")
            .to_string();
        assert!(
            err_msg.contains("Unknown component 'my-nav'"),
            "Error should mention undefined component: {err_msg}"
        );
        assert!(
            err_msg.contains("[site.layout.components.my-nav]"),
            "Error should suggest defining the component: {err_msg}"
        );
    }

    #[test]
    fn test_resolve_no_preset() {
        let config = LayoutConfig {
            header: Some(RegionSpec::Enabled(false)),
            ..Default::default()
        };

        let resolved = config.resolve();
        assert!(matches!(resolved.header, Some(RegionSpec::Enabled(false))));
        assert!(resolved.left_sidebar.is_none());
    }

    #[test]
    fn test_resolve_preset_only() {
        let config = LayoutConfig {
            preset: Some(LayoutPreset::Docs),
            ..Default::default()
        };

        let resolved = config.resolve();
        assert!(resolved.header.is_some());
        assert!(
            resolved
                .left_sidebar
                .as_ref()
                .is_some_and(|r| r.is_enabled())
        );
        assert!(
            resolved
                .right_sidebar
                .as_ref()
                .is_some_and(|r| r.is_enabled())
        );
    }

    #[test]
    fn test_resolve_preset_with_override() {
        let config = LayoutConfig {
            preset: Some(LayoutPreset::Docs),
            left_sidebar: Some(RegionSpec::Enabled(false)),
            ..Default::default()
        };

        let resolved = config.resolve();
        assert!(
            resolved
                .left_sidebar
                .as_ref()
                .is_some_and(|r| !r.is_enabled())
        );
        assert!(
            resolved
                .right_sidebar
                .as_ref()
                .is_some_and(|r| r.is_enabled())
        );
    }

    #[test]
    fn test_resolve_preset_with_components() -> Result<()> {
        let toml = r#"
            preset = "docs"

            [components.custom-nav]
            type = "nav-tree"
            collapsible = false
        "#;
        let config: LayoutConfig = toml::from_str(toml)?;

        let resolved = config.resolve();
        assert!(resolved.components.contains_key("custom-nav"));

        Ok(())
    }

    #[test]
    fn test_resolve_named_component_reference() -> Result<()> {
        let toml = r#"
            [components.my-nav]
            type = "nav-tree"
            collapsible = false
            depth = 2

            [left-sidebar]
            middle = "my-nav"
        "#;
        let config: LayoutConfig = toml::from_str(toml)?;

        let resolved = config.resolve();

        let sidebar = resolved
            .left_sidebar
            .as_ref()
            .expect("left-sidebar should be present");
        let region_config = sidebar.config().expect("should be config");
        let middle = region_config
            .middle
            .as_ref()
            .expect("middle should be present");

        assert_eq!(middle.len(), 1);
        if let ComponentSpec::Config(ComponentConfig::NavTree {
            collapsible, depth, ..
        }) = &middle[0]
        {
            assert_eq!(*collapsible, Some(false));
            assert_eq!(*depth, Some(2));
        } else {
            panic!("Expected ComponentSpec::Config(ComponentConfig::NavTree) after resolution");
        }

        Ok(())
    }

    #[test]
    fn test_resolve_builtin_name_unchanged() -> Result<()> {
        let toml = r#"
            [header]
            start = "logo"
        "#;
        let config: LayoutConfig = toml::from_str(toml)?;

        let resolved = config.resolve();

        let header = resolved.header.as_ref().expect("header should be present");
        let region_config = header.config().expect("should be config");
        let start = region_config
            .start
            .as_ref()
            .expect("start should be present");

        assert_eq!(start.len(), 1);
        assert!(matches!(&start[0], ComponentSpec::Name(n) if n == "logo"));

        Ok(())
    }

    #[test]
    fn test_resolve_mixed_named_and_builtin() -> Result<()> {
        let toml = r#"
            [components.custom-toc]
            type = "toc-tree"
            depth = 4

            [header]
            start = "logo"
            end = ["custom-toc", "color-mode"]
        "#;
        let config: LayoutConfig = toml::from_str(toml)?;

        let resolved = config.resolve();

        let header = resolved.header.as_ref().expect("header should be present");
        let region_config = header.config().expect("should be config");

        let start = region_config
            .start
            .as_ref()
            .expect("start should be present");
        assert!(matches!(&start[0], ComponentSpec::Name(n) if n == "logo"));

        let end = region_config.end.as_ref().expect("end should be present");
        assert_eq!(end.len(), 2);

        assert!(
            matches!(
                &end[0],
                ComponentSpec::Config(ComponentConfig::TocTree { .. })
            ),
            "Expected custom-toc to be expanded"
        );

        assert!(matches!(&end[1], ComponentSpec::Name(n) if n == "color-mode"));

        Ok(())
    }

    #[test]
    fn test_resolve_for_route_no_override() {
        let config = LayoutConfig {
            preset: Some(LayoutPreset::Docs),
            overrides: vec![LayoutOverride {
                routes: vec!["/blog/**".to_string()],
                left_sidebar: Some(RegionSpec::Enabled(false)),
                ..Default::default()
            }],
            ..Default::default()
        };

        let resolved = config.resolve_for_route("/docs/guide/");

        assert!(
            resolved
                .left_sidebar
                .as_ref()
                .is_some_and(|r| r.is_enabled())
        );
    }

    #[test]
    fn test_resolve_for_route_with_matching_override() {
        let config = LayoutConfig {
            preset: Some(LayoutPreset::Docs),
            overrides: vec![LayoutOverride {
                routes: vec!["/blog/**".to_string()],
                left_sidebar: Some(RegionSpec::Enabled(false)),
                bottom: Some(RegionSpec::Enabled(false)),
                ..Default::default()
            }],
            ..Default::default()
        };

        let resolved = config.resolve_for_route("/blog/my-post/");

        assert!(
            resolved
                .left_sidebar
                .as_ref()
                .is_some_and(|r| !r.is_enabled())
        );
        assert!(resolved.bottom.as_ref().is_some_and(|r| !r.is_enabled()));
        assert!(resolved.header.as_ref().is_some_and(|r| r.is_enabled()));
    }

    #[test]
    fn test_resolve_for_route_first_match_wins() {
        let config = LayoutConfig {
            preset: Some(LayoutPreset::Docs),
            overrides: vec![
                LayoutOverride {
                    routes: vec!["/blog/special/**".to_string()],
                    left_sidebar: Some(RegionSpec::Enabled(false)),
                    right_sidebar: Some(RegionSpec::Enabled(false)),
                    ..Default::default()
                },
                LayoutOverride {
                    routes: vec!["/blog/**".to_string()],
                    left_sidebar: Some(RegionSpec::Enabled(false)),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let resolved = config.resolve_for_route("/blog/special/post/");
        assert!(
            resolved
                .left_sidebar
                .as_ref()
                .is_some_and(|r| !r.is_enabled())
        );
        assert!(
            resolved
                .right_sidebar
                .as_ref()
                .is_some_and(|r| !r.is_enabled())
        );

        let resolved = config.resolve_for_route("/blog/regular/");
        assert!(
            resolved
                .left_sidebar
                .as_ref()
                .is_some_and(|r| !r.is_enabled())
        );
        assert!(
            resolved
                .right_sidebar
                .as_ref()
                .is_some_and(|r| r.is_enabled())
        );
    }

    #[test]
    fn test_resolve_for_route_glob_patterns() {
        let config = LayoutConfig {
            overrides: vec![
                LayoutOverride {
                    routes: vec!["/api/**".to_string()],
                    header: Some(RegionSpec::Enabled(false)),
                    ..Default::default()
                },
                LayoutOverride {
                    routes: vec!["/docs/*/intro/".to_string()],
                    footer: Some(RegionSpec::Enabled(false)),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        let resolved = config.resolve_for_route("/api/users/");
        assert!(resolved.header.as_ref().is_some_and(|r| !r.is_enabled()));

        let resolved = config.resolve_for_route("/docs/guide/intro/");
        assert!(resolved.footer.as_ref().is_some_and(|r| !r.is_enabled()));

        let resolved = config.resolve_for_route("/docs/guide/advanced/");
        assert!(resolved.footer.is_none());
    }

    #[test]
    fn test_resolve_for_route_clears_overrides() {
        let config = LayoutConfig {
            overrides: vec![LayoutOverride {
                routes: vec!["/blog/**".to_string()],
                left_sidebar: Some(RegionSpec::Enabled(false)),
                ..Default::default()
            }],
            ..Default::default()
        };

        let resolved = config.resolve_for_route("/blog/post/");
        assert!(resolved.overrides.is_empty());
    }

    #[test]
    fn test_resolve_for_route_with_preset_override() {
        let config = LayoutConfig {
            preset: Some(LayoutPreset::Docs),
            overrides: vec![LayoutOverride {
                routes: vec!["/blog/**".to_string()],
                preset: Some(LayoutPreset::Blog),
                ..Default::default()
            }],
            ..Default::default()
        };

        let resolved = config.resolve_for_route("/docs/guide/");
        assert!(
            resolved
                .left_sidebar
                .as_ref()
                .is_some_and(|r| r.is_enabled())
        );
        assert_eq!(resolved.preset, Some(LayoutPreset::Docs));

        let resolved = config.resolve_for_route("/blog/post/");
        assert!(
            resolved
                .left_sidebar
                .as_ref()
                .is_some_and(|r| !r.is_enabled())
        );
        assert_eq!(resolved.preset, Some(LayoutPreset::Blog));
    }

    #[test]
    fn test_resolve_for_route_preset_override_with_region_override() {
        let config = LayoutConfig {
            preset: Some(LayoutPreset::Docs),
            overrides: vec![LayoutOverride {
                routes: vec!["/blog/**".to_string()],
                preset: Some(LayoutPreset::Blog),
                footer: Some(RegionSpec::Enabled(false)),
                ..Default::default()
            }],
            ..Default::default()
        };

        let resolved = config.resolve_for_route("/blog/post/");

        assert!(
            resolved
                .left_sidebar
                .as_ref()
                .is_some_and(|r| !r.is_enabled())
        );
        assert!(
            resolved
                .right_sidebar
                .as_ref()
                .is_some_and(|r| r.is_enabled())
        );
        assert!(resolved.footer.as_ref().is_some_and(|r| !r.is_enabled()));
    }

    #[test]
    fn test_resolve_for_route_landing_preset_override() {
        let config = LayoutConfig {
            preset: Some(LayoutPreset::Docs),
            overrides: vec![LayoutOverride {
                routes: vec!["/", "/features/"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                preset: Some(LayoutPreset::Landing),
                ..Default::default()
            }],
            ..Default::default()
        };

        let resolved = config.resolve_for_route("/");
        assert!(
            resolved
                .left_sidebar
                .as_ref()
                .is_some_and(|r| !r.is_enabled())
        );
        assert!(
            resolved
                .right_sidebar
                .as_ref()
                .is_some_and(|r| !r.is_enabled())
        );
        assert_eq!(resolved.preset, Some(LayoutPreset::Landing));

        let resolved = config.resolve_for_route("/docs/guide/");
        assert!(
            resolved
                .left_sidebar
                .as_ref()
                .is_some_and(|r| r.is_enabled())
        );
        assert_eq!(resolved.preset, Some(LayoutPreset::Docs));
    }

    #[test]
    fn test_override_preset_parsing() -> Result<()> {
        let toml = r#"
            preset = "docs"

            [[overrides]]
            routes = ["/blog/**"]
            preset = "blog"

            [[overrides]]
            routes = ["/"]
            preset = "landing"
            footer = false
        "#;
        let config: LayoutConfig = toml::from_str(toml)?;

        assert_eq!(config.preset, Some(LayoutPreset::Docs));
        assert_eq!(config.overrides.len(), 2);
        assert_eq!(config.overrides[0].preset, Some(LayoutPreset::Blog));
        assert_eq!(config.overrides[1].preset, Some(LayoutPreset::Landing));
        assert!(matches!(
            config.overrides[1].footer,
            Some(RegionSpec::Enabled(false))
        ));

        Ok(())
    }

    #[test]
    fn test_resolve_for_route_expands_named_components_in_override() -> Result<()> {
        let toml = r#"
            preset = "docs"

            [components.custom-nav]
            type = "nav-tree"
            collapsible = false
            depth = 2

            [[overrides]]
            routes = ["/blog/**"]
            preset = "blog"
            left-sidebar.middle = "custom-nav"
        "#;
        let config: LayoutConfig = toml::from_str(toml)?;

        let resolved = config.resolve_for_route("/blog/post/");

        let sidebar = resolved
            .left_sidebar
            .as_ref()
            .expect("left-sidebar should be present");
        let region_config = sidebar.config().expect("should be config");
        let middle = region_config
            .middle
            .as_ref()
            .expect("middle should be present");

        assert_eq!(middle.len(), 1);
        if let ComponentSpec::Config(ComponentConfig::NavTree {
            collapsible, depth, ..
        }) = &middle[0]
        {
            assert_eq!(*collapsible, Some(false));
            assert_eq!(*depth, Some(2));
        } else {
            panic!("Expected ComponentSpec::Config(ComponentConfig::NavTree) after resolution");
        }

        Ok(())
    }

    #[test]
    fn test_resolve_for_route_expands_components_from_preset_override() -> Result<()> {
        let toml = r#"
            [components.custom-toc]
            type = "toc-tree"
            depth = 5

            [right-sidebar]
            start = "custom-toc"

            [[overrides]]
            routes = ["/blog/**"]
            preset = "blog"
            right-sidebar.start = "custom-toc"
        "#;
        let config: LayoutConfig = toml::from_str(toml)?;

        let resolved = config.resolve_for_route("/blog/post/");

        let sidebar = resolved
            .right_sidebar
            .as_ref()
            .expect("right-sidebar should be present");
        let region_config = sidebar.config().expect("should be config");
        let start = region_config
            .start
            .as_ref()
            .expect("start should be present");

        assert_eq!(start.len(), 1);
        if let ComponentSpec::Config(ComponentConfig::TocTree { depth, .. }) = &start[0] {
            assert_eq!(*depth, Some(5));
        } else {
            panic!("Expected ComponentSpec::Config(ComponentConfig::TocTree) after resolution");
        }

        Ok(())
    }

    #[test]
    fn test_resolve_for_route_global_config_persists_through_preset_override() -> Result<()> {
        let toml = r#"
            preset = "docs"

            [header]
            end = ["search", "color-mode"]

            [[overrides]]
            routes = ["/blog/**"]
            preset = "blog"
        "#;
        let config: LayoutConfig = toml::from_str(toml)?;

        let resolved = config.resolve_for_route("/docs/guide/");
        let header = resolved.header.as_ref().expect("header should be present");
        let region_config = header.config().expect("should be config");
        let end = region_config.end.as_ref().expect("end should be present");
        assert_eq!(end.len(), 2);
        assert!(matches!(&end[0], ComponentSpec::Name(n) if n == "search"));

        let resolved = config.resolve_for_route("/blog/post/");
        let header = resolved.header.as_ref().expect("header should be present");
        let region_config = header.config().expect("should be config");
        let end = region_config.end.as_ref().expect("end should be present");
        assert_eq!(end.len(), 2);
        assert!(
            matches!(&end[0], ComponentSpec::Name(n) if n == "search"),
            "Global header.end config should persist through preset override"
        );

        Ok(())
    }

    #[test]
    fn test_resolve_for_route_override_can_still_override_global() -> Result<()> {
        let toml = r#"
            preset = "docs"

            [header]
            end = ["search", "color-mode"]

            [[overrides]]
            routes = ["/blog/**"]
            preset = "blog"
            header.end = ["color-mode"]
        "#;
        let config: LayoutConfig = toml::from_str(toml)?;

        let resolved = config.resolve_for_route("/docs/guide/");
        let header = resolved.header.as_ref().expect("header should be present");
        let region_config = header.config().expect("should be config");
        let end = region_config.end.as_ref().expect("end should be present");
        assert_eq!(end.len(), 2);

        let resolved = config.resolve_for_route("/blog/post/");
        let header = resolved.header.as_ref().expect("header should be present");
        let region_config = header.config().expect("should be config");
        let end = region_config.end.as_ref().expect("end should be present");
        assert_eq!(end.len(), 1);
        assert!(
            matches!(&end[0], ComponentSpec::Name(n) if n == "color-mode"),
            "Override's explicit header.end should take precedence over global"
        );

        Ok(())
    }

    #[test]
    fn test_deep_merge_setting_only_end_preserves_start_and_middle() -> Result<()> {
        let toml = r#"
            preset = "docs"

            [header]
            end = ["search", "color-mode"]
        "#;
        let config: LayoutConfig = toml::from_str(toml)?;

        let resolved = config.resolve();

        let header = resolved.header.as_ref().expect("header should be present");
        let region_config = header.config().expect("should be config");

        let start = region_config
            .start
            .as_ref()
            .expect("start should be present from preset");
        assert_eq!(start.len(), 1);
        assert!(
            matches!(&start[0], ComponentSpec::Name(n) if n == "logo"),
            "preset's header.start should be preserved"
        );

        let middle = region_config
            .middle
            .as_ref()
            .expect("middle should be present from preset");
        assert_eq!(middle.len(), 1);
        assert!(
            matches!(&middle[0], ComponentSpec::Name(n) if n == "nav-menu"),
            "preset's header.middle should be preserved"
        );

        let end = region_config.end.as_ref().expect("end should be present");
        assert_eq!(end.len(), 2);
        assert!(matches!(&end[0], ComponentSpec::Name(n) if n == "search"));
        assert!(matches!(&end[1], ComponentSpec::Name(n) if n == "color-mode"));

        Ok(())
    }

    #[test]
    fn test_deep_merge_explicitly_empty_clears_subregion() -> Result<()> {
        let toml = r#"
            preset = "docs"

            [header]
            start = []
        "#;
        let config: LayoutConfig = toml::from_str(toml)?;

        let resolved = config.resolve();

        let header = resolved.header.as_ref().expect("header should be present");
        let region_config = header.config().expect("should be config");

        let start = region_config
            .start
            .as_ref()
            .expect("start should be Some (explicit empty)");
        assert!(
            start.is_empty(),
            "header.start = [] should clear the preset's start"
        );

        let middle = region_config
            .middle
            .as_ref()
            .expect("middle should be present from preset");
        assert_eq!(middle.len(), 1);

        Ok(())
    }

    #[test]
    fn test_deep_merge_in_route_override() -> Result<()> {
        let toml = r#"
            preset = "docs"

            [[overrides]]
            routes = ["/blog/**"]
            preset = "blog"
            header.end = ["rss-feed", "color-mode"]
        "#;
        let config: LayoutConfig = toml::from_str(toml)?;

        let resolved = config.resolve_for_route("/blog/post/");

        let header = resolved.header.as_ref().expect("header should be present");
        let region_config = header.config().expect("should be config");

        let start = region_config
            .start
            .as_ref()
            .expect("start should be present from blog preset");
        assert_eq!(start.len(), 1);
        assert!(
            matches!(&start[0], ComponentSpec::Name(n) if n == "logo"),
            "blog preset's header.start should be preserved"
        );

        let middle = region_config
            .middle
            .as_ref()
            .expect("middle should be present from blog preset");
        assert_eq!(middle.len(), 1);

        let end = region_config.end.as_ref().expect("end should be present");
        assert_eq!(end.len(), 2);
        assert!(matches!(&end[0], ComponentSpec::Name(n) if n == "rss-feed"));
        assert!(matches!(&end[1], ComponentSpec::Name(n) if n == "color-mode"));

        Ok(())
    }

    #[test]
    fn test_deep_merge_global_then_override_layers() -> Result<()> {
        let toml = r#"
            preset = "docs"

            [header]
            end = ["search", "color-mode"]

            [[overrides]]
            routes = ["/landing/"]
            preset = "landing"
            header.start = ["big-logo"]
        "#;
        let config: LayoutConfig = toml::from_str(toml)?;

        let resolved = config.resolve_for_route("/landing/");

        let header = resolved.header.as_ref().expect("header should be present");
        let region_config = header.config().expect("should be config");

        let start = region_config
            .start
            .as_ref()
            .expect("start should be present from override");
        assert_eq!(start.len(), 1);
        assert!(matches!(&start[0], ComponentSpec::Name(n) if n == "big-logo"));

        let middle = region_config
            .middle
            .as_ref()
            .expect("middle should be present from preset");
        assert_eq!(middle.len(), 1);
        assert!(matches!(&middle[0], ComponentSpec::Name(n) if n == "nav-menu"));

        let end = region_config.end.as_ref().expect("end should be present");
        assert_eq!(end.len(), 2);
        assert!(
            matches!(&end[0], ComponentSpec::Name(n) if n == "search"),
            "global header.end should persist through preset override"
        );

        Ok(())
    }
}
