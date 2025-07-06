use std::process::Command;

use crate::{
    environments::{Devbox, Mise},
    execution::Python,
    AsyncToolCommand, Tool, ToolStdio, ToolType,
};
use common::eyre::{bail, Result};

/// Trait for packages that are installed within runtime environments
///
/// Unlike `Tool` which represents standalone executables on PATH, `Package` represents
/// dependencies that are installed within specific runtime environments (like R packages,
/// Python packages, etc.). These packages are typically not available as standalone
/// executables but are libraries/modules within their respective runtimes.
pub trait Package: Sync + Send {
    /// The name of the package
    fn name(&self) -> &'static str;

    /// A URL for the package
    fn url(&self) -> &'static str;

    /// A description of the package
    fn description(&self) -> &'static str;

    /// Check if the package is installed in the runtime
    fn is_installed(&self) -> bool;

    /// Get the command to install this package
    ///
    /// Returns an AsyncToolCommand that will automatically handle environment detection
    /// and tool wrapping (mise, devbox, etc.) when executed.
    fn install_tool_command(&self) -> Option<AsyncToolCommand>;

    /// Configuration files that indicate this package is needed
    fn config_files(&self) -> Vec<&'static str> {
        vec![]
    }
}

/// Get a list of packages used by Stencila
pub fn packages() -> Vec<Box<dyn Package>> {
    vec![Box::new(Renv)]
}

/// Get a package by name
pub fn get_package(name: &str) -> Option<Box<dyn Package>> {
    packages()
        .into_iter()
        .find(|package| package.name() == name)
}

/// Ensure a package is installed, installing it if necessary
///
/// This is a convenience function that checks if the package is installed and
/// installs it if not, with proper error handling and progress output.
pub async fn ensure_package_installed(package: &dyn Package) -> Result<()> {
    if !package.is_installed() {
        eprintln!("📥 Installing {}...", package.name());
        if let Some(mut install_cmd) = package.install_tool_command() {
            let status = install_cmd
                .stdout(ToolStdio::Inherit)
                .stderr(ToolStdio::Inherit)
                .status()
                .await?;
            if !status.success() {
                bail!("Failed to install {}", package.name());
            }
        } else {
            bail!("No install command available for {}", package.name());
        }
    }
    Ok(())
}

pub struct Npm;

impl Tool for Npm {
    fn name(&self) -> &'static str {
        "npm"
    }

    fn url(&self) -> &'static str {
        "https://www.npmjs.com/"
    }

    fn description(&self) -> &'static str {
        "Node.js package and environment manager"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Packages
    }

    fn config_files(&self) -> Vec<&'static str> {
        vec!["package.json"]
    }
}

pub struct Rig;

impl Tool for Rig {
    fn name(&self) -> &'static str {
        "rig"
    }

    fn url(&self) -> &'static str {
        "https://github.com/r-lib/rig"
    }

    fn description(&self) -> &'static str {
        "The R installation manager"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Packages
    }

    fn install_tools(&self) -> Vec<Box<dyn Tool>> {
        vec![Box::new(Mise)]
    }

    fn exec_command(&self, cmd: &str, args: &[String]) -> Option<Command> {
        // Only wrap R commands
        if cmd != "R" && cmd != "Rscript" {
            return None;
        }

        self.path()?;

        let mut command = Command::new(self.executable_name());

        // Note that we can't simply wrap the command and args here
        let mut args = args.to_vec();
        if cmd == "Rscript" && args.len() == 1 {
            args.insert(0, "--script".into());
        }

        // Note that we do not include `cmd` in the args passed on
        command.args(["run"]).args(args);

        Some(command)
    }

    fn install_command(&self, tool: &dyn Tool) -> Option<Command> {
        if tool.name() != "r" {
            return None;
        }

        let mut command = Command::new(self.executable_name());
        // Add the latest release
        command.args(["add", "release"]);
        Some(command)
    }
}

pub struct Uv;

impl Tool for Uv {
    fn name(&self) -> &'static str {
        "uv"
    }

    fn url(&self) -> &'static str {
        "https://docs.astral.sh/uv/"
    }

    fn description(&self) -> &'static str {
        "Python package and environment manager"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Packages
    }

    fn install_tools(&self) -> Vec<Box<dyn Tool>> {
        vec![Box::new(Devbox)]
    }

    fn install_script(&self) -> Option<(&'static str, Vec<&'static str>)> {
        Some(("https://astral.sh/uv/install.sh", vec![]))
    }

    fn config_files(&self) -> Vec<&'static str> {
        vec!["pyproject.toml"]
    }

    fn exec_command(&self, cmd: &str, args: &[String]) -> Option<Command> {
        // Only wrap python/python3 commands
        if cmd != "python" && cmd != "python3" {
            return None;
        }

        self.path()?;

        let mut command = Command::new(self.executable_name());
        command.args(["run", cmd]).args(args);
        Some(command)
    }

    fn install_command(&self, tool: &dyn Tool) -> Option<Command> {
        // Only install Python
        if tool.name() != Python.name() {
            return None;
        }

        self.path()?;

        // Use `use` here so that a mise.toml get created or added to
        let mut command = Command::new(self.executable_name());
        command.args(["python", "install"]);
        Some(command)
    }
}

pub struct Renv;

impl Package for Renv {
    fn name(&self) -> &'static str {
        "renv"
    }

    fn url(&self) -> &'static str {
        "https://rstudio.github.io/renv/"
    }

    fn description(&self) -> &'static str {
        "R package management and environment isolation"
    }

    fn is_installed(&self) -> bool {
        std::process::Command::new("Rscript")
            .args(&["-e", "library(renv)"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    fn install_tool_command(&self) -> Option<AsyncToolCommand> {
        let mut command = AsyncToolCommand::new("Rscript");
        command.args(&[
            "-e",
            "install.packages(c('pak', 'renv'), repos='https://cran.rstudio.com/')",
        ]);
        Some(command)
    }

    fn config_files(&self) -> Vec<&'static str> {
        vec!["renv.lock", "DESCRIPTION"]
    }
}
