//! Layout presets
//!
//! This module contains named preset configurations for common site types
//! such as documentation, blogs, landing pages, and API reference sites.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::Display;

use crate::RowConfig;

use super::components::ComponentSpec;
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
        let header = Some(RegionSpec::Config(RegionConfig {
            start: Some(vec![ComponentSpec::Name("logo".into())]),
            middle: Some(vec![ComponentSpec::Name("nav-menu".into())]),
            ..Default::default()
        }));

        let top = Some(RegionSpec::Config(RegionConfig {
            start: Some(vec![ComponentSpec::Name("breadcrumbs".into())]),
            ..Default::default()
        }));

        let bottom = Some(RegionSpec::Config(RegionConfig {
            middle: Some(vec![ComponentSpec::Name("prev-next".into())]),
            ..Default::default()
        }));

        let left_sidebar = Some(RegionSpec::Config(RegionConfig {
            start: Some(vec![ComponentSpec::Name("nav-tree".into())]),
            ..Default::default()
        }));

        let right_sidebar = Some(RegionSpec::Config(RegionConfig {
            start: Some(vec![ComponentSpec::Name("toc-tree".into())]),
            end: Some(vec![ComponentSpec::Name("edit-source".into())]),
            ..Default::default()
        }));

        let footer = Some(RegionSpec::Config(RegionConfig {
            rows: Some(vec![
                RowConfig {
                    middle: Some(vec![ComponentSpec::Name("nav-groups".into())]),
                    ..Default::default()
                },
                RowConfig {
                    start: Some(vec![ComponentSpec::Name("color-mode".into())]),
                    middle: Some(vec![ComponentSpec::Name("copyright".into())]),
                    end: Some(vec![ComponentSpec::Name("social-links".into())]),
                },
            ]),
            ..Default::default()
        }));

        match self {
            Self::Landing => LayoutConfig {
                header,
                footer,
                ..Default::default()
            },
            Self::Blog => LayoutConfig {
                header,
                right_sidebar,
                footer,
                ..Default::default()
            },
            Self::Docs => LayoutConfig {
                header,
                left_sidebar,
                top,
                bottom,
                right_sidebar,
                footer,
                ..Default::default()
            },
            Self::Api => LayoutConfig {
                header,
                left_sidebar,
                top,
                bottom,
                footer,
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
    fn test_preset_display() {
        assert_eq!(LayoutPreset::Docs.to_string(), "docs");
        assert_eq!(LayoutPreset::Blog.to_string(), "blog");
        assert_eq!(LayoutPreset::Landing.to_string(), "landing");
        assert_eq!(LayoutPreset::Api.to_string(), "api");
    }
}
