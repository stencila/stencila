//! Site layout configuration types
//!
//! This module contains types for configuring the layout of site pages using
//! a region-based system where components can be placed in any region's sub-regions.
//!
//! ## Architecture
//!
//! The layout consists of regions (header, left-sidebar, top, content, bottom,
//! right-sidebar, footer), each with sub-regions (start, middle, end). Components
//! can be placed in any sub-region.
//!
//! ## Example
//!
//! ```toml
//! [site.layout.header]
//! start = "logo"
//! middle = { type = "nav-links", links = [...] }
//! end = ["icon-links", "color-mode"]
//!
//! [site.layout.left-sidebar]
//! middle = { type = "nav-tree", collapsible = true }
//! ```

use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum::Display;

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
    pub fn validate(&self) -> eyre::Result<()> {
        for (index, override_config) in self.overrides.iter().enumerate() {
            override_config
                .validate()
                .map_err(|e| eyre::eyre!("Invalid layout override at index {}: {}", index, e))?;
        }
        Ok(())
    }

    /// Resolve the layout configuration by merging preset defaults with explicit config
    ///
    /// If a preset is specified, its defaults are used as the base and any explicit
    /// region configurations override those defaults. If no preset is specified,
    /// the explicit config is used as-is.
    pub fn resolve(&self) -> Self {
        let base = match &self.preset {
            Some(preset) => preset.defaults(),
            None => Self::default(),
        };

        Self {
            preset: self.preset,
            components: {
                let mut merged = base.components;
                merged.extend(self.components.clone());
                merged
            },
            header: merge_region(&base.header, &self.header),
            left_sidebar: merge_region(&base.left_sidebar, &self.left_sidebar),
            top: merge_region(&base.top, &self.top),
            bottom: merge_region(&base.bottom, &self.bottom),
            right_sidebar: merge_region(&base.right_sidebar, &self.right_sidebar),
            footer: merge_region(&base.footer, &self.footer),
            overrides: self.overrides.clone(),
        }
    }
}

/// Merge two optional region specs (override takes precedence over base)
fn merge_region(
    base: &Option<RegionSpec>,
    override_spec: &Option<RegionSpec>,
) -> Option<RegionSpec> {
    match (base, override_spec) {
        // Override completely replaces base
        (_, Some(override_spec)) => Some(override_spec.clone()),
        // No override, use base
        (Some(base), None) => Some(base.clone()),
        // Neither specified
        (None, None) => None,
    }
}

/// Named layout presets for common documentation patterns
#[derive(
    Debug, Clone, Copy, Default, Display, Serialize, Deserialize, PartialEq, Eq, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum LayoutPreset {
    /// Documentation site: nav-tree left, toc-tree right, breadcrumbs, page-nav
    #[default]
    Docs,

    /// Blog/article site: no left sidebar, toc-tree right, no page-nav
    Blog,

    /// Landing page: no sidebars, centered content
    Landing,

    /// API reference: nav-tree left (flat), no right sidebar
    Api,
}

