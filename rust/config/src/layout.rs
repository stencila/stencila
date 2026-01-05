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

use glob::Pattern;
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
fn merge_region(
    base: &Option<RegionSpec>,
    override_spec: &Option<RegionSpec>,
) -> Option<RegionSpec> {
    match (base, override_spec) {
        // Both are Config: deep merge sub-regions
        (Some(RegionSpec::Config(base_config)), Some(RegionSpec::Config(override_config))) => {
            Some(RegionSpec::Config(RegionConfig {
                // Override enabled flag if specified, otherwise inherit
                enabled: override_config.enabled.or(base_config.enabled),
                // Merge each sub-region: override takes precedence if specified
                start: merge_subregion(&base_config.start, &override_config.start),
                middle: merge_subregion(&base_config.middle, &override_config.middle),
                end: merge_subregion(&base_config.end, &override_config.end),
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
fn merge_subregion(
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
fn resolve_region(
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
        }),
    })
}

/// Resolve named component references in a component list
fn resolve_component_list(
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
fn resolve_component_spec(
    spec: ComponentSpec,
    components: &HashMap<String, ComponentConfig>,
) -> ComponentSpec {
    match spec {
        ComponentSpec::Name(name) => {
            if let Some(config) = components.get(&name) {
                ComponentSpec::Config(ComponentWithCondition {
                    condition: None,
                    component: config.clone(),
                })
            } else {
                // Not a named component - assume it's a built-in type
                ComponentSpec::Name(name)
            }
        }
        ComponentSpec::Config(config) => ComponentSpec::Config(config),
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

/// Built-in component type names (kebab-case as used in TOML)
const BUILTIN_COMPONENT_TYPES: &[&str] = &[
    "logo",
    "title",
    "breadcrumbs",
    "nav-tree",
    "toc-tree",
    "page-nav",
    "color-mode",
    "copyright",
];

/// Check if a name is a built-in component type
fn is_builtin_component_type(name: &str) -> bool {
    BUILTIN_COMPONENT_TYPES.contains(&name)
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
/// ## Layering Order
///
/// When resolving layout for a route, configuration is layered as follows:
///
/// 1. **Override preset** (if specified) - provides the base region defaults
/// 2. **Global explicit config** - `[site.layout.header]` etc. merged on top
/// 3. **Override explicit regions** - `header`, `left-sidebar`, etc. merged on top
///
/// This means global customizations (like adding search to the header) persist
/// across all routes, even those that switch to a different preset. To remove
/// a global customization for specific routes, explicitly override that region.
///
/// ## Examples
///
/// ```toml
/// [site.layout]
/// preset = "docs"
/// header.end = ["search", "color-mode"]  # Global: search on all pages
///
/// # Blog routes use blog preset, but keep the global header customization
/// [[site.layout.overrides]]
/// routes = ["/blog/**"]
/// preset = "blog"
///
/// # Landing pages use landing preset with explicit header override
/// [[site.layout.overrides]]
/// routes = ["/", "/features/"]
/// preset = "landing"
/// header.end = ["color-mode"]  # Override global: no search on landing
///
/// # Just override specific regions (no preset change)
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

    /// Preset to use for matching routes
    ///
    /// When specified, the preset's defaults are used as the base for this override.
    /// Global explicit config is then merged on top, followed by any explicit
    /// region overrides in this override.
    pub preset: Option<LayoutPreset>,

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

    #[test]
    fn test_resolve_named_component_reference() -> Result<()> {
        // Named component references should be expanded
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

        // The reference should be expanded to the full config
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
        if let ComponentSpec::Config(c) = &middle[0] {
            if let ComponentConfig::NavTree {
                collapsible, depth, ..
            } = &c.component
            {
                assert_eq!(*collapsible, Some(false));
                assert_eq!(*depth, Some(2));
            } else {
                panic!("Expected NavTree component");
            }
        } else {
            panic!("Expected ComponentSpec::Config after resolution");
        }

        Ok(())
    }

    #[test]
    fn test_resolve_builtin_name_unchanged() -> Result<()> {
        // Built-in component names should remain as Name variants
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
        // "logo" is not in components, so it should remain as Name
        assert!(matches!(&start[0], ComponentSpec::Name(n) if n == "logo"));

        Ok(())
    }

    #[test]
    fn test_resolve_mixed_named_and_builtin() -> Result<()> {
        // Can mix named component references with built-in names
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

        // start should have "logo" unchanged
        let start = region_config
            .start
            .as_ref()
            .expect("start should be present");
        assert!(matches!(&start[0], ComponentSpec::Name(n) if n == "logo"));

        // end should have expanded custom-toc and unchanged color-mode
        let end = region_config.end.as_ref().expect("end should be present");
        assert_eq!(end.len(), 2);

        // custom-toc should be expanded
        if let ComponentSpec::Config(c) = &end[0] {
            assert!(matches!(c.component, ComponentConfig::TocTree { .. }));
        } else {
            panic!("Expected custom-toc to be expanded");
        }

        // color-mode should remain as Name
        assert!(matches!(&end[1], ComponentSpec::Name(n) if n == "color-mode"));

        Ok(())
    }

    #[test]
    fn test_validate_builtin_component_names() -> Result<()> {
        // Built-in component names should pass validation
        let toml = r#"
            [header]
            start = "logo"
            middle = "title"
            end = ["breadcrumbs", "nav-tree", "toc-tree", "page-nav", "color-mode", "copyright"]
        "#;
        let config: LayoutConfig = toml::from_str(toml)?;
        config.validate()?;

        Ok(())
    }

    #[test]
    fn test_validate_named_component_reference() -> Result<()> {
        // Named components defined in components should pass validation
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
        // Unknown component name should fail validation
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
        // Reference to component not defined in components map should fail
        let toml = r#"
            [left-sidebar]
            middle = "my-nav"
        "#;
        // Note: my-nav is NOT defined in components
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
    fn test_resolve_for_route_no_override() {
        // Route that doesn't match any override should return base resolved config
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

        // Should have left sidebar (from docs preset, not affected by blog override)
        assert!(
            resolved
                .left_sidebar
                .as_ref()
                .is_some_and(|r| r.is_enabled())
        );
    }

    #[test]
    fn test_resolve_for_route_with_matching_override() {
        // Route that matches override should have override applied
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

        // Left sidebar should be disabled (from override)
        assert!(
            resolved
                .left_sidebar
                .as_ref()
                .is_some_and(|r| !r.is_enabled())
        );
        // Bottom should be disabled (from override)
        assert!(resolved.bottom.as_ref().is_some_and(|r| !r.is_enabled()));
        // Header should still be enabled (from preset, not overridden)
        assert!(resolved.header.as_ref().is_some_and(|r| r.is_enabled()));
    }

    #[test]
    fn test_resolve_for_route_first_match_wins() {
        // First matching override should be applied
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
                    // right_sidebar not overridden here
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        // /blog/special/post/ should match first override (both sidebars disabled)
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

        // /blog/regular/ should match second override (only left sidebar disabled)
        let resolved = config.resolve_for_route("/blog/regular/");
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
    fn test_resolve_for_route_glob_patterns() {
        // Test various glob patterns
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

        // /api/users/ should match first override
        let resolved = config.resolve_for_route("/api/users/");
        assert!(resolved.header.as_ref().is_some_and(|r| !r.is_enabled()));

        // /docs/guide/intro/ should match second override
        let resolved = config.resolve_for_route("/docs/guide/intro/");
        assert!(resolved.footer.as_ref().is_some_and(|r| !r.is_enabled()));

        // /docs/guide/advanced/ should not match second override
        let resolved = config.resolve_for_route("/docs/guide/advanced/");
        assert!(resolved.footer.is_none()); // No footer config at all
    }

    #[test]
    fn test_resolve_for_route_clears_overrides() {
        // Resolved config should have empty overrides (they've been applied)
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
        // Override with a different preset should use that preset's defaults
        let config = LayoutConfig {
            preset: Some(LayoutPreset::Docs),
            overrides: vec![LayoutOverride {
                routes: vec!["/blog/**".to_string()],
                preset: Some(LayoutPreset::Blog),
                ..Default::default()
            }],
            ..Default::default()
        };

        // Non-blog route should use docs preset (left sidebar enabled)
        let resolved = config.resolve_for_route("/docs/guide/");
        assert!(
            resolved
                .left_sidebar
                .as_ref()
                .is_some_and(|r| r.is_enabled())
        );
        assert_eq!(resolved.preset, Some(LayoutPreset::Docs));

        // Blog route should use blog preset (left sidebar disabled)
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
        // Override with preset AND explicit region should apply region on top of preset
        let config = LayoutConfig {
            preset: Some(LayoutPreset::Docs),
            overrides: vec![LayoutOverride {
                routes: vec!["/blog/**".to_string()],
                preset: Some(LayoutPreset::Blog),
                // Blog preset disables left sidebar, but we'll enable the right sidebar
                // (which blog preset has enabled by default) and disable footer
                footer: Some(RegionSpec::Enabled(false)),
                ..Default::default()
            }],
            ..Default::default()
        };

        let resolved = config.resolve_for_route("/blog/post/");

        // Left sidebar should be disabled (from blog preset)
        assert!(
            resolved
                .left_sidebar
                .as_ref()
                .is_some_and(|r| !r.is_enabled())
        );
        // Right sidebar should be enabled (from blog preset)
        assert!(
            resolved
                .right_sidebar
                .as_ref()
                .is_some_and(|r| r.is_enabled())
        );
        // Footer should be disabled (explicit override)
        assert!(resolved.footer.as_ref().is_some_and(|r| !r.is_enabled()));
    }

    #[test]
    fn test_resolve_for_route_landing_preset_override() {
        // Test landing preset override (disables both sidebars)
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

        // Landing pages should have no sidebars
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

        // Docs pages should have sidebars
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
        // Test that preset can be parsed in override TOML
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
        // Regression test: named components should be expanded even when
        // an override uses a different preset
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

        // Resolve for blog route
        let resolved = config.resolve_for_route("/blog/post/");

        // The named component "custom-nav" should be expanded
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
        if let ComponentSpec::Config(c) = &middle[0] {
            if let ComponentConfig::NavTree {
                collapsible, depth, ..
            } = &c.component
            {
                assert_eq!(*collapsible, Some(false));
                assert_eq!(*depth, Some(2));
            } else {
                panic!("Expected NavTree component");
            }
        } else {
            panic!("Expected ComponentSpec::Config after resolution, got Name");
        }

        Ok(())
    }

    #[test]
    fn test_resolve_for_route_expands_components_from_preset_override() -> Result<()> {
        // Regression test: when override uses a preset, its default regions
        // should also have named components expanded
        let toml = r#"
            [components.custom-toc]
            type = "toc-tree"
            depth = 5

            # Override right-sidebar to use custom component
            [right-sidebar]
            start = "custom-toc"

            [[overrides]]
            routes = ["/blog/**"]
            preset = "blog"
            # Blog preset has right-sidebar enabled by default with toc-tree
            # We override it to use our custom component
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
        // custom-toc should be expanded to TocTree with depth=5
        if let ComponentSpec::Config(c) = &start[0] {
            if let ComponentConfig::TocTree { depth, .. } = &c.component {
                assert_eq!(*depth, Some(5));
            } else {
                panic!("Expected TocTree component");
            }
        } else {
            panic!("Expected ComponentSpec::Config after resolution");
        }

        Ok(())
    }

    #[test]
    fn test_resolve_for_route_global_config_persists_through_preset_override() -> Result<()> {
        // Test that global explicit config persists even when an override switches presets.
        // This is the intended behavior: if you set header.end = ["search"] globally,
        // you want search on ALL pages, not just non-overridden pages.
        let toml = r#"
            preset = "docs"

            # Global customization: add search to header on all pages
            [header]
            end = ["search", "color-mode"]

            [[overrides]]
            routes = ["/blog/**"]
            preset = "blog"
            # Note: we're NOT overriding header here
        "#;
        let config: LayoutConfig = toml::from_str(toml)?;

        // Non-blog route should have search in header
        let resolved = config.resolve_for_route("/docs/guide/");
        let header = resolved.header.as_ref().expect("header should be present");
        let region_config = header.config().expect("should be config");
        let end = region_config.end.as_ref().expect("end should be present");
        assert_eq!(end.len(), 2);
        assert!(matches!(&end[0], ComponentSpec::Name(n) if n == "search"));

        // Blog route should ALSO have search in header (global config persists)
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
        // Test that override's explicit regions still take precedence over global config
        let toml = r#"
            preset = "docs"

            # Global customization
            [header]
            end = ["search", "color-mode"]

            [[overrides]]
            routes = ["/blog/**"]
            preset = "blog"
            # Explicitly override header for blog (overrides global)
            header.end = ["color-mode"]
        "#;
        let config: LayoutConfig = toml::from_str(toml)?;

        // Non-blog route should have search + color-mode
        let resolved = config.resolve_for_route("/docs/guide/");
        let header = resolved.header.as_ref().expect("header should be present");
        let region_config = header.config().expect("should be config");
        let end = region_config.end.as_ref().expect("end should be present");
        assert_eq!(end.len(), 2);

        // Blog route should only have color-mode (override takes precedence)
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
        // Test that setting only header.end preserves preset's header.start and header.middle
        let toml = r#"
            preset = "docs"

            # Only customize header.end - should preserve preset's start and middle
            [header]
            end = ["search", "color-mode"]
        "#;
        let config: LayoutConfig = toml::from_str(toml)?;

        let resolved = config.resolve();

        let header = resolved.header.as_ref().expect("header should be present");
        let region_config = header.config().expect("should be config");

        // start should be preserved from docs preset ("logo")
        let start = region_config
            .start
            .as_ref()
            .expect("start should be present from preset");
        assert_eq!(start.len(), 1);
        assert!(
            matches!(&start[0], ComponentSpec::Name(n) if n == "logo"),
            "preset's header.start should be preserved"
        );

        // middle should be preserved from docs preset ("title")
        let middle = region_config
            .middle
            .as_ref()
            .expect("middle should be present from preset");
        assert_eq!(middle.len(), 1);
        assert!(
            matches!(&middle[0], ComponentSpec::Name(n) if n == "title"),
            "preset's header.middle should be preserved"
        );

        // end should be our custom config
        let end = region_config.end.as_ref().expect("end should be present");
        assert_eq!(end.len(), 2);
        assert!(matches!(&end[0], ComponentSpec::Name(n) if n == "search"));
        assert!(matches!(&end[1], ComponentSpec::Name(n) if n == "color-mode"));

        Ok(())
    }

    #[test]
    fn test_deep_merge_explicitly_empty_clears_subregion() -> Result<()> {
        // Test that setting header.start = [] explicitly clears it
        let toml = r#"
            preset = "docs"

            [header]
            start = []
        "#;
        let config: LayoutConfig = toml::from_str(toml)?;

        let resolved = config.resolve();

        let header = resolved.header.as_ref().expect("header should be present");
        let region_config = header.config().expect("should be config");

        // start should be explicitly empty (not inherited from preset)
        let start = region_config
            .start
            .as_ref()
            .expect("start should be Some (explicit empty)");
        assert!(
            start.is_empty(),
            "header.start = [] should clear the preset's start"
        );

        // middle should still be preserved from preset
        let middle = region_config
            .middle
            .as_ref()
            .expect("middle should be present from preset");
        assert_eq!(middle.len(), 1);

        Ok(())
    }

    #[test]
    fn test_deep_merge_in_route_override() -> Result<()> {
        // Test deep merge works in resolve_for_route with override preset
        let toml = r#"
            preset = "docs"

            [[overrides]]
            routes = ["/blog/**"]
            preset = "blog"
            # Only customize header.end for blog - should preserve blog preset's start/middle
            header.end = ["rss-feed", "color-mode"]
        "#;
        let config: LayoutConfig = toml::from_str(toml)?;

        let resolved = config.resolve_for_route("/blog/post/");

        let header = resolved.header.as_ref().expect("header should be present");
        let region_config = header.config().expect("should be config");

        // start should be preserved from blog preset ("logo")
        let start = region_config
            .start
            .as_ref()
            .expect("start should be present from blog preset");
        assert_eq!(start.len(), 1);
        assert!(
            matches!(&start[0], ComponentSpec::Name(n) if n == "logo"),
            "blog preset's header.start should be preserved"
        );

        // middle should be preserved from blog preset ("title")
        let middle = region_config
            .middle
            .as_ref()
            .expect("middle should be present from blog preset");
        assert_eq!(middle.len(), 1);

        // end should be our custom override
        let end = region_config.end.as_ref().expect("end should be present");
        assert_eq!(end.len(), 2);
        assert!(matches!(&end[0], ComponentSpec::Name(n) if n == "rss-feed"));
        assert!(matches!(&end[1], ComponentSpec::Name(n) if n == "color-mode"));

        Ok(())
    }

    #[test]
    fn test_deep_merge_global_then_override_layers() -> Result<()> {
        // Test that deep merge works through all layers:
        // preset defaults -> global explicit -> override explicit
        let toml = r#"
            preset = "docs"

            # Global: customize only header.end
            [header]
            end = ["search", "color-mode"]

            [[overrides]]
            routes = ["/landing/"]
            preset = "landing"
            # Override: customize only header.start
            header.start = ["big-logo"]
        "#;
        let config: LayoutConfig = toml::from_str(toml)?;

        let resolved = config.resolve_for_route("/landing/");

        let header = resolved.header.as_ref().expect("header should be present");
        let region_config = header.config().expect("should be config");

        // start should be from override ("big-logo")
        let start = region_config
            .start
            .as_ref()
            .expect("start should be present from override");
        assert_eq!(start.len(), 1);
        assert!(matches!(&start[0], ComponentSpec::Name(n) if n == "big-logo"));

        // middle should be from landing preset ("title")
        let middle = region_config
            .middle
            .as_ref()
            .expect("middle should be present from preset");
        assert_eq!(middle.len(), 1);
        assert!(matches!(&middle[0], ComponentSpec::Name(n) if n == "title"));

        // end should be from global config (preserved through preset switch)
        let end = region_config.end.as_ref().expect("end should be present");
        assert_eq!(end.len(), 2);
        assert!(
            matches!(&end[0], ComponentSpec::Name(n) if n == "search"),
            "global header.end should persist through preset override"
        );

        Ok(())
    }
}
