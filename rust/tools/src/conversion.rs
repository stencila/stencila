use crate::{
    VersionReq,
    environments::{Apt, Devbox, Mise},
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

pub struct Chromium;

impl Tool for Chromium {
    fn name(&self) -> &'static str {
        "chromium"
    }

    fn url(&self) -> &'static str {
        "https://www.chromium.org/"
    }

    fn description(&self) -> &'static str {
        "Open-source browser for HTML rendering and conversion"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Conversion
    }

    fn version_required(&self) -> VersionReq {
        VersionReq::parse(">=131").expect("invalid semver")
    }

    fn installation_tools(&self) -> Vec<Box<dyn Tool>> {
        vec![Box::new(Apt)]
    }
}

pub struct Chrome;

impl Tool for Chrome {
    fn name(&self) -> &'static str {
        "chrome"
    }

    fn executable_name(&self) -> &'static str {
        "google-chrome"
    }

    fn url(&self) -> &'static str {
        "https://www.google.com/chrome/"
    }

    fn description(&self) -> &'static str {
        "Google Chrome browser for HTML rendering and conversion"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Conversion
    }

    fn version_required(&self) -> VersionReq {
        VersionReq::parse(">=131").expect("invalid semver")
    }

    fn installation_tools(&self) -> Vec<Box<dyn Tool>> {
        vec![]
    }
}

pub struct Convert;

impl Tool for Convert {
    fn name(&self) -> &'static str {
        "convert"
    }

    fn url(&self) -> &'static str {
        "https://imagemagick.org/"
    }

    fn description(&self) -> &'static str {
        "Convert digital images"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Conversion
    }

    fn installation_tools(&self) -> Vec<Box<dyn Tool>> {
        vec![Box::new(Devbox), Box::new(Apt)]
    }
}

pub struct Ffmpeg;

impl Tool for Ffmpeg {
    fn name(&self) -> &'static str {
        "ffmpeg"
    }

    fn url(&self) -> &'static str {
        "https://ffmpeg.org/"
    }

    fn description(&self) -> &'static str {
        "A complete, cross-platform solution to record, convert and stream audio and video"
    }

    fn r#type(&self) -> ToolType {
        ToolType::Conversion
    }

    fn installation_tools(&self) -> Vec<Box<dyn Tool>> {
        vec![Box::new(Mise), Box::new(Devbox), Box::new(Apt)]
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
