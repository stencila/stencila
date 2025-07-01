use crate::{environments::{Mise, Devbox}, Tool, ToolType};

pub struct Ruff;

impl Tool for Ruff {
    fn name(&self) -> &'static str {
        "ruff"
    }

    fn url(&self) -> &'static str {
        "https://docs.astral.sh/ruff/"
    }

    fn install_script(&self) -> Option<(&'static str, Vec<&'static str>)> {
        Some(("https://astral.sh/ruff/install.sh", vec![]))
    }

    fn description(&self) -> &'static str {
        "Python linter and code formatter"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Linting
    }

    fn install_tools(&self) -> Vec<Box<dyn Tool>> {
        vec![Box::new(Mise), Box::new(Devbox)]
    }
}
