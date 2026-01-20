use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use indexmap::IndexMap;

use eyre::{OptionExt, Result, bail, eyre};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum::Display;
use toml_edit::{DocumentMut, InlineTable, Item, Table, value};

use crate::{
    CONFIG_FILENAME, ConfigRelativePath, DOMAIN_REGEX, SpreadMode, find_config_file,
    layout::LayoutConfig, reviews::ReviewsSpec, validate_placeholders,
};

/// Additional formats to generate alongside HTML during site rendering
///
/// Controls which format files are generated and which format-specific
/// buttons (e.g., copy-markdown) are displayed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SiteFormat {
    /// Markdown format (generates page.md files)
    Md,
    // Future: JsonLd, Docx, Pdf etc
}

/// Logo configuration - simple string or responsive object
///
/// Supports both simple usage with a single logo path and advanced usage
/// with responsive variants and dark mode support.
///
/// Example (simple):
/// ```toml
/// [site]
/// logo = "logo.svg"
/// ```
///
/// Example (responsive):
/// ```toml
/// [site.logo]
/// default = "logo.svg"
/// mobile = "logo-mobile.svg"
/// dark = "logo-dark.svg"
/// link = "/"
/// alt = "Company Logo"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum LogoSpec {
    /// Simple path to logo image
    Path(String),

    /// Responsive logo configuration with variants
    Config(LogoConfig),
}

/// Responsive logo configuration with breakpoint and dark mode variants
///
/// All fields are optional. Missing variants fall back through a cascade:
/// - `dark-mobile` → `dark` → `mobile` → `default`
/// - `dark-tablet` → `dark` → `tablet` → `default`
/// - `dark` → `default`
/// - `mobile` → `default`
/// - `tablet` → `default`
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct LogoConfig {
    /// Default logo image path (used for desktop light mode)
    pub default: Option<String>,

    /// Logo for mobile breakpoint (< 640px)
    pub mobile: Option<String>,

    /// Logo for tablet breakpoint (640px - 768px)
    pub tablet: Option<String>,

    /// Logo for dark mode (desktop)
    pub dark: Option<String>,

    /// Logo for dark mode on mobile
    pub dark_mobile: Option<String>,

    /// Logo for dark mode on tablet
    pub dark_tablet: Option<String>,

    /// Link target when logo is clicked (default: "/")
    pub link: Option<String>,

    /// Alt text for accessibility (used as aria-label on the link)
    pub alt: Option<String>,
}

/// Author specification - simple string or full Author object
///
/// Supports both simple usage with a name string and advanced usage
/// with a full Stencila Author object for richer metadata.
///
/// Example (simple):
/// ```toml
/// [site]
/// author = "Acme Inc"
/// ```
///
/// Example (full):
/// ```toml
/// [site.author]
/// type = "Organization"
/// name = "Acme Inc"
/// url = "https://acme.com"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum AuthorSpec {
    /// Simple name string (will be used as-is for copyright)
    Name(String),

    /// Full Author object (Person, Organization, etc.)
    /// Uses Author::name() for copyright holder text
    #[schemars(schema_with = "author_schema")]
    Author(stencila_schema::Author),
}

impl AuthorSpec {
    /// Get the display name for copyright purposes
    pub fn name(&self) -> String {
        match self {
            AuthorSpec::Name(name) => name.clone(),
            AuthorSpec::Author(author) => author.name(),
        }
    }
}

/// Navigation item for site.nav configuration
///
/// Defines hierarchical navigation structure used by nav-tree and prev-next components.
/// Supports three forms for flexible TOML configuration:
///
/// 1. Route string shorthand - label derived from route:
/// ```toml
/// nav = ["/", "/docs/getting-started/", "/about/"]
/// ```
///
/// 2. Link with explicit label:
/// ```toml
/// nav = [
///   { label = "Home", route = "/" },
///   { label = "Getting Started", route = "/docs/getting-started/" },
/// ]
/// ```
///
/// 3. Group with nested children:
/// ```toml
/// nav = [
///   "/",
///   { label = "Docs", children = [
///     "/docs/getting-started/",
///     "/docs/configuration/",
///   ]},
///   { label = "Guides", route = "/guides/", children = [
///     "/guides/deployment/",
///   ]},
/// ]
/// ```
///
/// Note: Only internal routes are supported. External URLs should be placed
/// in header/footer components, not in site navigation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum NavItem {
    /// Route path shorthand - label derived from route
    ///
    /// Example: `"/docs/guide/"` → label "Guide", href "/docs/guide/"
    Route(String),

    /// Group with nested children
    ///
    /// Groups can optionally link to a page (making the header clickable).
    /// If `route` is omitted, the header only toggles expand/collapse.
    ///
    /// Note: This variant must come before `Link` in the enum because serde's
    /// untagged deserialization tries variants in order. Since `Group` has
    /// a required `children` field, it's more specific than `Link`.
    Group {
        /// Stable identifier for filtering (optional)
        ///
        /// Use when label may change for UX copy reasons. Filter with `#id` syntax.
        id: Option<String>,

        /// Display text for the group header
        label: String,

        /// Optional route for the group header link
        ///
        /// When set, clicking the group header navigates to this page.
        /// When omitted, the header only toggles expand/collapse.
        route: Option<String>,

        /// Nested navigation items
        children: Vec<NavItem>,

        /// Optional icon name for nav rendering
        ///
        /// Icon format: "banana" (default set) or "lucide:banana" (explicit library)
        icon: Option<String>,

        /// Optional section title for grouping within nav-menu dropdowns
        #[serde(rename = "section-title")]
        section_title: Option<String>,
    },

    /// Link with explicit label
    ///
    /// Use when you need a custom label different from the route-derived one.
    Link {
        /// Stable identifier for filtering (optional)
        ///
        /// Use when label may change for UX copy reasons. Filter with `#id` syntax.
        id: Option<String>,

        /// Display text for the navigation item
        label: String,

        /// Internal route path (must start with "/")
        route: String,

        /// Optional icon name for nav rendering
        ///
        /// Icon format: "banana" (default set) or "lucide:banana" (explicit library)
        icon: Option<String>,

        /// Optional short description for nav-menu dropdowns
        description: Option<String>,
    },
}

