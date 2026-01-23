//! Layout overrides
//!
//! This module contains the `LayoutOverride` type for route-specific
//! layout configuration overrides.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

use super::presets::LayoutPreset;
use super::regions::{RegionSpec, ResponsiveConfig};

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
/// header.end = ["site-search", "color-mode"]  # Global: site-search on all pages
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
/// header.end = ["color-mode"]  # Override global: no site-search on landing
///
/// # Just override specific regions (no preset change)
/// [[site.layout.overrides]]
/// routes = ["/api/**"]
/// left-sidebar.middle = "api-nav"
/// right-sidebar = false
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
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

    /// Global responsive configuration override
    pub responsive: Option<ResponsiveConfig>,
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
    fn test_override_empty_routes_validation() {
        let override_config = LayoutOverride {
            routes: vec![],
            left_sidebar: Some(RegionSpec::Enabled(false)),
            ..Default::default()
        };

        let result = override_config.validate();
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
        let override_config = LayoutOverride {
            routes: vec!["/blog/**".to_string()],
            left_sidebar: Some(RegionSpec::Enabled(false)),
            ..Default::default()
        };

        override_config.validate()?;
        Ok(())
    }

    #[test]
    fn test_override_basic_parsing() -> Result<()> {
        let toml = r#"
            routes = ["/blog/**"]
            left-sidebar = false
            bottom = false
        "#;
        let override_config: LayoutOverride = toml::from_str(toml)?;

        assert_eq!(override_config.routes, vec!["/blog/**"]);
        assert!(matches!(
            override_config.left_sidebar,
            Some(RegionSpec::Enabled(false))
        ));
        assert!(matches!(
            override_config.bottom,
            Some(RegionSpec::Enabled(false))
        ));

        Ok(())
    }

    #[test]
    fn test_override_with_preset_parsing() -> Result<()> {
        let toml = r#"
            routes = ["/blog/**"]
            preset = "blog"
        "#;
        let override_config: LayoutOverride = toml::from_str(toml)?;

        assert_eq!(override_config.routes, vec!["/blog/**"]);
        assert_eq!(override_config.preset, Some(LayoutPreset::Blog));

        Ok(())
    }

    #[test]
    fn test_override_with_subregion_parsing() -> Result<()> {
        let toml = r#"
            routes = ["/api/**"]
            right-sidebar = false
            left-sidebar.middle = "api-nav"
        "#;
        let override_config: LayoutOverride = toml::from_str(toml)?;

        assert_eq!(override_config.routes, vec!["/api/**"]);
        assert!(matches!(
            override_config.right_sidebar,
            Some(RegionSpec::Enabled(false))
        ));

        if let Some(RegionSpec::Config(config)) = &override_config.left_sidebar {
            let middle = config.middle.as_ref().expect("middle should be present");
            assert!(
                matches!(&middle[0], super::super::components::ComponentSpec::Name(n) if n == "api-nav")
            );
        } else {
            panic!("Expected RegionSpec::Config for left-sidebar");
        }

        Ok(())
    }

    #[test]
    fn test_override_multiple_routes() -> Result<()> {
        let toml = r#"
            routes = ["/", "/features/"]
            preset = "landing"
        "#;
        let override_config: LayoutOverride = toml::from_str(toml)?;

        assert_eq!(override_config.routes.len(), 2);
        assert!(override_config.routes.contains(&"/".to_string()));
        assert!(override_config.routes.contains(&"/features/".to_string()));

        Ok(())
    }
}