impl LayoutPreset {
    /// Get the default layout configuration for this preset
    pub fn defaults(&self) -> LayoutConfig {
        match self {
            Self::Docs => LayoutConfig {
                header: Some(RegionSpec::Config(RegionConfig {
                    start: Some(vec![ComponentSpec::Name("logo".into())]),
                    middle: Some(vec![ComponentSpec::Name("title".into())]),
                    end: Some(vec![ComponentSpec::Name("color-mode".into())]),
                    ..Default::default()
                })),
                left_sidebar: Some(RegionSpec::Config(RegionConfig {
                    middle: Some(vec![ComponentSpec::Name("nav-tree".into())]),
                    ..Default::default()
                })),
                top: Some(RegionSpec::Config(RegionConfig {
                    start: Some(vec![ComponentSpec::Name("breadcrumbs".into())]),
                    ..Default::default()
                })),
                bottom: Some(RegionSpec::Config(RegionConfig {
                    middle: Some(vec![ComponentSpec::Name("page-nav".into())]),
                    ..Default::default()
                })),
                right_sidebar: Some(RegionSpec::Config(RegionConfig {
                    start: Some(vec![ComponentSpec::Name("toc-tree".into())]),
                    ..Default::default()
                })),
                footer: Some(RegionSpec::Config(RegionConfig {
                    middle: Some(vec![ComponentSpec::Name("copyright".into())]),
                    ..Default::default()
                })),
                ..Default::default()
            },
            Self::Blog => LayoutConfig {
                header: Some(RegionSpec::Config(RegionConfig {
                    start: Some(vec![ComponentSpec::Name("logo".into())]),
                    middle: Some(vec![ComponentSpec::Name("title".into())]),
                    end: Some(vec![ComponentSpec::Name("color-mode".into())]),
                    ..Default::default()
                })),
                left_sidebar: Some(RegionSpec::Enabled(false)),
                top: Some(RegionSpec::Enabled(false)),
                bottom: Some(RegionSpec::Enabled(false)),
                right_sidebar: Some(RegionSpec::Config(RegionConfig {
                    start: Some(vec![ComponentSpec::Name("toc-tree".into())]),
                    ..Default::default()
                })),
                footer: Some(RegionSpec::Config(RegionConfig {
                    middle: Some(vec![ComponentSpec::Name("copyright".into())]),
                    ..Default::default()
                })),
                ..Default::default()
            },
            Self::Landing => LayoutConfig {
                header: Some(RegionSpec::Config(RegionConfig {
                    start: Some(vec![ComponentSpec::Name("logo".into())]),
                    middle: Some(vec![ComponentSpec::Name("title".into())]),
                    end: Some(vec![ComponentSpec::Name("color-mode".into())]),
                    ..Default::default()
                })),
                left_sidebar: Some(RegionSpec::Enabled(false)),
                top: Some(RegionSpec::Enabled(false)),
                bottom: Some(RegionSpec::Enabled(false)),
                right_sidebar: Some(RegionSpec::Enabled(false)),
                footer: Some(RegionSpec::Config(RegionConfig {
                    middle: Some(vec![ComponentSpec::Name("copyright".into())]),
                    ..Default::default()
                })),
                ..Default::default()
            },
            Self::Api => LayoutConfig {
                header: Some(RegionSpec::Config(RegionConfig {
                    start: Some(vec![ComponentSpec::Name("logo".into())]),
                    middle: Some(vec![ComponentSpec::Name("title".into())]),
                    end: Some(vec![ComponentSpec::Name("color-mode".into())]),
                    ..Default::default()
                })),
                left_sidebar: Some(RegionSpec::Config(RegionConfig {
                    middle: Some(vec![ComponentSpec::Config(ComponentWithCondition {
                        condition: None,
                        component: ComponentConfig::NavTree {
                            items: None,
                            collapsible: Some(false),
                            depth: None,
                        },
                    })]),
                    ..Default::default()
                })),
                top: Some(RegionSpec::Config(RegionConfig {
                    start: Some(vec![ComponentSpec::Name("breadcrumbs".into())]),
                    ..Default::default()
                })),
                bottom: Some(RegionSpec::Config(RegionConfig {
                    middle: Some(vec![ComponentSpec::Name("page-nav".into())]),
                    ..Default::default()
                })),
                right_sidebar: Some(RegionSpec::Enabled(false)),
                footer: Some(RegionSpec::Config(RegionConfig {
                    middle: Some(vec![ComponentSpec::Name("copyright".into())]),
                    ..Default::default()
                })),
                ..Default::default()
            },
        }
    }
}

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
/// All sub-regions are Option:
/// - None = inherit from base/defaults
/// - Some([]) = explicitly empty
/// - Some([...]) = these components
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

/// Component specification: string name or full configuration
///
/// Resolution order for string names:
/// 1. Check site.layout.components for named component
/// 2. Fall back to built-in component type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum ComponentSpec {
    /// Simple type name: "logo" or named component: "main-nav"
    Name(String),

    /// Full config with optional condition
    Config(ComponentWithCondition),
}

/// Wrapper that adds optional `if` condition to any component
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ComponentWithCondition {
    /// Condition for showing this component
    ///
    /// Supported conditions:
    /// - `site.search.enabled` - Search feature is configured
    /// - `document.headings` - Current document has headings
    /// - `site.multi-page` - Site has multiple pages
    #[serde(rename = "if")]
    pub condition: Option<String>,

    /// The component configuration
    #[serde(flatten)]
    pub component: ComponentConfig,
}