impl NavItem {
    /// Validate that all routes in the nav item are internal (start with "/")
    pub fn validate(&self) -> Result<()> {
        match self {
            NavItem::Route(route) => {
                if !route.starts_with('/') {
                    bail!(
                        "Invalid nav route `{route}`: must be an internal route starting with '/'"
                    );
                }
            }
            NavItem::Group {
                label,
                route,
                children,
                ..
            } => {
                if let Some(route) = route
                    && !route.starts_with('/')
                {
                    bail!(
                        "Invalid nav route `{route}` in group `{label}`: must be an internal route starting with '/'"
                    );
                }
                for child in children {
                    child.validate()?;
                }
            }
            NavItem::Link { label, route, .. } => {
                if !route.starts_with('/') {
                    bail!(
                        "Invalid nav route `{route}` for `{label}`: must be an internal route starting with '/'"
                    );
                }
            }
        }
        Ok(())
    }
}

/// Featured/promotional content for nav-menu dropdowns
///
/// Displays promotional content in the dropdown panel of a nav group.
/// Configured in `site.featured`, keyed by the dropdown's parent group route or label.
///
/// Example:
/// ```toml
/// [site.featured.docs]
/// badge = "New"
/// icon = "rocket"
/// title = "Quick Start Guide"
/// description = "Get up and running in minutes"
/// cta = { label = "Get Started", route = "/docs/getting-started/" }
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct FeaturedContent {
    /// Badge text displayed above the title (e.g., "Featured", "New", "Spotlight")
    ///
    /// When set, displays a small uppercase label above the title to draw attention.
    pub badge: Option<String>,

    /// Icon name displayed above the title
    ///
    /// Icon format: "rocket" (default lucide set) or "tabler:rocket" (explicit library).
    /// Displays in a rounded box with accent color background.
    pub icon: Option<String>,

    /// Image path (relative to site root)
    pub image: Option<String>,

    /// Promotional title
    pub title: String,

    /// Short description text
    pub description: Option<String>,

    /// Call-to-action button
    pub cta: Option<FeaturedCta>,
}

/// Call-to-action button for featured content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct FeaturedCta {
    /// Button label text
    pub label: String,

    /// Target route
    pub route: String,
}

impl FeaturedContent {
    /// Validate featured content configuration
    ///
    /// Warns if both `icon` and `image` are specified (only one will be shown).
    pub fn validate(&self, key: &str) {
        if self.icon.is_some() && self.image.is_some() {
            tracing::warn!(
                "Featured content `{key}` has both `icon` and `image` specified; only the icon will be displayed"
            );
        }
    }

    /// Validate that image path exists relative to the site root
    pub fn validate_image_path(&self, key: &str, site_root: &Path) {
        if let Some(image) = &self.image {
            let image_path = site_root.join(image);
            if !image_path.exists() {
                tracing::warn!(
                    "Featured content `{key}` has image path `{image}` that does not exist in site root"
                );
            }
        }
    }
}

/// Simple JSON schema for Author - describes it as an object with type, name, and url
fn author_schema(_gen: &mut schemars::SchemaGenerator) -> schemars::Schema {
    serde_json::from_value(serde_json::json!({
        "type": "object",
        "properties": {
            "type": { "type": "string", "enum": ["Person", "Organization"] },
            "name": { "type": "string" },
            "url": { "type": "string", "format": "uri" }
        },
        "required": ["type", "name"]
    }))
    .expect("valid schema")
}

