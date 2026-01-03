//! Site layout configuration types
//!
//! This module contains types for configuring the layout of site pages,
//! including sidebars, navigation, headers, and footers.

use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Site layout configuration
///
/// Controls the layout structure of site pages including header, sidebars,
/// footer, and navigation.
///
/// Example:
/// ```toml
/// [site.layout]
/// left-sidebar = true
/// right-sidebar = true
/// ```
///
/// Or with full sidebar configuration:
/// ```toml
/// [site.layout]
/// left-sidebar = { nav = "auto", collapsible = true, depth = 3 }
/// right-sidebar = { content = "headings", depth = 3 }
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct SiteLayout {
    /// Header configuration
    ///
    /// Configure the site header with logo, title, navigation tabs, and icon links.
    pub header: Option<LayoutHeader>,

    /// Left sidebar configuration
    ///
    /// Can be a boolean to enable/disable, or a configuration object.
    /// Defaults to `true` (enabled with auto-generated navigation).
    /// Set to `false` to explicitly disable the left sidebar.
    pub left_sidebar: Option<LayoutLeftSidebar>,

    /// Right sidebar configuration
    ///
    /// Can be a boolean to enable/disable, or a configuration object.
    /// When `true`, displays a right sidebar with document headings.
    /// When `false` or not specified, the right sidebar is hidden.
    pub right_sidebar: Option<LayoutRightSidebar>,

    /// Enable prev/next page navigation
    ///
    /// When `true`, displays prev/next links at the bottom of the content area.
    /// Defaults to `true` when left sidebar navigation is enabled.
    pub page_nav: Option<bool>,

    /// Footer configuration
    ///
    /// Configure the site footer with link groups, icon links, and copyright text.
    pub footer: Option<LayoutFooter>,

    /// Named navigation configurations
    ///
    /// Define reusable navigation trees that can be referenced by name
    /// in sidebar configurations.
    ///
    /// Example:
    /// ```toml
    /// [site.layout.navs.api]
    /// items = [
    ///   "/api/getting-started/",
    ///   { group = "Endpoints", children = ["/api/documents/", "/api/nodes/"] }
    /// ]
    /// ```
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub navs: HashMap<String, NavConfig>,

    /// Route-specific layout overrides
    ///
    /// Allows different layouts for specific routes using glob patterns.
    /// Overrides are evaluated in array order; the first matching entry wins.
    ///
    /// Example:
    /// ```toml
    /// [[site.layout.overrides]]
    /// routes = ["/blog/**"]
    /// left-sidebar = false
    ///
    /// [[site.layout.overrides]]
    /// routes = ["/api/**"]
    /// left-sidebar = { nav = "api" }
    /// ```
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub overrides: Vec<LayoutOverride>,
}

impl SiteLayout {
    /// Check if layout has any active sections
    pub fn has_any(&self) -> bool {
        self.has_header()
            || self.has_left_sidebar()
            || self.has_right_sidebar()
            || self.has_page_nav()
            || self.has_footer()
    }

    /// Check if the header is enabled
    pub fn has_header(&self) -> bool {
        self.header.is_some()
    }

    /// Get the header configuration if enabled
    pub fn header_config(&self) -> Option<&LayoutHeader> {
        self.header.as_ref()
    }

    /// Check if the left sidebar is enabled
    ///
    /// Defaults to `true` when not specified.
    pub fn has_left_sidebar(&self) -> bool {
        self.left_sidebar
            .as_ref()
            .map(|sidebar| sidebar.is_enabled())
            .unwrap_or(true)
    }

    /// Check if the right sidebar is enabled
    pub fn has_right_sidebar(&self) -> bool {
        self.right_sidebar
            .as_ref()
            .map(|sidebar| sidebar.is_enabled())
            .unwrap_or(false)
    }

    /// Check if the right sidebar is explicitly disabled
    ///
    /// Returns true only if user explicitly set `right-sidebar = false`.
    /// Returns false if not configured (None) or if enabled.
    pub fn is_right_sidebar_explicitly_disabled(&self) -> bool {
        self.right_sidebar
            .as_ref()
            .map(|sidebar| sidebar.is_explicitly_disabled())
            .unwrap_or(false)
    }

    /// Get the right sidebar configuration if enabled
    ///
    /// Returns default config when enabled with boolean shorthand.
    pub fn right_sidebar_config(&self) -> Option<RightSidebarConfig> {
        match &self.right_sidebar {
            None => None,
            Some(LayoutRightSidebar::Enabled(false)) => None,
            Some(LayoutRightSidebar::Enabled(true)) => Some(RightSidebarConfig::default()),
            Some(LayoutRightSidebar::Config(config)) => Some(config.clone()),
        }
    }