/// Component configuration (internally tagged by type)
///
/// All fields are Option - bare string usage gets defaults from site config
/// at resolution time.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ComponentConfig {
    /// Site logo image
    Logo {
        /// Path to logo image (relative to site root)
        src: Option<String>,

        /// Link target when logo is clicked (default: "/")
        link: Option<String>,
    },

    /// Site title text
    Title {
        /// Title text (defaults to site.title)
        text: Option<String>,
    },

    /// Breadcrumb navigation trail
    Breadcrumbs,

    /// Hierarchical navigation tree
    NavTree {
        /// Specific items to show (defaults to auto-generated from site structure)
        items: Option<Vec<NavTreeItem>>,

        /// Whether groups are collapsible (default: true)
        collapsible: Option<bool>,

        /// Maximum depth (default: 3)
        depth: Option<u8>,
    },

    /// Table of contents tree from document headings
    TocTree {
        /// Title above the TOC (default: "On this page")
        title: Option<String>,

        /// Maximum heading depth (default: 3)
        depth: Option<u8>,
    },

    /// Previous/next page navigation links
    PageNav,

    /// Light/dark mode toggle
    ColorMode {
        /// Display style (default: icon)
        style: Option<ColorModeStyle>,
    },

    /// Copyright text
    Copyright {
        /// Copyright text (defaults to site.copyright)
        text: Option<String>,
    },
}

/// Navigation tree item for explicit nav configuration
///
/// Used in `NavTree.items` to explicitly define navigation structure instead of
/// auto-generating from the file system. This gives full control over ordering,
/// labels, grouping, and which pages appear in navigation.
///
/// ## Variants
///
/// ### Route (string shorthand)
///
/// A simple route path. The label is derived from the route (e.g., "/docs/guide/"
/// becomes "Guide"). Use this for quick references to existing pages:
/// ```toml
/// items = ["/docs/", "/docs/getting-started/", "/docs/guide/"]
/// ```
///
/// ### Link (object with label)
///
/// Explicit label and target. Use when you need a custom label or linking to
/// external URLs:
/// ```toml
/// items = [
///   { label = "Getting Started", target = "/docs/getting-started/" },
///   { label = "GitHub", target = "https://github.com/example", icon = "github" }
/// ]
/// ```
///
/// ### Group (object with children)
///
/// A collapsible group containing nested items. The group header can optionally link
/// to a page, or just act as an expand/collapse toggle:
/// ```toml
/// items = [
///   # Group with clickable header linking to /docs/
///   { label = "Guides", target = "/docs/", children = [
///     "/docs/installation/",
///     "/docs/configuration/"
///   ]},
///   # Group without target - header only expands/collapses
///   { label = "Community", children = [
///     { label = "Discord", target = "https://discord.gg/example", icon = "message-circle" },
///     { label = "GitHub", target = "https://github.com/example", icon = "github" }
///   ]}
/// ]
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum NavTreeItem {
    /// Route path shorthand - label derived from route
    ///
    /// Example: `"/docs/guide/"` → label "Guide", href "/docs/guide/"
    Route(String),

    /// Link with explicit label and optional icon
    ///
    /// Use for custom labels or external links.
    Link {
        /// Display text for the navigation item
        label: String,

        /// URL or route path to link to
        target: String,

        /// Optional icon name (e.g., "github", "book", "settings")
        icon: Option<String>,
    },

    /// Collapsible group with nested children
    ///
    /// Groups can optionally link to a page (making the header clickable).
    /// If `target` is omitted, the header just expands/collapses the children.
    Group {
        /// Display text for the group header
        label: String,

        /// Optional URL or route path for the group header link
        ///
        /// When set, clicking the group header navigates to this page.
        /// When omitted, the header only toggles expand/collapse.
        target: Option<String>,

        /// Optional icon name
        icon: Option<String>,

        /// Nested navigation items (can include routes, links, or more groups)
        children: Vec<NavTreeItem>,
    },
}