/// Configuration for a site
///
/// Site settings are associated with a workspace (see `WorkspaceConfig`).
/// The workspace ID is used to identify the site in Stencila Cloud.
///
/// Example:
/// ```toml
/// [site]
/// domain = "docs.example.org"
/// root = "docs"
/// exclude = ["**/*.draft.md", "_drafts/**"]
///
/// [site.routes]
/// "/" = "index.md"
/// "/about/" = "README.md"
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct SiteConfig {
    /// Custom domain for the site
    ///
    /// This is a cached value that is kept in sync with Stencila Cloud
    /// when site details are fetched or the domain is modified.
    /// The canonical source is the Stencila Cloud API.
    #[schemars(regex(pattern = r"^([a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z]{2,}$"))]
    pub domain: Option<String>,

    /// Site title
    ///
    /// Used by the Title component and as fallback metadata.
    /// When not specified, the Title component will render empty.
    ///
    /// Example:
    /// ```toml
    /// [site]
    /// title = "My Documentation"
    /// ```
    pub title: Option<String>,

    /// Site author
    ///
    /// Used as the default copyright holder and for site metadata.
    /// Can be a simple string or a full Author object for richer metadata.
    ///
    /// Example (simple):
    /// ```toml
    /// [site]
    /// author = "Acme Inc"
    /// ```
    ///
    /// Example (full):
    /// ```toml
    /// [site.author]
    /// type = "Organization"
    /// name = "Acme Inc"
    /// url = "https://acme.com"
    /// ```
    pub author: Option<AuthorSpec>,

    /// Site logo configuration
    ///
    /// Can be a simple path string or a responsive configuration with
    /// breakpoint and dark mode variants.
    ///
    /// Example (simple):
    /// ```toml
    /// [site]
    /// logo = "logo.svg"
    /// ```
    ///
    /// Example (responsive):
    /// ```toml
    /// [site.logo]
    /// default = "logo.svg"
    /// dark = "logo-dark.svg"
    /// mobile = "logo-mobile.svg"
    /// ```
    pub logo: Option<LogoSpec>,

    /// Icon assignments for nav items
    ///
    /// Applied during nav construction. Icons specified directly on NavItem take precedence.
    ///
    /// **Key formats** (lookup order, most to least specific):
    /// 1. Full route: `"/docs/config/"` - exact match
    /// 2. Without slashes: `"docs/config"` - flexible path matching
    /// 3. Label: `"Features"` - matches nav item labels
    /// 4. Bare segment: `config` - matches last path segment (e.g., both `/docs/config/` and `/api/config/`)
    ///
    /// For unambiguous matching of nested routes, use full routes or paths without leading slash.
    ///
    /// **Icon format**: `"banana"` (default lucide set) or `"lucide:banana"` (explicit library)
    ///
    /// Example:
    /// ```toml
    /// [site.icons]
    /// "/" = "home"
    /// "docs/config" = "bolt"  # Specific: only /docs/config/
    /// docs = "book"           # Matches any route ending in /docs/
    /// "Features" = "sparkles" # Matches by label
    /// ```
    pub icons: Option<HashMap<String, String>>,

    /// Descriptions for navigation items
    ///
    /// Used by nav components (e.g., `nav-menu`) to display descriptions.
    /// Descriptions specified directly on NavItem take precedence.
    ///
    /// **Key formats** (lookup order, most to least specific):
    /// 1. Full route: `"/docs/config/"` - exact match
    /// 2. Without slashes: `"docs/config"` - flexible path matching
    /// 3. Label: `"Features"` - matches nav item labels
    /// 4. Bare segment: `config` - matches last path segment
    ///
    /// For unambiguous matching of nested routes, use full routes or paths without leading slash.
    ///
    /// Example:
    /// ```toml
    /// [site.descriptions]
    /// "docs/getting-started" = "Quick start guide"  # Specific
    /// docs = "Documentation and guides"             # Any /docs/ route
    /// "Features" = "Explore all capabilities"       # By label
    /// ```
    pub descriptions: Option<HashMap<String, String>>,

    /// Social/external links for the site
    ///
    /// Keyed by platform name (github, discord, linkedin, etc.). Values can be
    /// shortcuts (expanded automatically) or full URLs. Used by the `social-links`
    /// component. Icons are automatically determined from the platform key.
    ///
    /// Supported platforms and shortcuts:
    ///
    /// - `bluesky = "handle.bsky.social"` → bsky.app/profile/...
    /// - `discord = "invite"` → discord.gg/invite
    /// - `facebook = "page"` → facebook.com/page
    /// - `github = "org"` or `"org/repo"` → github.com/org or github.com/org/repo
    /// - `gitlab = "org"` or `"org/repo"` → gitlab.com/org or gitlab.com/org/repo
    /// - `instagram = "handle"` → instagram.com/handle
    /// - `linkedin = "in/name"` or `"company/name"` → linkedin.com/...
    /// - `mastodon` → requires full URL (federated)
    /// - `reddit = "r/sub"` or `"u/user"` → reddit.com/...
    /// - `twitch = "channel"` → twitch.tv/channel
    /// - `x = "handle"` or `twitter = "handle"` → x.com/handle
    /// - `youtube = "@channel"` → youtube.com/@channel
    ///
    /// Note: `twitter` and `x` are treated as aliases. Both are accepted,
    /// but `x` takes precedence if both are specified.
    ///
    /// Order is preserved - links appear in the order defined.
    ///
    /// Example:
    /// ```toml
    /// [site.socials]
    /// github = "org/repo"
    /// discord = "invite-code"
    /// linkedin = "company/name"
    /// x = "handle"
    /// mastodon = "https://mastodon.social/@handle"
    /// ```
    pub socials: Option<IndexMap<String, String>>,

    /// Featured/promotional content for nav-menu dropdowns
    ///
    /// Displays promotional content in the dropdown panel of a nav group.
    /// Keyed by the **dropdown's parent group** (not leaf items).
    ///
    /// **Key formats** (lookup order, most to least specific):
    /// 1. Full route: `"/docs/config/"` - exact match
    /// 2. Without slashes: `"docs/config"` - flexible path matching
    /// 3. Label: `"Features"` - matches nav group labels
    /// 4. Bare segment: `config` - matches last path segment
    ///
    /// For unambiguous matching, use full routes or paths without leading slash.
    ///
    /// Example:
    /// ```toml
    /// [site.featured.docs]  # Matches /docs/ dropdown (bare segment)
    /// title = "Quick Start"
    /// description = "Get up and running"
    /// cta = { label = "Start", route = "/docs/getting-started/" }
    /// ```
    pub featured: Option<HashMap<String, FeaturedContent>>,

    /// Site navigation structure
    ///
    /// Defines the hierarchical navigation used by nav-tree and prev-next components.
    /// If not specified, navigation is auto-generated from document routes.
    ///
    /// Example:
    /// ```toml
    /// [site]
    /// nav = [
    ///   "/",
    ///   { label = "Docs", children = [
    ///     "/docs/getting-started/",
    ///     "/docs/configuration/",
    ///   ]},
    ///   "/about/",
    /// ]
    /// ```
    pub nav: Option<Vec<NavItem>>,

    /// Root directory for site content
    ///
    /// Path relative to the config file containing this setting.
    /// When set, only files within this directory will be published
    /// to the site, and routes will be calculated relative to this
    /// directory rather than the workspace root.
    ///
    /// Example: If set to "docs" in /myproject/stencila.toml,
    /// then /myproject/docs/guide.md → /guide/ (not /docs/guide/)
    pub root: Option<ConfigRelativePath>,

    /// Glob patterns for files to exclude when publishing
    ///
    /// Files matching these patterns will be excluded from publishing.
    /// Exclude patterns take precedence over include patterns.
    /// Patterns are relative to `root` (if set) or the workspace root.
    /// Default exclusions (`.git/`, `node_modules/`, etc.) are applied automatically.
    ///
    /// Example: `["**/*.draft.md", "temp/**"]`
    pub exclude: Option<Vec<String>>,

    /// Custom routes for serving content
    ///
    /// Routes map URL paths to files, redirects, or spread configurations.
    /// The key is the URL path (or path template for spreads), and the value can be:
    /// - A simple string for the file path: `"/about/" = "README.md"`
    /// - An object for redirects: `"/old/" = { redirect = "/new/", status = 301 }`
    /// - An object for spreads: `"/{region}/" = { file = "report.smd", arguments = { region = ["north", "south"] } }`
    ///
    /// Example:
    /// ```toml
    /// [site.routes]
    /// "/" = "index.md"
    /// "/about/" = "README.md"
    /// "/old-page/" = { redirect = "/new-page/", status = 301 }
    /// "/{region}/{species}/" = { file = "report.smd", arguments = { region = ["north", "south"], species = ["ABC", "DEF"] } }
    /// ```
    #[schemars(with = "Option<HashMap<String, RouteTarget>>")]
    pub routes: Option<HashMap<String, RouteTarget>>,

    /// Site layout configuration
    ///
    /// Controls the layout structure of site pages including header, sidebars,
    /// footer, and navigation. When not specified, pages are rendered without
    /// the layout wrapper.
    ///
    /// Example:
    /// ```toml
    /// [site.layout]
    /// left-sidebar = true
    /// right-sidebar = true
    /// ```
    pub layout: Option<LayoutConfig>,

    /// Glide configuration for client-side navigation
    ///
    /// When enabled, internal link clicks are intercepted and content
    /// is swapped without full page reloads, using View Transitions API
    /// when available.
    ///
    /// Example:
    /// ```toml
    /// [site.glide]
    /// prefetch = 25
    /// ```
    pub glide: Option<GlideConfig>,

    /// Additional formats to generate alongside HTML
    ///
    /// Controls which format files are generated during site rendering and
    /// which format-specific buttons are displayed. When a format is not
    /// in this list, its corresponding button (e.g., copy-markdown) is hidden.
    ///
    /// Default: `["md"]` (generates page.md files)
    ///
    /// Example:
    /// ```toml
    /// [site]
    /// formats = ["md"]  # Generate page.md files, show copy-markdown button
    /// formats = []      # No additional formats, hide format buttons
    /// ```
    pub formats: Option<Vec<SiteFormat>>,

    /// Site reviews configuration
    ///
    /// Enables readers to submit comments and suggestions on site pages.
    /// Requires `workspace.id` to be configured for cloud enforcement of
    /// public/anon settings.
    ///
    /// Can be a simple boolean or a detailed configuration object.
    ///
    /// Example (simple):
    /// ```toml
    /// [site]
    /// reviews = true   # Enable with defaults
    /// ```
    ///
    /// Example (detailed):
    /// ```toml
    /// [site.reviews]
    /// enabled = true
    /// public = true           # Non-team members can submit
    /// anon = false            # Require GitHub auth
    /// position = "bottom-right"
    /// types = ["comment", "suggestion"]
    /// min-selection = 10
    /// max-selection = 5000
    /// shortcuts = false
    /// include = ["docs/**"]   # Only show on docs pages
    /// exclude = ["api/**"]    # Hide from API reference
    /// ```
    pub reviews: Option<ReviewsSpec>,
}

