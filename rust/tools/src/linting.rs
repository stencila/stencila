use crate::{
    R,
    environments::{Apt, Devbox, Mise},
    packages::Uv,
    tool::{PACKAGE, Tool, ToolType},
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

pub struct LintR;

impl Tool for LintR {
    fn name(&self) -> &'static str {
        "lintr"
    }

    fn url(&self) -> &'static str {
        "https://lintr.r-lib.org/"
    }

    fn description(&self) -> &'static str {
        "Static code analysis for R"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Linting
    }

    fn installation_tools(&self) -> Vec<Box<dyn Tool>> {
        vec![Box::new(R)]
    }

    fn executable_name(&self) -> &'static str {
        PACKAGE
    }
}

pub struct StyleR;

impl Tool for StyleR {
    fn name(&self) -> &'static str {
        "styler"
    }

    fn url(&self) -> &'static str {
        "https://styler.r-lib.org/"
    }

    fn description(&self) -> &'static str {
        "Formats R code according to the tidyverse style guide"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Linting
    }

    fn installation_tools(&self) -> Vec<Box<dyn Tool>> {
        vec![Box::new(R)]
    }

    fn executable_name(&self) -> &'static str {
        PACKAGE
    }
}
