use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use clap::ValueEnum;
use eyre::{Result, eyre};
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

mod agents;
pub mod cli;
mod init;
mod layout;
mod mcp;
mod models;
mod outputs;
mod remotes;
mod singleton;
mod site;
mod site_access;
mod site_actions;
mod site_remotes;
mod site_reviews;
mod site_uploads;
mod utils;
mod watch;
mod workspace;

use crate::workspace::WorkspaceConfig;

pub use {
    agents::AgentsConfig,
    layout::{
        ColorModeStyle, ComponentConfig, ComponentSpec, CopyMarkdownStyle, CustomSocialLink,
        EditOnService, EditSourceStyle, LayoutConfig, LayoutOverride, LayoutPreset, NavGroupsIcons,
        NavMenuDropdownStyle, NavMenuGroups, NavMenuIcons, NavMenuTrigger, NavTreeIcons,
        PrevNextStyle, RegionConfig, RegionSpec, RowConfig, SocialLinkPlatform, SocialLinksStyle,
    },
    mcp::{McpConfig, McpServerEntry, McpTransportConfig},
    models::{KNOWN_MODEL_PROVIDERS, ModelsConfig},
    outputs::{OutputCommand, OutputConfig, OutputTarget, config_add_output, config_remove_output},
    remotes::{
        RemoteSpread, RemoteValue, config_add_remote, config_set_remote_spread,
        config_update_remote_watch,
    },
    singleton::{ConfigChangeEvent, get, load_and_validate, subscribe},
    site::{
        AuthorSpec, AutoIndexConfig, AutoIndexSpec, FeaturedContent, FeaturedCta, GlideConfig,
        LogoConfig, LogoSpec, NavItem, RedirectStatus, RouteSpread, SearchConfig, SearchSpec,
        SiteConfig, SiteFormat, config_add_redirect_route, config_add_route, config_remove_route,
        config_set_route_spread,
    },
    site_access::{AccessLevel, SiteAccessConfig},
    site_actions::{SiteActionsConfig, SiteActionsDirection, SiteActionsMode, SiteActionsPosition},
    site_remotes::{SiteRemoteFormat, SiteRemoteSyncDirection, SiteRemotesConfig, SiteRemotesSpec},
    site_reviews::{SiteReviewType, SiteReviewsConfig, SiteReviewsSpec},
    site_uploads::{SiteUploadsConfig, SiteUploadsSpec},
    utils::{ConfigTarget, set_value, unset_value},
    watch::watch,
};

// Crate-internal re-exports
pub(crate) use utils::find_config_file;

#[cfg(test)]
mod tests;

/// Main configuration file name
pub const CONFIG_FILENAME: &str = "stencila.toml";

/// Local configuration file name (for local overrides, typically gitignored)
pub const CONFIG_LOCAL_FILENAME: &str = "stencila.local.toml";

/// Reserved placeholders that are auto-bound from git refs
const RESERVED_PLACEHOLDERS: &[&str] = &["tag", "branch", "i"];

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

/// Stencila configuration
#[skip_serializing_none]
#[derive(Debug, Default, Clone, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The workspace directory this config was loaded from
    ///
    /// This is set when the config is loaded and is not serialized.
    #[serde(skip)]
    #[schemars(skip)]
    pub workspace_dir: PathBuf,

    /// Workspace configuration.
    pub workspace: Option<WorkspaceConfig>,

    /// Remote synchronization configuration.
    ///
    /// Maps local paths to remote service URLs. The key is the local path
    /// (file, directory, or pattern), and the value can be:
    /// - A simple URL string: `"site" = "https://example.stencila.site/"`
    /// - An object with watch: `"file.md" = { url = "...", watch = "w123" }`
    /// - Multiple remotes: `"file.md" = [{ url = "...", watch = "..." }, "https://..."]`
    ///
    /// Directory paths are implicitly recursive, matching all files within.
    ///
    /// ```toml
    /// # Remotes for a site and specific files
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

    /// Outputs configuration.
    ///
    /// Defines files to be rendered/converted and uploaded to Stencila Cloud
    /// workspace outputs. The key is the output path template, and the value can be:
    /// - A simple source path: `"report.pdf" = "report.md"`
    /// - A configuration object: `"report.pdf" = { source = "report.md", command = "render" }`
    /// - A static file: `"data.csv" = {}` (copies file as-is)
    /// - A pattern: `"exports/*.csv" = { pattern = "exports/*.csv" }`
    /// - A spread: `"{region}/report.pdf" = { source = "report.md", arguments = { region = ["north", "south"] } }`
    ///
    /// ```toml
    /// # Outputs with source, static, and spread variants
    /// [outputs]
    /// "report.pdf" = "report.md"
    /// "data/results.csv" = {}
    /// "{region}/report.pdf" = { source = "report.md", arguments = { region = ["north", "south"] } }
    /// ```
    #[schemars(with = "Option<HashMap<String, OutputTarget>>")]
    pub outputs: Option<HashMap<String, OutputTarget>>,

    /// Site configuration.
    pub site: Option<SiteConfig>,

    /// Agent configuration.
    ///
    /// Controls which agent is used by default.
    pub agents: Option<AgentsConfig>,

    /// Model context protocol (MCP) server configuration.
    ///
    /// Defines MCP servers that agents can connect to.
    ///
    /// ```toml
    /// [mcp.servers.filesystem]
    /// transport.type = "stdio"
    /// transport.command = "npx"
    /// transport.args = ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
    /// ```
    pub mcp: Option<McpConfig>,

    /// Model configuration.
    ///
    /// Controls provider ordering/selection for model-backed features.
    pub models: Option<ModelsConfig>,
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
    pub fn path_is_site_root(&self, path: &Path) -> bool {
        if let Some(site_config) = &self.site
            && let Some(site_root) = &site_config.root
        {
            let site_root_path = self.workspace_dir.join(site_root);

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
    pub fn path_is_in_site_root(&self, path: &Path) -> bool {
        if let Some(site_config) = &self.site
            && let Some(site_root) = &site_config.root
        {
            let site_root_path = self.workspace_dir.join(site_root);

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
