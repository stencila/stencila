use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use clap::ValueEnum;
use eyre::{Result, eyre};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum::Display;
use url::Url;

mod outputs;
pub use outputs::*;

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

/// Main configuration file name
pub const CONFIG_FILENAME: &str = "stencila.toml";

/// Local configuration file name (for local overrides, typically gitignored)
pub const CONFIG_LOCAL_FILENAME: &str = "stencila.local.toml";

/// Reserved placeholders that are auto-bound from git refs
const RESERVED_PLACEHOLDERS: &[&str] = &["tag", "branch"];

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

    // Validate all output configurations
    if let Some(outputs) = &config.outputs {
        for (path_key, target) in outputs {
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
        key: "workspace.id",
        command: "stencila site create",
        reason: "Workspace IDs are automatically assigned when creating a workspace and must be registered with Stencila Cloud.",
    },
    ManagedConfigKey {
        key: "site.domain",
        command: "stencila site domain set <domain>",
        reason: "Custom domains require DNS validation, SSL provisioning, and synchronization with Stencila Cloud.",
    },
];

/// Configuration for a Stencila Cloud workspace
///
/// Workspaces are the primary entity in Stencila Cloud, representing a
/// GitHub repository. Sites and watches are scoped under workspaces.
///
/// Example:
/// ```toml
/// [workspace]
/// id = "ws3x9k2m7fab"
/// ```
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct WorkspaceConfig {
    /// The workspace public ID from Stencila Cloud
    ///
    /// This is automatically assigned when a workspace is created via
    /// `stencila site create` or when pushing to a site for the first time.
    /// The workspace ID is derived from the GitHub repository URL.
    #[schemars(regex(pattern = r"^ws[a-z0-9]{10}$"))]
    pub id: Option<String>,
}

