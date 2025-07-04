use std::{fs::read_dir, path::PathBuf, process::Command};

use directories::UserDirs;
use which::which;

use crate::{
    environments::{Devbox, Mise},
    execution::Python,
    Tool, ToolType,
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

    fn path(&self) -> Option<PathBuf> {
        // First try standard PATH lookup
        if let Ok(path) = which(self.executable_name()) {
            return Some(path);
        }

        // Check mise installation directories
        // This is necessary where we have just installed Rig (and potentially also Mise) in the same
        // process and so they may not yet be on the path.
        // Look for rig in ~/.local/share/mise/installs/ubi-r-lib-rig/*/bin/rig
        let home = UserDirs::new().map(|usr| usr.home_dir().to_path_buf())?;
        let mise_base = home.join(".local/share/mise/installs");

        // Check ubi-r-lib-rig directory
        let rig_dir = mise_base.join("ubi-r-lib-rig");
        if rig_dir.exists() {
            // Try latest directory first
            let latest_path = rig_dir.join("latest/rig");
            if latest_path.exists() {
                return Some(latest_path);
            }

            // Find version directories
            if let Ok(entries) = read_dir(&rig_dir) {
                for entry in entries.flatten() {
                    let version_dir = entry.path();

                    // Try bin/rig first
                    let bin_path = version_dir.join("bin/rig");
                    if bin_path.exists() {
                        return Some(bin_path);
                    }

                    // Try rig directly in version directory (for ubi tools)
                    let direct_path = version_dir.join("rig");
                    if direct_path.exists() {
                        return Some(direct_path);
                    }
                }
            }
        }

        None
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
