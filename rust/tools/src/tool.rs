use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    path::{Path, PathBuf},
    process::Command,
    sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
    },
};

use mcp_types::{Tool as McpTool, ToolInputSchema as McpToolInputSchema};
pub use semver::{Version, VersionReq};
use which::which;

use common::{
    async_recursion::async_recursion,
    clap::{self, ValueEnum},
    eyre::{OptionExt, Result, bail},
    once_cell::sync::Lazy,
    regex::Regex,
    reqwest,
    serde::Serialize,
    strum::Display,
    tempfile::env::temp_dir,
    tokio::fs::write,
    tracing,
};
use version::STENCILA_USER_AGENT;

use crate::{ToolCommand, ToolStdio, command::AsyncToolCommand, json_map};

/// The type of a kernel
#[derive(Debug, Clone, PartialEq, Eq, Display, Serialize, ValueEnum)]
#[serde(crate = "common::serde")]
#[strum(serialize_all = "lowercase")]
pub enum ToolType {
    Collaboration,
    Conversion,
    Environments,
    Execution,
    Linting,
    Packages,
}

/// Tools should return this string from `executable_name`
/// to indicate that they are a package
pub(crate) const PACKAGE: &str = "<package>";

pub trait Tool: Sync + Send {
    /// The name of the tool
    fn name(&self) -> &'static str;

    /// A URL for the tool
    fn url(&self) -> &'static str;

    /// A description of the tool
    fn description(&self) -> &'static str;

    /// The type of the tool
    fn r#type(&self) -> ToolType;