    /// Check if page navigation is enabled
    ///
    /// Defaults to `true` when left sidebar is enabled.
    pub fn has_page_nav(&self) -> bool {
        self.page_nav.unwrap_or_else(|| self.has_left_sidebar())
    }

    /// Check if the footer is enabled
    pub fn has_footer(&self) -> bool {
        self.footer.is_some()
    }

    /// Get the footer configuration if enabled
    pub fn footer_config(&self) -> Option<&LayoutFooter> {
        self.footer.as_ref()
    }

    /// Get the left sidebar configuration if enabled
    ///
    /// Returns default config when not specified (since left sidebar defaults to enabled).
    pub fn left_sidebar_config(&self) -> Option<LeftSidebarConfig> {
        match &self.left_sidebar {
            None => Some(LeftSidebarConfig::default()),
            Some(LayoutLeftSidebar::Enabled(true)) => Some(LeftSidebarConfig::default()),
            Some(LayoutLeftSidebar::Enabled(false)) => None,
            Some(LayoutLeftSidebar::Config(config)) => Some(config.clone()),
        }
    }
}

/// Header configuration
///
/// Controls the site header appearance including logo, title, navigation links,
/// and icon links.
///
/// Example:
/// ```toml
/// [site.layout.header]
/// logo = "images/logo.svg"
/// title = "My Site"
///
/// [[site.layout.header.links]]
/// label = "Docs"
/// target = "/docs/"
///
/// [[site.layout.header.icons]]
/// icon = "github"
/// target = "https://github.com/example/repo"
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct LayoutHeader {
    /// Path to logo image (relative to site root)
    pub logo: Option<String>,

    /// Site title displayed in header
    pub title: Option<String>,

    /// Navigation top-level links
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<TextLink>,

    /// Icon links (e.g., GitHub, Discord)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub icons: Vec<IconLink>,
}

/// Footer configuration
///
/// Controls the site footer appearance with link groups, icon links,
/// and copyright text.
///
/// Example:
/// ```toml
/// [site.layout.footer]
/// copyright = "© 2024 Stencila Inc."
///
/// [[site.layout.footer.groups]]
/// title = "Product"
/// links = [
///   { label = "Features", target = "/features/" },
///   { label = "Pricing", target = "/pricing/" },
/// ]
///
/// [[site.layout.footer.icons]]
/// icon = "github"
/// target = "https://github.com/stencila/stencila"
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct LayoutFooter {
    /// Groups of links displayed in columns
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<FooterGroup>,

    /// Icon links (e.g., social media)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub icons: Vec<IconLink>,

    /// Copyright text displayed at the bottom
    pub copyright: Option<String>,
}

/// A group of links in the footer
///
/// Displayed as a column with a title and list of links.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct FooterGroup {
    /// Group title displayed above the links
    pub title: String,

    /// Links in this group
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<TextLink>,
}

/// A text link (used in header tabs and footer groups)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct TextLink {
    /// Display label for the link
    pub label: String,

    /// URL to link to
    pub target: String,
}

/// An icon link (used in header and footer)
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct IconLink {
    /// Icon name (Lucide icon name, e.g., "github", "discord")
    pub icon: String,

    /// URL to link to
    pub target: String,

    /// Accessible label (used for aria-label and tooltip)
    pub label: Option<String>,
}

/// Left sidebar configuration
///
/// Supports both boolean shorthand and full configuration:
/// - `left-sidebar = false` → Sidebar disabled
/// - `left-sidebar = true` → Sidebar with auto navigation
/// - `left-sidebar = { nav = "api" }` → Sidebar with named navigation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(untagged)]
pub enum LayoutLeftSidebar {
    /// Boolean shorthand for enable/disable
    Enabled(bool),
    /// Full sidebar configuration
    Config(LeftSidebarConfig),
}

impl LayoutLeftSidebar {
    /// Check if the sidebar is enabled
    pub fn is_enabled(&self) -> bool {
        match self {
            LayoutLeftSidebar::Enabled(enabled) => *enabled,
            LayoutLeftSidebar::Config(_) => true,
        }
    }

    /// Get the configuration if this is a Config variant
    pub fn config(&self) -> Option<&LeftSidebarConfig> {
        match self {
            LayoutLeftSidebar::Config(config) => Some(config),
            LayoutLeftSidebar::Enabled(_) => None,
        }
    }
}

