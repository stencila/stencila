use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use eyre::{Result, eyre};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum::Display;

/// Main configuration file name
pub const CONFIG_FILENAME: &str = "stencila.toml";

/// Local configuration file name (for local overrides, typically gitignored)
pub const CONFIG_LOCAL_FILENAME: &str = "stencila.local.toml";

mod utils;
use utils::build_figment;
pub use utils::{
    ConfigTarget, config_set, config_unset, config_update_remote_watch, config_value,
    find_config_file,
};

pub mod cli;

#[cfg(test)]
mod tests;

/// A path that is resolved relative to the configuration file it was defined in
///
/// This wrapper provides JsonSchema implementation and stores paths as strings.
/// The paths will be resolved relative to the config file directory during use.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[schemars(transparent)]
#[serde(transparent)]
pub struct ConfigRelativePath(pub String);

impl ConfigRelativePath {
    /// Get the path string as originally specified in the config
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Resolve the path relative to a base directory
    pub fn resolve(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(&self.0)
    }
}

/// Resolve a configuration at a path
///
/// Searches up the directory tree from the path, and then finally in
/// `~/.config/stencila`, looking for `stencila.toml` and `stencila.local.toml`
/// files and merges them into a path-specific config.
///
/// # Precedence (highest to lowest)
///
/// 1. Current directory: `stencila.local.toml` → `stencila.toml`
/// 2. Parent directories: `../stencila.local.toml` → `../stencila.toml` (and so on)
/// 3. User config: `~/.config/stencila/stencila.toml`
///
/// # Error Handling
///
/// Missing config files are silently ignored. Malformed config files are logged
/// as warnings and skipped. Only returns an error if no valid config could be
/// constructed or if path normalization fails.
pub fn config(path: &Path) -> Result<Config> {
    let figment = build_figment(path, true)?;
    let config: Config = figment.extract().map_err(|error| eyre!("{error}"))?;

    // Validate all route configurations
    if let Some(routes) = &config.routes {
        for (path_key, target) in routes {
            target.validate(path_key)?;
        }
    }

    Ok(config)
}

/// A configuration key that is managed by a specific command
///
/// These keys should not be set directly via `stencila config set` because
/// they require special validation, API calls, or side effects that are
/// handled by dedicated commands.
pub(crate) struct ManagedConfigKey {
    /// The config key pattern (e.g., "site.id", "site.domain")
    pub key: &'static str,

    /// The command to use instead (e.g., "stencila site create")
    pub command: &'static str,

    /// Explanation of why this should use the dedicated command
    pub reason: &'static str,
}

/// Registry of configuration keys that should be managed through specific commands
static MANAGED_CONFIG_KEYS: &[ManagedConfigKey] = &[
    ManagedConfigKey {
        key: "site.id",
        command: "stencila site create",
        reason: "Site IDs are automatically assigned when creating a site and must be registered with Stencila Cloud.",
    },
    ManagedConfigKey {
        key: "site.domain",
        command: "stencila site domain set <domain>",
        reason: "Custom domains require DNS validation, SSL provisioning, and synchronization with Stencila Cloud.",
    },
];

/// Stencila configuration
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct Config {
    /// Remote synchronization configuration
    ///
    /// Defines mappings between local files/directories and remote services
    /// (Google Docs, Microsoft 365, Stencila Sites). Directory paths are
    /// implicitly recursive, that is, they match all files within that directory.
    pub remotes: Option<Vec<RemoteConfig>>,

    /// Site configuration
    pub site: Option<SiteConfig>,

    /// Custom routes for serving content
    ///
    /// Routes map URL paths to files or redirects. The key is the URL path,
    /// and the value can be either:
    /// - A simple string for the file path: `"/about/" = "README.md"`
    /// - An object for redirects: `"/old/" = { redirect = "/new/", status = 301 }`
    ///
    /// Routes can be used by both remote sites (e.g., stencila.site) and
    /// local development servers.
    ///
    /// Example:
    /// ```toml
    /// [routes]
    /// "/" = "index.md"
    /// "/about/" = "README.md"
    /// "/old-page/" = { redirect = "/new-page/", status = 301 }
    /// "/external/" = { redirect = "https://example.com" }
    /// ```
    #[schemars(with = "Option<HashMap<String, RouteTarget>>")]
    pub routes: Option<HashMap<String, RouteTarget>>,
}

