use std::path::Path;

use eyre::{Result, eyre};
use serde::{Deserialize, Serialize};

mod utils;
use serde_with::skip_serializing_none;
use utils::build_figment;
pub use utils::{ConfigTarget, config_set, config_unset, config_value, find_config_file};

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(test)]
mod tests;

/// Resolve a configuration at a path
///
/// Searches up the directory tree from the path, and then finally in
/// `~/.config/stencila`, looking for `stencila.yaml` and `stencila.local.yaml`
/// files and merges them into a path-specific config.
///
/// # Precedence (highest to lowest)
///
/// 1. Current directory: `stencila.local.yaml` → `stencila.yaml`
/// 2. Parent directories: `../stencila.local.yaml` → `../stencila.yaml` (and so on)
/// 3. User config: `~/.config/stencila/stencila.yaml`
///
/// # Error Handling
///
/// Missing config files are silently ignored. Malformed config files are logged
/// as warnings and skipped. Only returns an error if no valid config could be
/// constructed or if path normalization fails.
pub fn config(path: &Path) -> Result<Config> {
    let figment = build_figment(path, true)?;
    figment.extract().map_err(|error| eyre!("{error}"))
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
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Config {
    /// Site configuration
    pub site: Option<SiteConfig>,

    /// Custom routes for serving content
    ///
    /// Routes can be used by both remote sites (e.g., stencila.site) and
    /// local development servers to map URL paths to files or redirects.
    pub routes: Option<Vec<RouteConfig>>,
}

/// Configuration for a site
#[skip_serializing_none]
#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct SiteConfig {
    /// The id of the Stencila Site
    ///
    /// Eight characters, lowercase letters or digits, returned by
    /// Stencila Cloud when a site is created.
    pub id: Option<String>,

    /// Custom domain for the site
    ///
    /// This is a cached value that is kept in sync with Stencila Cloud
    /// when site details are fetched or the domain is modified.
    /// The canonical source is the Stencila Cloud API.
    pub domain: Option<String>,
}

/// A route configuration for a site
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RouteConfig {
    /// The path pattern for this route
    pub path: String,

    /// The file to serve for this route
    pub file: Option<String>,

    /// A redirect URL for this route
    pub redirect: Option<String>,

    /// HTTP status code for this route
    pub status: Option<u16>,
}