/// Display style for color mode switcher
#[derive(
    Debug, Clone, Copy, Default, Display, Serialize, Deserialize, PartialEq, Eq, JsonSchema,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ColorModeStyle {
    /// Sun/moon icon only (default)
    #[default]
    Icon,

    /// "Light"/"Dark" text label only
    Label,

    /// Icon and label
    Both,
}

/// Route-specific layout override
///
/// First matching override wins (order matters in the array).
///
/// Example:
/// ```toml
/// [[site.layout.overrides]]
/// routes = ["/blog/**"]
/// left-sidebar = false
/// bottom = false
///
/// [[site.layout.overrides]]
/// routes = ["/api/**"]
/// left-sidebar.middle = "api-nav"
/// right-sidebar = false
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct LayoutOverride {
    /// Glob patterns for routes this override applies to (required, non-empty)
    ///
    /// Examples: `["/blog/**"]`, `["/docs/api/**", "/api/**"]`
    pub routes: Vec<String>,

    /// Header region override
    pub header: Option<RegionSpec>,

    /// Left sidebar region override
    pub left_sidebar: Option<RegionSpec>,

    /// Top region override
    pub top: Option<RegionSpec>,

    /// Bottom region override
    pub bottom: Option<RegionSpec>,

    /// Right sidebar region override
    pub right_sidebar: Option<RegionSpec>,

    /// Footer region override
    pub footer: Option<RegionSpec>,
}

impl LayoutOverride {
    /// Validate the layout override configuration
    pub fn validate(&self) -> eyre::Result<()> {
        if self.routes.is_empty() {
            eyre::bail!(
                "Layout override must have at least one route pattern in `routes`. \
                 An override with empty routes would never match any page."
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use eyre::Result;

    use super::*;

    #[test]
    fn test_preset_parsing() -> Result<()> {
        let toml = r#"preset = "docs""#;
        let layout: LayoutConfig = toml::from_str(toml)?;
        assert_eq!(layout.preset, Some(LayoutPreset::Docs));

        let toml = r#"preset = "blog""#;
        let layout: LayoutConfig = toml::from_str(toml)?;
        assert_eq!(layout.preset, Some(LayoutPreset::Blog));

        let toml = r#"preset = "landing""#;
        let layout: LayoutConfig = toml::from_str(toml)?;
        assert_eq!(layout.preset, Some(LayoutPreset::Landing));

        let toml = r#"preset = "api""#;
        let layout: LayoutConfig = toml::from_str(toml)?;
        assert_eq!(layout.preset, Some(LayoutPreset::Api));

        Ok(())
    }

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
            ComponentSpec::Config(c) if matches!(c.component, ComponentConfig::Logo { .. })
        ));

        Ok(())
    }