/// Sidebar configuration options
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct LeftSidebarConfig {
    /// Navigation source
    ///
    /// - `"auto"` - Auto-generate navigation from file structure (default)
    /// - Any other value - Use a named navigation config from `site.layout.navs`
    pub nav: Option<String>,

    /// Maximum depth for auto-generated navigation
    ///
    /// Only applies when `nav = "auto"`. Limits how deep the navigation
    /// tree will go. Default is 5.
    pub depth: Option<u8>,

    /// Whether navigation groups are collapsible
    ///
    /// When `true` (default), groups can be expanded/collapsed.
    /// When `false`, all groups are always expanded.
    pub collapsible: Option<bool>,

    /// Initial expansion depth for navigation groups
    ///
    /// Controls how many levels of navigation are expanded by default:
    /// - `0` - All groups start collapsed (only groups with active page are expanded)
    /// - `1` - Top-level groups expanded
    /// - `2` - Two levels expanded
    /// - etc.
    ///
    /// Default is to expand all levels. The client-side component may persist
    /// user preferences in local storage, overriding this on subsequent visits.
    pub expanded: Option<u8>,
}

/// Right sidebar configuration
///
/// Supports both boolean shorthand and full configuration:
/// - `right-sidebar = false` → Sidebar disabled
/// - `right-sidebar = true` → Sidebar with headings (default content)
/// - `right-sidebar = { content = "headings", depth = 3 }` → Full configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(untagged)]
pub enum LayoutRightSidebar {
    /// Boolean shorthand for enable/disable
    Enabled(bool),
    /// Full sidebar configuration
    Config(RightSidebarConfig),
}

impl LayoutRightSidebar {
    /// Check if the sidebar is enabled
    pub fn is_enabled(&self) -> bool {
        match self {
            LayoutRightSidebar::Enabled(enabled) => *enabled,
            LayoutRightSidebar::Config(_) => true,
        }
    }

    /// Check if the sidebar is explicitly disabled
    ///
    /// Returns true only if the user explicitly set `right-sidebar = false`.
    /// Returns false for Config variants or Enabled(true).
    pub fn is_explicitly_disabled(&self) -> bool {
        matches!(self, LayoutRightSidebar::Enabled(false))
    }

    /// Get the configuration if this is a Config variant
    pub fn config(&self) -> Option<&RightSidebarConfig> {
        match self {
            LayoutRightSidebar::Config(config) => Some(config),
            LayoutRightSidebar::Enabled(_) => None,
        }
    }
}

/// Right sidebar configuration options
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct RightSidebarConfig {
    /// Content type for the right sidebar
    ///
    /// - `"headings"` - Display document headings as table of contents (default)
    /// - Future: other content types may be added
    pub content: Option<String>,

    /// Title displayed above the content
    ///
    /// Default: "On this page" for headings content
    pub title: Option<String>,

    /// Maximum heading depth to include (1-6)
    ///
    /// Only applies when `content = "headings"`. Limits how deep the heading
    /// tree will go. Default is 3 (h1, h2, h3).
    pub depth: Option<u8>,
}

/// Route-specific layout override
///
/// Allows different layout settings for specific routes using glob patterns.
/// The first matching override wins (order matters in the array).
///
/// Example:
/// ```toml
/// # Blog pages: no left sidebar
/// [[site.layout.overrides]]
/// routes = ["/blog/**"]
/// left-sidebar = false
/// page-nav = false
///
/// # API docs: use named nav
/// [[site.layout.overrides]]
/// routes = ["/api/**"]
/// left-sidebar = { nav = "api" }
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct LayoutOverride {
    /// Glob patterns for routes this override applies to
    ///
    /// Examples: `["/blog/**"]`, `["/docs/api/**", "/api/**"]`
    /// First matching override wins.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub routes: Vec<String>,

    /// Header configuration override
    ///
    /// Use `false` to hide header, or provide config to replace it.
    pub header: Option<LayoutHeaderOverride>,

    /// Left sidebar configuration override
    ///
    /// Use `false` to disable, `true` for auto nav, or config object.
    pub left_sidebar: Option<LayoutLeftSidebar>,

    /// Right sidebar configuration override
    ///
    /// Use `false` to disable, `true` for default headings, or config object.
    pub right_sidebar: Option<LayoutRightSidebar>,

    /// Footer configuration override
    ///
    /// Use `false` to hide footer, or provide config to replace it.
    pub footer: Option<LayoutFooterOverride>,

    /// Page navigation override
    ///
    /// Use `false` to disable prev/next links.
    pub page_nav: Option<bool>,
}

