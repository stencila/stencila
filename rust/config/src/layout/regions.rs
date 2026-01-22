//! Layout region types
//!
//! This module contains types for configuring layout regions (header, sidebars,
//! top, bottom, footer) and their sub-regions (start, middle, end).

use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum::Display;

use super::components::{ComponentConfig, ComponentSpec};

/// Region specification that can be enabled/disabled or fully configured
///
/// Supports boolean shorthand and full configuration:
/// - `region = false` → Region disabled
/// - `region = true` → Region with smart defaults
/// - `region = { start = [...], middle = [...] }` → Full configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum RegionSpec {
    /// Boolean: false = disabled, true = use smart defaults
    Enabled(bool),

    /// Full configuration with sub-regions
    Config(RegionConfig),
}

impl Default for RegionSpec {
    fn default() -> Self {
        Self::Enabled(true)
    }
}

impl RegionSpec {
    /// Check if the region is enabled
    pub fn is_enabled(&self) -> bool {
        match self {
            Self::Enabled(enabled) => *enabled,
            Self::Config(config) => config.enabled.unwrap_or(true),
        }
    }

    /// Get the configuration if this is a Config variant
    pub fn config(&self) -> Option<&RegionConfig> {
        match self {
            Self::Config(config) => Some(config),
            Self::Enabled(_) => None,
        }
    }
}

/// Region with sub-regions (start, middle, end)
///
/// All sub-regions are optional:
///
/// - `null`: inherit from base/defaults
/// - `[]`: explicitly empty region
/// - `[...]`: region with these components
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct RegionConfig {
    /// Explicit enable/disable (for use in overrides that also set sub-regions)
    pub enabled: Option<bool>,

    /// Components in the start sub-region (left for horizontal, top for vertical)
    #[serde(default, deserialize_with = "deserialize_component_list")]
    pub start: Option<Vec<ComponentSpec>>,

    /// Components in the middle sub-region (center)
    #[serde(default, deserialize_with = "deserialize_component_list")]
    pub middle: Option<Vec<ComponentSpec>>,

    /// Components in the end sub-region (right for horizontal, bottom for vertical)
    #[serde(default, deserialize_with = "deserialize_component_list")]
    pub end: Option<Vec<ComponentSpec>>,

    /// Multiple rows, each with their own start/middle/end sub-regions
    ///
    /// When specified, `start`, `middle`, and `end` are ignored and each row
    /// is rendered separately. This enables multi-row layouts within a region.
    ///
    /// Example:
    /// ```toml
    /// [site.layout.bottom]
    /// rows = [
    ///   { middle = "prev-next" },
    ///   { start = "edit-source", end = "last-edited" }
    /// ]
    /// ```
    pub rows: Option<Vec<RowConfig>>,

    /// Responsive configuration (only applicable to sidebars)
    ///
    /// Controls when the sidebar becomes collapsible and how the toggle appears.
    pub responsive: Option<ResponsiveConfig>,
}

/// A single row within a region, containing start/middle/end sub-regions
///
/// Used with `RegionConfig.rows` for multi-row layouts. Each row has
/// the same sub-region structure as a single-row region.
///
/// Example TOML:
/// ```toml
/// [site.layout.bottom]
/// rows = [
///   { middle = "prev-next" },
///   { start = "edit-source", end = "last-edited" }
/// ]
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct RowConfig {
    /// Components in the start sub-region (left for horizontal, top for vertical)
    #[serde(default, deserialize_with = "deserialize_component_list")]
    pub start: Option<Vec<ComponentSpec>>,

    /// Components in the middle sub-region (center)
    #[serde(default, deserialize_with = "deserialize_component_list")]
    pub middle: Option<Vec<ComponentSpec>>,

    /// Components in the end sub-region (right for horizontal, bottom for vertical)
    #[serde(default, deserialize_with = "deserialize_component_list")]
    pub end: Option<Vec<ComponentSpec>>,
}

