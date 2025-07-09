//! Unified tool management.
//!
//! This crate provides an interface for managing various development tools
//! including environment managers (e.g. devbox, mise, nix), programming language runtimes
//! (e.g. python, node, r), package managers (e.g. uv, npm), linters (e.g. ruff), and conversion tools
//! (e.g. pandoc, xelatex). It supports automatic tool discovery, installation, version
//! management, and nested environment orchestration.

use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Output, Stdio},
    sync::Mutex,
};

use ask::{ask_with, Answer, AskLevel, AskOptions};
use common::{
    async_recursion::async_recursion,
    clap::{self, ValueEnum},
    eyre::{bail, OptionExt, Result},
    itertools::Itertools,
    once_cell::sync::Lazy,
    regex::Regex,
    reqwest,
    serde::Serialize,
    strum::Display,
    tempfile::env::temp_dir,
    tokio::{self, fs::write, process::Command as AsyncCommand},
    tracing,
};
use version::STENCILA_VERSION;

use derive_more::{Deref, DerefMut};
use mcp_types::{Tool as McpTool, ToolInputSchema as McpToolInputSchema};
pub use semver::{Version, VersionReq};
use which::which;

use collaboration::*;
use conversion::*;
use environments::*;
use execution::*;
use linting::*;
use packages::*;
pub use packages::{ensure_package_installed, get_package, Package};

pub mod cli;
mod collaboration;
mod conversion;
mod environments;
mod execution;
mod linting;
mod packages;

/// Get a list of tools used by Stencila
pub fn list() -> Vec<Box<dyn Tool>> {
    vec![
        // Environments
        Box::new(Devbox),
        Box::new(Mise),
        Box::new(Nix),
        Box::new(Pixi),
        Box::new(Rig),
        // Packages
        Box::new(Npm),
        Box::new(Uv),
        // Execution
        Box::new(Bash),
        Box::new(Node),
        Box::new(Python),
        Box::new(R),
        // Linting
        Box::new(Ruff),
        // Conversion
        Box::new(Agg),
        Box::new(Pandoc),
        Box::new(Xelatex),
        // Collaboration
        Box::new(Git),
    ]
}

/// Get a tool by name
pub fn get(name: &str) -> Option<Box<dyn Tool>> {
    list().into_iter().find(|tool| tool.name() == name)
}

