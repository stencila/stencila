use stencila_codec_dom_trait::html_escape::{self, encode_double_quoted_attribute, encode_safe};

/// Pre-computed layout data for a specific route
///
/// This struct contains all layout-related data needed to render
/// a page, including the navigation tree with active/expanded states.
#[derive(Debug, Clone)]
pub struct ResolvedLayout {
    /// Resolved header configuration (if enabled)
    pub header: Option<ResolvedHeader>,

    /// Resolved left sidebar configuration (if enabled)
    pub left_sidebar: Option<ResolvedLeftSidebar>,

    /// Resolved right sidebar configuration (if enabled)
    pub right_sidebar: Option<ResolvedRightSidebar>,

    /// Resolved footer configuration (if enabled)
    pub footer: Option<ResolvedFooter>,

    /// Breadcrumb trail for the current route
    pub breadcrumbs: Vec<BreadcrumbItem>,

    /// Page navigation links (prev/next)
    pub page_nav: Option<PageNavLinks>,

    /// Current route (for client-side active state updates)
    pub current_route: String,
}

/// Pre-computed header data for a specific route
#[derive(Debug, Clone)]
pub struct ResolvedHeader {
    /// Logo path (resolved relative to base URL)
    pub logo: Option<String>,

    /// Site title
    pub title: Option<String>,

    /// Navigation links with active state computed
    pub links: Vec<ResolvedNavLink>,

    /// Icon links
    pub icons: Vec<ResolvedIconLink>,
}

/// A resolved tab with active state
#[derive(Debug, Clone)]
pub struct ResolvedNavLink {
    /// Display label
    pub label: String,

    /// URL to link to
    pub href: String,

    /// Whether this tab is active (current route starts with tab href)
    pub active: bool,
}

/// A resolved icon link
#[derive(Debug, Clone)]
pub struct ResolvedIconLink {
    /// Icon name (Lucide icon name)
    pub icon: String,

    /// URL to link to
    pub href: String,

    /// Accessible label
    pub label: String,
}

/// Pre-computed footer data
#[derive(Debug, Clone)]
pub struct ResolvedFooter {
    /// Groups of links
    pub groups: Vec<ResolvedFooterGroup>,

    /// Icon links
    pub icons: Vec<ResolvedIconLink>,

    /// Copyright text
    pub copyright: Option<String>,
}

/// A resolved footer link group
#[derive(Debug, Clone)]
pub struct ResolvedFooterGroup {
    /// Group title
    pub title: String,

    /// Links in this group
    pub links: Vec<ResolvedNavLink>,
}

/// Resolved left sidebar configuration
///
/// Contains the navigation tree and display options for the left sidebar.
#[derive(Debug, Clone)]
pub struct ResolvedLeftSidebar {
    /// Navigation tree items
    pub nav_tree: Vec<NavTreeItem>,

    /// Whether nav groups are collapsible
    pub collapsible: bool,

    /// Initial expansion depth (None = expand all)
    pub expanded_depth: Option<u8>,
}

/// Resolved right sidebar configuration
///
/// Contains the table of contents (headings) for the right sidebar.
#[derive(Debug, Clone)]
pub struct ResolvedRightSidebar {
    /// Title displayed above the headings (e.g., "On this page")
    pub title: String,

    /// Headings for table of contents
    pub headings: Vec<HeadingItem>,
}

/// A breadcrumb item in the navigation trail
#[derive(Debug, Clone)]
pub struct BreadcrumbItem {
    /// Display label
    pub label: String,

    /// URL to link to
    pub href: String,

    /// Whether this is the current page (last item)
    pub current: bool,
}

/// Page navigation links (prev/next)
#[derive(Debug, Clone)]
pub struct PageNavLinks {
    /// Link to previous page
    pub prev: Option<PageLink>,

    /// Link to next page
    pub next: Option<PageLink>,
}

/// A page link for navigation
#[derive(Debug, Clone)]
pub struct PageLink {
    /// Display label
    pub label: String,

    /// URL to link to
    pub href: String,
}

/// A navigation tree item for site layouts
#[derive(Debug, Clone)]
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

/// A heading item for right sidebar table of contents
#[derive(Debug, Clone)]
pub struct HeadingItem {
    /// Node ID for linking (used as anchor)
    pub id: String,

    /// Heading text content
    pub text: String,

    /// Heading level (1-6)
    pub level: u8,

    /// Child headings (nested subheadings)
    pub children: Vec<HeadingItem>,
}