/// Header override - can be disabled or replaced
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(untagged)]
pub enum LayoutHeaderOverride {
    /// Disable header with `false`
    Enabled(bool),
    /// Replace header with config
    Config(LayoutHeader),
}

/// Footer override - can be disabled or replaced
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(untagged)]
pub enum LayoutFooterOverride {
    /// Disable footer with `false`
    Enabled(bool),
    /// Replace footer with config
    Config(LayoutFooter),
}

/// Named navigation configuration
///
/// Defines a navigation tree that can be referenced by name.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
pub struct NavConfig {
    /// Navigation items in this configuration
    pub items: Vec<NavItem>,
}

/// A navigation item
///
/// Can be:
/// - A route string (e.g., `"/docs/intro/"`)
/// - A link with label and target URL
/// - A group with children
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, JsonSchema)]
#[serde(untagged)]
pub enum NavItem {
    /// A route path - label will be derived from the route
    Route(String),

    /// A link with explicit label
    Link {
        /// Display label for the link
        label: String,
        /// URL to link to
        target: String,
        /// Optional icon name (Lucide icon)
        #[serde(skip_serializing_if = "Option::is_none")]
        icon: Option<String>,
    },

    /// A group of navigation items
    Group {
        /// Group title/label
        group: String,
        /// Child navigation items
        children: Vec<NavItem>,
    },
}

#[cfg(test)]
mod tests {
    use eyre::{OptionExt, Result};

    use super::*;

    #[test]
    fn test_header_config() -> Result<()> {
        let toml = r#"
            [header]
            logo = "images/logo.svg"
            title = "My Site"

            [[header.links]]
            label = "Docs"
            target = "/docs/"

            [[header.links]]
            label = "API"
            target = "/api/"

            [[header.icons]]
            icon = "github"
            target = "https://github.com/example/repo"
            label = "GitHub"
        "#;
        let layout: SiteLayout = toml::from_str(toml)?;

        assert!(layout.has_header());
        let header = layout.header_config().expect("header should be present");
        assert_eq!(header.logo, Some("images/logo.svg".to_string()));
        assert_eq!(header.title, Some("My Site".to_string()));
        assert_eq!(header.links.len(), 2);
        assert_eq!(header.links[0].label, "Docs");
        assert_eq!(header.links[0].target, "/docs/");
        assert_eq!(header.icons.len(), 1);
        assert_eq!(header.icons[0].icon, "github");
        assert_eq!(header.icons[0].label, Some("GitHub".to_string()));

        Ok(())
    }

    #[test]
    fn test_header_minimal() -> Result<()> {
        let toml = r#"
            [header]
            title = "Simple Site"
        "#;
        let layout: SiteLayout = toml::from_str(toml)?;

        assert!(layout.has_header());
        let header = layout.header_config().expect("header should be present");
        assert_eq!(header.logo, None);
        assert_eq!(header.title, Some("Simple Site".to_string()));
        assert!(header.links.is_empty());
        assert!(header.icons.is_empty());

        Ok(())
    }

    #[test]
    fn test_no_header() -> Result<()> {
        let toml = r#"
            left-sidebar = true
        "#;
        let layout: SiteLayout = toml::from_str(toml)?;

        assert!(!layout.has_header());
        assert!(layout.header_config().is_none());

        Ok(())
    }

    #[test]
    fn test_layout_sidebar_bool() -> Result<()> {
        let toml = r#"left-sidebar = true"#;
        let layout: SiteLayout = toml::from_str(toml)?;
        assert!(layout.has_left_sidebar());
        assert!(layout.left_sidebar_config().is_some());

        let toml = r#"left-sidebar = false"#;
        let layout: SiteLayout = toml::from_str(toml)?;
        assert!(!layout.has_left_sidebar());
        assert!(layout.left_sidebar_config().is_none());

        Ok(())
    }

    #[test]
    fn test_layout_sidebar_config() -> Result<()> {
        let toml = r#"
            [left-sidebar]
            nav = "auto"
            collapsible = true
            depth = 2
        "#;
        let layout: SiteLayout = toml::from_str(toml)?;
        assert!(layout.has_left_sidebar());

        let config = layout
            .left_sidebar_config()
            .ok_or_eyre("expected left sidebar")?;
        assert_eq!(config.nav, Some("auto".to_string()));
        assert_eq!(config.collapsible, Some(true));
        assert_eq!(config.depth, Some(2));

        Ok(())
    }

