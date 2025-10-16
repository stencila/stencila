use std::{env, process::Command};

use crate::{
    AsyncToolCommand,
    tool::{Tool, ToolType, find_config_in_ancestors},
};

pub struct Apt;

impl Tool for Apt {
    fn name(&self) -> &'static str {
        "apt"
    }

    fn url(&self) -> &'static str {
        "https://wiki.debian.org/Apt"
    }

    fn description(&self) -> &'static str {
        "Package manager for Ubuntu and Debian systems"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Environments
    }

    fn executable_name(&self) -> &'static str {
        "apt-get"
    }

    fn install_tool(&self, tool: &dyn Tool, force: bool) -> Option<AsyncToolCommand> {
        self.path()?;

        let pkg = match tool.name() {
            "chromium" => "chromium-browser",
            "convert" => "imagemagick",
            "make" => "build-essential",
            "node" => "nodejs",
            "pip" => "python3-pip",
            "python" => "python3",
            "r" => "r-base",
            "xelatex" => "texlive",
            name => name,
        };

        let mut command = AsyncToolCommand::new("sudo");
        command.args(["apt-get", "install", "-y"]);
        if force {
            command.arg("--force-yes");
        }
        command.arg(pkg);

        Some(command)
    }
}

pub struct Devbox;

impl Tool for Devbox {
    fn name(&self) -> &'static str {
        "devbox"
    }

    fn url(&self) -> &'static str {
        "https://www.jetify.com/devbox/"
    }

    fn installation_script(&self) -> Option<(&'static str, Vec<&'static str>)> {
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

    fn execute_command(&self, cmd: &str, args: &[String]) -> Option<Command> {
        self.path()?;

        let mut command = Command::new(self.executable_name());
        command.args(["run", "--", cmd]).args(args);
        Some(command)
    }

    fn install_tool(&self, tool: &dyn Tool, _force: bool) -> Option<AsyncToolCommand> {
        self.path()?;

        let pkg = match tool.name() {
            "node" => "nodejs",
            "r" => "R",
            "xelatex" => "texlivePackages.xelatex-dev",
            "convert" => "imagemagick",
            name => name,
        };

        // Check if devbox.json exists in current directory or ancestors
        let current_dir = env::current_dir().ok()?;
        let has_config = find_config_in_ancestors(&current_dir, &self.config_files()).is_some();

        if !has_config {
            // Use shell to run both commands sequentially
            let mut command = AsyncToolCommand::new("sh");
            command.args(["-c", &format!("devbox init && devbox add {pkg}")]);
            Some(command)
        } else {
            let mut command = self.async_command();
            command.args(["add", pkg]);
            Some(command)
        }
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

    fn installation_script(&self) -> Option<(&'static str, Vec<&'static str>)> {
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

    fn execute_command(&self, cmd: &str, args: &[String]) -> Option<Command> {
        self.path()?;

        let mut command = Command::new(self.executable_name());
        command.args(["exec", "--", cmd]).args(args);
        Some(command)
    }

    fn install_tool(&self, tool: &dyn Tool, _force: bool) -> Option<AsyncToolCommand> {
        self.path()?;

        let mut command = self.async_command();
        match tool.name() {
            "agg" => {
                command.args(["use", "ubi:asciinema/agg"]);
            }
            "rig" => {
                // Use `install` for rig because it is more of a system dependency
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

    fn installation_script(&self) -> Option<(&'static str, Vec<&'static str>)> {
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

    fn execute_command(&self, cmd: &str, args: &[String]) -> Option<Command> {
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
#[allow(dead_code)]
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

    fn execute_command(&self, cmd: &str, args: &[String]) -> Option<Command> {
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

    fn installation_script(&self) -> Option<(&'static str, Vec<&'static str>)> {
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

    fn execute_command(&self, cmd: &str, args: &[String]) -> Option<Command> {
        self.path()?;

        let mut command = Command::new(self.executable_name());
        command.args(["run", cmd]).args(args);
        Some(command)
    }
}