/// Find out if a tool is installed in the current environment
///
/// Errors if the tool is unknown.
pub fn is_installed(name: &str) -> Result<bool> {
    let tool = get(name).ok_or_eyre("Unknown tool")?;
    Ok(tool_is_installed(tool.as_ref()))
}

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
        which(self.executable_name()).ok()
    }

    /// Get the path to the tool within environment managers (if any)
    ///
    /// This method checks if environment managers are configured and uses them to find the tool.
    /// Falls back to the regular `path()` method if no environment managers are detected.
    fn path_in_env(&self) -> Option<PathBuf> {
        let cwd = std::env::current_dir().ok()?;
        let detected_managers = detect_managers(&cwd, &[ToolType::Environments]);

        for (manager, _config_path) in detected_managers {
            // Skip the is_installed() check to avoid recursion and
            // because we'll try the command anyway
            if let Some(mut command) =
                manager.exec_command("which", &[self.executable_name().to_string()])
            {
                if let Ok(output) = command.output() {
                    if output.status.success() {
                        let path_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                        if !path_str.is_empty() {
                            return Some(PathBuf::from(path_str));
                        }
                    }
                }
            }
        }

        // Fall back to system path
        self.path()
    }

    /// Check if the tool is installed (available on the system)
    fn is_installed(&self) -> bool {
        // For the trait method, use a simpler version that doesn't use the cache
        // The cache will be used when calling the public is_tool_installed function

        // For environment tools, use regular path() to avoid recursion
        // For other tools, use path_in_env() to check environment managers first
        let path = if matches!(self.r#type(), ToolType::Environments) {
            self.path()
        } else {
            self.path_in_env()
        };

        if path.is_none() {
            return false;
        }

        // Check version requirements
        if matches!(self.r#type(), ToolType::Environments) {
            // Use `version_available` for environments to avoid recursion
            if let Some(version) = self.version_available() {
                if !self.version_required().matches(&version) {
                    return false;
                }
            }
        } else if let Some(version) = self.version_available_in_env() {
            if !self.version_required().matches(&version) {
                return false;
            }
        }

        // If is installed but version is not known, return true
        // for best effort use
        true
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
            if manager.is_installed() {
                let version_args: Vec<String> = self
                    .version_command()
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                if let Some(mut command) =
                    manager.exec_command(self.executable_name(), &version_args)
                {
                    if let Ok(output) = command.output() {
                        if output.status.success() {
                            let output_str = String::from_utf8_lossy(&output.stdout);
                            if let Some(version) = self.parse_version(&output_str) {
                                return Some(version);
                            }
                        }
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
    fn install_tools(&self) -> Vec<Box<dyn Tool>> {
        Vec::new()
    }

    /// Get the installation script details for this tool
    ///
    /// Returns `None` if the tool doesn't support script-based installation.
    /// Returns a tuple of (url, arguments) where arguments are passed to the script.
    /// Note the the install script is used as a fallback if the tools has no `install_tools`
    fn install_script(&self) -> Option<(&'static str, Vec<&'static str>)> {
        None
    }

    /// Check if the tool can be installed automatically
    fn is_installable(&self) -> bool {
        !self.install_tools().is_empty() || self.install_script().is_some()
    }

    /// Build a command that executes within this tool's environment
    ///
    /// Given a command and its arguments, returns a new command that will
    /// execute the original command within the tool's managed environment.
    /// Returns `None` if this tool doesn't provide environment management.
    fn exec_command(&self, _cmd: &str, _args: &[String]) -> Option<Command> {
        None
    }

    /// Build a command to install another [`Tool`]
    ///
    /// This method should be implemented by tools that can install other tools (like mise, nix, devbox).
    /// Returns `None` if this tool doesn't support installing other tools.
    fn install_command(&self, _tool: &dyn Tool) -> Option<Command> {
        None
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
pub fn clear_installed_cache() {
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

/// Check if a tool is installed, with version-aware caching
/// This function works with trait objects and manages the cache
fn tool_is_installed(tool: &dyn Tool) -> bool {
    let cache_key = tool_cache_key(tool);

    // Check the cache first to avoid repeated PATH lookups for recently installed tools
    if let Ok(cache) = INSTALLED_TOOLS_CACHE.lock() {
        if cache.contains(&cache_key) {
            return true;
        }
    }

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
pub(crate) async fn install(tool: &dyn Tool, force: bool) -> Result<()> {
    install_with_depth(tool, 0, force).await
}

/// Internal install function with dependency resolution and depth tracking
#[async_recursion]
async fn install_with_depth(tool: &dyn Tool, depth: u32, force: bool) -> Result<()> {
    const MAX_DEPTH: u32 = 3;

    if depth > MAX_DEPTH {
        bail!(
            "Maximum dependency chain depth ({}) exceeded when installing {}",
            MAX_DEPTH,
            tool.name()
        );
    }

    // Check if already installed
    if !force && tool_is_installed(tool) {
        return Ok(());
    }

    let install_tools = tool.install_tools();
    if !install_tools.is_empty() {
        // Find the first available installer tool
        for installer in &install_tools {
            if installer.is_installed() {
                return install_via_installer(installer.as_ref(), tool).await;
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
        install_with_depth(first_installer.as_ref(), depth + 1, false).await?;
        install_via_installer(first_installer.as_ref(), tool).await
    } else {
        install_via_script(tool).await
    }
}

/// Install the tool using another tool
async fn install_via_installer(installer: &dyn Tool, tool: &dyn Tool) -> Result<()> {
    if let Some(mut command) = installer.install_command(tool) {
        tracing::debug!("Installing `{}` using `{}`", tool.name(), installer.name());

        let output = command.output()?;
        if output.status.success() {
            // Add to cache since we just installed it
            // Use the tool's cache key which includes version requirements
            if let Ok(mut cache) = INSTALLED_TOOLS_CACHE.lock() {
                cache.insert(tool_cache_key(tool));
            }
            Ok(())
        } else {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!(
                "Failed to install `{}` with `{}`:\n\n{stdout}\n\n{stderr}",
                tool.name(),
                installer.name()
            )
        }
    } else {
        tracing::warn!("Tool `{}` defines `{}` as an installer but the latter does not provide a command to install it", tool.name(), installer.name());

        // Fall back to using script
        install_via_script(tool).await
    }
}

/// Install the tool using its installation script
///
/// Downloads and executes the installation script. Returns an error if
/// installation is not supported or fails.
async fn install_via_script(tool: &dyn Tool) -> Result<()> {
    let (url, script_args) = tool
        .install_script()
        .ok_or_eyre("This tool does not support automated installation")?;

    tracing::debug!("Installing `{}` using install script", tool.name());

    // Create a client that follows redirects and sets user agent to avoid 403s
    let client = reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::limited(10))
        .user_agent(format!("stencila/{STENCILA_VERSION}"))
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
    let output = command.output()?;

    if output.status.success() {
        // Add to cache since we just installed it
        // Use the tool's cache key which includes version requirements
        if let Ok(mut cache) = INSTALLED_TOOLS_CACHE.lock() {
            cache.insert(tool_cache_key(tool));
        }
        Ok(())
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Installation script failed:\n\n{stdout}\n\n{stderr}")
    }
}

/// Macro to create a [`serde_json::Map`] needed within MCP tool definition
#[macro_export]
macro_rules! json_map {
    // Count the number of key-value pairs at compile time
    (@count) => { 0 };
    (@count $key:expr => $value:expr) => { 1 };
    (@count $key:expr => $value:expr, $($rest:tt)*) => {
        1 + json_map!(@count $($rest)*)
    };

    // Empty map
    {} => {
        serde_json::Map::new()
    };

    // Map with pre-allocated capacity
    { $($key:expr => $value:expr),+ $(,)? } => {
        {
            let capacity = json_map!(@count $($key => $value),+);
            let mut map = common::serde_json::Map::with_capacity(capacity);
            $(
                map.insert($key.to_string(), common::serde_json::json!($value));
            )+
            map
        }
    };
}

/// The stdio config to use with one of the tools streams
///
/// It is necessary for us to implement this because [`Stdio`] is not gettable
/// from the underlying command nor clone-able and so we are unable to take the
/// same approach to setting these on wrapped commands as we do for environment
/// variables etc.
#[derive(Default, Debug, Clone, Copy)]
pub enum ToolStdio {
    Inherit,
    Piped,
    #[default]
    Null,
}

impl From<ToolStdio> for Stdio {
    fn from(value: ToolStdio) -> Self {
        match value {
            ToolStdio::Inherit => Stdio::inherit(),
            ToolStdio::Piped => Stdio::piped(),
            ToolStdio::Null => Stdio::null(),
        }
    }
}

/// A wrapper around `std::process::Command` that automatically runs commands
/// through detected environment and package managers with support for nested environments.
///
/// This wrapper can automatically nest tools to provide both
/// tool version management and package management. For example:
///
/// `python script.py` with mise + uv becomes `mise exec -- uv run python script.py`
#[derive(Debug, Deref, DerefMut)]
pub struct ToolCommand {
    #[deref]
    #[deref_mut]
    inner: Command,

    stdin: ToolStdio,
    stdout: ToolStdio,
    stderr: ToolStdio,
}

impl ToolCommand {
    /// Creates a new `ToolCommand` for the given program.
    ///
    /// The program and arguments will be executed through an environment manager
    /// if one is detected in the current working directory.
    pub fn new<S: AsRef<std::ffi::OsStr>>(program: S) -> Self {
        Self {
            inner: Command::new(program),
            stdin: ToolStdio::default(),
            stdout: ToolStdio::default(),
            stderr: ToolStdio::default(),
        }
    }

    /// Adds an argument to pass to the program.
    pub fn arg<S: AsRef<std::ffi::OsStr>>(&mut self, arg: S) -> &mut Self {
        self.inner.arg(arg);
        self
    }

    /// Adds multiple arguments to pass to the program.
    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        self.inner.args(args);
        self
    }

    /// Sets the standard input (stdin) configuration for the child process.
    pub fn stdin(&mut self, cfg: ToolStdio) -> &mut Self {
        self.inner.stdin(Stdio::from(cfg));
        self.stdin = cfg;
        self
    }

    /// Sets the standard output (stdout) configuration for the child process.
    pub fn stdout(&mut self, cfg: ToolStdio) -> &mut Self {
        self.inner.stdout(Stdio::from(cfg));
        self.stdout = cfg;
        self
    }

    /// Sets the standard error (stderr) configuration for the child process.
    pub fn stderr(&mut self, cfg: ToolStdio) -> &mut Self {
        self.inner.stderr(Stdio::from(cfg));
        self.stderr = cfg;
        self
    }

    /// Sets the working directory for the child process.
    pub fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.inner.current_dir(dir);
        self
    }

    /// Inserts or updates an environment variable mapping.
    pub fn env<K, V>(&mut self, key: K, val: V) -> &mut Self
    where
        K: AsRef<std::ffi::OsStr>,
        V: AsRef<std::ffi::OsStr>,
    {
        self.inner.env(key, val);
        self
    }

    /// Adds or updates multiple environment variable mappings.
    pub fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<std::ffi::OsStr>,
        V: AsRef<std::ffi::OsStr>,
    {
        self.inner.envs(vars);
        self
    }

    /// Executes the command as a child process, waiting for it to finish and
    /// collecting all of its output.
    ///
    /// If an environment manager is detected, the command will be wrapped
    /// to run within that environment.
    pub fn output(&mut self) -> Result<Output> {
        // Set stdout and stderr to piped for output capture
        self.stdout = ToolStdio::Piped;
        self.stderr = ToolStdio::Piped;
        Ok(self.wrap_if_needed()?.output()?)
    }

    /// Executes the command as a child process, waiting for it to finish and
    /// collecting its status.
    ///
    /// If an environment manager is detected, the command will be wrapped
    /// to run within that environment.
    pub fn status(&mut self) -> Result<ExitStatus> {
        Ok(self.wrap_if_needed()?.status()?)
    }

    /// Executes the command as a child process, returning a handle to it.
    ///
    /// If an environment manager is detected, the command will be wrapped
    /// to run within that environment.
    pub fn spawn(&mut self) -> Result<std::process::Child> {
        Ok(self.wrap_if_needed()?.spawn()?)
    }

    /// Wraps the command with environment managers if detected
    fn wrap_if_needed(&mut self) -> Result<&mut Command> {
        let program = self.inner.get_program().to_string_lossy().to_string();

        // Check if tool needs auto-installation (sync version cannot install)
        if let Some(tool) = get(&program) {
            let name = tool.name();
            let name_ver = tool.name_and_version_required();

            if !tool.is_installed() {
                bail!(
                    "{name_ver} is required for this operation but is not installed and cannot be auto-installed. Please install {name_ver} (e.g. using `stencila tools install {name}`) and try again"
                );
            }
        }

        // Get the current directory for environment detection
        let cwd = self
            .inner
            .get_current_dir()
            .map(|p| p.to_path_buf())
            .or_else(|| std::env::current_dir().ok());

        if let Some(cwd) = cwd {
            // Get the args from the original command
            let args: Vec<String> = self
                .inner
                .get_args()
                .map(|arg| arg.to_string_lossy().to_string())
                .collect();

            // Build nested environment command
            if let Some(mut wrapped_cmd) = build_nested_command(&program, &args, &cwd) {
                // Log the wrapped command
                tracing::debug!(
                    "ToolCommand wrapped: {} {} -> {} {}",
                    program,
                    args.join(" "),
                    wrapped_cmd.get_program().to_string_lossy(),
                    wrapped_cmd
                        .get_args()
                        .map(|arg| arg.to_string_lossy().to_string())
                        .join(" ")
                );

                // Copy over cwd
                if let Some(dir) = self.inner.get_current_dir() {
                    wrapped_cmd.current_dir(dir);
                }

                // Copy environment variables
                for (key, value) in self.inner.get_envs() {
                    if let (Some(key), Some(value)) = (key.to_str(), value) {
                        wrapped_cmd.env(key, value);
                    }
                }

                // Set stdio configs
                wrapped_cmd.stdin(Stdio::from(self.stdin));
                wrapped_cmd.stdout(Stdio::from(self.stdout));
                wrapped_cmd.stderr(Stdio::from(self.stderr));

                // Replace inner command with wrapped version
                self.inner = wrapped_cmd;
            }
        }

        Ok(&mut self.inner)
    }
}

/// An async wrapper around `tokio::process::Command` that automatically runs commands
/// through detected environment and package managers with support for nested environments.
///
/// This wrapper can automatically nest tools to provide both
/// tool version management and package management. For example:
///
/// `python script.py` with mise + uv becomes `mise exec -- uv run python script.py`
#[derive(Debug, Deref, DerefMut)]
pub struct AsyncToolCommand {
    #[deref]
    #[deref_mut]
    inner: AsyncCommand,

    stdin: ToolStdio,
    stdout: ToolStdio,
    stderr: ToolStdio,
}

impl AsyncToolCommand {
    /// Creates a new `AsyncToolCommand` for the given program.
    ///
    /// The program and arguments will be executed through an environment manager
    /// if one is detected in the current working directory.
    pub fn new<S: AsRef<std::ffi::OsStr>>(program: S) -> Self {
        Self {
            inner: AsyncCommand::new(program),
            stdin: ToolStdio::default(),
            stdout: ToolStdio::default(),
            stderr: ToolStdio::default(),
        }
    }

    /// Adds an argument to pass to the program.
    pub fn arg<S: AsRef<std::ffi::OsStr>>(&mut self, arg: S) -> &mut Self {
        self.inner.arg(arg);
        self
    }

    /// Adds multiple arguments to pass to the program.
    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<std::ffi::OsStr>,
    {
        self.inner.args(args);
        self
    }

    /// Sets the standard input (stdin) configuration for the child process.
    pub fn stdin(&mut self, cfg: ToolStdio) -> &mut Self {
        self.inner.stdin(Stdio::from(cfg));
        self.stdin = cfg;
        self
    }

    /// Sets the standard output (stdout) configuration for the child process.
    pub fn stdout(&mut self, cfg: ToolStdio) -> &mut Self {
        self.inner.stdout(Stdio::from(cfg));
        self.stdout = cfg;
        self
    }

    /// Sets the standard error (stderr) configuration for the child process.
    pub fn stderr(&mut self, cfg: ToolStdio) -> &mut Self {
        self.inner.stderr(Stdio::from(cfg));
        self.stderr = cfg;
        self
    }

    /// Sets the working directory for the child process.
    pub fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.inner.current_dir(dir);
        self
    }

    /// Inserts or updates an environment variable mapping.
    pub fn env<K, V>(&mut self, key: K, val: V) -> &mut Self
    where
        K: AsRef<std::ffi::OsStr>,
        V: AsRef<std::ffi::OsStr>,
    {
        self.inner.env(key, val);
        self
    }

    /// Adds or updates multiple environment variable mappings.
    pub fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<std::ffi::OsStr>,
        V: AsRef<std::ffi::OsStr>,
    {
        self.inner.envs(vars);
        self
    }

    /// Executes the command as a child process, waiting for it to finish and
    /// collecting all of its output.
    ///
    /// If an environment manager is detected, the command will be wrapped
    /// to run within that environment.
    pub async fn output(&mut self) -> Result<std::process::Output> {
        // Set stdout and stderr to piped for output capture
        self.stdout = ToolStdio::Piped;
        self.stderr = ToolStdio::Piped;
        Ok(self.wrap_if_needed().await?.output().await?)
    }

    /// Executes the command as a child process, waiting for it to finish and
    /// collecting its status.
    ///
    /// If an environment manager is detected, the command will be wrapped
    /// to run within that environment.
    pub async fn status(&mut self) -> Result<std::process::ExitStatus> {
        Ok(self.wrap_if_needed().await?.status().await?)
    }

    /// Executes the command as a child process, returning a handle to it.
    ///
    /// If an environment manager is detected, the command will be wrapped
    /// to run within that environment.
    pub async fn spawn(&mut self) -> Result<tokio::process::Child> {
        Ok(self.wrap_if_needed().await?.spawn()?)
    }

    /// Wraps the command with environment managers if detected
    async fn wrap_if_needed(&mut self) -> Result<&mut AsyncCommand> {
        let program = self
            .inner
            .as_std()
            .get_program()
            .to_string_lossy()
            .to_string();

        // Auto-install tool if it's a known tool and not yet installed
        if let Some(tool) = get(&program) {
            if !tool_is_installed(tool.as_ref()) {
                let name = tool.name();
                let name_ver = tool.name_and_version_required();

                let answer = ask_with(
                    &format!("{name_ver} is required for this operation but is not yet installed. Would you like to install it now?"),
                    AskOptions {
                        level: AskLevel::Warning,
                        default: Some(Answer::Yes),
                        ..Default::default()
                    },
                )
                .await
                .unwrap_or(Answer::No);

                if answer.is_yes() {
                    tracing::info!("Installing `{name}`");
                    if let Err(error) = install(tool.as_ref(), false).await {
                        tracing::warn!("Failed to install {name}: {error}");
                    }
                } else {
                    bail!(format!("Please install {name_ver} (e.g. using `stencila tools install {name}`) and try again"));
                }
            }
        }

        // Get the current directory for environment detection
        let cwd = self
            .inner
            .as_std()
            .get_current_dir()
            .map(|p| p.to_path_buf())
            .or_else(|| std::env::current_dir().ok());

        if let Some(cwd) = cwd {
            // Get the args from the original command
            let args: Vec<String> = self
                .inner
                .as_std()
                .get_args()
                .map(|arg| arg.to_string_lossy().to_string())
                .collect();

            // Build nested environment command
            if let Some(wrapped_cmd) = build_nested_command(&program, &args, &cwd) {
                // Extract the wrapped command details
                let wrapped_program = wrapped_cmd.get_program().to_string_lossy().to_string();
                let wrapped_args: Vec<String> = wrapped_cmd
                    .get_args()
                    .map(|arg| arg.to_string_lossy().to_string())
                    .collect();

                // Log the wrapped command
                tracing::debug!(
                    "AsyncToolCommand wrapped: {} {} -> {} {}",
                    program,
                    args.join(" "),
                    wrapped_program,
                    wrapped_args.join(" ")
                );

                // Create new async command with wrapped details
                let mut wrapped_cmd = AsyncCommand::new(&wrapped_program);
                wrapped_cmd.args(&wrapped_args);

                // Copy over cwd
                if let Some(dir) = self.inner.as_std().get_current_dir() {
                    wrapped_cmd.current_dir(dir);
                }

                // Copy environment variables
                for (key, value) in self.inner.as_std().get_envs() {
                    if let (Some(key), Some(value)) = (key.to_str(), value) {
                        wrapped_cmd.env(key, value);
                    }
                }

                // Set stdio configs
                wrapped_cmd.stdin(Stdio::from(self.stdin));
                wrapped_cmd.stdout(Stdio::from(self.stdout));
                wrapped_cmd.stderr(Stdio::from(self.stderr));

                // Replace inner command with wrapped version
                self.inner = wrapped_cmd;
            }
        }

        Ok(&mut self.inner)
    }
}

/// Find a config file in the given path or any of its ancestor directories
///
/// Searches up the directory tree from the given path (or its parent directory if
/// the path is a file) looking for any of the specified config files.
/// Returns the path to the first matching config file found.
pub fn find_config_in_ancestors(start_path: &Path, config_files: &[&str]) -> Option<PathBuf> {
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
    for manager in list()
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

/// Build a nested tool command for the given command and path
///
/// This function detects all applicable environment and package managers and creates a nested
/// command structure. For example:
///
/// `python script.py` with mise + uv becomes `mise exec -- uv run python script.py`
fn build_nested_command(command: &str, args: &[String], path: &Path) -> Option<Command> {
    // Find all capable managers in order
    let mut capable_managers: Vec<Box<dyn Tool>> = Vec::new();

    // First, check for package managers with config files in the project
    let detected_package_managers = detect_managers(path, &[ToolType::Packages]);
    for (manager, ..) in detected_package_managers {
        if manager.is_installed() && manager.exec_command(command, args).is_some() {
            capable_managers.push(manager);
        }
    }

    // Then, check for environment managers with config files in the project
    let detected_env_managers = detect_managers(path, &[ToolType::Environments]);
    for (manager, ..) in detected_env_managers {
        if manager.is_installed() && manager.exec_command(command, args).is_some() {
            capable_managers.push(manager);
        }
    }

    // If no capable managers found, return None
    if capable_managers.is_empty() {
        return None;
    }

    // Build nested command from innermost to outermost
    // Package managers (like rig, uv) should be innermost, environment managers (like mise) outermost
    let mut current_cmd = command.to_string();
    let mut current_args = args.to_vec();

    for manager in capable_managers.iter() {
        // Skip if the manager would wrap itself (e.g., mise wrapping "mise install")
        if manager.executable_name() == current_cmd {
            continue;
        }

        if let Some(wrapped_cmd) = manager.exec_command(&current_cmd, &current_args) {
            current_cmd = wrapped_cmd.get_program().to_string_lossy().to_string();
            current_args = wrapped_cmd
                .get_args()
                .map(|arg| arg.to_string_lossy().to_string())
                .collect();
        }
    }

    // Return the final wrapped command
    let mut final_cmd = Command::new(current_cmd);
    final_cmd.args(current_args);
    Some(final_cmd)
}