impl SiteConfig {
    /// Validate the site configuration
    pub fn validate(&self) -> Result<()> {
        if let Some(domain) = &self.domain
            && !DOMAIN_REGEX.is_match(domain)
        {
            bail!(
                "Invalid domain `{domain}`: must be a valid domain name (e.g., 'docs.example.org')"
            );
        }

        if let Some(layout) = &self.layout {
            layout.validate()?;
        }

        // Validate reviews configuration
        if let Some(reviews) = &self.reviews {
            reviews.validate()?;
        }

        // Validate featured content (icon + image conflict)
        if let Some(featured) = &self.featured {
            for (key, content) in featured {
                content.validate(key);
            }
        }

        Ok(())
    }

    /// Validate paths in the site configuration
    ///
    /// Warns about missing image paths in featured content.
    /// Call this after resolving the site root.
    pub fn validate_paths(&self, site_root: &Path) {
        if let Some(featured) = &self.featured {
            for (key, content) in featured {
                content.validate_image_path(key, site_root);
            }
        }
    }

    /// Get the root path
    pub fn resolve_root(&self, base_dir: &Path) -> Option<PathBuf> {
        self.root.as_ref().map(|root| root.resolve(base_dir))
    }

    /// Check if a format is enabled for generation
    ///
    /// Returns true if the format is in the `formats` list.
    /// When `formats` is None, defaults to `["md"]`.
    pub fn is_format_enabled(&self, format: SiteFormat) -> bool {
        self.formats
            .as_ref()
            .map(|f| f.contains(&format))
            .unwrap_or_else(|| {
                // Default: md is enabled
                matches!(format, SiteFormat::Md)
            })
    }
}

