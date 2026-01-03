use serde::{Deserialize, Serialize};
use stencila_codec_dom_trait::html_escape::{self, encode_double_quoted_attribute, encode_safe};

/// Pre-computed layout data for a specific route
///
/// This struct contains all layout-related data needed to render
/// a page, including the navigation tree with active/expanded states.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedLayout {
    /// Resolved header configuration (if enabled)
    pub header: Option<ResolvedHeader>,

    /// Whether left sidebar is enabled
    pub left_sidebar: bool,

    /// Whether right sidebar is enabled
    pub right_sidebar: bool,

    /// Navigation tree for left sidebar (if enabled)
    pub nav_tree: Option<Vec<NavTreeItem>>,

    /// Whether nav groups are collapsible
    pub collapsible: bool,

    /// Initial expansion depth (None = expand all)
    pub expanded_depth: Option<u8>,

    /// Current route (for client-side active state updates)
    pub current_route: String,
}

/// Pre-computed header data for a specific route
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedHeader {
    /// Logo path (resolved relative to base URL)
    pub logo: Option<String>,

    /// Site title
    pub title: Option<String>,

    /// Navigation tabs with active state computed
    pub tabs: Vec<ResolvedTab>,

    /// Icon links
    pub icons: Vec<ResolvedIconLink>,
}

/// A resolved tab with active state
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedTab {
    /// Display label
    pub label: String,

    /// URL to link to
    pub href: String,

    /// Whether this tab is active (current route starts with tab href)
    pub active: bool,
}

/// A resolved icon link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedIconLink {
    /// Icon name (Lucide icon name)
    pub icon: String,

    /// URL to link to
    pub href: String,

    /// Accessible label
    pub label: String,
}

/// A navigation tree item for site layouts
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NavTreeItem {
    /// Display label for this item
    pub label: String,

    /// URL to navigate to (None for group headers)
    pub href: Option<String>,

    /// Optional icon name (Lucide icon)
    pub icon: Option<String>,

    /// Whether this item is the current page
    pub active: bool,

    /// Whether this group is expanded (only relevant for items with children)
    pub expanded: bool,

    /// Child navigation items (for groups/directories)
    pub children: Option<Vec<NavTreeItem>>,
}

/// Render header from resolved layout
///
/// Generates HTML for the site header with logo, title, tabs, and icons.
pub fn render_header(layout: &ResolvedLayout) -> Option<String> {
    let header = layout.header.as_ref()?;

    let mut html = String::from(r#"<stencila-header slot="header">"#);

    // Logo and title
    if header.logo.is_some() || header.title.is_some() {
        html.push_str(
            r#"
      <a href="/" class="header-brand">"#,
        );

        if let Some(ref logo) = header.logo {
            html.push_str(&format!(
                r#"
        <img src="{}" alt="" class="header-logo" />"#,
                encode_double_quoted_attribute(logo)
            ));
        }

        if let Some(ref title) = header.title {
            html.push_str(&format!(
                r#"
        <span class="header-title">{}</span>"#,
                encode_safe(title)
            ));
        }

        html.push_str(
            r#"
      </a>"#,
        );
    }

    // Navigation tabs
    if !header.tabs.is_empty() {
        html.push_str(
            r#"
      <stencila-nav-tabs role="navigation" aria-label="Main navigation">"#,
        );

        for tab in &header.tabs {
            let active_class = if tab.active { " active" } else { "" };
            let aria_current = if tab.active {
                r#" aria-current="page""#
            } else {
                ""
            };

            html.push_str(&format!(
                r#"
        <a href="{}" class="nav-tab{active_class}"{aria_current}>{}</a>"#,
                encode_double_quoted_attribute(&tab.href),
                encode_safe(&tab.label)
            ));
        }

        html.push_str(
            r#"
      </stencila-nav-tabs>"#,
        );
    }

    // Spacer to push icons to the right
    html.push_str(
        r#"
      <div class="header-spacer"></div>"#,
    );

    // Icon links
    if !header.icons.is_empty() {
        html.push_str(
            r#"
      <div class="header-icons">"#,
        );

        for icon_link in &header.icons {
            html.push_str(&format!(
                r#"
        <a href="{}" class="header-icon-link" aria-label="{}" target="_blank" rel="noopener noreferrer" data-icon="{}">
          <span class="icon-placeholder"></span>
        </a>"#,
                encode_double_quoted_attribute(&icon_link.href),
                encode_double_quoted_attribute(&icon_link.label),
                encode_double_quoted_attribute(&icon_link.icon)
            ));
        }

        html.push_str(
            r#"
      </div>"#,
        );
    }

    html.push_str(
        r#"
    </stencila-header>"#,
    );

    Some(html)
}

/// Render navigation tree from resolved layout
///
/// Generates ARIA-compliant HTML for the navigation tree.
pub fn render_nav(layout: &ResolvedLayout) -> Option<String> {
    /// Render a nav tree item recursively
    fn render_item(item: &NavTreeItem, collapsible: bool) -> String {
        let active_class = if item.active { " active" } else { "" };
        let aria_current = if item.active {
            r#" aria-current="page""#
        } else {
            ""
        };

        if let Some(ref children) = item.children {
            // Group with children
            let expanded = if item.expanded { " open" } else { "" };
            let aria_expanded = if item.expanded { "true" } else { "false" };

            if collapsible {
                let mut html = format!(
                    r#"
            <li role="treeitem" aria-expanded="{aria_expanded}">
              <details{expanded}>
                <summary class="nav-group{active_class}">{}</summary>
                <ul role="group">"#,
                    html_escape::encode_text(&item.label)
                );

                for child in children {
                    html.push_str(&render_item(child, collapsible));
                }

                html.push_str(
                    r#"
                </ul>
              </details>
            </li>"#,
                );
                html
            } else {
                let mut html = format!(
                    r#"
            <li role="treeitem">
              <span class="nav-group{active_class}">{}</span>
              <ul role="group">"#,
                    html_escape::encode_text(&item.label)
                );

                for child in children {
                    html.push_str(&render_item(child, collapsible));
                }

                html.push_str(
                    r#"
              </ul>
            </li>"#,
                );
                html
            }
        } else if let Some(ref href) = item.href {
            // Leaf link
            format!(
                r#"
            <li role="treeitem"><a href="{}" class="nav-link{active_class}"{aria_current}>{}</a></li>"#,
                html_escape::encode_double_quoted_attribute(href),
                html_escape::encode_text(&item.label)
            )
        } else {
            // Label only (no link)
            format!(
                r#"
            <li role="treeitem"><span class="nav-label">{}</span></li>"#,
                html_escape::encode_text(&item.label)
            )
        }
    }

    if !layout.left_sidebar {
        return None;
    }

    let nav_tree = layout.nav_tree.as_ref()?;
    if nav_tree.is_empty() {
        return None;
    }

    let mut html = String::from(
        r#"<nav slot="left-sidebar" role="navigation" aria-label="Main navigation">
        <stencila-nav-tree>
          <ul role="tree">"#,
    );

    for item in nav_tree {
        html.push_str(&render_item(item, layout.collapsible));
    }

    html.push_str(
        r#"
          </ul>
        </stencila-nav-tree>
      </nav>"#,
    );

    Some(html)
}
