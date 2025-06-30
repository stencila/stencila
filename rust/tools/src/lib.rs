use std::{
    collections::HashMap,
    fmt::Debug,
    io,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Output},
};

use common::{
    async_recursion::async_recursion,
    clap::{self, ValueEnum},
    eyre::{bail, OptionExt, Result},
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
        Box::new(Devbox) as Box<dyn Tool>,
        Box::new(Mise) as Box<dyn Tool>,
        Box::new(Nix) as Box<dyn Tool>,
        Box::new(Pixi) as Box<dyn Tool>,
        // Packages
        Box::new(Npm) as Box<dyn Tool>,
        Box::new(Uv) as Box<dyn Tool>,
        // Execution
        Box::new(Bash) as Box<dyn Tool>,
        Box::new(Node) as Box<dyn Tool>,
        Box::new(Python) as Box<dyn Tool>,
        Box::new(R) as Box<dyn Tool>,
        // Linting
        Box::new(Ruff) as Box<dyn Tool>,
        // Conversion
        Box::new(Pandoc) as Box<dyn Tool>,
        Box::new(Xelatex) as Box<dyn Tool>,
        // Collaboration
        Box::new(Git) as Box<dyn Tool>,
    ]
}

/// Get a tool by name
pub fn get(name: &str) -> Option<Box<dyn Tool>> {
    list().into_iter().find(|tool| tool.name() == name)
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

    /// Check if the tool is installed (available on the system)
    fn is_installed(&self) -> bool {
        self.path().is_some()
    }

    /// The version required by Stencila
    ///
    /// Defaults to any version and should be overridden if a
    fn version_required(&self) -> VersionReq {
        VersionReq::STAR
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

        let unknown = Version::new(0, 0, 0);

        let version_args = self.version_command();
        let Ok(output) = Command::new(path).args(version_args).output() else {
            return Some(unknown);
        };

        let output = String::from_utf8_lossy(&output.stdout);

        let Some(line) = output.lines().next() else {
            return Some(unknown);
        };

        // Some tools have a version string like `3.141592653-2.6` or `2025-05-09` so
        // replace hyphens with dots so that we can extract as many parts as possible.
        let line = line.replace("-", ".");

        static REGEX: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"(\d+)(?:\.(\d+))?(?:\.(\d+))?").expect("invalid regex"));

        let Some(captures) = REGEX.captures(&line) else {
            return Some(unknown);
        };

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

