//! Layout presets
//!
//! This module contains named preset configurations for common site types
//! such as documentation, blogs, landing pages, and API reference sites.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::Display;

use super::components::{ComponentConfig, ComponentSpec};
use super::config::LayoutConfig;
use super::regions::{RegionConfig, RegionSpec};

/// Named layout presets for common documentation patterns
#[derive(
    Debug, Clone, Copy, Default, Display, Serialize, Deserialize, PartialEq, Eq, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum LayoutPreset {
    /// Documentation site: nav-tree left, toc-tree right, breadcrumbs, prev-next
    #[default]
    Docs,

    /// Blog/article site: no left sidebar, toc-tree right, no prev-next
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
                    middle: Some(vec![ComponentSpec::Name("prev-next".into())]),
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
                    middle: Some(vec![ComponentSpec::Config(ComponentConfig::NavTree {
                        title: None,
                        depth: None,
                        collapsible: Some(false),
                        expanded: None,
                        scroll_to_active: None,
                    })]),
                    ..Default::default()
                })),
                top: Some(RegionSpec::Config(RegionConfig {
                    start: Some(vec![ComponentSpec::Name("breadcrumbs".into())]),
                    ..Default::default()
                })),
                bottom: Some(RegionSpec::Config(RegionConfig {
                    middle: Some(vec![ComponentSpec::Name("prev-next".into())]),
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

#[cfg(test)]
mod tests {
    use eyre::Result;

    use super::*;

    #[test]
    fn test_preset_parsing() -> Result<()> {
        let config: LayoutConfig = toml::from_str(r#"preset = "docs""#)?;
        assert_eq!(config.preset, Some(LayoutPreset::Docs));

        let config: LayoutConfig = toml::from_str(r#"preset = "blog""#)?;
        assert_eq!(config.preset, Some(LayoutPreset::Blog));

        let config: LayoutConfig = toml::from_str(r#"preset = "landing""#)?;
        assert_eq!(config.preset, Some(LayoutPreset::Landing));

        let config: LayoutConfig = toml::from_str(r#"preset = "api""#)?;
        assert_eq!(config.preset, Some(LayoutPreset::Api));

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
    fn test_preset_display() {
        assert_eq!(LayoutPreset::Docs.to_string(), "docs");
        assert_eq!(LayoutPreset::Blog.to_string(), "blog");
        assert_eq!(LayoutPreset::Landing.to_string(), "landing");
        assert_eq!(LayoutPreset::Api.to_string(), "api");
    }
}
