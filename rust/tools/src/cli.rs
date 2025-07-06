use std::{
    fs::canonicalize,
    path::{Path, PathBuf},
    process::exit,
};

use cli_utils::{
    color_print::cstr,
    format::Format,
    tabulated::{Attribute, Cell, CellAlignment, Color, Tabulated},
    AsFormat, Code, ToStdout,
};
use common::{
    clap::{self, Args, Parser, Subcommand},
    eyre::{bail, Result},
    itertools::Itertools,
    once_cell::sync::Lazy,
    serde_json::json,
};
use directories::UserDirs;
use pathdiff::diff_paths;

use crate::{detect_managers, AsyncToolCommand, ToolStdio, ToolType};

/// Manage tools and environments used by Stencila
///
/// Provides a unified interface for managing various tools including
/// programming languages, package managers, linters, and converters.
/// It automatically detects and integrates with environment managers like devbox,
/// mise, and uv to provide isolated and reproducible environments.
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><blue>Examples</blue></bold>
  <dim># List all available tools</dim>
  <blue>></blue> stencila tools

  <dim># Show details about a specific tool</dim>
  <blue>></blue> stencila tools show python

  <dim># Install a tool</dim>
  <blue>></blue> stencila tools install mise

  <dim># Install multiple tools</dim>
  <blue>></blue> stencila tools install mise uv ruff

  <dim># Install all dependencies from config files</dim>
  <blue>></blue> stencila tools install

  <dim># Detect environment configuration in current directory</dim>
  <blue>></blue> stencila tools env

  <dim># Run a command with automatic environment detection</dim>
  <blue>></blue> stencila tools run -- python script.py
"
);

#[derive(Debug, Subcommand)]
enum Command {
    List(List),
    Show(Show),
    Install(Install),
    Env(Env),
    Run(Run),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let Some(command) = self.command else {
            return List::default().run().await;
        };

        match command {
            Command::List(list) => list.run().await,
            Command::Show(show) => show.run().await,
            Command::Install(install) => install.run().await,
            Command::Env(detect) => detect.run().await,
            Command::Run(run) => run.run().await,
        }
    }
}

/// List available tools and their installation status
///   
/// Displays a table of all tools that Stencila can manage, including their type,
/// required version, available version, and installation path. The versions and paths
/// shown reflect the currently active environment managers (devbox, mise, etc.) if
/// configured in the current directory, otherwise system-wide installations.
#[derive(Debug, Default, Args)]
#[command(after_long_help = LIST_AFTER_LONG_HELP)]
struct List {
    /// Only list tools of a particular type
    #[arg(short, long)]
    r#type: Option<ToolType>,

    /// Only list tools that are installed
    ///
    /// This filters out tools that are not installed or cannot be found in PATH
    #[arg(long)]
    installed: bool,

    /// Only list tools that can be installed automatically
    ///
    /// This filters to only show tools that have installation scripts available
    #[arg(long)]
    installable: bool,

    /// Output format for tool specifications
    ///
    /// Export tools as Model Context Protocol (MCP) tool specifications.
    /// This is useful for integrating with AI assistants and other MCP-compatible systems.
    /// See https://modelcontextprotocol.io/docs/concepts/tools for more details.
    #[arg(long, short, value_name = "FORMAT")]
    r#as: Option<AsFormat>,
}

pub static LIST_AFTER_LONG_HELP: &str = cstr!(
    "<bold><blue>Examples</blue></bold>
  <dim># List all tools</dim>
  <blue>></blue> stencila tools list

  <dim># List only installed tools</dim>
  <blue>></blue> stencila tools list --installed

  <dim># List only installable tools</dim>
  <blue>></blue> stencila tools list --installable

  <dim># List only execution tools (programming languages)</dim>
  <blue>></blue> stencila tools list --type execution

  <dim># Export tool list as Model Context Protocol tool specifications</dim>
  <blue>></blue> stencila tools list --as json

  <dim># Display tool list as YAML</dim>
  <blue>></blue> stencila tools list --as yaml
"
);