    /// The name of the tool's executable used by Stencila
    ///
    /// Used to search for the tool on the $PATH.
    /// By default, the same as `self.name()`.
    fn executable_name(&self) -> &'static str {
        self.name()
    }

    /// The path to the tool (if any)
    ///
    /// Searches on the $PATH for tool, using its name. If an environment
    /// tool such as `mise` or `devbox` is available then the path
    /// should be within any environments defined by them.
    fn path(&self) -> Option<PathBuf> {
        if self.executable_name() == PACKAGE {
            return Some(PACKAGE.into());
        }

        which(self.executable_name()).ok()
    }

    /// Get the path to the tool within environment managers (if any)
    ///
    /// This method checks if environment managers are configured and uses them to find the tool.
    /// Falls back to the regular `path()` method if no environment managers are detected.
    fn path_in_env(&self) -> Option<PathBuf> {
        if self.executable_name() == PACKAGE {
            return Some(PACKAGE.into());
        }

        let cwd = std::env::current_dir().ok()?;
        let detected_managers = detect_managers(&cwd, &[ToolType::Environments]);

        for (manager, _config_path) in detected_managers {
            // Skip the is_installed() check to avoid recursion and
            // because we'll try the command anyway
            if let Some(mut command) =
                manager.execute_command("which", &[self.executable_name().to_string()])
                && let Ok(output) = command.output()
                && output.status.success()
            {
                let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path_str.is_empty() {
                    return Some(PathBuf::from(path_str));
                }
            }
        }

        // Fall back to system path
        self.path()
    }

    /// The version required by Stencila
    ///
    /// Defaults to any version and should be overridden if a
    fn version_required(&self) -> VersionReq {
        VersionReq::STAR
    }

    /// Get the name and semver requirements of the tool
    fn name_and_version_required(&self) -> String {
        let mut result = self.name().to_string();
        let version_required = self.version_required();
        if version_required != VersionReq::STAR {
            result.push_str(&version_required.to_string());
        }
        result
    }

    /// Get the command arguments to retrieve the version of this tool
    ///
    /// Most tools use `--version`, but some may use different arguments.
    /// Override this method for tools that use non-standard version commands.
    fn version_command(&self) -> Vec<&'static str> {
        vec!["--version"]
    }

    /// The version available (if any)
    ///
    /// Defaults to calling the discovered binary with the version command
    /// and then parsing it into a version.
    ///
    /// Note that Stencila uses (and returns from this function) semantic version numbers
    /// which muse have major.minor.patch components. Therefore this function will add
    /// minor and patch versions of 0 is necessary. As such the version returned here
    /// may not exactly match the string returned by the tool.
    fn version_available(&self) -> Option<Version> {
        let path = self.path()?;

        let version_args = self.version_command();
        let Ok(output) = Command::new(path).args(version_args).output() else {
            return Some(Version::new(0, 0, 0));
        };

        let output_str = String::from_utf8_lossy(&output.stdout);
        self.parse_version(&output_str)
            .or(Some(Version::new(0, 0, 0)))
    }

    /// Get the version available within environment managers (if any)
    ///
    /// This method checks if environment managers are configured and uses them to get the tool version.
    /// Falls back to the regular `version_available()` method if no environment managers are detected.
    fn version_available_in_env(&self) -> Option<Version> {
        let cwd = std::env::current_dir().ok()?;
        let detected_managers = detect_managers(&cwd, &[ToolType::Environments]);

        for (manager, _config_path) in detected_managers {
            if is_installed(manager.as_ref()) {
                let version_args: Vec<String> = self
                    .version_command()
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                if let Some(mut command) =
                    manager.execute_command(self.executable_name(), &version_args)
                    && let Ok(output) = command.output()
                    && output.status.success()
                {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    if let Some(version) = self.parse_version(&output_str) {
                        return Some(version);
                    }
                }
            }
        }

        // Fall back to system version
        self.version_available()
    }

    /// Parse version string output into a Version
    ///
    /// Takes the stdout output from a version command and extracts the version number.
    /// Handles various version string formats by normalizing hyphens to dots.
    fn parse_version(&self, output: &str) -> Option<Version> {
        let line = output.lines().next()?;

        // Some tools have a version string like `3.141592653-2.6` or `2025-05-09` so
        // replace hyphens with dots so that we can extract as many parts as possible.
        let line = line.replace("-", ".");

        static REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(\d+)(?:\.(\d+))?(?:\.(\d+))?").expect("invalid regex"));

        let captures = REGEX.captures(&line)?;

        let part = |index| {
            captures
                .get(index)
                .map(|m| m.as_str())
                .and_then(|m| m.parse().ok())
                .unwrap_or_default()
        };
        let major = part(1);
        let minor = part(2);
        let patch = part(3);

        Some(Version::new(major, minor, patch))
    }

    /// Create a Model Context Protocol (MCP) specification for the tool
    ///
    /// This default implementation simply exposes the tool with an input schema
    /// as an array of  
    fn mcp_tool(&self) -> McpTool {
        let version = self
            .version_available()
            .map(|version| ["(version ", &version.to_string(), ")"].concat())
            .unwrap_or_else(|| "(unavailable)".to_string());

        McpTool {
            name: self.name().into(),
            description: Some([self.description(), " ", self.url(), " ", &version].concat()),
            input_schema: self.mcp_tool_inputs(),
        }
    }

    /// Create a Model Context Protocol (MCP) specification for the tool
    ///
    /// This default implementation simply exposes the tool with an input schema
    /// as an array of  
    fn mcp_tool_inputs(&self) -> McpToolInputSchema {
        McpToolInputSchema {
            type_: "object".into(),
            required: vec!["arguments".into()],
            properties: HashMap::from([(
                "arguments".into(),
                json_map! {
                    "type" => "array",
                    "items"  => "string"
                },
            )]),
        }
    }

    /// Get the configuration files used by this tool
    ///
    /// Returns a list of filenames that can be used to detect if this tool
    /// is configured in a project. Environment managers should override this.
    fn config_files(&self) -> Vec<&'static str> {
        vec![]
    }

    /// Get the tools that can install *this* tool, in priority order
    ///
    /// Returns an empty vector if this tool can be installed directly (e.g., via script).
    /// Returns a vector of tools in priority order that can install this tool.
    /// The first available tool will be used for installation.
    fn installation_tools(&self) -> Vec<Box<dyn Tool>> {
        Vec::new()
    }

    /// Get the installation script details for this tool
    ///
    /// Returns `None` if the tool doesn't support script-based installation.
    /// Returns a tuple of (url, arguments) where arguments are passed to the script.
    /// Note the the install script is used as a fallback if the tools has no `install_tools`
    fn installation_script(&self) -> Option<(&'static str, Vec<&'static str>)> {
        None
    }

    /// Check if the tool is installed
    fn is_installed(&self) -> bool
    where
        Self: Sized,
    {
        is_installed(self)
    }

    /// Check if the tool can be installed automatically
    fn is_installable(&self) -> bool {
        !self.installation_tools().is_empty() || self.installation_script().is_some()
    }

    /// Build a command that executes within this tool's environment
    ///
    /// Given a command and its arguments, returns a new command that will
    /// execute the original command within the tool's managed environment.
    /// Returns `None` if this tool doesn't provide environment management.
    fn execute_command(&self, _cmd: &str, _args: &[String]) -> Option<Command> {
        None
    }

    /// Build a command to install another [`Tool`]
    ///
    /// This method should be implemented by tools that can install other tools
    /// (i.e. environment managers like mise, nix, devbox).
    /// Returns `None` if this tool doesn't support installing other tools.
    fn install_tool(&self, _tool: &dyn Tool, _force: bool) -> Option<AsyncToolCommand> {
        None
    }

    /// Build a command to check if a package is installed
    ///
    /// This method is usually only implemented by tools that are package
    /// managers (e.g. uv, npm). It is necessary because packages do not have an
    /// executable on the path and we need to check with their installers if
    /// they are installed.
    ///
    /// Returns `None` if this tool doesn't support package management.
    fn is_package_installed(&self, _tool: &dyn Tool) -> Option<ToolCommand> {
        None
    }

    /// Create a ToolCommand for this tool
    ///
    /// Convenience method that creates a ToolCommand using this tool's executable name.
    fn command(&self) -> ToolCommand {
        ToolCommand::new(self.executable_name())
    }

    /// Create an AsyncToolCommand for this tool
    ///
    /// Convenience method that creates an AsyncToolCommand using this tool's executable name.
    fn async_command(&self) -> AsyncToolCommand {
        AsyncToolCommand::new(self.executable_name())
    }
}

