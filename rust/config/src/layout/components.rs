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
    ///
    /// Displays site navigation from `site.nav` configuration (or auto-generated
    /// from routes if not specified). Supports collapsible groups, active page
    /// highlighting, and keyboard navigation.
    ///
    /// Example:
    /// ```toml
    /// [site.layout.left-sidebar]
    /// start = "nav-tree"  # Uses defaults
    ///
    /// # Or with configuration:
    /// start = { type = "nav-tree", title = "Documentation", expanded = "current-path" }
    /// ```
    NavTree {
        /// Optional title above the nav tree (e.g., "Navigation", "Docs")
        title: Option<String>,

        /// Maximum depth to display (default: unlimited)
        ///
        /// Limits how deep the navigation tree renders. Useful for large sites
        /// where you want to show only top-level sections.
        depth: Option<u8>,

        /// Whether groups are collapsible (default: true)
        ///
        /// When true, group headers can be clicked to expand/collapse children.
        /// When false, all groups are always expanded.
        collapsible: Option<bool>,

        /// Default expansion state for collapsible groups (default: all)
        ///
        /// Controls initial expand/collapse state:
        /// - all: All groups expanded
        /// - none: All groups collapsed
        /// - first-level: Only top-level groups expanded
        /// - current-path: Expand groups containing the active page
        expanded: Option<NavTreeExpanded>,

        /// Auto-scroll nav container to show active item on page load (default: true)
        #[serde(rename = "scroll-to-active")]
        scroll_to_active: Option<bool>,

        /// Include only items matching these patterns
        ///
        /// Supports routes ("/docs/*"), IDs ("#features"), and labels ("Features").
        /// See filtering documentation for pattern syntax.
        include: Option<Vec<String>>,

        /// Exclude items matching these patterns
        ///
        /// Supports routes ("/docs/*"), IDs ("#features"), and labels ("Features").
        /// Exclude takes precedence over include.
        exclude: Option<Vec<String>>,

        /// Whether to show icons from site.icons (default: hide)
        icons: Option<NavTreeIcons>,
    },

    /// Top-level navigation menu bar
    ///
    /// Displays horizontal navigation with mega-dropdown panels on desktop
    /// and accordion-style menu on mobile. Uses site.nav as data source.
    ///
    /// Example:
    /// ```toml
    /// [site.layout.header]
    /// middle = "nav-menu"  # Uses defaults
    ///
    /// # Or with configuration:
    /// middle = { type = "nav-menu", groups = "dropdowns", trigger = "click" }
    /// ```
    NavMenu {
        /// Include only items matching these patterns
        ///
        /// Supports routes ("/docs/*"), IDs ("#features"), and labels ("Features").
        include: Option<Vec<String>>,

        /// Exclude items matching these patterns
        ///
        /// Supports routes ("/docs/*"), IDs ("#features"), and labels ("Features").
        /// Exclude takes precedence over include.
        exclude: Option<Vec<String>>,

        /// Maximum depth to display (default: unlimited, 1 = top-level only)
        depth: Option<u8>,

        /// How to render groups (default: auto)
        ///
        /// - auto: Groups with children become dropdowns, others are links
        /// - dropdowns: All groups become dropdown menus
        /// - links: All groups render as simple links (requires route)
        groups: Option<NavMenuGroups>,

        /// Whether to show icons (default: show)
        ///
        /// - show: Show icons on all items that have them
        /// - hide: Never show icons
        /// - dropdowns-only: Only show icons inside dropdown panels
        icons: Option<NavMenuIcons>,

        /// Whether to show descriptions in dropdowns (default: true)
        descriptions: Option<bool>,

        /// Dropdown trigger behavior (default: hover)
        trigger: Option<NavMenuTrigger>,

        /// Dropdown panel style (default: full-width)
        #[serde(rename = "dropdown-style")]
        dropdown_style: Option<NavMenuDropdownStyle>,

        /// Hover delay in ms before opening (default: 150)
        #[serde(rename = "hover-delay")]
        hover_delay: Option<u16>,

        /// Close delay in ms after mouse leaves (default: 300)
        #[serde(rename = "close-delay")]
        close_delay: Option<u16>,

        /// Mobile breakpoint in pixels (default: 1024)
        #[serde(rename = "mobile-breakpoint")]
        mobile_breakpoint: Option<u16>,
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

    /// Copyright notice with auto-updating year
    ///
    /// Displays a copyright notice with optional auto-updating year.
    /// When used as a bare `"copyright"` string, uses `site.author` as the holder
    /// and current year.
    ///
    /// Example:
    /// ```toml
    /// [site.layout.footer]
    /// middle = "copyright"  # Uses site.author, current year
    ///
    /// # With year range:
    /// middle = { type = "copyright", start-year = 2020 }
    ///
    /// # With custom holder:
    /// middle = { type = "copyright", holder = "Acme Inc", link = "https://acme.com" }
    ///
    /// # Full custom text (no auto-year):
    /// middle = { type = "copyright", text = "Custom copyright notice" }
    /// ```
    Copyright {
        /// Full custom text (overrides all other fields)
        ///
        /// When provided, this text is used verbatim with no auto-year.
        /// Example: "© 2024 Acme Inc. All rights reserved."
        text: Option<String>,

        /// Copyright holder name (defaults to site.author)
        ///
        /// Example: "Acme Inc"
        holder: Option<String>,

        /// Start year for copyright range (e.g., 2020 in "2020-2024")
        ///
        /// If not set, only current year is shown.
        #[serde(rename = "start-year")]
        start_year: Option<u16>,

        /// Link URL for the holder name
        ///
        /// When provided, the holder name becomes a clickable link.
        link: Option<String>,
    },
}

