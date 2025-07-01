use std::process::Command;

use crate::{Tool, ToolType};

pub struct Devbox;

impl Tool for Devbox {
    fn name(&self) -> &'static str {
        "devbox"
    }

    fn url(&self) -> &'static str {
        "https://www.jetpack.io/devbox/"
    }

    fn install_script(&self) -> Option<(&'static str, Vec<&'static str>)> {
        // Use the --force argument to avoid waiting for interactive inputs
        Some(("https://get.jetify.com/devbox", vec!["--force"]))
    }

    fn version_command(&self) -> Vec<&'static str> {
        vec!["version"]
    }

    fn description(&self) -> &'static str {
        "Isolated development environments with Nix"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Environments
    }

    fn config_files(&self) -> Vec<&'static str> {
        vec!["devbox.json"]
    }

    fn exec_command(&self, cmd: &str, args: &[String]) -> Option<Command> {
        self.path()?;

        let mut command = Command::new(self.executable_name());
        command.args(["run", "--", cmd]).args(args);
        Some(command)
    }
}

pub struct Mise;

impl Tool for Mise {
    fn name(&self) -> &'static str {
        "mise"
    }

    fn url(&self) -> &'static str {
        "https://mise.jdx.dev/"
    }

    fn install_script(&self) -> Option<(&'static str, Vec<&'static str>)> {
        Some(("https://mise.run", vec![]))
    }

    fn description(&self) -> &'static str {
        "Polyglot tool version manager and task runner"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Environments
    }

    fn config_files(&self) -> Vec<&'static str> {
        vec![
            "mise.toml",
            ".mise.toml",
            "mise.local.toml",
            ".mise.local.toml",
        ]
    }

    fn exec_command(&self, cmd: &str, args: &[String]) -> Option<Command> {
        self.path()?;

        let mut command = Command::new(self.executable_name());
        command.args(["exec", "--", cmd]).args(args);
        Some(command)
    }

    fn install_command(&self, tool: &dyn Tool) -> Option<Command> {
        self.path()?;

        let mut command = Command::new(self.executable_name());
        match tool.name() {
            "rig" => {
                // Use `install --global` for rig because it is more of a system dependency
                // rather than a project dependency
                command.args(["install", "ubi:r-lib/rig"]);
            }
            _ => {
                // For other tools, use the "use" command to add tool to mise.toml
                command.args(["use", tool.name()]);
            }
        }

        Some(command)
    }
}

pub struct Nix;

impl Tool for Nix {
    fn name(&self) -> &'static str {
        "nix"
    }

    fn url(&self) -> &'static str {
        "https://nixos.org/"
    }

    fn install_script(&self) -> Option<(&'static str, Vec<&'static str>)> {
        Some(("https://nixos.org/nix/install", vec![]))
    }

    fn description(&self) -> &'static str {
        "Reproducible builds and development environments"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Environments
    }

    fn config_files(&self) -> Vec<&'static str> {
        vec!["flake.nix"]
    }

    fn exec_command(&self, cmd: &str, args: &[String]) -> Option<Command> {
        self.path()?;

        let mut command = Command::new(self.executable_name());
        command.args(["develop", "--command", cmd]).args(args);
        Some(command)
    }
}

/// Legacy Nix shell environment manager for shell.nix and default.nix files
///
/// This is a separate tool from `Nix` to handle the different command syntax
/// between modern flakes (`nix develop`) and legacy shell files (`nix-shell`).
/// This tool is not exposed in the public tools list but is used internally
/// for environment detection and command wrapping.
pub struct NixShell;

impl Tool for NixShell {
    fn name(&self) -> &'static str {
        "nix-shell"
    }

    fn url(&self) -> &'static str {
        "https://nixos.org/"
    }

    fn description(&self) -> &'static str {
        "Legacy Nix shell environments"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Environments
    }

    fn executable_name(&self) -> &'static str {
        "nix-shell"
    }

    fn config_files(&self) -> Vec<&'static str> {
        vec!["shell.nix", "default.nix"]
    }

    fn exec_command(&self, cmd: &str, args: &[String]) -> Option<Command> {
        self.path()?;

        let mut command = Command::new(self.executable_name());
        command.args(["--run", cmd]).args(args);
        Some(command)
    }
}

pub struct Pixi;

impl Tool for Pixi {
    fn name(&self) -> &'static str {
        "pixi"
    }

    fn url(&self) -> &'static str {
        "https://pixi.sh/"
    }

    fn install_script(&self) -> Option<(&'static str, Vec<&'static str>)> {
        Some(("https://pixi.sh/install.sh", vec![]))
    }

    fn description(&self) -> &'static str {
        "Conda-based package and environment manager"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Environments
    }

    fn config_files(&self) -> Vec<&'static str> {
        vec!["pixi.toml"]
    }

    fn exec_command(&self, cmd: &str, args: &[String]) -> Option<Command> {
        self.path()?;

        let mut command = Command::new(self.executable_name());
        command.args(["run", cmd]).args(args);
        Some(command)
    }
}