/// Custom deserializer that accepts a single component or array of components
fn deserialize_component_list<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<ComponentSpec>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, SeqAccess, Visitor};

    struct ComponentListVisitor;

    impl<'de> Visitor<'de> for ComponentListVisitor {
        type Value = Option<Vec<ComponentSpec>>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("null, a string, an object, or an array of components")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(None)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(vec![ComponentSpec::Name(value.to_string())]))
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(vec![ComponentSpec::Name(value)]))
        }

        fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
        where
            M: de::MapAccess<'de>,
        {
            let component = ComponentSpec::deserialize(de::value::MapAccessDeserializer::new(map))?;
            Ok(Some(vec![component]))
        }

        fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
        where
            S: SeqAccess<'de>,
        {
            let mut components = Vec::new();
            while let Some(component) = seq.next_element::<ComponentSpec>()? {
                components.push(component);
            }
            Ok(Some(components))
        }
    }

    deserializer.deserialize_any(ComponentListVisitor)
}

/// Toggle button style for collapsible sidebars
#[derive(
    Debug, Clone, Copy, Default, Display, Serialize, Deserialize, PartialEq, Eq, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum SidebarToggleStyle {
    /// Fixed edge buttons (buttons fixed to left/right viewport edges)
    #[default]
    FixedEdge,

    /// Header buttons (toggle buttons inside header region)
    Header,

    /// Hamburger menu (single button for all sidebars)
    Hamburger,
}

/// Responsive configuration for layout sidebars
///
/// Controls when sidebars collapse and how toggle buttons appear.
///
/// Example:
/// ```toml
/// [site.layout.responsive]
/// breakpoint = 1024
/// toggle-style = "fixed-edge"
///
/// [site.layout.left-sidebar]
/// responsive.collapsible = false
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ResponsiveConfig {
    /// Breakpoint at which sidebars collapse (in pixels)
    ///
    /// Default: 1024
    pub breakpoint: Option<u16>,

    /// Whether the sidebar is collapsible
    ///
    /// Default: true
    pub collapsible: Option<bool>,

    /// Toggle button style
    ///
    /// Default: fixed-edge
    pub toggle_style: Option<SidebarToggleStyle>,
}

/// Merge two optional region specs with deep merging of sub-regions
///
/// When both base and override are `RegionSpec::Config`, individual sub-regions
/// (start, middle, end) are merged rather than the whole region being replaced.
/// This allows setting only `header.end` without losing preset's `header.start`.
///
/// Sub-region merge semantics:
/// - `None` = inherit from base
/// - `Some([])` = explicitly empty (clears base)
/// - `Some([...])` = override with these components
pub(crate) fn merge_region(
    base: &Option<RegionSpec>,
    override_spec: &Option<RegionSpec>,
) -> Option<RegionSpec> {
    match (base, override_spec) {
        // Both are Config: deep merge sub-regions
        (Some(RegionSpec::Config(base_config)), Some(RegionSpec::Config(override_config))) => {
            // Override enabled flag if specified, otherwise inherit
            let enabled = override_config.enabled.or(base_config.enabled);

            // Merge each sub-region: override takes precedence if specified
            let start = merge_subregion(&base_config.start, &override_config.start);
            let middle = merge_subregion(&base_config.middle, &override_config.middle);
            let end = merge_subregion(&base_config.end, &override_config.end);

            // Rows: if override specifies rows, use them; if override specifies any
            // subregion but not rows, clear rows (intent is single-row mode);
            // otherwise inherit from base
            let override_has_subregions = override_config.start.is_some()
                || override_config.middle.is_some()
                || override_config.end.is_some();
            let rows = if override_config.rows.is_some() {
                override_config.rows.clone()
            } else if override_has_subregions {
                // Override specifies subregions without rows - clear rows
                None
            } else {
                base_config.rows.clone()
            };

            // Override responsive if specified, otherwise inherit
            let responsive = override_config
                .responsive
                .clone()
                .or(base_config.responsive.clone());

            Some(RegionSpec::Config(RegionConfig {
                enabled,
                start,
                middle,
                end,
                rows,
                responsive,
            }))
        }
        // Override is Enabled (bool): replaces base entirely (explicit enable/disable)
        (_, Some(RegionSpec::Enabled(enabled))) => Some(RegionSpec::Enabled(*enabled)),
        // Override is Config but base is Enabled or None: use override as-is
        (_, Some(override_spec)) => Some(override_spec.clone()),
        // No override, use base
        (Some(base), None) => Some(base.clone()),
        // Neither specified
        (None, None) => None,
    }
}

