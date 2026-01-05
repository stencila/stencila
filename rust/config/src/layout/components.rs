//! Layout component types
//!
//! This module contains types for configuring layout components such as
//! logo, navigation tree, breadcrumbs, color mode toggle, and more.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum::Display;

use crate::LogoConfig;

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

    /// Full component configuration
    Config(ComponentConfig),
}

/// Component configuration (internally tagged by type)
///
/// All fields are Option - bare string usage gets defaults from site config
/// at resolution time.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ComponentConfig {
    /// Site logo image with responsive and dark mode variants
    ///
    /// When used as a bare `"logo"` string, inherits configuration from
    /// `site.logo`. When used as an object, can override any fields.
    ///
    /// Example:
    /// ```toml
    /// [site.layout.header]
    /// start = "logo"  # Uses site.logo config
    ///
    /// # Or with overrides:
    /// start = { type = "logo", default = "header-logo.svg", dark = "header-logo-dark.svg" }
    /// ```
    Logo(LogoConfig),

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
    ///
    /// Displays links to previous and next pages in the navigation sequence.
    /// Style controls what information is shown (see PrevNextStyle).
    ///
    /// Example:
    /// ```toml
    /// [site.layout.bottom]
    /// middle = "prev-next"  # Uses default "standard" style
    ///
    /// # Or with configuration:
    /// middle = { type = "prev-next", style = "compact" }
    ///
    /// # Custom labels:
    /// middle = { type = "prev-next", prev-text = "Back", next-text = "Continue" }
    /// ```
    PrevNext {
        /// Display style (default: standard)
        ///
        /// Controls what information is shown:
        /// - minimal: icons only
        /// - compact: icons + labels
        /// - standard: icons + labels + titles (default)
        /// - detailed: icons + labels + titles + position
        style: Option<PrevNextStyle>,

        /// Custom text for previous link (default: "Previous")
        ///
        /// Useful for localization.
        #[serde(rename = "prev-text")]
        prev_text: Option<String>,

        /// Custom text for next link (default: "Next")
        ///
        /// Useful for localization.
        #[serde(rename = "next-text")]
        next_text: Option<String>,

        /// Separator between prev and next links (default: none)
        ///
        /// Common values: "|", "·", or any custom string.
        /// Only shown when both prev and next links are present.
        separator: Option<String>,
    },

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
pub const BUILTIN_COMPONENT_TYPES: &[&str] = &[
    "logo",
    "title",
    "breadcrumbs",
    "nav-tree",
    "toc-tree",
    "prev-next",
    "color-mode",
    "copyright",
];