/// Stencila configuration
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct Config {
    /// Workspace configuration
    ///
    /// Workspaces are the primary entity in Stencila Cloud, representing a
    /// GitHub repository. Sites and watches are scoped under workspaces.
    pub workspace: Option<WorkspaceConfig>,

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

    /// Outputs configuration
    ///
    /// Defines files to be rendered/converted and uploaded to Stencila Cloud
    /// workspace outputs. The key is the output path template, and the value can be:
    /// - A simple source path: `"report.pdf" = "report.md"`
    /// - A configuration object: `"report.pdf" = { source = "report.md", command = "render" }`
    /// - A static file: `"data.csv" = {}` (copies file as-is)
    /// - A pattern: `"exports/*.csv" = { pattern = "exports/*.csv" }`
    /// - A spread: `"{region}/report.pdf" = { source = "report.md", arguments = { region = ["north", "south"] } }`
    ///
    /// Example:
    /// ```toml
    /// [outputs]
    /// "report.pdf" = "report.md"
    /// "data/results.csv" = {}
    /// "{region}/report.pdf" = { source = "report.md", arguments = { region = ["north", "south"] } }
    /// ```
    #[schemars(with = "Option<HashMap<String, OutputTarget>>")]
    pub outputs: Option<HashMap<String, OutputTarget>>,
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
///
/// Site settings are associated with a workspace (see `WorkspaceConfig`).
/// The workspace ID is used to identify the site in Stencila Cloud.
///
/// Example:
/// ```toml
/// [site]
/// watch = "wAbCdEfGh1"
/// domain = "docs.example.org"
/// root = "docs"
/// exclude = ["**/*.draft.md", "_drafts/**"]
/// ```
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct SiteConfig {
    /// Watch ID from Stencila Cloud
    ///
    /// If watching is enabled for this site, this field contains the watch ID.
    /// The watch enables unidirectional sync from repository to site - when
    /// changes are pushed to the repository, the site is automatically updated.
    #[schemars(regex(pattern = r"^wa[a-z0-9]{10}$"))]
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
    /// Example: `{ service = "gdoc", title = "Report {region}", arguments = { region = ["north", "south"] } }`
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
    #[schemars(regex(pattern = r"^wa[a-z0-9]{10}$"))]
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
    /// Placeholders like `{param}` are replaced with arguments.
    /// Example: "Report - {region}"
    pub title: Option<String>,

    /// Spread mode
    ///
    /// - `grid`: Cartesian product of all arguments (default)
    /// - `zip`: Positional pairing of values (all params must have same length)
    pub spread: Option<SpreadMode>,

    /// Arguments for spread variants
    ///
    /// Keys are parameter names, values are arrays of possible values.
    /// Example: `{ region = ["north", "south"], species = ["A", "B"] }`
    pub arguments: HashMap<String, Vec<String>>,
}

/// Spread mode for multi-variant execution
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum SpreadMode {
    /// Cartesian product of all arguments (default)
    #[default]
    Grid,
    /// Positional pairing of values (all params must have same length)
    Zip,
}

/// Processing command for outputs
///
/// Determines how source files are processed before upload:
/// - `render`: Execute code, apply parameters, then convert to output format
/// - `convert`: Pure format transformation (no code execution)
/// - `none`: Copy file as-is (static upload)
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize, JsonSchema, Display, ValueEnum,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum OutputCommand {
    /// Execute code and convert to output format (default for different extensions)
    #[default]
    Render,

    /// Format transformation only, no code execution
    Convert,

    /// Copy file as-is (default for same extensions)
    None,
}

/// Target for an output - either a simple source path or a full configuration
///
/// Outputs define files to be rendered/converted and uploaded to Stencila Cloud
/// workspace outputs. The key is the output path template.
///
/// Example in TOML:
/// ```toml
/// [outputs]
/// # Simple: source path (rendered if extension differs)
/// "report.pdf" = "report.md"
///
/// # Full config with options
/// "report.docx" = { source = "report.md", command = "render" }
///
/// # Static file (omit source = use key as source)
/// "data/results.csv" = {}
///
/// # Pattern for multiple files
/// "exports/*.csv" = { pattern = "exports/*.csv" }
///
/// # Spread with parameters
/// "{region}/report.pdf" = { source = "report.md", arguments = { region = ["north", "south"] } }
/// ```
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum OutputTarget {
    /// Simple source path (rendered if extension differs from key)
    ///
    /// Example: `"report.pdf" = "report.md"`
    Source(ConfigRelativePath),

    /// Full configuration object
    ///
    /// Example: `"report.pdf" = { source = "report.md", command = "render" }`
    /// Example: `"data.csv" = {}` (static, source = output path)
    Config(OutputConfig),
}

impl OutputTarget {
    /// Validate the output target configuration
    ///
    /// Ensures that:
    /// - `arguments` and `spread` are only allowed with `command = render`
    /// - If `arguments` is present, it must be non-empty
    /// - `source` and `pattern` cannot both be set
    /// - If `refs` is present, it must be non-empty
    /// - Pattern keys must include `*.ext` suffix
    pub fn validate(&self, key: &str) -> Result<()> {
        match self {
            OutputTarget::Source(_) => Ok(()),
            OutputTarget::Config(config) => config.validate(key),
        }
    }

    /// Get the source path if this is a simple source target
    pub fn source(&self) -> Option<&ConfigRelativePath> {
        match self {
            OutputTarget::Source(path) => Some(path),
            OutputTarget::Config(_) => None,
        }
    }

    /// Get the configuration if this is a config target
    pub fn config(&self) -> Option<&OutputConfig> {
        match self {
            OutputTarget::Config(config) => Some(config),
            OutputTarget::Source(_) => None,
        }
    }

    /// Check if this is a spread output (has arguments)
    pub fn is_spread(&self) -> bool {
        match self {
            OutputTarget::Source(_) => false,
            OutputTarget::Config(config) => config.arguments.is_some(),
        }
    }

    /// Check if this is a pattern output
    pub fn is_pattern(&self) -> bool {
        match self {
            OutputTarget::Source(_) => false,
            OutputTarget::Config(config) => config.pattern.is_some(),
        }
    }
}

/// Full output configuration
///
/// Provides detailed control over how an output is processed and uploaded.
///
/// Example:
/// ```toml
/// [outputs]
/// "report.pdf" = { source = "report.md", command = "render" }
/// "{region}/report.pdf" = { source = "report.md", arguments = { region = ["north", "south"] } }
/// "exports/*.csv" = { pattern = "exports/*.csv", exclude = ["temp-*.csv"] }
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct OutputConfig {
    /// Source file path (for single-file outputs)
    ///
    /// Path relative to the config file. If not specified and `pattern` is not set,
    /// the output key is used as the source path.
    pub source: Option<String>,

    /// Glob pattern for matching multiple source files
    ///
    /// Mutually exclusive with `source`. The output key must include `*.ext`
    /// to specify the output format (e.g., `"reports/*.pdf"`).
    pub pattern: Option<String>,

    /// Processing command
    ///
    /// - `render`: Execute code, apply parameters, convert to output format (default if extensions differ)
    /// - `convert`: Format transformation only, no code execution
    /// - `none`: Copy file as-is (default if extensions are the same)
    pub command: Option<OutputCommand>,

    /// Spread mode for parameter variants
    ///
    /// Only valid with `command = render`.
    /// - `grid`: Cartesian product of all arguments (default)
    /// - `zip`: Positional pairing of values
    pub spread: Option<SpreadMode>,

    /// Parameter values for spread variants
    ///
    /// Only valid with `command = render`. Keys are parameter names,
    /// values are arrays of possible values.
    ///
    /// Example: `{ region = ["north", "south"], species = ["A", "B"] }`
    pub arguments: Option<HashMap<String, Vec<String>>>,

    /// Git ref patterns to filter when this output is processed and uploaded
    ///
    /// If set, the output is only processed when the current git ref matches
    /// one of these patterns. Supports glob matching.
    ///
    /// Examples: `["main"]`, `["release/*"]`, `["v*"]`
    pub refs: Option<Vec<String>>,

    /// Glob patterns to exclude from pattern matches
    ///
    /// Paths are relative to the repository root.
    /// Only applies when `pattern` is set.
    ///
    /// Example: `["temp-*.csv", "draft-*"]`
    pub exclude: Option<Vec<String>>,
}

impl OutputConfig {
    /// Validate the output configuration
    pub fn validate(&self, key: &str) -> Result<()> {
        // Source and pattern are mutually exclusive
        if self.source.is_some() && self.pattern.is_some() {
            return Err(eyre!(
                "Output '{}' cannot have both `source` and `pattern`",
                key
            ));
        }

        // If pattern is set, key must include *.ext suffix
        if self.pattern.is_some() && !key.contains("*.") {
            return Err(eyre!(
                "Output '{}' with `pattern` must include `*.ext` suffix to specify output format (e.g., 'reports/*.pdf')",
                key
            ));
        }

        // Arguments and spread are only allowed with command = render
        let command = self.command.unwrap_or_default();
        if command != OutputCommand::Render {
            if self.arguments.is_some() {
                return Err(eyre!(
                    "Output '{}' has `arguments` but `command` is not `render`",
                    key
                ));
            }
            if self.spread.is_some() {
                return Err(eyre!(
                    "Output '{}' has `spread` but `command` is not `render`",
                    key
                ));
            }
        }

        // If arguments is present, it must be non-empty
        if let Some(args) = &self.arguments
            && args.is_empty()
        {
            return Err(eyre!("Output '{}' has empty `arguments`", key));
        }

        // If refs is present, it must be non-empty
        if let Some(refs) = &self.refs
            && refs.is_empty()
        {
            return Err(eyre!("Output '{}' has empty `refs`", key));
        }

        // Exclude only applies with pattern
        if self.exclude.is_some() && self.pattern.is_none() {
            return Err(eyre!("Output '{}' has `exclude` but no `pattern`", key));
        }

        // Validate that all placeholders have corresponding arguments
        // (except reserved placeholders like {tag} and {branch})
        validate_placeholders(key, self.arguments.as_ref(), "Output")?;

        Ok(())
    }
}

/// Extract placeholder names from a template string
///
/// Finds all `{name}` patterns and returns the names.
/// Example: `"{region}/{species}/report.pdf"` returns `["region", "species"]`
fn extract_placeholders(template: &str) -> Vec<&str> {
    let mut placeholders = Vec::new();
    let mut chars = template.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' {
            let mut name = String::new();
            for c in chars.by_ref() {
                if c == '}' {
                    break;
                }
                name.push(c);
            }
            if !name.is_empty() {
                // Find the slice in the original string
                if let Some(start) = template.find(&format!("{{{name}}}")) {
                    let name_start = start + 1;
                    let name_end = name_start + name.len();
                    placeholders.push(&template[name_start..name_end]);
                }
            }
        }
    }

    placeholders
}

/// Validate that all placeholders in a template have corresponding arguments
///
/// Reserved placeholders (`{tag}`, `{branch}`) are auto-bound from git refs
/// and don't require arguments.
///
/// Returns an error if any non-reserved placeholder is missing from arguments.
pub fn validate_placeholders(
    template: &str,
    arguments: Option<&HashMap<String, Vec<String>>>,
    context: &str,
) -> Result<()> {
    let placeholders = extract_placeholders(template);

    for placeholder in placeholders {
        // Skip reserved placeholders
        if RESERVED_PLACEHOLDERS.contains(&placeholder) {
            continue;
        }

        // Check if placeholder has a corresponding argument
        let has_argument = arguments
            .map(|args| args.contains_key(placeholder))
            .unwrap_or(false);

        if !has_argument {
            return Err(eyre!(
                "{} '{}' has placeholder '{{{}}}' but no matching argument",
                context,
                template,
                placeholder
            ));
        }
    }

    Ok(())
}