/// Merge two optional sub-region component lists
///
/// - `None` in override = inherit from base
/// - `Some([])` in override = explicitly empty
/// - `Some([...])` in override = use override components
pub(crate) fn merge_subregion(
    base: &Option<Vec<ComponentSpec>>,
    override_spec: &Option<Vec<ComponentSpec>>,
) -> Option<Vec<ComponentSpec>> {
    match override_spec {
        // Override specified (even if empty): use override
        Some(override_list) => Some(override_list.clone()),
        // No override: inherit from base
        None => base.clone(),
    }
}

/// Resolve named component references in a region spec
pub(crate) fn resolve_region(
    spec: Option<RegionSpec>,
    components: &HashMap<String, ComponentConfig>,
) -> Option<RegionSpec> {
    spec.map(|spec| match spec {
        RegionSpec::Enabled(enabled) => RegionSpec::Enabled(enabled),
        RegionSpec::Config(config) => RegionSpec::Config(RegionConfig {
            enabled: config.enabled,
            start: resolve_component_list(config.start, components),
            middle: resolve_component_list(config.middle, components),
            end: resolve_component_list(config.end, components),
            rows: config.rows.map(|rows| {
                rows.into_iter()
                    .map(|row| RowConfig {
                        start: resolve_component_list(row.start, components),
                        middle: resolve_component_list(row.middle, components),
                        end: resolve_component_list(row.end, components),
                    })
                    .collect()
            }),
            responsive: config.responsive,
        }),
    })
}

/// Resolve named component references in a component list
pub(crate) fn resolve_component_list(
    specs: Option<Vec<ComponentSpec>>,
    components: &HashMap<String, ComponentConfig>,
) -> Option<Vec<ComponentSpec>> {
    specs.map(|specs| {
        specs
            .into_iter()
            .map(|spec| resolve_component_spec(spec, components))
            .collect()
    })
}

/// Resolve a single component spec, expanding named references
///
/// If the spec is a `Name` that exists in `components`, it's expanded to a `Config`.
/// Otherwise, the spec is returned unchanged (names not in `components` are assumed
/// to be built-in component types like "logo", "nav-tree", etc.).
pub(crate) fn resolve_component_spec(
    spec: ComponentSpec,
    components: &HashMap<String, ComponentConfig>,
) -> ComponentSpec {
    match spec {
        ComponentSpec::Name(name) => {
            if let Some(config) = components.get(&name) {
                ComponentSpec::Config(config.clone())
            } else {
                // Not a named component - assume it's a built-in type
                ComponentSpec::Name(name)
            }
        }
        ComponentSpec::Config(config) => ComponentSpec::Config(config),
    }
}

#[cfg(test)]
mod tests {
    use eyre::Result;

    use super::*;

    #[test]
    fn test_region_spec_default() {
        let spec = RegionSpec::default();
        assert!(matches!(spec, RegionSpec::Enabled(true)));
        assert!(spec.is_enabled());
    }

    #[test]
    fn test_region_spec_enabled() {
        let enabled = RegionSpec::Enabled(true);
        assert!(enabled.is_enabled());
        assert!(enabled.config().is_none());

        let disabled = RegionSpec::Enabled(false);
        assert!(!disabled.is_enabled());
        assert!(disabled.config().is_none());
    }

    #[test]
    fn test_region_spec_config() {
        let config = RegionSpec::Config(RegionConfig {
            enabled: Some(false),
            ..Default::default()
        });
        assert!(!config.is_enabled());
        assert!(config.config().is_some());

        let config_default_enabled = RegionSpec::Config(RegionConfig::default());
        assert!(config_default_enabled.is_enabled());
    }

    #[test]
    fn test_region_config_parsing() -> Result<()> {
        let toml = r#"
            start = "logo"
            middle = ["nav-tree"]
            end = ["color-mode"]
        "#;
        let config: RegionConfig = toml::from_str(toml)?;

        assert!(config.start.is_some());
        assert_eq!(config.start.as_ref().map(|v| v.len()), Some(1));
        assert!(config.middle.is_some());
        assert!(config.end.is_some());

        Ok(())
    }

