use crate::{
    command::AsyncToolCommand,
    environments::{Devbox, Mise},
    packages::{Rig, Uv},
    tool::{Tool, ToolType},
    VersionReq,
};

pub struct Bash;

impl Tool for Bash {
    fn name(&self) -> &'static str {
        "bash"
    }

    fn url(&self) -> &'static str {
        "https://www.gnu.org/software/bash/"
    }

    fn description(&self) -> &'static str {
        "Unix shell and command language"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Execution
    }

    fn installation_tools(&self) -> Vec<Box<dyn Tool>> {
        vec![Box::new(Devbox)]
    }
}

pub struct Node;

impl Tool for Node {
    fn name(&self) -> &'static str {
        "node"
    }

    fn url(&self) -> &'static str {
        "https://nodejs.org/"
    }

    fn description(&self) -> &'static str {
        "JavaScript runtime and execution environment"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Execution
    }

    fn installation_tools(&self) -> Vec<Box<dyn Tool>> {
        vec![Box::new(Mise), Box::new(Devbox)]
    }
}

pub struct Python;

impl Tool for Python {
    fn name(&self) -> &'static str {
        "python"
    }

    fn url(&self) -> &'static str {
        "https://www.python.org/"
    }

    fn description(&self) -> &'static str {
        "General-purpose programming language interpreter"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Execution
    }

    fn executable_name(&self) -> &'static str {
        "python3"
    }

    fn version_required(&self) -> VersionReq {
        VersionReq::parse("3").expect("invalid semver")
    }

    fn installation_tools(&self) -> Vec<Box<dyn Tool>> {
        vec![Box::new(Uv), Box::new(Mise), Box::new(Devbox)]
    }
}

pub struct R;

impl Tool for R {
    fn name(&self) -> &'static str {
        "r"
    }

    fn url(&self) -> &'static str {
        "https://www.r-project.org/"
    }

    fn description(&self) -> &'static str {
        "Statistical computing and graphics language"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Execution
    }

    fn executable_name(&self) -> &'static str {
        "Rscript"
    }

    fn installation_tools(&self) -> Vec<Box<dyn Tool>> {
        // At time of writing, `mise use asdf:r` is possible but involves a source compile
        // which is slow and error prone (many dev dependencies) so do not include Mise here
        vec![Box::new(Rig), Box::new(Devbox)]
    }

    fn install_package(&self, package: &str) -> Option<AsyncToolCommand> {
        let mut command = AsyncToolCommand::new(self.executable_name());
        command.args([
            "-e",
            &format!("install.packages('{package}', repos='https://cran.rstudio.com/')"),
        ]);
        Some(command)
    }
}