/// Target for a route - either a file path, a redirect, or a spread
///
/// Routes can either serve a file, redirect to another URL, or generate
/// multiple variants using spread parameters.
/// This enum allows for a clean representation where simple file
/// paths are strings, and redirects/spreads are objects.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum RouteTarget {
    /// Serve a file at this path
    ///
    /// Path relative to the workspace directory (or `site.root` if configured).
    ///
    /// Example in TOML:
    /// ```toml
    /// [site.routes]
    /// "/about/" = "README.md"
    /// ```
    File(ConfigRelativePath),

    /// Redirect to another URL
    ///
    /// Example in TOML:
    /// ```toml
    /// [site.routes]
    /// "/old/" = { redirect = "/new/", status = 301 }
    /// ```
    Redirect(RouteRedirect),

    /// Spread configuration for multi-variant routes
    ///
    /// Example in TOML:
    /// ```toml
    /// [site.routes]
    /// "/{region}/{species}/" = { file = "report.smd", arguments = { region = ["north", "south"], species = ["A", "B"] } }
    /// ```
    Spread(RouteSpread),
}

impl RouteTarget {
    /// Validate the route target configuration
    ///
    /// Ensures that:
    /// - Route path starts with '/'
    /// - `status` can only be used with `redirect`
    /// - Spread routes have a non-empty file and arguments
    pub fn validate(&self, path: &str) -> Result<()> {
        // All routes must start with '/'
        if !path.starts_with('/') {
            bail!("Route '{path}' must start with '/'");
        }

        match self {
            RouteTarget::File(_) => Ok(()),
            RouteTarget::Redirect(redirect) => {
                if redirect.redirect.is_empty() {
                    bail!("Route '{path}' has an empty redirect URL");
                }
                Ok(())
            }
            RouteTarget::Spread(spread) => {
                if spread.file.is_empty() {
                    bail!("Spread route '{path}' has an empty file");
                }
                if spread.arguments.is_empty() {
                    bail!("Spread route '{path}' has no arguments");
                }
                // Validate that all placeholders have corresponding arguments
                // (except reserved placeholders like {tag} and {branch})
                validate_placeholders(path, Some(&spread.arguments), "Route")?;
                Ok(())
            }
        }
    }

    /// Get the file path if this is a file route
    pub fn file(&self) -> Option<&ConfigRelativePath> {
        match self {
            RouteTarget::File(path) => Some(path),
            RouteTarget::Redirect(_) | RouteTarget::Spread(_) => None,
        }
    }

    /// Get the redirect info if this is a redirect route
    pub fn redirect(&self) -> Option<&RouteRedirect> {
        match self {
            RouteTarget::Redirect(redirect) => Some(redirect),
            RouteTarget::File(_) | RouteTarget::Spread(_) => None,
        }
    }

    /// Get the spread configuration if this is a spread route
    pub fn spread(&self) -> Option<&RouteSpread> {
        match self {
            RouteTarget::Spread(spread) => Some(spread),
            RouteTarget::File(_) | RouteTarget::Redirect(_) => None,
        }
    }