    #[test]
    fn test_responsive_config_parsing() -> Result<()> {
        let toml = r#"
            breakpoint = 1024
            collapsible = true
            toggle-style = "fixed-edge"
        "#;
        let config: ResponsiveConfig = toml::from_str(toml)?;

        assert_eq!(config.breakpoint, Some(1024));
        assert_eq!(config.collapsible, Some(true));
        assert_eq!(config.toggle_style, Some(SidebarToggleStyle::FixedEdge));

        Ok(())
    }

    #[test]
    fn test_sidebar_toggle_style_parsing() -> Result<()> {
        let styles = [
            ("fixed-edge", SidebarToggleStyle::FixedEdge),
            ("header", SidebarToggleStyle::Header),
            ("hamburger", SidebarToggleStyle::Hamburger),
        ];

        for (toml_value, expected) in styles {
            let toml = format!(r#"toggle-style = "{}""#, toml_value);
            let config: ResponsiveConfig = toml::from_str(&toml)?;
            assert_eq!(config.toggle_style, Some(expected));
        }

        Ok(())
    }

    #[test]
    fn test_merge_region_both_config() {
        let base = Some(RegionSpec::Config(RegionConfig {
            start: Some(vec![ComponentSpec::Name("logo".into())]),
            middle: Some(vec![ComponentSpec::Name("title".into())]),
            end: Some(vec![ComponentSpec::Name("search".into())]),
            ..Default::default()
        }));

        let override_spec = Some(RegionSpec::Config(RegionConfig {
            end: Some(vec![ComponentSpec::Name("color-mode".into())]),
            ..Default::default()
        }));

        let merged = merge_region(&base, &override_spec);

        if let Some(RegionSpec::Config(config)) = merged {
            // start should be inherited from base
            assert_eq!(config.start.as_ref().map(|v| v.len()), Some(1));
            // middle should be inherited from base
            assert_eq!(config.middle.as_ref().map(|v| v.len()), Some(1));
            // end should be overridden
            assert_eq!(config.end.as_ref().map(|v| v.len()), Some(1));
            assert!(
                matches!(&config.end.as_ref().expect("end should be present")[0], ComponentSpec::Name(n) if n == "color-mode")
            );
        } else {
            panic!("Expected RegionSpec::Config");
        }
    }

    #[test]
    fn test_merge_region_override_with_bool() {
        let base = Some(RegionSpec::Config(RegionConfig {
            start: Some(vec![ComponentSpec::Name("logo".into())]),
            ..Default::default()
        }));

        let override_spec = Some(RegionSpec::Enabled(false));

        let merged = merge_region(&base, &override_spec);
        assert!(matches!(merged, Some(RegionSpec::Enabled(false))));
    }

    #[test]
    fn test_merge_region_no_override() {
        let base = Some(RegionSpec::Config(RegionConfig {
            start: Some(vec![ComponentSpec::Name("logo".into())]),
            ..Default::default()
        }));

        let merged = merge_region(&base, &None);

        if let Some(RegionSpec::Config(config)) = merged {
            assert!(config.start.is_some());
        } else {
            panic!("Expected RegionSpec::Config");
        }
    }

    #[test]
    fn test_merge_subregion() {
        let base = Some(vec![ComponentSpec::Name("logo".into())]);
        let override_spec = Some(vec![ComponentSpec::Name("title".into())]);

        // Override takes precedence
        let merged = merge_subregion(&base, &override_spec);
        assert_eq!(merged.as_ref().map(|v| v.len()), Some(1));
        assert!(
            matches!(&merged.as_ref().expect("merged should be Some")[0], ComponentSpec::Name(n) if n == "title")
        );

        // None override inherits from base
        let merged = merge_subregion(&base, &None);
        assert!(
            matches!(&merged.as_ref().expect("merged should be Some")[0], ComponentSpec::Name(n) if n == "logo")
        );

        // Empty override clears base
        let merged = merge_subregion(&base, &Some(vec![]));
        assert!(merged.as_ref().expect("merged should be Some").is_empty());
    }

    #[test]
    fn test_resolve_component_spec() {
        let mut components = HashMap::new();
        components.insert(
            "custom-nav".to_string(),
            ComponentConfig::NavTree {
                title: None,
                depth: Some(2),
                collapsible: Some(false),
                expanded: None,
                scroll_to_active: None,
                include: None,
                exclude: None,
                icons: None,
            },
        );

        // Named component that exists in components map should be expanded
        let spec = ComponentSpec::Name("custom-nav".into());
        let resolved = resolve_component_spec(spec, &components);
        assert!(matches!(
            resolved,
            ComponentSpec::Config(ComponentConfig::NavTree { .. })
        ));

        // Built-in name should remain unchanged
        let spec = ComponentSpec::Name("logo".into());
        let resolved = resolve_component_spec(spec, &components);
        assert!(matches!(resolved, ComponentSpec::Name(n) if n == "logo"));

        // Config should remain unchanged
        let spec = ComponentSpec::Config(ComponentConfig::Breadcrumbs);
        let resolved = resolve_component_spec(spec, &components);
        assert!(matches!(
            resolved,
            ComponentSpec::Config(ComponentConfig::Breadcrumbs)
        ));
    }

    #[test]
    fn test_row_config_parsing() -> Result<()> {
        let toml = r#"
            rows = [
                { middle = "prev-next" },
                { start = "edit-source", end = "copyright" }
            ]
        "#;
        let config: RegionConfig = toml::from_str(toml)?;

        let rows = config.rows.expect("rows should be present");
        assert_eq!(rows.len(), 2);

        // First row has only middle
        assert!(rows[0].start.is_none());
        assert!(rows[0].middle.is_some());
        assert!(rows[0].end.is_none());

        // Second row has start and end
        assert!(rows[1].start.is_some());
        assert!(rows[1].middle.is_none());
        assert!(rows[1].end.is_some());

        Ok(())
    }

    #[test]
    fn test_merge_region_subregion_override_clears_rows() {
        // Base has rows
        let base = Some(RegionSpec::Config(RegionConfig {
            rows: Some(vec![
                RowConfig {
                    middle: Some(vec![ComponentSpec::Name("prev-next".into())]),
                    ..Default::default()
                },
                RowConfig {
                    start: Some(vec![ComponentSpec::Name("edit-source".into())]),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        }));

        // Override specifies subregions without rows - should clear rows
        let override_spec = Some(RegionSpec::Config(RegionConfig {
            middle: Some(vec![ComponentSpec::Name("prev-next".into())]),
            ..Default::default()
        }));

        let merged = merge_region(&base, &override_spec);

        if let Some(RegionSpec::Config(config)) = merged {
            // rows should be cleared because override has subregions
            assert!(config.rows.is_none(), "rows should be cleared");
            // middle should be set from override
            assert!(config.middle.is_some());
        } else {
            panic!("Expected RegionSpec::Config");
        }
    }

    #[test]
    fn test_merge_region_rows_override_replaces_rows() {
        // Base has rows
        let base = Some(RegionSpec::Config(RegionConfig {
            rows: Some(vec![RowConfig {
                middle: Some(vec![ComponentSpec::Name("prev-next".into())]),
                ..Default::default()
            }]),
            ..Default::default()
        }));

        // Override specifies different rows
        let override_spec = Some(RegionSpec::Config(RegionConfig {
            rows: Some(vec![
                RowConfig {
                    start: Some(vec![ComponentSpec::Name("edit-source".into())]),
                    ..Default::default()
                },
                RowConfig {
                    end: Some(vec![ComponentSpec::Name("copyright".into())]),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        }));

        let merged = merge_region(&base, &override_spec);

        if let Some(RegionSpec::Config(config)) = merged {
            let rows = config.rows.expect("rows should be present");
            assert_eq!(rows.len(), 2, "should have 2 rows from override");
        } else {
            panic!("Expected RegionSpec::Config");
        }
    }

    #[test]
    fn test_merge_region_rows_inherited_when_no_override() {
        // Base has rows
        let base = Some(RegionSpec::Config(RegionConfig {
            rows: Some(vec![RowConfig {
                middle: Some(vec![ComponentSpec::Name("prev-next".into())]),
                ..Default::default()
            }]),
            ..Default::default()
        }));

        // Override only changes enabled, not subregions or rows
        let override_spec = Some(RegionSpec::Config(RegionConfig {
            enabled: Some(true),
            ..Default::default()
        }));

        let merged = merge_region(&base, &override_spec);

        if let Some(RegionSpec::Config(config)) = merged {
            // rows should be inherited from base
            let rows = config.rows.expect("rows should be inherited");
            assert_eq!(rows.len(), 1);
        } else {
            panic!("Expected RegionSpec::Config");
        }
    }
}