/// Built-in component type names (kebab-case as used in TOML)
pub const BUILTIN_COMPONENT_TYPES: &[&str] = &[
    "logo",
    "title",
    "breadcrumbs",
    "nav-tree",
    "nav-menu",
    "toc-tree",
    "prev-next",
    "color-mode",
    "copyright",
];

/// Check if a name is a built-in component type
pub fn is_builtin_component_type(name: &str) -> bool {
    BUILTIN_COMPONENT_TYPES.contains(&name)
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

/// Default expansion state for NavTree groups
///
/// Controls the initial expand/collapse state of collapsible navigation groups.
#[derive(
    Debug, Clone, Copy, Default, Display, Serialize, Deserialize, PartialEq, Eq, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum NavTreeExpanded {
    /// All groups expanded (default)
    ///
    /// Every collapsible group is initially open, showing all navigation items.
    #[default]
    All,

    /// All groups collapsed
    ///
    /// Every collapsible group is initially closed. Users must click to expand.
    None,

    /// Only top-level groups expanded
    ///
    /// First-level groups are open, but nested groups are collapsed.
    FirstLevel,

    /// Expand groups containing the active page
    ///
    /// Only groups that are ancestors of the current page are expanded.
    /// This keeps the navigation focused on the user's current location.
    CurrentPath,
}

/// Whether to show icons in nav-tree
#[derive(
    Debug, Clone, Copy, Default, Display, Serialize, Deserialize, PartialEq, Eq, JsonSchema,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum NavTreeIcons {
    /// Show icons from site.icons
    Show,

    /// Hide icons (default for nav-tree)
    #[default]
    Hide,
}

/// How to render groups in nav-menu
#[derive(
    Debug, Clone, Copy, Default, Display, Serialize, Deserialize, PartialEq, Eq, JsonSchema,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum NavMenuGroups {
    /// Groups with children become dropdowns, others are links (default)
    #[default]
    Auto,

    /// All groups become dropdown menus
    Dropdowns,

    /// All groups render as simple links (requires route)
    Links,
}

/// Whether to show icons in nav-menu
#[derive(
    Debug, Clone, Copy, Default, Display, Serialize, Deserialize, PartialEq, Eq, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum NavMenuIcons {
    /// Show icons on all items that have them (default)
    #[default]
    Show,

    /// Never show icons
    Hide,

    /// Only show icons inside dropdown panels
    DropdownsOnly,
}

/// Dropdown trigger behavior for nav-menu
#[derive(
    Debug, Clone, Copy, Default, Display, Serialize, Deserialize, PartialEq, Eq, JsonSchema,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum NavMenuTrigger {
    /// Open dropdowns on hover with delay (default)
    #[default]
    Hover,

    /// Open dropdowns on click only
    Click,
}

/// Dropdown panel style for nav-menu
#[derive(
    Debug, Clone, Copy, Default, Display, Serialize, Deserialize, PartialEq, Eq, JsonSchema,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum NavMenuDropdownStyle {
    /// Full-width dropdown panels (default)
    #[default]
    FullWidth,

    /// Dropdown aligned to trigger position
    Aligned,
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
    fn test_nav_tree_basic_parsing() -> Result<()> {
        let config: ComponentConfig = toml::from_str(r#"type = "nav-tree""#)?;
        assert!(matches!(
            config,
            ComponentConfig::NavTree {
                title: None,
                depth: None,
                collapsible: None,
                expanded: None,
                scroll_to_active: None,
                include: None,
                exclude: None,
                icons: None,
            }
        ));

        Ok(())
    }

    #[test]
    fn test_nav_tree_with_title() -> Result<()> {
        let config: ComponentConfig = toml::from_str(
            r#"type = "nav-tree"
title = "Documentation""#,
        )?;

        if let ComponentConfig::NavTree { title, .. } = config {
            assert_eq!(title.as_deref(), Some("Documentation"));
        } else {
            panic!("Expected ComponentConfig::NavTree");
        }

        Ok(())
    }

    #[test]
    fn test_nav_tree_with_depth() -> Result<()> {
        let config: ComponentConfig = toml::from_str(
            r#"type = "nav-tree"
depth = 2"#,
        )?;

        if let ComponentConfig::NavTree { depth, .. } = config {
            assert_eq!(depth, Some(2));
        } else {
            panic!("Expected ComponentConfig::NavTree");
        }

        Ok(())
    }

    #[test]
    fn test_nav_tree_expanded_all() -> Result<()> {
        let config: ComponentConfig = toml::from_str(
            r#"type = "nav-tree"
expanded = "all""#,
        )?;

        if let ComponentConfig::NavTree { expanded, .. } = config {
            assert_eq!(expanded, Some(NavTreeExpanded::All));
        } else {
            panic!("Expected ComponentConfig::NavTree");
        }

        Ok(())
    }

    #[test]
    fn test_nav_tree_expanded_none() -> Result<()> {
        let config: ComponentConfig = toml::from_str(
            r#"type = "nav-tree"
expanded = "none""#,
        )?;

        if let ComponentConfig::NavTree { expanded, .. } = config {
            assert_eq!(expanded, Some(NavTreeExpanded::None));
        } else {
            panic!("Expected ComponentConfig::NavTree");
        }

        Ok(())
    }

    #[test]
    fn test_nav_tree_expanded_first_level() -> Result<()> {
        let config: ComponentConfig = toml::from_str(
            r#"type = "nav-tree"
expanded = "first-level""#,
        )?;

        if let ComponentConfig::NavTree { expanded, .. } = config {
            assert_eq!(expanded, Some(NavTreeExpanded::FirstLevel));
        } else {
            panic!("Expected ComponentConfig::NavTree");
        }

        Ok(())
    }

    #[test]
    fn test_nav_tree_expanded_current_path() -> Result<()> {
        let config: ComponentConfig = toml::from_str(
            r#"type = "nav-tree"
expanded = "current-path""#,
        )?;

        if let ComponentConfig::NavTree { expanded, .. } = config {
            assert_eq!(expanded, Some(NavTreeExpanded::CurrentPath));
        } else {
            panic!("Expected ComponentConfig::NavTree");
        }

        Ok(())
    }

    #[test]
    fn test_nav_tree_collapsible() -> Result<()> {
        let config: ComponentConfig = toml::from_str(
            r#"type = "nav-tree"
collapsible = false"#,
        )?;

        if let ComponentConfig::NavTree { collapsible, .. } = config {
            assert_eq!(collapsible, Some(false));
        } else {
            panic!("Expected ComponentConfig::NavTree");
        }

        Ok(())
    }

    #[test]
    fn test_nav_tree_scroll_to_active() -> Result<()> {
        let config: ComponentConfig = toml::from_str(
            r#"type = "nav-tree"
scroll-to-active = false"#,
        )?;

        if let ComponentConfig::NavTree {
            scroll_to_active, ..
        } = config
        {
            assert_eq!(scroll_to_active, Some(false));
        } else {
            panic!("Expected ComponentConfig::NavTree");
        }

        Ok(())
    }

    #[test]
    fn test_nav_tree_all_options() -> Result<()> {
        let config: ComponentConfig = toml::from_str(
            r#"type = "nav-tree"
title = "Site Navigation"
depth = 3
collapsible = true
expanded = "current-path"
scroll-to-active = true"#,
        )?;

        if let ComponentConfig::NavTree {
            title,
            depth,
            collapsible,
            expanded,
            scroll_to_active,
            ..
        } = config
        {
            assert_eq!(title.as_deref(), Some("Site Navigation"));
            assert_eq!(depth, Some(3));
            assert_eq!(collapsible, Some(true));
            assert_eq!(expanded, Some(NavTreeExpanded::CurrentPath));
            assert_eq!(scroll_to_active, Some(true));
        } else {
            panic!("Expected ComponentConfig::NavTree");
        }

        Ok(())
    }

    #[test]
    fn test_copyright_basic_parsing() -> Result<()> {
        let config: ComponentConfig = toml::from_str(r#"type = "copyright""#)?;
        assert!(matches!(
            config,
            ComponentConfig::Copyright {
                text: None,
                holder: None,
                start_year: None,
                link: None,
            }
        ));

        Ok(())
    }

    #[test]
    fn test_copyright_with_holder_parsing() -> Result<()> {
        let config: ComponentConfig = toml::from_str(
            r#"type = "copyright"
holder = "Acme Inc""#,
        )?;

        if let ComponentConfig::Copyright { holder, .. } = config {
            assert_eq!(holder.as_deref(), Some("Acme Inc"));
        } else {
            panic!("Expected ComponentConfig::Copyright");
        }

        Ok(())
    }

    #[test]
    fn test_copyright_with_start_year_parsing() -> Result<()> {
        let config: ComponentConfig = toml::from_str(
            r#"type = "copyright"
start-year = 2020"#,
        )?;

        if let ComponentConfig::Copyright { start_year, .. } = config {
            assert_eq!(start_year, Some(2020));
        } else {
            panic!("Expected ComponentConfig::Copyright");
        }

        Ok(())
    }

    #[test]
    fn test_copyright_with_link_parsing() -> Result<()> {
        let config: ComponentConfig = toml::from_str(
            r#"type = "copyright"
holder = "Acme Inc"
link = "https://acme.com""#,
        )?;

        if let ComponentConfig::Copyright { holder, link, .. } = config {
            assert_eq!(holder.as_deref(), Some("Acme Inc"));
            assert_eq!(link.as_deref(), Some("https://acme.com"));
        } else {
            panic!("Expected ComponentConfig::Copyright");
        }

        Ok(())
    }

    #[test]
    fn test_copyright_with_text_parsing() -> Result<()> {
        let config: ComponentConfig = toml::from_str(
            r#"type = "copyright"
text = "© 2024 Custom Copyright Notice""#,
        )?;

        if let ComponentConfig::Copyright { text, .. } = config {
            assert_eq!(text.as_deref(), Some("© 2024 Custom Copyright Notice"));
        } else {
            panic!("Expected ComponentConfig::Copyright");
        }

        Ok(())
    }

    #[test]
    fn test_copyright_all_options_parsing() -> Result<()> {
        let config: ComponentConfig = toml::from_str(
            r#"type = "copyright"
holder = "Acme Inc"
start-year = 2020
link = "https://acme.com""#,
        )?;

        if let ComponentConfig::Copyright {
            text,
            holder,
            start_year,
            link,
        } = config
        {
            assert!(text.is_none());
            assert_eq!(holder.as_deref(), Some("Acme Inc"));
            assert_eq!(start_year, Some(2020));
            assert_eq!(link.as_deref(), Some("https://acme.com"));
        } else {
            panic!("Expected ComponentConfig::Copyright");
        }

        Ok(())
    }
}