    /// Check if this is a spread route
    pub fn is_spread(&self) -> bool {
        matches!(self, RouteTarget::Spread(_))
    }
}

/// A redirect configuration
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct RouteRedirect {
    /// The URL to redirect to
    ///
    /// Can be an absolute URL or a relative path.
    ///
    /// Examples:
    /// - /new-location/ - Redirect to another path on the same site
    /// - https://example.com - Redirect to an external URL
    pub redirect: String,

    /// HTTP status code for the redirect
    ///
    /// Determines the type of redirect. Common values:
    /// - 301 - Moved Permanently (permanent redirect)
    /// - 302 - Found (temporary redirect, default)
    /// - 303 - See Other
    /// - 307 - Temporary Redirect
    /// - 308 - Permanent Redirect
    ///
    /// If not specified, defaults to 302 (temporary redirect).
    pub status: Option<RedirectStatus>,
}

/// HTTP status code for redirects
#[derive(Debug, Display, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(into = "u16", try_from = "u16")]
#[schemars(schema_with = "redirect_status_schema")]
pub enum RedirectStatus {
    /// 301 Moved Permanently
    MovedPermanently = 301,
    /// 302 Found (temporary redirect)
    Found = 302,
    /// 303 See Other
    SeeOther = 303,
    /// 307 Temporary Redirect
    TemporaryRedirect = 307,
    /// 308 Permanent Redirect
    PermanentRedirect = 308,
}

fn redirect_status_schema(_generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
    serde_json::from_value(serde_json::json!({
        "type": "integer",
        "enum": [301, 302, 303, 307, 308],
        "description": "HTTP redirect status code."
    }))
    .expect("invalid JSON Schema")
}

impl From<RedirectStatus> for u16 {
    fn from(status: RedirectStatus) -> Self {
        status as u16
    }
}

impl TryFrom<u16> for RedirectStatus {
    type Error = String;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            301 => Ok(RedirectStatus::MovedPermanently),
            302 => Ok(RedirectStatus::Found),
            303 => Ok(RedirectStatus::SeeOther),
            307 => Ok(RedirectStatus::TemporaryRedirect),
            308 => Ok(RedirectStatus::PermanentRedirect),
            _ => Err(format!(
                "Invalid redirect status code: {value} (must be 301, 302, 303, 307, or 308)",
            )),
        }
    }
}

/// A spread configuration for multi-variant routes
///
/// Used to generate multiple route variants from a single source file
/// with different parameter values.
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct RouteSpread {
    /// The source file for this spread route
    ///
    /// Path relative to the workspace directory (or `site.root` if configured).
    pub file: String,

    /// Spread mode
    ///
    /// - `grid`: Cartesian product of all parameter values (default)
    /// - `zip`: Positional pairing of values (all params must have same length)
    pub spread: Option<SpreadMode>,

    /// Parameter values for spread variants
    ///
    /// Keys are parameter names, values are arrays of possible values.
    /// Example: `{ region = ["north", "south"], species = ["A", "B"] }`
    pub arguments: HashMap<String, Vec<String>>,
}

/// Configuration for client-side navigation (glide)
///
/// When enabled, internal link clicks are intercepted and content
/// is swapped without full page reloads, using View Transitions API
/// when available.
///
/// Example:
/// ```toml
/// [site.glide]
/// prefetch = 25
/// ```
#[skip_serializing_none]
#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(default)]
pub struct GlideConfig {
    /// Enable client-side navigation
    ///
    /// When true, internal links use AJAX navigation with View Transitions.
    /// Default: true
    pub enabled: Option<bool>,

    /// Maximum prefetches per session
    ///
    /// Pages are fetched on hover/focus before click, up to this limit.
    /// Set to 0 to disable prefetching. Only applies when glide is enabled.
    /// Default: 20
    pub prefetch: Option<usize>,

    /// Maximum number of pages to cache
    ///
    /// Controls how many pages are kept in the LRU cache for instant
    /// back/forward navigation. Set to 0 to disable caching.
    /// Default: 10
    pub cache: Option<usize>,
}

