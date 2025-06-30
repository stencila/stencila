use crate::{Tool, ToolType};

pub struct Npm;

impl Tool for Npm {
    fn name(&self) -> &'static str {
        "npm"
    }

    fn url(&self) -> &'static str {
        "https://www.npmjs.com/"
    }

    fn description(&self) -> &'static str {
        "Node.js package manager and registry"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Packages
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
        "Python package installer and resolver"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Packages
    }
}
