use crate::{
    environments::{Apt, Devbox, Mise},
    packages::Uv,
    tool::{Tool, ToolType},
};

pub struct Ruff;

impl Tool for Ruff {
    fn name(&self) -> &'static str {
        "ruff"
    }

    fn url(&self) -> &'static str {
        "https://docs.astral.sh/ruff/"
    }

    fn description(&self) -> &'static str {
        "Python linter and code formatter"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Linting
    }

    fn installation_tools(&self) -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(Uv),
            Box::new(Mise),
            Box::new(Devbox),
            Box::new(Apt),
        ]
    }

    fn installation_script(&self) -> Option<(&'static str, Vec<&'static str>)> {
        Some(("https://astral.sh/ruff/install.sh", vec![]))
    }
}

pub struct Pyright;

impl Tool for Pyright {
    fn name(&self) -> &'static str {
        "pyright"
    }

    fn url(&self) -> &'static str {
        "https://github.com/microsoft/pyright"
    }

    fn description(&self) -> &'static str {
        "Static type checker for Python"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Linting
    }

    fn installation_tools(&self) -> Vec<Box<dyn Tool>> {
        vec![
            Box::new(Uv),
            Box::new(Mise),
            Box::new(Devbox),
            Box::new(Apt),
        ]
    }
}