impl Debug for dyn Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Tool {}", self.name())
    }
}

/// Global cache of tools that have been installed and verified in this process
/// This helps avoid repeated installation attempts when environment managers
/// install tools that aren't immediately visible in the current process PATH
/// Key format: "tool_name@version_req" (e.g., "pandoc@^2.0.0")
static INSTALLED_TOOLS_CACHE: Lazy<Mutex<HashSet<String>>> =
    Lazy::new(|| Mutex::new(HashSet::new()));

/// Clear the installed tools cache
///
/// This can be useful for testing or when tools are manually uninstalled
/// and you want to force re-detection of tool availability.
#[allow(unused)]
fn clear_installed_cache() {
    if let Ok(mut cache) = INSTALLED_TOOLS_CACHE.lock() {
        cache.clear();
    }
}

/// Generate a cache key for a tool that includes version requirements
fn tool_cache_key(tool: &dyn Tool) -> String {
    let version_req = tool.version_required();
    if version_req == VersionReq::STAR {
        // If no specific version is required, just use the tool name
        tool.name().to_string()
    } else {
        // Include version requirement in the key
        format!("{}@{}", tool.name(), version_req)
    }
}

/// Global flag to indicate dry run mode for tool installations
static DRY_RUN: AtomicBool = AtomicBool::new(false);

/// Set the dry run mode for tool installations
pub fn set_dry_run(dry_run: bool) {
    DRY_RUN.store(dry_run, Ordering::Relaxed);
}

/// Check if we're in dry run mode
pub fn is_dry_run() -> bool {
    DRY_RUN.load(Ordering::Relaxed)
}