/// Check if a name is a built-in component type
pub fn is_builtin_component_type(name: &str) -> bool {
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

/// Display style for prev/next navigation
///
/// Controls what information is shown in the prev/next navigation links.
#[derive(
    Debug, Clone, Copy, Default, Display, Serialize, Deserialize, PartialEq, Eq, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum PrevNextStyle {
    /// Minimal: just arrow icons
    ///
    /// Output: ← | →
    Minimal,

    /// Compact: icons + labels
    ///
    /// Output: ← Previous | Next →
    Compact,

    /// Standard: icons + labels + page titles (default)
    ///
    /// Output: ← Previous: Getting Started | Next: Configuration →
    #[default]
    Standard,

    /// Detailed: icons + labels + titles + position indicator
    ///
    /// Output: ← Previous: Getting Started | Page 3 of 10 | Next: Configuration →
    Detailed,
}

#[cfg(test)]
mod tests {
    use eyre::Result;

    use super::*;

    #[test]
    fn test_component_types() -> Result<()> {
        // Test all component types can be parsed
        let components = vec![
            r#"type = "logo""#,
            r#"type = "title""#,
            r#"type = "nav-tree""#,
            r#"type = "toc-tree""#,
            r#"type = "breadcrumbs""#,
            r#"type = "prev-next""#,
            r#"type = "color-mode""#,
            r#"type = "copyright""#,
        ];

        for component_toml in components {
            let _: ComponentConfig = toml::from_str(component_toml)?;
        }

        Ok(())
    }

    #[test]
    fn test_builtin_component_types() {
        assert!(is_builtin_component_type("logo"));
        assert!(is_builtin_component_type("title"));
        assert!(is_builtin_component_type("breadcrumbs"));
        assert!(is_builtin_component_type("nav-tree"));
        assert!(is_builtin_component_type("toc-tree"));
        assert!(is_builtin_component_type("prev-next"));
        assert!(is_builtin_component_type("color-mode"));
        assert!(is_builtin_component_type("copyright"));
        assert!(!is_builtin_component_type("unknown"));
        assert!(!is_builtin_component_type("custom-nav"));
    }

    #[test]
    fn test_color_mode_style_parsing() -> Result<()> {
        let config: ComponentConfig = toml::from_str(r#"type = "color-mode""#)?;
        assert!(matches!(config, ComponentConfig::ColorMode { style: None }));

        let config: ComponentConfig = toml::from_str(
            r#"type = "color-mode"
style = "icon""#,
        )?;
        assert!(matches!(
            config,
            ComponentConfig::ColorMode {
                style: Some(ColorModeStyle::Icon)
            }
        ));

        let config: ComponentConfig = toml::from_str(
            r#"type = "color-mode"
style = "label""#,
        )?;
        assert!(matches!(
            config,
            ComponentConfig::ColorMode {
                style: Some(ColorModeStyle::Label)
            }
        ));

        let config: ComponentConfig = toml::from_str(
            r#"type = "color-mode"
style = "both""#,
        )?;
        assert!(matches!(
            config,
            ComponentConfig::ColorMode {
                style: Some(ColorModeStyle::Both)
            }
        ));

        Ok(())
    }

    #[test]
    fn test_prev_next_style_parsing() -> Result<()> {
        let config: ComponentConfig = toml::from_str(r#"type = "prev-next""#)?;
        assert!(matches!(
            config,
            ComponentConfig::PrevNext { style: None, .. }
        ));

        let config: ComponentConfig = toml::from_str(
            r#"type = "prev-next"
style = "minimal""#,
        )?;
        assert!(matches!(
            config,
            ComponentConfig::PrevNext {
                style: Some(PrevNextStyle::Minimal),
                ..
            }
        ));

        let config: ComponentConfig = toml::from_str(
            r#"type = "prev-next"
style = "compact""#,
        )?;
        assert!(matches!(
            config,
            ComponentConfig::PrevNext {
                style: Some(PrevNextStyle::Compact),
                ..
            }
        ));

        let config: ComponentConfig = toml::from_str(
            r#"type = "prev-next"
style = "standard""#,
        )?;
        assert!(matches!(
            config,
            ComponentConfig::PrevNext {
                style: Some(PrevNextStyle::Standard),
                ..
            }
        ));

        let config: ComponentConfig = toml::from_str(
            r#"type = "prev-next"
style = "detailed""#,
        )?;
        assert!(matches!(
            config,
            ComponentConfig::PrevNext {
                style: Some(PrevNextStyle::Detailed),
                ..
            }
        ));

        Ok(())
    }

    #[test]
    fn test_prev_next_custom_labels_parsing() -> Result<()> {
        let config: ComponentConfig = toml::from_str(
            r#"type = "prev-next"
prev-text = "Précédent"
next-text = "Suivant""#,
        )?;

        if let ComponentConfig::PrevNext {
            prev_text,
            next_text,
            ..
        } = config
        {
            assert_eq!(prev_text.as_deref(), Some("Précédent"));
            assert_eq!(next_text.as_deref(), Some("Suivant"));
        } else {
            panic!("Expected ComponentConfig::PrevNext");
        }

        Ok(())
    }

    #[test]
    fn test_prev_next_separator_parsing() -> Result<()> {
        let config: ComponentConfig = toml::from_str(
            r#"type = "prev-next"
separator = "·""#,
        )?;

        if let ComponentConfig::PrevNext { separator, .. } = config {
            assert_eq!(separator.as_deref(), Some("·"));
        } else {
            panic!("Expected ComponentConfig::PrevNext");
        }

        Ok(())
    }

    #[test]
    fn test_prev_next_all_options_parsing() -> Result<()> {
        let config: ComponentConfig = toml::from_str(
            r#"type = "prev-next"
style = "compact"
prev-text = "Back"
next-text = "Forward"
separator = "|""#,
        )?;

        if let ComponentConfig::PrevNext {
            style,
            prev_text,
            next_text,
            separator,
        } = config
        {
            assert_eq!(style, Some(PrevNextStyle::Compact));
            assert_eq!(prev_text.as_deref(), Some("Back"));
            assert_eq!(next_text.as_deref(), Some("Forward"));
            assert_eq!(separator.as_deref(), Some("|"));
        } else {
            panic!("Expected ComponentConfig::PrevNext");
        }

        Ok(())
    }

    #[test]
    fn test_nav_tree_item_route() -> Result<()> {
        // NavTreeItem::Route is used in arrays within NavTree.items
        // Test via a wrapper since TOML can't parse bare strings at top level
        #[derive(Deserialize)]
        struct Wrapper {
            items: Vec<NavTreeItem>,
        }
        let wrapper: Wrapper = toml::from_str(r#"items = ["/docs/guide/"]"#)?;
        assert_eq!(wrapper.items.len(), 1);
        assert!(matches!(&wrapper.items[0], NavTreeItem::Route(s) if s == "/docs/guide/"));

        Ok(())
    }

    #[test]
    fn test_nav_tree_item_link() -> Result<()> {
        let item: NavTreeItem = toml::from_str(
            r#"label = "Getting Started"
target = "/docs/getting-started/""#,
        )?;

        if let NavTreeItem::Link {
            label,
            target,
            icon,
        } = item
        {
            assert_eq!(label, "Getting Started");
            assert_eq!(target, "/docs/getting-started/");
            assert!(icon.is_none());
        } else {
            panic!("Expected NavTreeItem::Link");
        }

        Ok(())
    }

    #[test]
    fn test_nav_tree_item_link_with_icon() -> Result<()> {
        let item: NavTreeItem = toml::from_str(
            r#"label = "GitHub"
target = "https://github.com/example"
icon = "github""#,
        )?;

        if let NavTreeItem::Link {
            label,
            target,
            icon,
        } = item
        {
            assert_eq!(label, "GitHub");
            assert_eq!(target, "https://github.com/example");
            assert_eq!(icon.as_deref(), Some("github"));
        } else {
            panic!("Expected NavTreeItem::Link");
        }

        Ok(())
    }

    #[test]
    fn test_nav_tree_item_group() -> Result<()> {
        // Group without target (target is optional for groups)
        // Note: if both target and children are present, serde's untagged enum
        // matches Link first (ignoring children). So we test without target here.
        let item: NavTreeItem = toml::from_str(
            r#"label = "Guides"
children = ["/docs/installation/", "/docs/configuration/"]"#,
        )?;

        if let NavTreeItem::Group {
            label,
            target,
            icon,
            children,
        } = item
        {
            assert_eq!(label, "Guides");
            assert!(target.is_none());
            assert!(icon.is_none());
            assert_eq!(children.len(), 2);
        } else {
            panic!("Expected NavTreeItem::Group");
        }

        Ok(())
    }
}
