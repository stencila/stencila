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
        let Some(path) = self.path() else { return None };

        let mut command = Command::new(path);
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
        let Some(path) = self.path() else { return None };

        let mut command = Command::new(path);
        command.args(["exec", "--", cmd]).args(args);
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
        let Some(path) = self.path() else { return None };

        let mut command = Command::new(path);
        command.args(["run", cmd]).args(args);
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
        "Python package installer and environment manager"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Environments
    }

    fn config_files(&self) -> Vec<&'static str> {
        vec!["pyproject.toml"]
    }

    fn exec_command(&self, cmd: &str, args: &[String]) -> Option<Command> {
        // Only wrap python/python3 commands
        if cmd != "python" && cmd != "python3" {
            return None;
        }

        let Some(path) = self.path() else { return None };

        let mut command = Command::new(path);
        command.args(["run", cmd]).args(args);
        Some(command)
    }
}