/// Check if a tool is installed, with version-aware caching
/// This function works with trait objects and manages the cache
pub(crate) fn is_installed(tool: &dyn Tool) -> bool {
    let cache_key = tool_cache_key(tool);

    // Check the cache first to avoid repeated lookups for recently installed tools
    if let Ok(cache) = INSTALLED_TOOLS_CACHE.lock()
        && cache.contains(&cache_key)
    {
        return true;
    }

    if tool.executable_name() == PACKAGE {
        // If the tool is a package and thus has no path, then just check with its
        // installer tools if if exists
        let mut is_installed = false;

        for installer in tool.installation_tools() {
            if let Some(mut cmd) = installer.is_package_installed(tool)
                && cmd
                    .status()
                    .map(|status| status.success())
                    .unwrap_or_default()
            {
                is_installed = true;
                break;
            }
        }

        if !is_installed {
            return false;
        }
    } else {
        // For environment tools, use regular path() to avoid recursion
        // For other tools, use path_in_env() to check environment managers first
        let path = if matches!(tool.r#type(), ToolType::Environments) {
            tool.path()
        } else {
            tool.path_in_env()
        };

        if path.is_none() {
            return false;
        }

        // Check version requirements
        let version_satisfied = if matches!(tool.r#type(), ToolType::Environments) {
            // Use `version_available` for environments to avoid recursion
            if let Some(version) = tool.version_available() {
                tool.version_required().matches(&version)
            } else {
                // If version is unknown, assume it's satisfied for best effort
                true
            }
        } else if let Some(version) = tool.version_available_in_env() {
            tool.version_required().matches(&version)
        } else {
            // If version is unknown, assume it's satisfied for best effort
            true
        };

        if !version_satisfied {
            return false;
        }
    }

    // Add to cache only if both path and version requirements are satisfied
    if let Ok(mut cache) = INSTALLED_TOOLS_CACHE.lock() {
        cache.insert(cache_key);
    }

    true
}

/// Install the tool using its installation script or dependency
///
/// Automatically resolves dependencies and installs tools. Returns an error if
/// installation is not supported or fails.
#[tracing::instrument(skip_all)]
pub(crate) async fn install_tool(tool: &dyn Tool, force: bool, display: bool) -> Result<()> {
    install_with_depth(tool, 0, force, display).await
}

/// Internal install function with dependency resolution and depth tracking
#[tracing::instrument(skip_all)]
#[async_recursion]
async fn install_with_depth(tool: &dyn Tool, depth: u32, force: bool, display: bool) -> Result<()> {
    const MAX_DEPTH: u32 = 3;

    if depth > MAX_DEPTH {
        bail!(
            "Maximum dependency chain depth ({}) exceeded when installing {}",
            MAX_DEPTH,
            tool.name()
        );
    }

    // Check if already installed
    if !force && is_installed(tool) {
        return Ok(());
    }

    let install_tools = tool.installation_tools();
    if !install_tools.is_empty() {
        // Find the first available installer tool
        for installer in &install_tools {
            if is_installed(installer.as_ref()) {
                return install_via_tool(installer.as_ref(), tool, force, display).await;
            }
        }

        // No installer is available, try to install the first one
        let first_installer = install_tools.first().expect("checked is_empty above");
        tracing::debug!(
            "Installing installer `{}` to install `{}`",
            first_installer.name(),
            tool.name()
        );

        // Note that force is always false here
        install_with_depth(first_installer.as_ref(), depth + 1, false, display).await
    } else {
        install_via_script(tool, display).await
    }
}

/// Install the tool using another tool
#[tracing::instrument(skip_all)]
async fn install_via_tool(
    installer: &dyn Tool,
    tool: &dyn Tool,
    force: bool,
    display: bool,
) -> Result<()> {
    if let Some(mut command) = installer.install_tool(tool, force) {
        tracing::debug!("Installing `{}` using `{}`", tool.name(), installer.name());

        // Check if in dry run mode
        if is_dry_run() {
            tracing::info!(
                "Dry run mode, skipping install of `{}` with `{}`",
                tool.name(),
                installer.name()
            );
            return Ok(());
        }

        if display {
            let status = command
                .stdout(ToolStdio::Inherit)
                .stderr(ToolStdio::Inherit)
                .status()
                .await?;
            if !status.success() {
                bail!(
                    "Failed to install `{}` with `{}`",
                    tool.name(),
                    installer.name()
                )
            }
        } else {
            let output = command.output().await?;
            if !output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                bail!(
                    "Failed to install `{}` with `{}`:\n\n{stdout}\n\n{stderr}",
                    tool.name(),
                    installer.name()
                )
            }
        }

        // Add to cache since we just installed it
        // Use the tool's cache key which includes version requirements
        if let Ok(mut cache) = INSTALLED_TOOLS_CACHE.lock() {
            cache.insert(tool_cache_key(tool));
        }
        Ok(())
    } else {
        tracing::warn!(
            "Tool `{}` defines `{}` as an installer but the latter does not provide a command to install it",
            tool.name(),
            installer.name()
        );

        // Fall back to using script
        install_via_script(tool, display).await
    }
}

