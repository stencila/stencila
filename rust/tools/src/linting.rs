use crate::{Tool, ToolType};

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
}