/// Add a route to the [site.routes] section of stencila.toml
///
/// This function:
/// 1. Finds the nearest stencila.toml (or creates one if none exists)
/// 2. Adds the route entry mapping route path to file path
/// 3. Avoids duplicates - does nothing if route already exists
///
/// Returns the path to the modified config file.
pub fn config_add_route(file_path: &Path, route: &str) -> Result<PathBuf> {
    use crate::CONFIG_FILENAME;

    // Canonicalize file_path first to get absolute path
    let file_path = file_path.canonicalize()?;

    // Find the nearest stencila.toml starting from the file's directory
    let search_dir = if file_path.is_file() {
        file_path
            .parent()
            .ok_or_eyre("File has no parent directory")?
    } else {
        file_path.as_path()
    };

    let config_path = find_config_file(search_dir, CONFIG_FILENAME)
        .unwrap_or_else(|| search_dir.join(CONFIG_FILENAME));

    // Canonicalize config_path so we can compute workspace-relative paths
    let config_path = if config_path.exists() {
        config_path.canonicalize().unwrap_or(config_path)
    } else {
        // Config doesn't exist yet - canonicalize parent and rejoin filename
        config_path
            .parent()
            .and_then(|p| p.canonicalize().ok())
            .map(|p| p.join(CONFIG_FILENAME))
            .unwrap_or(config_path)
    };

    // Load existing config or create empty
    let contents = if config_path.exists() {
        fs::read_to_string(&config_path)?
    } else {
        String::new()
    };

    let mut doc = contents
        .parse::<DocumentMut>()
        .unwrap_or_else(|_| DocumentMut::new());

    // Ensure [site] table exists
    if doc.get("site").is_none() {
        doc["site"] = Item::Table(Table::new());
    }

    // Get workspace directory (parent of config file)
    let workspace_dir = config_path
        .parent()
        .ok_or_eyre("Config file has no parent directory")?;

    // Extract site.root value before getting mutable references (to avoid borrow issues)
    let site_root_str = doc
        .get("site")
        .and_then(|s| s.as_table())
        .and_then(|t| t.get("root"))
        .and_then(|r| r.as_str())
        .map(|s| s.to_string());

    // Compute base directory for relative paths
    let base_dir = if let Some(root_str) = &site_root_str {
        let site_root = workspace_dir.join(root_str);
        if let Ok(canonical_root) = site_root.canonicalize() {
            // If file is within site.root, use site.root as base
            if file_path.starts_with(&canonical_root) {
                canonical_root
            } else {
                workspace_dir.to_path_buf()
            }
        } else {
            workspace_dir.to_path_buf()
        }
    } else {
        workspace_dir.to_path_buf()
    };

    // Make file_path relative to base directory (file_path is already canonicalized)
    let file_relative = file_path.strip_prefix(&base_dir).unwrap_or(&file_path);
    let file_relative_str = file_relative.to_string_lossy().replace('\\', "/");

    let site_table = doc
        .get_mut("site")
        .and_then(|v| v.as_table_mut())
        .ok_or_eyre("site field is not a table")?;

    // Ensure [site.routes] table exists
    if site_table.get("routes").is_none() {
        site_table["routes"] = Item::Table(Table::new());
    }

    let routes_table = site_table
        .get_mut("routes")
        .and_then(|v| v.as_table_mut())
        .ok_or_eyre("site.routes field is not a table")?;

    // Check if route already exists
    if let Some(existing) = routes_table.get(route) {
        // Check if it's the same file path
        if let Some(existing_file) = existing.as_str()
            && existing_file == file_relative_str
        {
            // Route already exists with same file, nothing to do
            return Ok(config_path);
        }
        // Route exists but points to different file or is a different type
        // We'll update it to the new file
    }

    // Set the route to the file path
    routes_table[route] = value(&file_relative_str);

    // Write back to file
    fs::write(&config_path, doc.to_string())?;

    Ok(config_path)
}

/// Remove a route from the [site.routes] section of stencila.toml
///
/// This function:
/// 1. Finds the nearest stencila.toml
/// 2. Removes the route entry for the given key
///
/// Returns the path to the modified config file.
pub fn config_remove_route(route: &str) -> Result<PathBuf> {
    use crate::CONFIG_FILENAME;

    let cwd = std::env::current_dir()?;
    let config_path = find_config_file(&cwd, CONFIG_FILENAME)
        .ok_or_else(|| eyre!("No `{CONFIG_FILENAME}` found"))?;

    // Load existing config
    let contents = fs::read_to_string(&config_path)?;
    let mut doc = contents.parse::<DocumentMut>()?;

    // Get the site table
    let site_table = doc
        .get_mut("site")
        .ok_or_else(|| eyre!("No [site] section in `{CONFIG_FILENAME}`"))?
        .as_table_mut()
        .ok_or_else(|| eyre!("site field is not a table"))?;

    // Get the routes table
    let routes_table = site_table
        .get_mut("routes")
        .ok_or_else(|| eyre!("No [site.routes] section in `{CONFIG_FILENAME}`"))?
        .as_table_mut()
        .ok_or_else(|| eyre!("site.routes field is not a table"))?;

    // Remove the key
    routes_table
        .remove(route)
        .ok_or_else(|| eyre!("Route `{route}` not found"))?;

    // Write back to file
    fs::write(&config_path, doc.to_string())?;

    Ok(config_path)
}

