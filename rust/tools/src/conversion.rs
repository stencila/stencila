use semver::Version;

use crate::{
    VersionReq,
    environments::{Apt, Devbox, Mise},
    packages::Uv,
    tool::{Tool, ToolType},
};

pub struct Agg;

impl Tool for Agg {
    fn name(&self) -> &'static str {
        "agg"
    }

    fn url(&self) -> &'static str {
        "https://github.com/asciinema/agg"
    }

    fn description(&self) -> &'static str {
        "A command-line tool for generating animated GIFs and MP4 videos from asciicast recordings"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Conversion
    }

    fn installation_tools(&self) -> Vec<Box<dyn Tool>> {
        vec![Box::new(Mise)]
    }
}

pub struct MarkerPdf;

impl Tool for MarkerPdf {
    fn name(&self) -> &'static str {
        "marker"
    }

    fn executable_name(&self) -> &'static str {
        "marker_single"
    }

    fn url(&self) -> &'static str {
        "https://github.com/datalab-to/marker"
    }

    fn description(&self) -> &'static str {
        "Converts PDF, EPUB, MOBI to Markdown with high speed and accuracy"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Conversion
    }

    fn installation_tools(&self) -> Vec<Box<dyn Tool>> {
        vec![Box::new(Uv)]
    }

    fn version_available(&self) -> Option<Version> {
        // Marker does not seem to provide a --version or -V or any other
        // similar command AND takes a long time to start up. So to avoid long
        // duration attempts just return 0.0.0
        Some(Version::new(0, 0, 0))
    }

    fn version_available_in_env(&self) -> Option<Version> {
        Some(Version::new(0, 0, 0))
    }
}

pub struct MinerU;

impl Tool for MinerU {
    fn name(&self) -> &'static str {
        "mineru"
    }

    fn url(&self) -> &'static str {
        "https://github.com/opendatalab/MinerU/"
    }

    fn description(&self) -> &'static str {
        "Converts PDF into machine-readable formats"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Conversion
    }

    fn installation_tools(&self) -> Vec<Box<dyn Tool>> {
        vec![Box::new(Uv)]
    }
}

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

    fn installation_tools(&self) -> Vec<Box<dyn Tool>> {
        vec![Box::new(Mise), Box::new(Devbox), Box::new(Apt)]
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

    fn installation_tools(&self) -> Vec<Box<dyn Tool>> {
        vec![Box::new(Devbox), Box::new(Apt)]
    }
}
