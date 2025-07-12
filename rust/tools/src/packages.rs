use std::process::Command;

use crate::{
    command::AsyncToolCommand,
    environments::{Devbox, Mise},
    execution::{Python, R},
    package::Package,
    tool::{Tool, ToolType},
};

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

    fn installation_tools(&self) -> Vec<Box<dyn Tool>> {
        vec![Box::new(Mise)]
    }

    fn execute_command(&self, cmd: &str, args: &[String]) -> Option<Command> {
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

    fn install_tool(&self, tool: &dyn Tool) -> Option<Command> {
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

    fn installation_tools(&self) -> Vec<Box<dyn Tool>> {
        vec![Box::new(Devbox)]
    }

    fn installation_script(&self) -> Option<(&'static str, Vec<&'static str>)> {
        Some(("https://astral.sh/uv/install.sh", vec![]))
    }

    fn config_files(&self) -> Vec<&'static str> {
        vec!["pyproject.toml"]
    }

    fn execute_command(&self, cmd: &str, args: &[String]) -> Option<Command> {
        // Only wrap python/python3 commands
        if cmd != "python" && cmd != "python3" {
            return None;
        }

        self.path()?;

        let mut command = Command::new(self.executable_name());
        command.args(["run", cmd]).args(args);
        Some(command)
    }

    fn install_tool(&self, tool: &dyn Tool) -> Option<Command> {
        // Only install Python itself
        if tool.name() != Python.name() {
            return None;
        }

        self.path()?;

        let mut command = Command::new(self.executable_name());
        command.args(["python", "install"]);

        Some(command)
    }

    fn install_package(&self, package: &str) -> Option<AsyncToolCommand> {
        self.path()?;

        // Check if pyproject.toml exists, if not create it with uv init
        let current_dir = std::env::current_dir().ok()?;
        let pyproject_path = current_dir.join("pyproject.toml");

        let mut command = AsyncToolCommand::new(self.executable_name());

        if !pyproject_path.exists() {
            // Use bash to run both uv init and uv add in sequence
            command = AsyncToolCommand::new("bash");
            command.args([
                "-c",
                &format!("uv init --no-readme --no-pin-python && uv add {package}"),
            ]);
        } else {
            command.args(["add", package]);
        }

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

    fn config_files(&self) -> Vec<&'static str> {
        vec!["renv.lock", "DESCRIPTION"]
    }

    fn package_manager(&self) -> Box<dyn Tool> {
        Box::new(R)
    }

    fn install(&self) -> Option<AsyncToolCommand> {
        // Custom implementation to also install 'pak'
        let mut command = AsyncToolCommand::new("Rscript");
        command.args([
            "-e",
            "install.packages(c('pak', 'renv'), repos='https://cran.rstudio.com/')",
        ]);
        Some(command)
    }
}