    #[test]
    fn test_component_with_condition() -> Result<()> {
        let toml = r#"
            [right-sidebar]
            start = [{ type = "toc-tree", if = "document.headings" }]
        "#;
        let layout: LayoutConfig = toml::from_str(toml)?;

        let sidebar = layout
            .right_sidebar
            .expect("right-sidebar should be present");
        let config = sidebar.config().expect("should be config");
        let start = config.start.as_ref().expect("start should be present");
        assert_eq!(start.len(), 1);

        if let ComponentSpec::Config(c) = &start[0] {
            assert_eq!(c.condition, Some("document.headings".to_string()));
            assert!(matches!(c.component, ComponentConfig::TocTree { .. }));
        } else {
            panic!("Expected ComponentSpec::Config");
        }

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
            middle = "page-nav"

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
    fn test_component_types() -> Result<()> {
        // Test all component types can be parsed
        let components = vec![
            r#"type = "logo""#,
            r#"type = "title""#,
            r#"type = "nav-tree""#,
            r#"type = "toc-tree""#,
            r#"type = "breadcrumbs""#,
            r#"type = "page-nav""#,
            r#"type = "color-mode""#,
            r#"type = "copyright""#,
        ];

        for component_toml in components {
            let _: ComponentConfig = toml::from_str(component_toml)?;
        }

        Ok(())
    }

    #[test]
    fn test_override_empty_routes_validation() {
        // An override with empty routes should fail validation
        let layout = LayoutConfig {
            overrides: vec![LayoutOverride {
                routes: vec![], // Empty routes
                left_sidebar: Some(RegionSpec::Enabled(false)),
                ..Default::default()
            }],
            ..Default::default()
        };

        let result = layout.validate();
        assert!(result.is_err());
        let err_msg = result
            .expect_err("validation should fail for empty routes")
            .to_string();
        assert!(
            err_msg.contains("at least one route pattern"),
            "Error message should mention missing routes: {err_msg}"
        );
    }

    #[test]
    fn test_override_with_routes_validation() -> Result<()> {
        // An override with routes should pass validation
        let layout = LayoutConfig {
            overrides: vec![LayoutOverride {
                routes: vec!["/blog/**".to_string()],
                left_sidebar: Some(RegionSpec::Enabled(false)),
                ..Default::default()
            }],
            ..Default::default()
        };

        layout.validate()?;
        Ok(())
    }

    #[test]
    fn test_color_mode_style() -> Result<()> {
        let toml = r#"
            [header]
            end = [{ type = "color-mode", style = "both" }]
        "#;
        let layout: LayoutConfig = toml::from_str(toml)?;

        let header = layout.header.expect("header should be present");
        let config = header.config().expect("should be config");
        let end = config.end.as_ref().expect("end should be present");

        if let ComponentSpec::Config(c) = &end[0] {
            if let ComponentConfig::ColorMode { style } = &c.component {
                assert_eq!(*style, Some(ColorModeStyle::Both));
            } else {
                panic!("Expected ColorMode component");
            }
        } else {
            panic!("Expected ComponentSpec::Config");
        }

        Ok(())
    }

    #[test]
    fn test_preset_defaults() {
        // Each preset should have sensible defaults
        let docs = LayoutPreset::Docs.defaults();
        assert!(docs.header.is_some());
        assert!(docs.left_sidebar.as_ref().is_some_and(|r| r.is_enabled()));
        assert!(docs.right_sidebar.as_ref().is_some_and(|r| r.is_enabled()));

        let blog = LayoutPreset::Blog.defaults();
        assert!(blog.left_sidebar.as_ref().is_some_and(|r| !r.is_enabled()));
        assert!(blog.right_sidebar.as_ref().is_some_and(|r| r.is_enabled()));

        let landing = LayoutPreset::Landing.defaults();
        assert!(
            landing
                .left_sidebar
                .as_ref()
                .is_some_and(|r| !r.is_enabled())
        );
        assert!(
            landing
                .right_sidebar
                .as_ref()
                .is_some_and(|r| !r.is_enabled())
        );

        let api = LayoutPreset::Api.defaults();
        assert!(api.left_sidebar.as_ref().is_some_and(|r| r.is_enabled()));
        assert!(api.right_sidebar.as_ref().is_some_and(|r| !r.is_enabled()));
    }

    #[test]
    fn test_resolve_no_preset() {
        // Without a preset, resolve returns the explicit config
        let config = LayoutConfig {
            header: Some(RegionSpec::Enabled(false)),
            ..Default::default()
        };

        let resolved = config.resolve();
        assert!(matches!(resolved.header, Some(RegionSpec::Enabled(false))));
        // Other regions should remain None
        assert!(resolved.left_sidebar.is_none());
    }

    #[test]
    fn test_resolve_preset_only() {
        // With only a preset, resolve returns preset defaults
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
        // Explicit config should override preset defaults
        let config = LayoutConfig {
            preset: Some(LayoutPreset::Docs),
            // Override left sidebar to be disabled (docs normally has it enabled)
            left_sidebar: Some(RegionSpec::Enabled(false)),
            ..Default::default()
        };

        let resolved = config.resolve();
        // Left sidebar should be disabled (overridden)
        assert!(
            resolved
                .left_sidebar
                .as_ref()
                .is_some_and(|r| !r.is_enabled())
        );
        // Right sidebar should still be enabled (from preset)
        assert!(
            resolved
                .right_sidebar
                .as_ref()
                .is_some_and(|r| r.is_enabled())
        );
    }

    #[test]
    fn test_resolve_preset_with_components() -> Result<()> {
        // Named components should be merged
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
}
