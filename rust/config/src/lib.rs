use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use clap::ValueEnum;
use eyre::{Result, bail, eyre};
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Pattern for workspace IDs: ws followed by 10 lowercase alphanumeric chars
pub const WORKSPACE_ID_PATTERN: &str = r"^ws[a-z0-9]{10}$";

/// Pattern for watch IDs: wa followed by 10 lowercase alphanumeric chars
pub const WATCH_ID_PATTERN: &str = r"^wa[a-z0-9]{10}$";

/// Pattern for domain names
pub const DOMAIN_PATTERN: &str = r"^([a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?\.)+[a-z]{2,}$";

/// Compiled regex for workspace IDs
static WORKSPACE_ID_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(WORKSPACE_ID_PATTERN).expect("Invalid regex"));

/// Compiled regex for watch IDs
static WATCH_ID_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(WATCH_ID_PATTERN).expect("Invalid regex"));

/// Compiled regex for domain names
static DOMAIN_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(DOMAIN_PATTERN).expect("Invalid regex"));

pub mod cli;
mod init;
mod layout;
mod outputs;
mod remotes;
mod site;
mod utils;
mod watch;

pub use {
    layout::{
        ColorModeStyle, ComponentConfig, ComponentSpec, CopyMarkdownStyle, CustomSocialLink,
        EditOnService, EditSourceStyle, LayoutConfig, LayoutOverride, LayoutPreset, NavGroupsIcons,
        NavMenuDropdownStyle, NavMenuGroups, NavMenuIcons, NavMenuTrigger, NavTreeExpanded,
        NavTreeIcons, PrevNextStyle, RegionConfig, RegionSpec, RowConfig, SocialLinkPlatform,
        SocialLinksStyle,
    },
    outputs::{OutputCommand, OutputConfig, OutputTarget, config_add_output, config_remove_output},
    remotes::{
        RemoteSpread, RemoteValue, config_add_remote, config_set_remote_spread,
        config_update_remote_watch,
    },
    site::{
        AuthorSpec, FeaturedContent, FeaturedCta, GlideConfig, LogoConfig, LogoSpec, NavItem,
        RedirectStatus, RouteSpread, SiteConfig, SiteFormat, config_add_redirect_route,
        config_add_route, config_remove_route, config_set_route_spread,
    },
    utils::{ConfigTarget, config_set, config_unset, config_value, find_config_file},
    watch::watch,
};

use utils::build_figment;

#[cfg(test)]
mod tests;

/// Main configuration file name
pub const CONFIG_FILENAME: &str = "stencila.toml";

/// Local configuration file name (for local overrides, typically gitignored)
pub const CONFIG_LOCAL_FILENAME: &str = "stencila.local.toml";

/// Reserved placeholders that are auto-bound from git refs
const RESERVED_PLACEHOLDERS: &[&str] = &["tag", "branch", "i"];

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

    // Validate workspace configuration
    if let Some(workspace) = &config.workspace {
        workspace.validate()?;
    }

    // Validate site configuration
    if let Some(site) = &config.site {
        site.validate()?;
    }

    // Validate site navigation items (must be internal routes)
    if let Some(site) = &config.site
        && let Some(nav) = &site.nav
    {
        for item in nav {
            item.validate()?;
        }
    }

    // Validate all route configurations
    if let Some(site) = &config.site
        && let Some(routes) = &site.routes
    {
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

    /// The workspace watch ID from Stencila Cloud
    ///
    /// This is set when `stencila watch` is run without a file path to enable
    /// workspace-level watching. When enabled, `update.sh` is run on each git push.
    #[schemars(regex(pattern = r"^wa[a-z0-9]{10}$"))]
    pub watch: Option<String>,
}

impl WorkspaceConfig {
    /// Validate the workspace configuration
    pub fn validate(&self) -> Result<()> {
        if let Some(id) = &self.id
            && !WORKSPACE_ID_REGEX.is_match(id)
        {
            bail!(
                "Invalid workspace ID `{id}`: must match pattern 'ws' followed by 10 lowercase alphanumeric characters (e.g., 'ws3x9k2m7fab')"
            );
        }
        if let Some(watch) = &self.watch
            && !WATCH_ID_REGEX.is_match(watch)
        {
            bail!(
                "Invalid watch ID `{watch}`: must match pattern 'wa' followed by 10 lowercase alphanumeric characters (e.g., 'wa7x2k9m3fab')"
            );
        }
        Ok(())
    }
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

/// Spread mode for multi-variant execution
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize, Serialize, JsonSchema, ValueEnum,
)]
#[serde(rename_all = "lowercase")]
pub enum SpreadMode {
    /// Cartesian product of all arguments (default)
    #[default]
    Grid,
    /// Positional pairing of values (all params must have same length)
    Zip,
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