    #[test]
    fn test_nav_item_route() -> Result<()> {
        let toml = r#"items = ["/docs/intro/", "/docs/guide/"]"#;
        let config: NavConfig = toml::from_str(toml)?;
        assert_eq!(config.items.len(), 2);
        assert!(matches!(&config.items[0], NavItem::Route(r) if r == "/docs/intro/"));

        Ok(())
    }

    #[test]
    fn test_nav_item_link() -> Result<()> {
        let toml = r#"items = [{ label = "Home", target = "/" }]"#;
        let config: NavConfig = toml::from_str(toml)?;
        assert_eq!(config.items.len(), 1);
        assert!(
            matches!(&config.items[0], NavItem::Link { label, target, .. } if label == "Home" && target == "/")
        );

        Ok(())
    }

    #[test]
    fn test_nav_item_group() -> Result<()> {
        let toml = r#"
            [[items]]
            group = "Getting Started"
            children = ["/docs/install/", "/docs/quickstart/"]
        "#;
        let config: NavConfig = toml::from_str(toml)?;
        assert_eq!(config.items.len(), 1);
        if let NavItem::Group { group, children } = &config.items[0] {
            assert_eq!(group, "Getting Started");
            assert_eq!(children.len(), 2);
        } else {
            panic!("Expected Group variant");
        }

        Ok(())
    }

    #[test]
    fn test_named_navs() -> Result<()> {
        let toml = r#"
            [navs.api]
            items = ["/api/intro/"]

            [navs.docs]
            items = ["/docs/intro/"]
        "#;
        let layout: SiteLayout = toml::from_str(toml)?;
        assert!(!layout.navs.is_empty());
        assert!(layout.navs.contains_key("api"));
        assert!(layout.navs.contains_key("docs"));

        Ok(())
    }

    #[test]
    fn test_footer_config() -> Result<()> {
        let toml = r#"
            [footer]
            copyright = "© 2024 Stencila Inc."

            [[footer.groups]]
            title = "Product"
            links = [
                { label = "Features", target = "/features/" },
                { label = "Pricing", target = "/pricing/" },
            ]

            [[footer.groups]]
            title = "Resources"
            links = [
                { label = "Documentation", target = "/docs/" },
            ]

            [[footer.icons]]
            icon = "github"
            target = "https://github.com/stencila/stencila"
            label = "GitHub"
        "#;
        let layout: SiteLayout = toml::from_str(toml)?;

        assert!(layout.has_footer());
        let footer = layout.footer_config().expect("footer should be present");
        assert_eq!(footer.copyright, Some("© 2024 Stencila Inc.".to_string()));
        assert_eq!(footer.groups.len(), 2);
        assert_eq!(footer.groups[0].title, "Product");
        assert_eq!(footer.groups[0].links.len(), 2);
        assert_eq!(footer.groups[0].links[0].label, "Features");
        assert_eq!(footer.groups[1].title, "Resources");
        assert_eq!(footer.icons.len(), 1);
        assert_eq!(footer.icons[0].icon, "github");

        Ok(())
    }

    #[test]
    fn test_footer_minimal() -> Result<()> {
        let toml = r#"
            [footer]
            copyright = "© 2024"
        "#;
        let layout: SiteLayout = toml::from_str(toml)?;

        assert!(layout.has_footer());
        let footer = layout.footer_config().expect("footer should be present");
        assert_eq!(footer.copyright, Some("© 2024".to_string()));
        assert!(footer.groups.is_empty());
        assert!(footer.icons.is_empty());

        Ok(())
    }

    #[test]
    fn test_no_footer() -> Result<()> {
        let toml = r#"
            left-sidebar = true
        "#;
        let layout: SiteLayout = toml::from_str(toml)?;

        assert!(!layout.has_footer());
        assert!(layout.footer_config().is_none());

        Ok(())
    }

    #[test]
    fn test_page_nav_default() -> Result<()> {
        // Defaults to true when left sidebar is enabled (default)
        let layout = SiteLayout::default();
        assert!(layout.has_page_nav());

        // Explicit false
        let toml = r#"page-nav = false"#;
        let layout: SiteLayout = toml::from_str(toml)?;
        assert!(!layout.has_page_nav());

        // Explicit true
        let toml = r#"page-nav = true"#;
        let layout: SiteLayout = toml::from_str(toml)?;
        assert!(layout.has_page_nav());

        Ok(())
    }

