use std::process::Command;

use crate::{
    command::AsyncToolCommand,
    environments::{Apt, Devbox, Mise},
    execution::{Python, R},
    tool::{Tool, ToolType, PACKAGE},
    ToolCommand,
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

    fn installation_tools(&self) -> Vec<Box<dyn Tool>> {
        // Intentionally empty because npm is usually installed alongside nodejs
        vec![]
    }
}

pub struct Renv;

impl Tool for Renv {
    fn name(&self) -> &'static str {
        "renv"
    }

    fn url(&self) -> &'static str {
        "https://rstudio.github.io/renv/"
    }

    fn description(&self) -> &'static str {
        "R package management and environment isolation"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Packages
    }

    fn config_files(&self) -> Vec<&'static str> {
        vec!["renv.lock", "DESCRIPTION"]
    }

    fn installation_tools(&self) -> Vec<Box<dyn Tool>> {
        vec![Box::new(R)]
    }

    fn executable_name(&self) -> &'static str {
        PACKAGE
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

    fn install_tool(&self, tool: &dyn Tool, _force: bool) -> Option<AsyncToolCommand> {
        if tool.name() != "r" {
            return None;
        }

        let mut command = AsyncToolCommand::new(self.executable_name());
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
        vec![Box::new(Mise), Box::new(Devbox), Box::new(Apt)]
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

    fn install_tool(&self, tool: &dyn Tool, force: bool) -> Option<AsyncToolCommand> {
        self.path()?;

        let name = tool.name();

        // Install Python
        if name == Python.name() {
            let mut command = AsyncToolCommand::new(self.executable_name());
            command.args(["python", "install"]);
            return Some(command);
        }

        // Install as a tool if possible
        if let Some((tool, options)) = match name {
            "marker" => Some(("marker-pdf", vec!["--with", "psutil"])),
            "mineru" => Some(("mineru[core]", vec![])),
            _ => None,
        } {
            let mut command = AsyncToolCommand::new(self.executable_name());
            command.args(["tool", "install", tool]);
            command.args(options);
            if force {
                command.arg("--force");
            }
            return Some(command);
        }

        // Fallback to installing as a package
        // Check if pyproject.toml exists, if not create it with uv init
        let current_dir = std::env::current_dir().ok()?;
        let pyproject_path = current_dir.join("pyproject.toml");

        let mut command = AsyncToolCommand::new(self.executable_name());

        if !pyproject_path.exists() {
            // Use shell to run both uv init and uv add in sequence
            command = AsyncToolCommand::new("sh");
            command.args(["-c", &format!("uv init --bare && uv add {name}")]);
        } else {
            command.args(["add", name]);
        }

        Some(command)
    }

    fn is_package_installed(&self, tool: &dyn Tool) -> Option<ToolCommand> {
        let package = tool.name();
        let mut command = ToolCommand::new(self.executable_name());
        command.args(["-e", &format!("import {package}")]);
        Some(command)
    }
}