pub fn render_layout(content: &str, resolved_layout: &ResolvedLayout) -> String {
    // Check sidebar presence
    let has_left_sidebar = resolved_layout.left_sidebar.is_some();
    let has_right_sidebar = resolved_layout.right_sidebar.is_some();

    // Build layout attributes
    let mut layout_attrs = String::new();
    if has_left_sidebar {
        layout_attrs.push_str(" left-sidebar");
    }
    if has_right_sidebar {
        layout_attrs.push_str(" right-sidebar");
    }

    // Render header if available
    let header_html = render_header(resolved_layout).unwrap_or_default();

    // Render navigation tree (left sidebar) if available
    let nav_html = render_nav(resolved_layout).unwrap_or_default();

    // Render right sidebar (table of contents) if available
    let right_sidebar_html = render_right_sidebar(resolved_layout).unwrap_or_default();

    // Render footer if available
    let footer_html = render_footer(resolved_layout).unwrap_or_default();

    // Render breadcrumbs if available
    let breadcrumbs_html = render_breadcrumbs(resolved_layout).unwrap_or_default();

    // Render page navigation if available
    let page_nav_html = render_page_nav(resolved_layout).unwrap_or_default();

    // Hamburger button for mobile navigation - rendered when left sidebar is enabled
    // This is associated with the left sidebar, not the header
    let hamburger_html = if has_left_sidebar {
        MOBILE_NAV_TOGGLE_HTML
    } else {
        ""
    };

    // The "skip link" is an accessibility feature (WCAG 2.4.1) that allows keyboard
    // and screen reader users to bypass repetitive navigation elements and jump
    // directly to the main content. It's visually hidden until focused.
    format!(
        r##"<stencila-layout{layout_attrs}>
      <a href="#main-content" class="skip-link">Skip to content</a>{hamburger_html}
      {header_html}
      {nav_html}
      <main id="main-content" slot="content">
        {breadcrumbs_html}
        {content}
        {page_nav_html}
      </main>
      {right_sidebar_html}{footer_html}
    </stencila-layout>"##
    )
}