impl List {
    #[allow(clippy::print_stdout)]
    async fn run(self) -> Result<()> {
        let list = super::list();

        let list = list.into_iter().filter(|tool| {
            if self.installed && !tool.is_installed() {
                return false;
            }

            if self.installable && !tool.is_installable() {
                return false;
            }

            self.r#type
                .as_ref()
                .map(|tool_type| tool.r#type() == *tool_type)
                .unwrap_or(true)
        });

        if let Some(format) = self.r#as {
            let list = list.into_iter().map(|tool| tool.mcp_tool()).collect_vec();

            Code::new_from(format.into(), &list)?.to_stdout();

            return Ok(());
        }

        let mut table = Tabulated::new();
        table.set_header(["Name", "Description", "Type", "Version", "Path"]);

        for tool in list {
            let name = tool.name();
            let description = tool.description();
            let r#type = tool.r#type();
            let version = tool
                .version_available_in_env()
                .map_or_else(|| "-".into(), |version| version.to_string());
            let path = if let Some(path) = tool.path_in_env() {
                Cell::new(strip_home_dir(&path))
            } else if tool.is_installable() {
                Cell::new("installable").fg(Color::Yellow)
            } else {
                Cell::new("not installed").fg(Color::DarkRed)
            };

            table.add_row([
                Cell::new(name).add_attribute(Attribute::Bold),
                Cell::new(description),
                match r#type {
                    ToolType::Environments => Cell::new(r#type.to_string()).fg(Color::Magenta),
                    ToolType::Packages => Cell::new(r#type.to_string()).fg(Color::Blue),
                    ToolType::Execution => Cell::new(r#type.to_string()).fg(Color::Cyan),
                    ToolType::Linting => Cell::new(r#type.to_string()).fg(Color::Green),
                    ToolType::Conversion => Cell::new(r#type.to_string()).fg(Color::Yellow),
                    ToolType::Collaboration => Cell::new(r#type.to_string()).fg(Color::Red),
                },
                Cell::new(version).set_alignment(CellAlignment::Right),
                path,
            ]);
        }

        table.to_stdout();

        // Display active environment manager config files
        let cwd = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let detected_managers =
            detect_managers(&cwd, &[ToolType::Environments, ToolType::Packages]);

        if !detected_managers.is_empty() {
            println!();
            print!("Active environment config files: ");

            let config_paths: Vec<String> = detected_managers
                .into_iter()
                .map(|(_, config_path)| {
                    let relative_path = make_relative_with_dot(&cwd, &config_path)
                        .unwrap_or(config_path)
                        .display()
                        .to_string();

                    format!("\x1b[35m{}\x1b[0m", relative_path) // Magenta color
                })
                .collect();

            println!("{}", config_paths.join(", "));
        }

        Ok(())
    }
}

/// Show information about a specific tool
///
/// Displays information about a tool including its name, URL,
/// description, version requirements, installation status, and file path.
/// The version and path shown reflect the currently active environment
/// managers (devbox, mise, etc.) if configured in the current directory,
/// otherwise system-wide installation.
#[derive(Debug, Default, Args)]
#[command(after_long_help = SHOW_AFTER_LONG_HELP)]
struct Show {
    /// The name of the tool to show details for
    #[arg(value_name = "TOOL")]
    name: String,
}

pub static SHOW_AFTER_LONG_HELP: &str = cstr!(
    "<bold><blue>Examples</blue></bold>
  <dim># Show details about Pandoc</dim>
  <blue>></blue> stencila tools show pandoc

  <dim># Show details about uv</dim>
  <blue>></blue> stencila tools show uv

<bold><blue>Supported tools</blue></bold>
  <dim># See which tools are installed</dim>
  <blue>></blue> stencila tools list --installed
"
);

impl Show {
    #[allow(clippy::print_stderr)]
    async fn run(self) -> Result<()> {
        let Some(tool) = super::get(&self.name) else {
            eprintln!("🔍 No tool with name `{}`", self.name);
            exit(1)
        };

        let tool = json!({
            "Name": tool.name(),
            "URL": tool.url(),
            "Description": tool.description(),
            "Version required": tool.version_required(),
            "Version available": tool.version_available_in_env().map_or_else(|| "None".into(), |version| version.to_string()),
            "Path": tool.path_in_env().map_or_else(|| "None".into(), |path| strip_home_dir(&path)),
        });

        Code::new_from(Format::Yaml, &tool)?.to_stdout();

        Ok(())
    }
}

/// Install a tool or setup development environment
///
/// When provided with one or more tool names as arguments, installs those tools.
/// When run without arguments, automatically detects and installs environment managers,
/// tools, and dependencies based on configuration files found in the project directory.
#[derive(Debug, Args)]
#[command(after_long_help = INSTALL_AFTER_LONG_HELP)]
struct Install {
    /// The name(s) of the tool(s) to install (if not provided, installs all dependencies from config files)
    #[arg(value_name = "TOOL")]
    names: Vec<String>,

    /// The directory to setup when installing from config files (defaults to current directory)
    #[arg(long, short = 'C', value_name = "DIR")]
    path: Option<PathBuf>,

    /// Skip environment manager tool installation (only when installing from configs)
    #[arg(long)]
    skip_env: bool,

    /// Skip Python dependency installation (only when installing from configs)
    #[arg(long)]
    skip_python: bool,

    /// Skip R dependency installation (only when installing from configs)
    #[arg(long)]
    skip_r: bool,

    /// Force installation even if the tool is already installed
    #[arg(short, long)]
    force: bool,

    /// Show what would be done without executing (only when installing from configs)
    #[arg(long)]
    dry_run: bool,
}

pub static INSTALL_AFTER_LONG_HELP: &str = cstr!(
    "<bold><blue>Tool Installation Examples</blue></bold>
  <dim># Install mise (tool version manager)</dim>
  <blue>></blue> stencila tools install mise

  <dim># Install uv (Python package manager)</dim>
  <blue>></blue> stencila tools install uv

  <dim># Install multiple tools at once</dim>
  <blue>></blue> stencila tools install mise uv ruff

  <dim># Force reinstall an already installed tool</dim>
  <blue>></blue> stencila tools install --force ruff

<bold><blue>Environment Setup Examples</blue></bold>
  <dim># Install all dependencies from config files in current directory</dim>
  <blue>></blue> stencila tools install

  <dim># Install dependencies from config files in specific directory</dim>
  <blue>></blue> stencila tools install -C /path/to/project

  <dim># Show what would be installed without executing</dim>
  <blue>></blue> stencila tools install --dry-run

  <dim># Skip Python dependencies during setup</dim>
  <blue>></blue> stencila tools install --skip-python

<bold><blue>Setup phases (when no tool specified)</blue></bold>
  1. Install environment managers (mise, devbox, etc.) if needed
  2. Install tools from environment manager configs
  3. Setup Python dependencies (pyproject.toml, requirements.txt)
  4. Setup R dependencies (renv.lock, DESCRIPTION)

<bold><blue>Supported tools</blue></bold>
  <dim># See which tools can be installed</dim>
  <blue>></blue> stencila tools list --installable
"
);

impl Install {
    #[allow(clippy::print_stderr)]
    async fn run(self) -> Result<()> {
        if !self.names.is_empty() {
            // Install specific tools
            self.install_tools(&self.names).await
        } else {
            // Install from config files (setup mode)
            self.install_from_configs().await
        }
    }

    async fn install_tools(&self, names: &[String]) -> Result<()> {
        for name in names {
            self.install_tool(name).await?;
        }
        Ok(())
    }

    async fn install_tool(&self, name: &str) -> Result<()> {
        let Some(tool) = super::get(name) else {
            eprintln!("🔍 No tool with name `{}`", name);
            exit(1)
        };

        // Check if already installed (unless --force is used)
        if let Some(path) = tool.path_in_env() {
            if !self.force {
                eprintln!(
                    "👍 {} is already installed at {}",
                    tool.name(),
                    strip_home_dir(&path)
                );
                return Ok(());
            } else {
                eprintln!(
                    "🔄 {} is already installed at {}, but forcing reinstallation",
                    tool.name(),
                    strip_home_dir(&path)
                );
            }
        }

        // Check if installation is supported
        if !tool.is_installable() {
            eprintln!("❌ {} does not support automated installation", tool.name());
            eprintln!(
                "   Please visit {} for installation instructions",
                tool.url()
            );
            exit(1)
        }

        eprintln!("📥 Installing {}...", tool.name());

        match super::install(tool.as_ref(), self.force).await {
            Ok(()) => {
                eprintln!("✅ {} installed successfully", tool.name());

                // Verify installation
                if let Some(path) = tool.path_in_env() {
                    eprintln!("   Path: {}", strip_home_dir(&path));
                    if let Some(version) = tool.version_available_in_env() {
                        eprintln!("   Version: {}", version);
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to install {}: {}", tool.name(), e);
                eprintln!("   Please visit {} for manual installation", tool.url());
                exit(1)
            }
        }

        Ok(())
    }

    async fn install_from_configs(&self) -> Result<()> {
        let path = if let Some(path) = &self.path {
            if !path.exists() {
                bail!("Path does not exist: {}", path.display());
            }
            canonicalize(path).unwrap_or_else(|_| path.clone())
        } else {
            std::env::current_dir()?
        };

        eprintln!(
            "🔧 Installing dependencies from config files in {}",
            strip_home_dir(&path)
        );

        if !self.skip_env {
            self.install_environment_managers(&path).await?;
        }

        if !self.skip_python {
            self.install_python_dependencies(&path).await?;
        }

        if !self.skip_r {
            self.install_r_dependencies(&path).await?;
        }

        eprintln!("🎉 Installation complete!");
        Ok(())
    }

    async fn install_environment_managers(&self, path: &Path) -> Result<()> {
        let managers = detect_managers(path, &[ToolType::Environments]);

        if managers.is_empty() {
            if self.dry_run {
                eprintln!("📋 Would check for environment managers (none found)");
            }
            return Ok(());
        }

        for (manager, config_path) in managers {
            if self.dry_run {
                eprintln!(
                    "📋 Would install tools from {} using {}",
                    strip_home_dir(&config_path),
                    manager.name()
                );
                continue;
            }

            eprintln!(
                "🔧 Installing tools from {} using {}",
                strip_home_dir(&config_path),
                manager.name()
            );

            // Install the environment manager if needed
            if !manager.is_installed() {
                eprintln!("📥 Installing {}", manager.name());
                super::install(manager.as_ref(), self.force).await?;
            }

            // Install tools from the environment manager config
            let status = AsyncToolCommand::new(manager.executable_name())
                .arg("install")
                .current_dir(path)
                .stdout(ToolStdio::Inherit)
                .stderr(ToolStdio::Inherit)
                .status()
                .await?;
            if !status.success() {
                bail!(
                    "Failed to install tools from {}",
                    strip_home_dir(&config_path)
                );
            }
        }

        Ok(())
    }

    async fn install_python_dependencies(&self, path: &Path) -> Result<()> {
        let pyproject_path = path.join("pyproject.toml");
        let requirements_path = path.join("requirements.txt");

        if pyproject_path.exists() {
            if self.dry_run {
                eprintln!("📋 Would install Python dependencies from pyproject.toml");
                return Ok(());
            }

            eprintln!("🐍 Installing dependencies from pyproject.toml");

            // Install dependencies (creates venv automatically if needed)
            let status = AsyncToolCommand::new("uv")
                .arg("sync")
                .current_dir(path)
                .stdout(ToolStdio::Inherit)
                .stderr(ToolStdio::Inherit)
                .status()
                .await?;
            if !status.success() {
                bail!("Failed to install Python dependencies from pyproject.toml");
            }
        } else if requirements_path.exists() {
            if self.dry_run {
                eprintln!("📋 Would install Python dependencies from requirements.txt");
                return Ok(());
            }

            eprintln!("🐍 Installing dependencies from requirements.txt");

            // Create virtual environment first (uv pip requires it)
            let status = AsyncToolCommand::new("uv")
                .arg("venv")
                .current_dir(path)
                .stdout(ToolStdio::Inherit)
                .stderr(ToolStdio::Inherit)
                .status()
                .await?;
            if !status.success() {
                bail!("Failed to create Python virtual environment");
            }

            // Install dependencies
            let status = AsyncToolCommand::new("uv")
                .args(&["pip", "install", "-r", "requirements.txt"])
                .current_dir(path)
                .stdout(ToolStdio::Inherit)
                .stderr(ToolStdio::Inherit)
                .status()
                .await?;
            if !status.success() {
                bail!("Failed to install Python dependencies from requirements.txt");
            }
        }

        Ok(())
    }

    async fn install_r_dependencies(&self, path: &Path) -> Result<()> {
        let renv_path = path.join("renv.lock");
        let description_path = path.join("DESCRIPTION");

        if renv_path.exists() {
            if self.dry_run {
                eprintln!("📋 Would install R dependencies from renv.lock");
                return Ok(());
            }

            eprintln!("📦 Installing dependencies from renv.lock");

            let status = AsyncToolCommand::new("Rscript")
                .args(&["-e", "invisible(renv::restore())"])
                .current_dir(path)
                .stdout(ToolStdio::Inherit)
                .stderr(ToolStdio::Inherit)
                .status()
                .await?;
            if !status.success() {
                bail!("Failed to install R dependencies from renv.lock");
            }
        } else if description_path.exists() {
            if self.dry_run {
                eprintln!("📋 Would install R dependencies from DESCRIPTION");
                return Ok(());
            }

            eprintln!("📦 Installing dependencies from DESCRIPTION file");

            let status = AsyncToolCommand::new("Rscript")
                .args(&["-e", "invisible(renv::install())"])
                .current_dir(path)
                .stdout(ToolStdio::Inherit)
                .stderr(ToolStdio::Inherit)
                .status()
                .await?;
            if !status.success() {
                bail!("Failed to install R dependencies from DESCRIPTION file");
            }
        }

        Ok(())
    }
}

/// Detect environment manager configuration for a directory
///
/// Searches the specified directory (and parent directories) for configuration
/// files that indicate the presence of environment or package managers.
/// This helps understand what development environment is configured for a project.
///
/// Displays both the manager information and the content of the configuration
/// files for inspection.
#[derive(Debug, Args)]
#[command(after_long_help = ENV_AFTER_LONG_HELP)]
struct Env {
    /// The directory to check for environment manager configuration
    ///
    /// Searches this directory and all parent directories for configuration files.
    /// Configuration files include devbox.json, mise.toml, pixi.toml, and pyproject.toml.
    #[arg(default_value = ".", value_name = "PATH")]
    path: PathBuf,
}

pub static ENV_AFTER_LONG_HELP: &str = cstr!(
    "<bold><blue>Examples</blue></bold>
  <dim># Check current directory for environment configuration</dim>
  <blue>></blue> stencila tools env

  <dim># Check a specific project directory</dim>
  <blue>></blue> stencila tools env /path/to/project

  <dim># Check parent directory</dim>
  <blue>></blue> stencila tools env ..
"
);

impl Env {
    #[allow(clippy::print_stderr)]
    async fn run(self) -> Result<()> {
        if !self.path.exists() {
            bail!("Path does not exist: {}", self.path.display());
        }

        let path = canonicalize(&self.path).unwrap_or(self.path);
        let managers = detect_managers(&path, &[ToolType::Environments, ToolType::Packages]);

        if managers.is_empty() {
            eprintln!(
                "🔍 No environment or package manager configuration found for directory {}",
                strip_home_dir(&path)
            );
            exit(1)
        };

        for (tool, config_path) in managers {
            let env = json!({
                "Tool": tool.name(),
                "Config file": strip_home_dir(&config_path)
            });
            Code::new_from(Format::Yaml, &env)?.to_stdout();

            let content = std::fs::read_to_string(&config_path)?;
            let format = Format::from_path(&config_path);
            Code::new(format, &content).to_stdout();
        }

        Ok(())
    }
}

/// Run a command with automatic environment detection and setup
///
/// Mainly for testing configurations. Executes a command within the appropriate
/// development environment by automatically detecting and configuring
/// environment managers. This ensures commands run with the correct tool
/// versions and dependencies as specified in the project configuration.
///
/// The command automatically detects and chains environment managers:
/// (1) Environment managers (e.g devbox, mise, pixi) - for tool version management
/// (2) Package managers (e.g uv) - for language-specific dependencies
#[derive(Debug, Args)]
#[command(after_long_help = RUN_AFTER_LONG_HELP)]
struct Run {
    /// Working directory for the command
    ///
    /// Environment detection will be performed relative to this directory.
    /// If not specified, uses the current working directory.
    #[arg(long, short = 'C', value_name = "DIR")]
    cwd: Option<PathBuf>,

    /// The command and arguments to run (specify after --)
    ///
    /// All arguments after '--' are passed directly to the command.
    /// This allows commands with arguments that start with hyphens.
    #[arg(
        trailing_var_arg = true,
        allow_hyphen_values = true,
        value_name = "COMMAND"
    )]
    command: Vec<String>,
}

pub static RUN_AFTER_LONG_HELP: &str = cstr!(
    "<bold><blue>Note</blue></bold>
  Use '--' to separate the run command options from the command to execute.
  This prevents argument parsing conflicts
  
<bold><blue>Examples</blue></bold>
  <dim># Run Python script with automatic environment detection</dim>
  <blue>></blue> stencila tools run -- python script.py

  <dim># Run Python code</dim>
  <blue>></blue> stencila tools run -- python -c \"print('hello')\"

  <dim># Run from a different directory</dim>
  <blue>></blue> stencila tools run -C /path/to/project -- npm test

  <dim># Run a complex command with multiple arguments</dim>
  <blue>></blue> stencila tools run -- pandoc input.md -o output.pdf --pdf-engine=xelatex
"
);

impl Run {
    async fn run(self) -> Result<()> {
        if self.command.is_empty() {
            bail!("No command specified. Use -- to separate command from options, e.g.: stencila tools run -- pandoc --version");
        }

        let cmd = &self.command[0];
        let args = &self.command[1..];

        let mut command = AsyncToolCommand::new(cmd);
        command.args(args);

        if let Some(cwd) = &self.cwd {
            if !cwd.exists() {
                bail!("Working directory does not exist: {}", cwd.display());
            }
            command.current_dir(cwd);
        }

        let status = command.status().await?;
        if !status.success() {
            if let Some(code) = status.code() {
                exit(code);
            } else {
                exit(1);
            }
        }

        Ok(())
    }
}

/// Strip the home directory from a path and replace it with `~`
fn strip_home_dir(path: &Path) -> String {
    static HOME: Lazy<Option<PathBuf>> =
        Lazy::new(|| UserDirs::new().map(|dirs| dirs.home_dir().to_path_buf()));

    if let Some(rest) = HOME.as_ref().and_then(|home| path.strip_prefix(home).ok()) {
        PathBuf::from("~").join(rest)
    } else {
        path.to_path_buf()
    }
    .to_string_lossy()
    .to_string()
}

/// Make the target path relative to the base
fn make_relative_with_dot(base: &Path, target: &Path) -> Option<PathBuf> {
    diff_paths(target, base).map(|path| {
        if path.starts_with("..") {
            path
        } else {
            Path::new(".").join(path)
        }
    })
}
