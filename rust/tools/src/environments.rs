use crate::{Tool, ToolType};

pub struct Asdf;

impl Tool for Asdf {
    fn name(&self) -> &'static str {
        "asdf"
    }

    fn url(&self) -> &'static str {
        "https://asdf-vm.com/"
    }

    fn description(&self) -> &'static str {
        "Extensible version manager for multiple languages"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Environments
    }
}

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
}
