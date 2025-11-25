use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use eyre::{Result, eyre};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum::Display;
use url::Url;

/// Main configuration file name
pub const CONFIG_FILENAME: &str = "stencila.toml";

/// Local configuration file name (for local overrides, typically gitignored)
pub const CONFIG_LOCAL_FILENAME: &str = "stencila.local.toml";

mod utils;
use utils::build_figment;
pub use utils::{
    ConfigTarget, config_add_remote, config_add_route, config_set, config_set_remote_spread,
    config_set_route_spread, config_unset, config_update_remote_watch, config_update_site_watch,
    config_value, find_config_file,
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

    // Validate all remote configurations
    if let Some(remotes) = &config.remotes {
        for (path_key, value) in remotes {
            value.validate(path_key)?;
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
    /// Site configuration
    pub site: Option<SiteConfig>,

    /// Custom routes for serving content
    ///
    /// Routes map URL paths to files, redirects, or spread configurations.
    /// The key is the URL path (or path template for spreads), and the value can be:
    /// - A simple string for the file path: `"/about/" = "README.md"`
    /// - An object for redirects: `"/old/" = { redirect = "/new/", status = 301 }`
    /// - An object for spreads: `"/{region}/" = { file = "report.smd", arguments = { region = ["north", "south"] } }`
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
    /// "/{region}/{species}/" = { file = "report.smd", arguments = { region = ["north", "south"], species = ["ABC", "DEF"] } }
    /// ```
    #[schemars(with = "Option<HashMap<String, RouteTarget>>")]
    pub routes: Option<HashMap<String, RouteTarget>>,

    /// Remote synchronization configuration
    ///
    /// Maps local paths to remote service URLs. The key is the local path
    /// (file, directory, or pattern), and the value can be:
    /// - A simple URL string: `"site" = "https://example.stencila.site/"`
    /// - An object with watch: `"file.md" = { url = "...", watch = "w123" }`
    /// - Multiple remotes: `"file.md" = [{ url = "...", watch = "..." }, "https://..."]`
    ///
    /// Directory paths are implicitly recursive, matching all files within.
    ///
    /// Example:
    /// ```toml
    /// [remotes]
    /// "site" = "https://example.stencila.site/"
    /// "docs/report.md" = { url = "https://docs.google.com/...", watch = "w123" }
    /// "article.md" = [
    ///   { url = "https://docs.google.com/...", watch = "w456" },
    ///   "https://sharepoint.com/..."
    /// ]
    /// ```
    #[schemars(with = "Option<HashMap<String, RemoteValue>>")]
    pub remotes: Option<HashMap<String, RemoteValue>>,
}

impl Config {
    /// Check if a path is exactly the site root directory
    ///
    /// Unlike `path_is_in_site_root`, this returns `true` only if the path
    /// is exactly the configured `site.root`, not a subdirectory.
    ///
    /// Returns `false` if:
    /// - `site` is not configured
    /// - `site.root` is not configured
    /// - The path is not exactly the site root
    pub fn path_is_site_root(&self, path: &Path, workspace_dir: &Path) -> bool {
        if let Some(site_config) = &self.site
            && let Some(site_root) = &site_config.root
        {
            let site_root_path = site_root.resolve(workspace_dir);

            // Normalize both paths for comparison
            let path_canonical = path.canonicalize().ok();
            let site_root_canonical = site_root_path.canonicalize().ok();

            if let (Some(path_canon), Some(site_canon)) = (path_canonical, site_root_canonical) {
                return path_canon == site_canon;
            }
        }
        false
    }

    /// Check if a path is under the configured site root
    ///
    /// Returns true if the path is within (or is) the directory
    /// specified by `site.root` in the configuration.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to check (can be relative or absolute)
    /// * `workspace_dir` - The workspace directory used to resolve relative paths
    ///
    /// # Returns
    ///
    /// Returns `true` if:
    /// - `site.root` is configured AND
    /// - The path is under (or is) the site root directory
    ///
    /// Returns `false` if:
    /// - `site` is not configured
    /// - `site.root` is not configured
    /// - The path is not under the site root
    pub fn path_is_in_site_root(&self, path: &Path, workspace_dir: &Path) -> bool {
        if let Some(site_config) = &self.site
            && let Some(site_root) = &site_config.root
        {
            let site_root_path = site_root.resolve(workspace_dir);

            // Normalize both paths for comparison
            let path_canonical = path.canonicalize().ok();
            let site_root_canonical = site_root_path.canonicalize().ok();

            if let (Some(path_canon), Some(site_canon)) = (path_canonical, site_root_canonical) {
                // Check if path is under site_root or is the site_root itself
                return path_canon.starts_with(&site_canon) || path_canon == site_canon;
            }
        }
        false
    }
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

    /// Watch ID from Stencila Cloud
    ///
    /// If watching is enabled for this site, this field contains the watch ID.
    /// The watch enables unidirectional sync from repository to site - when
    /// changes are pushed to the repository, the site is automatically updated.
    #[schemars(regex(pattern = r"^\d+$"))]
    pub watch: Option<String>,

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

    /// Spread configuration for multi-variant routes
    ///
    /// Example in TOML:
    /// ```toml
    /// [routes]
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
            return Err(eyre!("Route '{}' must start with '/'", path));
        }

        match self {
            RouteTarget::File(_) => Ok(()),
            RouteTarget::Redirect(redirect) => {
                if redirect.redirect.is_empty() {
                    return Err(eyre!("Route '{}' has an empty redirect URL", path));
                }
                Ok(())
            }
            RouteTarget::Spread(spread) => {
                if spread.file.is_empty() {
                    return Err(eyre!("Spread route '{}' has an empty file", path));
                }
                if spread.arguments.is_empty() {
                    return Err(eyre!("Spread route '{}' has no arguments", path));
                }
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

/// Value for a remote configuration entry - can be single or multiple targets
///
/// Supports both simple cases (one URL) and complex cases (multiple URLs per path).
/// Each target can be a simple URL string or an object with a watch ID.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum RemoteValue {
    /// Multiple remote targets for the same path
    ///
    /// Example in TOML:
    /// ```toml
    /// [remotes]
    /// "article.md" = [
    ///   { url = "https://docs.google.com/...", watch = "w456" },
    ///   "https://sharepoint.com/..."
    /// ]
    /// ```
    Multiple(Vec<RemoteTarget>), // Keep first for correct deserialization of multiple remotes

    /// Single remote target
    ///
    /// Example in TOML:
    /// ```toml
    /// [remotes]
    /// "site" = "https://example.stencila.site/"
    /// "file.md" = { url = "https://...", watch = "w123" }
    /// ```
    Single(RemoteTarget),
}

impl RemoteValue {
    /// Convert to a vector of targets, flattening single or multiple variants
    pub fn to_vec(&self) -> Vec<&RemoteTarget> {
        match self {
            RemoteValue::Single(target) => vec![target],
            RemoteValue::Multiple(targets) => targets.iter().collect(),
        }
    }

    /// Find the watch ID for a specific URL, if it exists
    pub fn find_watch(&self, url: &str) -> Option<&str> {
        for target in self.to_vec() {
            if target.url() == Some(url) {
                return target.watch();
            }
        }
        None
    }

    /// Validate the remote value configuration
    ///
    /// Ensures that:
    /// - Each target is valid
    /// - Multiple targets array is not empty
    pub fn validate(&self, path: &str) -> Result<()> {
        match self {
            RemoteValue::Single(target) => target.validate(path)?,
            RemoteValue::Multiple(targets) => {
                if targets.is_empty() {
                    return Err(eyre!(
                        "Remote for path '{}' has an empty array of targets",
                        path
                    ));
                }
                for target in targets {
                    target.validate(path)?;
                }
            }
        }
        Ok(())
    }
}

/// A remote synchronization target
///
/// Can be:
/// - A simple URL string (for remotes without watch IDs)
/// - An object with URL and watch ID
/// - A spread configuration for multi-variant pushes
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum RemoteTarget {
    /// Simple URL string (no watch)
    ///
    /// Example: `"https://example.stencila.site/"`
    Url(Url),

    /// URL with watch information
    ///
    /// Example: `{ url = "https://...", watch = "w123" }`
    Watch(RemoteWatch),

    /// Spread configuration for multi-variant pushes
    ///
    /// Example: `{ service = "gdoc", title = "Report {region}", params = { region = ["north", "south"] } }`
    Spread(RemoteSpread),
}

impl RemoteTarget {
    /// Get the URL from this target as a string slice (if it has one)
    ///
    /// Returns None for Spread targets which have a service instead of a URL.
    pub fn url(&self) -> Option<&str> {
        match self {
            RemoteTarget::Url(url) => Some(url.as_str()),
            RemoteTarget::Watch(watch) => Some(watch.url.as_str()),
            RemoteTarget::Spread(_) => None,
        }
    }

    /// Get the URL from this target as an owned Url (if it has one)
    ///
    /// Returns None for Spread targets which have a service instead of a URL.
    pub fn url_owned(&self) -> Option<Url> {
        match self {
            RemoteTarget::Url(url) => Some(url.clone()),
            RemoteTarget::Watch(watch) => Some(watch.url.clone()),
            RemoteTarget::Spread(_) => None,
        }
    }

    /// Get the watch ID if this target has one
    pub fn watch(&self) -> Option<&str> {
        match self {
            RemoteTarget::Url(_) | RemoteTarget::Spread(_) => None,
            RemoteTarget::Watch(watch) => watch.watch.as_deref(),
        }
    }

    /// Get the spread configuration if this is a spread target
    pub fn spread(&self) -> Option<&RemoteSpread> {
        match self {
            RemoteTarget::Spread(spread) => Some(spread),
            _ => None,
        }
    }

    /// Check if this is a spread target
    pub fn is_spread(&self) -> bool {
        matches!(self, RemoteTarget::Spread(_))
    }

    /// Validate the remote value configuration
    ///
    /// Ensures that:
    /// - URL targets have non-empty URLs
    /// - Spread targets have a non-empty service
    /// - Multiple targets array is not empty
    pub fn validate(&self, path: &str) -> Result<()> {
        match self {
            RemoteTarget::Url(url) | RemoteTarget::Watch(RemoteWatch { url, .. }) => {
                if url.as_str().is_empty() {
                    return Err(eyre!("Remote for path `{path}` has an empty URL"));
                }
            }
            RemoteTarget::Spread(spread) => {
                if spread.service.is_empty() {
                    return Err(eyre!(
                        "Spread remote for path `{path}` has an empty service"
                    ));
                }
                if spread.arguments.is_empty() {
                    return Err(eyre!("Spread remote for path `{path}` has no `params`"));
                }
            }
        }

        Ok(())
    }
}

/// Remote synchronization information with watch ID
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct RemoteWatch {
    /// Remote URL
    ///
    /// The service type is inferred from the URL host:
    /// - Google Docs: https://docs.google.com/document/d/...
    /// - Microsoft 365: https://*.sharepoint.com/...
    /// - Stencila Sites: https://*.stencila.site/...
    pub url: Url,

    /// Watch ID from Stencila Cloud
    ///
    /// If this remote is being watched for automatic synchronization, this
    /// field contains the watch ID. Watch configuration (direction, PR mode,
    /// debounce) is stored in Stencila Cloud and queried via the API.
    #[schemars(regex(pattern = r"^w[a-zA-Z0-9]{9}$"))]
    pub watch: Option<String>,
}

/// Spread configuration for multi-variant pushes
///
/// Used in `[remotes]` to configure spread pushing of a document to multiple
/// remote variants with different parameter values.
///
/// Example:
/// ```toml
/// [remotes]
/// "report.smd" = { service = "gdoc", title = "Report {region}", arguments = { region = ["north", "south"] } }
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct RemoteSpread {
    /// Target service
    ///
    /// One of: "gdoc", "m365"
    pub service: String,

    /// Title template with placeholders
    ///
    /// Placeholders like `{param}` are replaced with parameter values.
    /// Example: "Report - {region}"
    pub title: Option<String>,

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

/// Spread mode for multi-variant execution
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SpreadMode {
    /// Cartesian product of all parameter values (default)
    #[default]
    Grid,
    /// Positional pairing of values (all params must have same length)
    Zip,
}