/// Install the tool using its installation script
///
/// Downloads and executes the installation script. Returns an error if
/// installation is not supported or fails.
async fn install_via_script(tool: &dyn Tool, display: bool) -> Result<()> {
    let (url, script_args) = tool
        .installation_script()
        .ok_or_eyre("This tool does not support automated installation")?;

    tracing::debug!("Installing `{}` using install script", tool.name());

    // Check if in dry run mode
    if is_dry_run() {
        tracing::info!(
            "Dry run mode, skipping install of `{}` using installation script",
            tool.name()
        );
        return Ok(());
    }

    // Create a client that follows redirects and sets user agent to avoid 403s
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(10))
        .user_agent(STENCILA_USER_AGENT)
        .build()?;

    // Download the installation script
    let response = client.get(url).send().await?;

    // Check if the response was successful
    if !response.status().is_success() {
        bail!(
            "Failed to download installation script: HTTP {}",
            response.status()
        );
    }

    let script = response.text().await?;

    // Validate that we got script content (not HTML error page)
    if script.trim().is_empty() || script.starts_with("<!DOCTYPE") || script.starts_with("<html") {
        bail!("Downloaded content does not appear to be a valid installation script");
    }

    // Create a temporary file for the script
    let temp_dir = temp_dir();
    let script_path = temp_dir.join("install.sh");
    write(&script_path, script).await?;

    // Make the script executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script_path, perms)?;
    }

    // Execute the installation script with bash (many install scripts require bash features)
    // Use the script-specific arguments provided by the tool
    let mut command = Command::new("bash");
    command.arg(&script_path);
    command.args(script_args);

    if display {
        let status = command
            .stdout(ToolStdio::Inherit)
            .stderr(ToolStdio::Inherit)
            .status()?;
        if !status.success() {
            bail!("Installation script for `{}` failed", tool.name())
        }
    } else {
        let output = command.output()?;
        if !output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!(
                "Installation script for `{}` failed:\n\n{stdout}\n\n{stderr}",
                tool.name()
            )
        }
    }

    // Add to cache since we just installed it
    // Use the tool's cache key which includes version requirements
    if let Ok(mut cache) = INSTALLED_TOOLS_CACHE.lock() {
        cache.insert(tool_cache_key(tool));
    }

    Ok(())
}

/// Find a config file in the given path or any of its ancestor directories
///
/// Searches up the directory tree from the given path (or its parent directory if
/// the path is a file) looking for any of the specified config files.
/// Returns the path to the first matching config file found.
pub(crate) fn find_config_in_ancestors(
    start_path: &Path,
    config_files: &[&str],
) -> Option<PathBuf> {
    let mut current = if start_path.is_file() {
        start_path.parent()?.to_path_buf()
    } else {
        start_path.to_path_buf()
    };

    loop {
        for config_file in config_files {
            let config_path = current.join(config_file);
            if config_path.exists() {
                return Some(config_path);
            }
        }

        if !current.pop() {
            break;
        }
    }

    None
}

/// Detect all environment and package managers configured for a given path
///
/// Searches up the directory tree from the given path (or its parent directory if
/// the path is a file) looking for manager config files.
/// Returns all detected managers and their config file paths.
pub fn detect_managers(path: &Path, types: &[ToolType]) -> Vec<(Box<dyn Tool>, PathBuf)> {
    let mut detected = Vec::new();
    for manager in super::list()
        .into_iter()
        .filter(|tool| types.contains(&tool.r#type()))
    {
        let config_files = manager.config_files();
        if let Some(config_path) = find_config_in_ancestors(path, &config_files) {
            detected.push((manager, config_path));
        }
    }
    detected
}