    #[test]
    fn test_override_basic() -> Result<()> {
        let toml = r#"
            [[overrides]]
            routes = ["/blog/**"]
            left-sidebar = false
            page-nav = false
        "#;
        let layout: SiteLayout = toml::from_str(toml)?;

        assert_eq!(layout.overrides.len(), 1);
        assert_eq!(layout.overrides[0].routes, vec!["/blog/**"]);
        assert!(matches!(
            layout.overrides[0].left_sidebar,
            Some(LayoutLeftSidebar::Enabled(false))
        ));
        assert_eq!(layout.overrides[0].page_nav, Some(false));

        Ok(())
    }

    #[test]
    fn test_override_with_sidebar_config() -> Result<()> {
        let toml = r#"
            [[overrides]]
            routes = ["/api/**"]
            [overrides.left-sidebar]
            nav = "api"
            collapsible = false
        "#;
        let layout: SiteLayout = toml::from_str(toml)?;

        assert_eq!(layout.overrides.len(), 1);
        if let Some(LayoutLeftSidebar::Config(config)) = &layout.overrides[0].left_sidebar {
            assert_eq!(config.nav, Some("api".to_string()));
            assert_eq!(config.collapsible, Some(false));
        } else {
            panic!("Expected LayoutSidebar::Config");
        }

        Ok(())
    }

    #[test]
    fn test_override_header_disabled() -> Result<()> {
        let toml = r#"
            [[overrides]]
            routes = ["/landing/**"]
            header = false
        "#;
        let layout: SiteLayout = toml::from_str(toml)?;

        assert!(matches!(
            layout.overrides[0].header,
            Some(LayoutHeaderOverride::Enabled(false))
        ));

        Ok(())
    }

    #[test]
    fn test_override_header_replaced() -> Result<()> {
        let toml = r#"
            [[overrides]]
            routes = ["/blog/**"]
            [overrides.header]
            logo = "blog-logo.svg"
            title = "Blog"
        "#;
        let layout: SiteLayout = toml::from_str(toml)?;

        if let Some(LayoutHeaderOverride::Config(header)) = &layout.overrides[0].header {
            assert_eq!(header.logo, Some("blog-logo.svg".to_string()));
            assert_eq!(header.title, Some("Blog".to_string()));
        } else {
            panic!("Expected LayoutHeaderOverride::Config");
        }

        Ok(())
    }

    #[test]
    fn test_override_footer_disabled() -> Result<()> {
        let toml = r#"
            [[overrides]]
            routes = ["/embed/**"]
            footer = false
        "#;
        let layout: SiteLayout = toml::from_str(toml)?;

        assert!(matches!(
            layout.overrides[0].footer,
            Some(LayoutFooterOverride::Enabled(false))
        ));

        Ok(())
    }

    #[test]
    fn test_multiple_overrides() -> Result<()> {
        let toml = r#"
            [[overrides]]
            routes = ["/docs/api/**"]
            right-sidebar = true

            [[overrides]]
            routes = ["/docs/**"]
            right-sidebar = false

            [[overrides]]
            routes = ["/blog/**"]
            left-sidebar = false
        "#;
        let layout: SiteLayout = toml::from_str(toml)?;

        assert_eq!(layout.overrides.len(), 3);
        assert_eq!(layout.overrides[0].routes, vec!["/docs/api/**"]);
        assert!(matches!(
            layout.overrides[0].right_sidebar,
            Some(LayoutRightSidebar::Enabled(true))
        ));
        assert_eq!(layout.overrides[1].routes, vec!["/docs/**"]);
        assert!(matches!(
            layout.overrides[1].right_sidebar,
            Some(LayoutRightSidebar::Enabled(false))
        ));
        assert_eq!(layout.overrides[2].routes, vec!["/blog/**"]);
        assert!(matches!(
            layout.overrides[2].left_sidebar,
            Some(LayoutLeftSidebar::Enabled(false))
        ));

        Ok(())
    }

    #[test]
    fn test_override_multiple_routes() -> Result<()> {
        let toml = r#"
            [[overrides]]
            routes = ["/landing/", "/home/"]
            left-sidebar = false
            header = false
        "#;
        let layout: SiteLayout = toml::from_str(toml)?;

        assert_eq!(layout.overrides[0].routes.len(), 2);
        assert_eq!(layout.overrides[0].routes[0], "/landing/");
        assert_eq!(layout.overrides[0].routes[1], "/home/");

        Ok(())
    }
}