/// Set a spread route configuration in the [site.routes] section of stencila.toml
///
/// This function:
/// 1. Finds the nearest stencila.toml (or creates one if none exists)
/// 2. Adds or replaces the spread route entry
///
/// Returns the path to the modified config file.
pub fn config_set_route_spread(route_template: &str, spread: &RouteSpread) -> Result<PathBuf> {
    use crate::CONFIG_FILENAME;

    // Parse spread.file as a path and canonicalize to get absolute path
    let file_path = Path::new(&spread.file).canonicalize()?;

    // Find the nearest stencila.toml starting from the file's directory
    let search_dir = if file_path.is_file() {
        file_path
            .parent()
            .ok_or_eyre("File has no parent directory")?
    } else {
        file_path.as_path()
    };

    let config_path = find_config_file(search_dir, CONFIG_FILENAME)
        .unwrap_or_else(|| search_dir.join(CONFIG_FILENAME));

    // Canonicalize config_path so we can compute workspace-relative paths
    let config_path = if config_path.exists() {
        config_path.canonicalize().unwrap_or(config_path)
    } else {
        // Config doesn't exist yet - canonicalize parent and rejoin filename
        config_path
            .parent()
            .and_then(|p| p.canonicalize().ok())
            .map(|p| p.join(CONFIG_FILENAME))
            .unwrap_or(config_path)
    };

    // Load existing config or create empty
    let contents = if config_path.exists() {
        fs::read_to_string(&config_path)?
    } else {
        String::new()
    };

    let mut doc = contents
        .parse::<DocumentMut>()
        .unwrap_or_else(|_| DocumentMut::new());

    // Ensure [site] table exists
    if doc.get("site").is_none() {
        doc["site"] = Item::Table(Table::new());
    }

    // Get workspace directory (parent of config file)
    let workspace_dir = config_path
        .parent()
        .ok_or_eyre("Config file has no parent directory")?;

    // Extract site.root value before getting mutable references (to avoid borrow issues)
    let site_root_str = doc
        .get("site")
        .and_then(|s| s.as_table())
        .and_then(|t| t.get("root"))
        .and_then(|r| r.as_str())
        .map(|s| s.to_string());

    // Compute base directory for relative paths
    let base_dir = if let Some(root_str) = &site_root_str {
        let site_root = workspace_dir.join(root_str);
        if let Ok(canonical_root) = site_root.canonicalize() {
            // If file is within site.root, use site.root as base
            if file_path.starts_with(&canonical_root) {
                canonical_root
            } else {
                workspace_dir.to_path_buf()
            }
        } else {
            workspace_dir.to_path_buf()
        }
    } else {
        workspace_dir.to_path_buf()
    };

    // Make file_path relative to base directory (file_path is already canonicalized)
    let file_relative = file_path.strip_prefix(&base_dir).unwrap_or(&file_path);
    let file_relative_str = file_relative.to_string_lossy().replace('\\', "/");

    let site_table = doc
        .get_mut("site")
        .and_then(|v| v.as_table_mut())
        .ok_or_eyre("site field is not a table")?;

    // Ensure [site.routes] table exists
    if site_table.get("routes").is_none() {
        site_table["routes"] = Item::Table(Table::new());
    }

    let routes_table = site_table
        .get_mut("routes")
        .and_then(|v| v.as_table_mut())
        .ok_or_eyre("site.routes field is not a table")?;

    // Build the spread config as an inline table
    let mut spread_table = InlineTable::new();
    spread_table.insert("file", file_relative_str.as_str().into());

    if let Some(ref spread_mode) = spread.spread {
        let spread_mode_str = match spread_mode {
            crate::SpreadMode::Grid => "grid",
            crate::SpreadMode::Zip => "zip",
        };
        spread_table.insert("spread", spread_mode_str.into());
    }

    // Build arguments as an inline table
    if !spread.arguments.is_empty() {
        let mut arguments_table = InlineTable::new();
        for (key, values) in &spread.arguments {
            let mut arr = toml_edit::Array::new();
            for v in values {
                arr.push(v.as_str());
            }
            arguments_table.insert(key.as_str(), toml_edit::Value::Array(arr));
        }
        spread_table.insert("arguments", toml_edit::Value::InlineTable(arguments_table));
    }

    // Set the route to the spread config
    routes_table[route_template] = value(spread_table);

    // Write back to file
    fs::write(&config_path, doc.to_string())?;

    Ok(config_path)
}

/// Add a redirect route to [site.routes] in the nearest stencila.toml
pub fn config_add_redirect_route(
    route: &str,
    redirect: &str,
    status: Option<u16>,
) -> Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    let config_path =
        find_config_file(&cwd, CONFIG_FILENAME).unwrap_or_else(|| cwd.join(CONFIG_FILENAME));

    // Load existing config or create empty
    let contents = if config_path.exists() {
        fs::read_to_string(&config_path)?
    } else {
        String::new()
    };

    let mut doc = contents
        .parse::<DocumentMut>()
        .unwrap_or_else(|_| DocumentMut::new());

    // Ensure [site] table exists
    if doc.get("site").is_none() {
        doc["site"] = Item::Table(Table::new());
    }

    let site_table = doc
        .get_mut("site")
        .and_then(|v| v.as_table_mut())
        .ok_or_eyre("site field is not a table")?;

    // Ensure [site.routes] table exists
    if site_table.get("routes").is_none() {
        site_table["routes"] = Item::Table(Table::new());
    }

    let routes_table = site_table
        .get_mut("routes")
        .and_then(|v| v.as_table_mut())
        .ok_or_eyre("site.routes field is not a table")?;

    // Build redirect config as an inline table
    let mut redirect_table = InlineTable::new();
    redirect_table.insert("redirect", redirect.into());
    if let Some(status) = status {
        redirect_table.insert("status", (status as i64).into());
    }

    routes_table[route] = value(redirect_table);

    // Write back to file
    fs::write(&config_path, doc.to_string())?;

    Ok(config_path)
}