/// SVG markup for the mobile navigation toggle button (hamburger menu)
///
/// This is defined as a constant rather than inline to:
/// 1. Keep the SVG in one maintainable location
/// 2. Ensure the hamburger works without JS/CSS (critical mobile UI)
/// 3. Allow both open (hamburger) and close (X) states via CSS toggle
pub const MOBILE_NAV_TOGGLE_HTML: &str = r#"<button class="mobile-nav-toggle" aria-label="Toggle navigation" aria-expanded="false" aria-controls="left-sidebar">
        <svg class="hamburger-open" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="3" y1="6" x2="21" y2="6"></line>
          <line x1="3" y1="12" x2="21" y2="12"></line>
          <line x1="3" y1="18" x2="21" y2="18"></line>
        </svg>
        <svg class="hamburger-close" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <line x1="18" y1="6" x2="6" y2="18"></line>
          <line x1="6" y1="6" x2="18" y2="18"></line>
        </svg>
      </button>"#;

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
    if !header.links.is_empty() {
        html.push_str(
            r#"
      <stencila-nav-tabs role="navigation" aria-label="Main navigation">"#,
        );

        for tab in &header.links {
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

    let left_sidebar = layout.left_sidebar.as_ref()?;
    if left_sidebar.nav_tree.is_empty() {
        return None;
    }

    let mut html = String::from(
        r#"<nav slot="left-sidebar" role="navigation" aria-label="Main navigation">
        <stencila-nav-tree>
          <ul role="tree">"#,
    );

    for item in &left_sidebar.nav_tree {
        html.push_str(&render_item(item, left_sidebar.collapsible));
    }

    html.push_str(
        r#"
          </ul>
        </stencila-nav-tree>
      </nav>"#,
    );

    Some(html)
}

/// Render right sidebar from resolved layout
///
/// Generates HTML for the right sidebar with table of contents (headings).
/// The headings are rendered as a nested list with links to each heading anchor.
pub fn render_right_sidebar(layout: &ResolvedLayout) -> Option<String> {
    let right_sidebar = layout.right_sidebar.as_ref()?;
    if right_sidebar.headings.is_empty() {
        return None;
    }

    let mut html = format!(
        r#"<nav slot="right-sidebar" class="toc" aria-label="Table of contents">
        <h2 class="toc-title">{}</h2>
        <stencila-toc-tree>
          <ul role="tree">"#,
        encode_safe(&right_sidebar.title)
    );

    for heading in &right_sidebar.headings {
        html.push_str(&render_heading_item(heading));
    }

    html.push_str(
        r#"
          </ul>
        </stencila-toc-tree>
      </nav>"#,
    );

    Some(html)
}

/// Render a single heading item and its children recursively
fn render_heading_item(item: &HeadingItem) -> String {
    let has_children = !item.children.is_empty();
    let escaped_id = encode_double_quoted_attribute(&item.id);
    let escaped_text = encode_safe(&item.text);

    if has_children {
        let mut html = format!(
            r##"
            <li role="treeitem" aria-expanded="true">
              <a href="#{escaped_id}" class="toc-link" data-level="{}">{escaped_text}</a>
              <ul role="group">"##,
            item.level
        );

        for child in &item.children {
            html.push_str(&render_heading_item(child));
        }

        html.push_str(
            r#"
              </ul>
            </li>"#,
        );
        html
    } else {
        format!(
            r##"
            <li role="treeitem">
              <a href="#{escaped_id}" class="toc-link" data-level="{}">{escaped_text}</a>
            </li>"##,
            item.level
        )
    }
}

/// Render footer from resolved layout
///
/// Generates HTML for the site footer with link groups, icons, and copyright.
pub fn render_footer(layout: &ResolvedLayout) -> Option<String> {
    let footer = layout.footer.as_ref()?;

    let mut html = String::from(r#"<stencila-footer slot="footer">"#);

    // Link groups
    if !footer.groups.is_empty() {
        html.push_str(
            r#"
      <div class="footer-groups">"#,
        );

        for group in &footer.groups {
            html.push_str(&format!(
                r#"
        <div class="footer-group">
          <h4 class="footer-group-title">{}</h4>
          <ul class="footer-group-links">"#,
                encode_safe(&group.title)
            ));

            for link in &group.links {
                html.push_str(&format!(
                    r#"
            <li><a href="{}">{}</a></li>"#,
                    encode_double_quoted_attribute(&link.href),
                    encode_safe(&link.label)
                ));
            }

            html.push_str(
                r#"
          </ul>
        </div>"#,
            );
        }

        html.push_str(
            r#"
      </div>"#,
        );
    }

    // Bottom section with icons and copyright
    let has_bottom = !footer.icons.is_empty() || footer.copyright.is_some();
    if has_bottom {
        html.push_str(
            r#"
      <div class="footer-bottom">"#,
        );

        // Icon links
        if !footer.icons.is_empty() {
            html.push_str(
                r#"
        <div class="footer-icons">"#,
            );

            for icon in &footer.icons {
                html.push_str(&format!(
                    r#"
          <a href="{}" class="footer-icon-link" aria-label="{}" target="_blank" rel="noopener noreferrer" data-icon="{}">
            <span class="icon-placeholder"></span>
          </a>"#,
                    encode_double_quoted_attribute(&icon.href),
                    encode_double_quoted_attribute(&icon.label),
                    encode_double_quoted_attribute(&icon.icon)
                ));
            }

            html.push_str(
                r#"
        </div>"#,
            );
        }

        // Copyright
        if let Some(ref copyright) = footer.copyright {
            html.push_str(&format!(
                r#"
        <p class="footer-copyright">{}</p>"#,
                encode_safe(copyright)
            ));
        }

        html.push_str(
            r#"
      </div>"#,
        );
    }

    html.push_str(
        r#"
    </stencila-footer>"#,
    );

    Some(html)
}

/// Render breadcrumbs from resolved layout
///
/// Generates HTML for the breadcrumb navigation trail.
pub fn render_breadcrumbs(layout: &ResolvedLayout) -> Option<String> {
    if layout.breadcrumbs.is_empty() {
        return None;
    }

    let mut html = String::from(
        r#"<stencila-breadcrumbs aria-label="Breadcrumb">
        <ol>"#,
    );

    for crumb in &layout.breadcrumbs {
        if crumb.current {
            html.push_str(&format!(
                r#"
          <li aria-current="page">{}</li>"#,
                encode_safe(&crumb.label)
            ));
        } else {
            html.push_str(&format!(
                r#"
          <li><a href="{}">{}</a></li>"#,
                encode_double_quoted_attribute(&crumb.href),
                encode_safe(&crumb.label)
            ));
        }
    }

    html.push_str(
        r#"
        </ol>
      </stencila-breadcrumbs>"#,
    );

    Some(html)
}

/// Render page navigation from resolved layout
///
/// Generates HTML for prev/next page links.
pub fn render_page_nav(layout: &ResolvedLayout) -> Option<String> {
    let page_nav = layout.page_nav.as_ref()?;

    // Only render if there's at least one link
    if page_nav.prev.is_none() && page_nav.next.is_none() {
        return None;
    }

    let mut html = String::from(r#"<stencila-page-nav>"#);

    // Previous link
    if let Some(ref prev) = page_nav.prev {
        html.push_str(&format!(
            r#"
        <a href="{}" class="page-nav-prev" rel="prev">
          <span class="page-nav-label">Previous</span>
          <span class="page-nav-title">{}</span>
        </a>"#,
            encode_double_quoted_attribute(&prev.href),
            encode_safe(&prev.label)
        ));
    } else {
        html.push_str(
            r#"
        <span class="page-nav-prev page-nav-empty"></span>"#,
        );
    }

    // Next link
    if let Some(ref next) = page_nav.next {
        html.push_str(&format!(
            r#"
        <a href="{}" class="page-nav-next" rel="next">
          <span class="page-nav-label">Next</span>
          <span class="page-nav-title">{}</span>
        </a>"#,
            encode_double_quoted_attribute(&next.href),
            encode_safe(&next.label)
        ));
    } else {
        html.push_str(
            r#"
        <span class="page-nav-next page-nav-empty"></span>"#,
        );
    }

    html.push_str(
        r#"
      </stencila-page-nav>"#,
    );

    Some(html)
}