/// Install the tool using its installation script or dependency
///
/// Automatically resolves dependencies and installs tools. Returns an error if
/// installation is not supported or fails.
pub async fn install(tool: &dyn Tool, force: bool) -> Result<()> {
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
    if !force && tool.is_installed() {
        return Ok(());
    }

    let install_tools = tool.install_tools();
    if !install_tools.is_empty() {
        // Find the first available installer tool
        for installer in &install_tools {
            if installer.is_installed() {
                tracing::debug!(
                    "Installing `{}` using available installer `{}`",
                    tool.name(),
                    installer.name()
                );
                return install_via_installer(installer.as_ref(), tool).await;
            }
        }

        // No installer is available, try to install the first one
        let first_installer = install_tools.first().expect("checked is_empty above");
        tracing::debug!(
            "Installing installer `{}` first for `{}`",
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
}

impl ToolCommand {
    /// Creates a new `ToolCommand` for the given program.
    ///
    /// The program and arguments will be executed through an environment manager
    /// if one is detected in the current working directory.
    pub fn new<S: AsRef<std::ffi::OsStr>>(program: S) -> Self {
        Self {
            inner: Command::new(program),
        }
    }

    /// Executes the command as a child process, waiting for it to finish and
    /// collecting all of its output.
    ///
    /// If an environment manager is detected, the command will be wrapped
    /// to run within that environment.
    pub fn output(&mut self) -> io::Result<Output> {
        self.wrap_if_needed().output()
    }

    /// Executes the command as a child process, waiting for it to finish and
    /// collecting its status.
    ///
    /// If an environment manager is detected, the command will be wrapped
    /// to run within that environment.
    pub fn status(&mut self) -> io::Result<ExitStatus> {
        self.wrap_if_needed().status()
    }

    /// Executes the command as a child process, returning a handle to it.
    ///
    /// If an environment manager is detected, the command will be wrapped
    /// to run within that environment.
    pub fn spawn(&mut self) -> io::Result<std::process::Child> {
        self.wrap_if_needed().spawn()
    }

    /// Wraps the command with environment managers if detected
    fn wrap_if_needed(&mut self) -> &mut Command {
        // Check if tool needs auto-installation (sync version cannot install)
        let program = self.inner.get_program().to_string_lossy().to_string();
        if let Some(tool) = get(&program) {
            if !tool.is_installed() {
                tracing::warn!(
                    "Tool {} is not installed and cannot be auto-installed in sync context. Use AsyncToolCommand or install manually.", 
                    tool.name()
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
            // Get the program and args from the original command
            let program = self.inner.get_program().to_string_lossy().to_string();
            let args: Vec<String> = self
                .inner
                .get_args()
                .map(|arg| arg.to_string_lossy().to_string())
                .collect();

            // Build nested environment command
            if let Some(mut wrapped_cmd) = build_nested_environment_command(&program, &args, &cwd) {
                // Copy over important properties from the original command
                if let Some(dir) = self.inner.get_current_dir() {
                    wrapped_cmd.current_dir(dir);
                }

                // Copy environment variables
                for (key, value) in self.inner.get_envs() {
                    if let (Some(key), Some(value)) = (key.to_str(), value) {
                        wrapped_cmd.env(key, value);
                    }
                }

                // Log the wrapped command
                let wrapped_program = wrapped_cmd.get_program().to_string_lossy();
                let wrapped_args: Vec<String> = wrapped_cmd
                    .get_args()
                    .map(|arg| arg.to_string_lossy().to_string())
                    .collect();
                tracing::debug!(
                    "ToolCommand wrapped: {} {} -> {} {}",
                    program,
                    args.join(" "),
                    wrapped_program,
                    wrapped_args.join(" ")
                );

                // Replace inner command with wrapped version
                self.inner = wrapped_cmd;
            }
        }

        &mut self.inner
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
}

impl AsyncToolCommand {
    /// Creates a new `AsyncToolCommand` for the given program.
    ///
    /// The program and arguments will be executed through an environment manager
    /// if one is detected in the current working directory.
    pub fn new<S: AsRef<std::ffi::OsStr>>(program: S) -> Self {
        Self {
            inner: AsyncCommand::new(program),
        }
    }

    /// Executes the command as a child process, waiting for it to finish and
    /// collecting all of its output.
    ///
    /// If an environment manager is detected, the command will be wrapped
    /// to run within that environment.
    pub async fn output(&mut self) -> io::Result<std::process::Output> {
        self.wrap_if_needed().await.output().await
    }

    /// Executes the command as a child process, waiting for it to finish and
    /// collecting its status.
    ///
    /// If an environment manager is detected, the command will be wrapped
    /// to run within that environment.
    pub async fn status(&mut self) -> io::Result<std::process::ExitStatus> {
        self.wrap_if_needed().await.status().await
    }

    /// Executes the command as a child process, returning a handle to it.
    ///
    /// If an environment manager is detected, the command will be wrapped
    /// to run within that environment.
    pub async fn spawn(&mut self) -> io::Result<tokio::process::Child> {
        self.wrap_if_needed().await.spawn()
    }

    /// Wraps the command with environment managers if detected
    async fn wrap_if_needed(&mut self) -> &mut AsyncCommand {
        // Auto-install tool if it's a known tool and not installed
        let program = self
            .inner
            .as_std()
            .get_program()
            .to_string_lossy()
            .to_string();

        if let Some(tool) = get(&program) {
            if !tool.is_installed() {
                tracing::info!("Auto-installing missing tool: {}", tool.name());
                if let Err(e) = install(tool.as_ref(), false).await {
                    tracing::warn!("Failed to auto-install {}: {}", tool.name(), e);
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
            // Get the program and args from the original command
            let program = self
                .inner
                .as_std()
                .get_program()
                .to_string_lossy()
                .to_string();
            let args: Vec<String> = self
                .inner
                .as_std()
                .get_args()
                .map(|arg| arg.to_string_lossy().to_string())
                .collect();

            // Build nested environment command
            if let Some(wrapped_cmd) = build_nested_environment_command(&program, &args, &cwd) {
                // Extract the wrapped command details
                let wrapped_program = wrapped_cmd.get_program().to_string_lossy().to_string();
                let wrapped_args: Vec<String> = wrapped_cmd
                    .get_args()
                    .map(|arg| arg.to_string_lossy().to_string())
                    .collect();

                // Create new async command with wrapped details
                let mut new_async_cmd = AsyncCommand::new(&wrapped_program);
                new_async_cmd.args(&wrapped_args);

                // Copy over important properties from the original command
                if let Some(dir) = self.inner.as_std().get_current_dir() {
                    new_async_cmd.current_dir(dir);
                }

                // Copy environment variables
                for (key, value) in self.inner.as_std().get_envs() {
                    if let (Some(key), Some(value)) = (key.to_str(), value) {
                        new_async_cmd.env(key, value);
                    }
                }

                // Log the wrapped command
                tracing::debug!(
                    "AsyncToolCommand wrapped: {} {} -> {} {}",
                    program,
                    args.join(" "),
                    wrapped_program,
                    wrapped_args.join(" ")
                );

                // Replace inner command with wrapped version
                self.inner = new_async_cmd;
            }
        }

        &mut self.inner
    }
}

/// Find a config file in the given path or any of its ancestor directories
///
/// Searches up the directory tree from the given path (or its parent directory if
/// the path is a file) looking for any of the specified config files.
/// Returns the path to the first matching config file found.
fn find_config_in_ancestors(start_path: &Path, config_files: &[&str]) -> Option<PathBuf> {
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
fn detect_all_managers(path: &Path) -> Vec<(Box<dyn Tool>, PathBuf)> {
    let managers: Vec<Box<dyn Tool>> = vec![
        Box::new(Devbox),
        Box::new(Mise),
        Box::new(Nix),
        Box::new(NixShell),
        Box::new(Pixi),
        Box::new(Uv),
    ];

    let mut detected = Vec::new();
    for manager in managers {
        let config_files = manager.config_files();
        if let Some(config_path) = find_config_in_ancestors(path, &config_files) {
            detected.push((manager, config_path));
        }
    }
    detected
}

/// Build a nested environment command for the given command and path
///
/// This function detects all applicable environment managers and creates a nested
/// command structure. For example:
///
/// `python script.py` with mise + uv becomes `mise exec -- uv run python script.py`
fn build_nested_environment_command(
    command: &str,
    args: &[String],
    path: &Path,
) -> Option<Command> {
    let all_managers = detect_all_managers(path);
    if all_managers.is_empty() {
        return None;
    }

    // Separate managers by type
    let mut environment_manager = None;
    let mut package_manager = None;

    for (manager, _config_path) in all_managers {
        match manager.r#type() {
            ToolType::Environments => {
                // Take the first environment manager found (prioritize by order in detect function)
                if environment_manager.is_none() {
                    environment_manager = Some(manager);
                }
            }
            ToolType::Packages => {
                // Only use package manager if it supports this command
                if manager.exec_command(command, args).is_some() {
                    package_manager = Some(manager);
                }
            }
            _ => {}
        }
    }

    match (environment_manager, package_manager) {
        // Both environment manager and package manager detected
        (Some(env_mgr), Some(pkg_mgr)) => {
            // Create inner command: package_manager exec command args
            if let Some(inner_cmd) = pkg_mgr.exec_command(command, args) {
                // Get the command and args from the inner command
                let inner_program = inner_cmd.get_program().to_string_lossy().to_string();
                let inner_args: Vec<String> = inner_cmd
                    .get_args()
                    .map(|arg| arg.to_string_lossy().to_string())
                    .collect();

                // Create outer command: environment_manager exec -- inner_command inner_args
                env_mgr.exec_command(&inner_program, &inner_args)
            } else {
                // Package manager doesn't support this command, just use environment manager
                env_mgr.exec_command(command, args)
            }
        }
        // Only environment manager detected
        (Some(env_mgr), None) => env_mgr.exec_command(command, args),
        // Only package manager detected
        (None, Some(pkg_mgr)) => pkg_mgr.exec_command(command, args),
        // No managers detected (shouldn't happen since we checked above)
        (None, None) => None,
    }
}
