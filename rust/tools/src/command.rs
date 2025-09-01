use std::{
    fmt::Debug,
    path::Path,
    process::{Command, ExitStatus, Output, Stdio},
};

use derive_more::{Deref, DerefMut};
use eyre::{Result, bail};
use itertools::Itertools;
use tokio::{self, process::Command as AsyncCommand};

use ask::{Answer, AskLevel, AskOptions, ask_with};

use crate::{
    get,
    tool::{Tool, ToolType, detect_managers, install_tool, is_dry_run, is_installed},
};

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

            if !is_installed(tool.as_ref()) {
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
            tracing::trace!(
                "Checking if `{program}` should be wrapped in directory `{}`",
                cwd.display()
            );

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
                    "ToolCommand wrapped:\n    {} {}\n    {} {}",
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
        if let Some(tool) = get(&program)
            && !is_installed(tool.as_ref())
        {
            // Skip auto-install in dry run mode
            if is_dry_run() {
                bail!(
                    "Tool `{}` would be installed but skipping in dry run mode",
                    program
                );
            }

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
                if let Err(error) = install_tool(tool.as_ref(), false, false).await {
                    tracing::warn!("Failed to install {name}: {error}");
                }
            } else {
                bail!(format!(
                    "Please install {name_ver} (e.g. using `stencila tools install {name}`) and try again"
                ));
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
                    "AsyncToolCommand wrapped `{} {}` into `{} {}`",
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
        if is_installed(manager.as_ref()) && manager.execute_command(command, args).is_some() {
            capable_managers.push(manager);
        }
    }

    // Then, check for environment managers with config files in the project
    let detected_env_managers = detect_managers(path, &[ToolType::Environments]);
    for (manager, ..) in detected_env_managers {
        if is_installed(manager.as_ref()) && manager.execute_command(command, args).is_some() {
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

        if let Some(wrapped_cmd) = manager.execute_command(&current_cmd, &current_args) {
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