/// Configuration for a site
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct SiteConfig {
    /// The id of the Stencila Site
    ///
    /// Returned by Stencila Cloud when a site is created.
    #[schemars(regex(pattern = r"^s[a-z0-9]{9}$"))]
    pub id: Option<String>,

    /// Custom domain for the site
    ///
    /// This is a cached value that is kept in sync with Stencila Cloud
    /// when site details are fetched or the domain is modified.
    /// The canonical source is the Stencila Cloud API.
    #[schemars(regex(pattern = r"^([a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z]{2,}$"))]
    pub domain: Option<String>,

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

    /// Glob patterns for files to include when publishing
    ///
    /// When specified, only files matching these patterns will be included.
    /// Patterns are relative to `root` (if set) or the workspace root.
    /// Supports standard glob syntax: `**/*.md`, `assets/**`, etc.
    ///
    /// Example: `["**/*.md", "**/*.html", "assets/**"]`
    pub include: Option<Vec<String>>,

    /// Glob patterns for files to exclude when publishing
    ///
    /// Files matching these patterns will be excluded from publishing.
    /// Exclude patterns take precedence over include patterns.
    /// Patterns are relative to `root` (if set) or the workspace root.
    /// Default exclusions (`.git/`, `node_modules/`, etc.) are applied automatically.
    ///
    /// Example: `["**/*.draft.md", "temp/**"]`
    pub exclude: Option<Vec<String>>,
}

/// Target for a route - either a file path or a redirect
///
/// Routes can either serve a file or redirect to another URL.
/// This enum allows for a clean representation where simple file
/// paths are strings, and redirects are objects.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum RouteTarget {
    /// Serve a file at this path
    ///
    /// Path relative to the workspace directory (or `site.root` if configured).
    ///
    /// Example in TOML:
    /// ```toml
    /// [routes]
    /// "/about/" = "README.md"
    /// ```
    File(ConfigRelativePath),

    /// Redirect to another URL
    ///
    /// Example in TOML:
    /// ```toml
    /// [routes]
    /// "/old/" = { redirect = "/new/", status = 301 }
    /// ```
    Redirect(RouteRedirect),
}

impl RouteTarget {
    /// Validate the route target configuration
    ///
    /// Ensures that:
    /// - `status` can only be used with `redirect`
    pub fn validate(&self, path: &str) -> Result<()> {
        match self {
            RouteTarget::File(_) => Ok(()),
            RouteTarget::Redirect(redirect) => {
                if redirect.redirect.is_empty() {
                    return Err(eyre!("Route '{}' has an empty redirect URL", path));
                }
                Ok(())
            }
        }
    }

    /// Get the file path if this is a file route
    pub fn file(&self) -> Option<&ConfigRelativePath> {
        match self {
            RouteTarget::File(path) => Some(path),
            RouteTarget::Redirect(_) => None,
        }
    }

    /// Get the redirect info if this is a redirect route
    pub fn redirect(&self) -> Option<&RouteRedirect> {
        match self {
            RouteTarget::File(_) => None,
            RouteTarget::Redirect(redirect) => Some(redirect),
        }
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

/// Configuration for a remote synchronization target
///
/// Remotes define how local files or directories map to external services
/// like Google Docs, Microsoft 365, or Stencila Sites. Each remote specifies
/// a path (file, directory, or pattern) and a URL to synchronize with.
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct RemoteConfig {
    /// Path relative to workspace root
    ///
    /// Can be:
    /// - Single file: "file.md", "report.ipynb"
    /// - Directory: "site", "docs" (implicitly includes all files recursively)
    /// - Pattern: "*.md" (for matching specific files, optional)
    ///
    /// Directory paths are automatically treated as recursive - they match
    /// all files within that directory and its subdirectories.
    pub path: ConfigRelativePath,

    /// Remote URL
    ///
    /// The service type is inferred from the URL host:
    /// - Google Docs: https://docs.google.com/document/d/...
    /// - Microsoft 365: https://*.sharepoint.com/...
    /// - Stencila Sites: https://*.stencila.site/...
    #[schemars(regex(pattern = r"^https?://"))]
    pub url: String,

    /// Watch ID from Stencila Cloud
    ///
    /// If this remote is being watched for automatic synchronization, this
    /// field contains the watch ID. Watch configuration (direction, PR mode,
    /// debounce) is stored in Stencila Cloud and queried via the API.
    ///
    /// If no watch exists, this field is omitted.
    #[schemars(regex(pattern = r"^w[a-zA-Z0-9]{9}$"))]
    pub watch: Option<String>,
}
