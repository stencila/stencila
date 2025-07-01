use crate::{
    environments::{Devbox, Mise},
    Tool, ToolType, VersionReq,
};

pub struct Pandoc;

impl Tool for Pandoc {
    fn name(&self) -> &'static str {
        "pandoc"
    }

    fn url(&self) -> &'static str {
        "https://pandoc.org/"
    }

    fn description(&self) -> &'static str {
        "A universal document converter"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Conversion
    }

    fn version_required(&self) -> VersionReq {
        VersionReq::parse("3").expect("invalid semver")
    }

    fn install_tools(&self) -> Vec<Box<dyn Tool>> {
        vec![Box::new(Mise), Box::new(Devbox)]
    }
}

pub struct Xelatex;

impl Tool for Xelatex {
    fn name(&self) -> &'static str {
        "xelatex"
    }

    fn url(&self) -> &'static str {
        "https://tug.org/xetex/"
    }

    fn description(&self) -> &'static str {
        "LaTeX to PDF processor with Unicode support"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Conversion
    }

    fn install_tools(&self) -> Vec<Box<dyn Tool>> {
        vec![Box::new(Devbox)]
    }
}
