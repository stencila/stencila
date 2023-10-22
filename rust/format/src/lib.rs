//! Provides the `Format` enum and utility functions for working with document formats

use std::path::{Path, PathBuf};

use common::{
    clap::{self, ValueEnum},
    eyre::{bail, Result},
    serde::Serialize,
    strum::{Display, EnumIter, EnumString},
};

#[derive(
    Debug,
    Display,
    Clone,
    Copy,
    PartialOrd,
    PartialEq,
    Eq,
    Ord,
    ValueEnum,
    EnumIter,
    EnumString,
    Serialize,
)]
#[strum(
    serialize_all = "lowercase",
    ascii_case_insensitive,
    crate = "common::strum"
)]
#[serde(rename_all = "lowercase", crate = "common::serde")]
pub enum Format {
    // Grouped and ordered as most appropriate for documentation
    // Markup formats
    Html,
    Jats,
    // Text formats
    Markdown,
    Text,
    // Data serialization formats
    Json,
    Json5,
    Yaml,
    /// Image formats
    Gif,
    Jpeg,
    Png,
    Svg,
    WebP,
    /// Audio formats
    Aac,
    Flac,
    Mp3,
    Ogg,
    Wav,
    /// Video formats
    Avi,
    Mkv,
    Mp4,
    Ogv,
    WebM,
    // Development focussed formats
    Debug,
}

impl Format {
    /// Get the name of a format to use when displayed
    pub fn name(&self) -> &str {
        use Format::*;
        match self {
            Aac => "AAC",
            Avi => "AVI",
            Debug => "Debug",
            Flac => "FLAC",
            Gif => "GIF",
            Html => "HTML",
            Jats => "JATS",
            Jpeg => "JPEG",
            Json => "JSON",
            Json5 => "JSON5",
            Markdown => "Markdown",
            Mkv => "Matroska",
            Mp3 => "MPEG-3",
            Mp4 => "MPEG-4",
            Ogg => "Ogg Vorbis",
            Ogv => "Ogg Vorbis Video",
            Png => "PNG",
            Svg => "SVG",
            Text => "Plain text",
            Wav => "WAV",
            WebM => "WebM",
            WebP => "WebP",
            Yaml => "YAML",
        }
    }

    /// Is this an image format?
    pub fn is_image(&self) -> bool {
        use Format::*;
        matches!(self, Gif | Jpeg | Png | Svg | WebP)
    }

    /// Is this an audio format?
    pub fn is_audio(&self) -> bool {
        use Format::*;
        matches!(self, Aac | Flac | Mp3 | Ogg | Wav)
    }

    /// Is this a video format?
    pub fn is_video(&self) -> bool {
        use Format::*;
        matches!(self, Avi | Mkv | Mp4 | Ogv | WebM)
    }

    /// Resolve a [`Format`] from a name for the format
    pub fn from_name(name: &str) -> Result<Self> {
        use Format::*;

        Ok(match name.to_lowercase().trim() {
            "debug" => Debug,
            "html" => Html,
            "jats" => Jats,
            "json" => Json,
            "json5" => Json5,
            "md" | "markdown" => Markdown,
            "text" | "txt" => Text,
            "yaml" | "yml" => Yaml,
            _ => bail!("No format matching name `{name}`"),
        })
    }

    /// Resolve a [`Format`] from a file path
    pub fn from_path(path: &Path) -> Result<Self> {
        if path.to_string_lossy().ends_with(".jats.xml") {
            return Ok(Format::Jats);
        }

        let name = match path.extension() {
            Some(ext) => ext,
            None => match path.file_name() {
                Some(name) => name,
                None => path.as_os_str(),
            },
        };

        Self::from_name(&name.to_string_lossy())
    }

    /// Resolve a [`Format`] from a string (e.g. a URL)
    pub fn from_string<S: AsRef<str>>(string: S) -> Result<Self> {
        Self::from_path(&PathBuf::from(string.as_ref()))
    }

    /// Get the default file name extension for a format
    pub fn get_extension(&self) -> String {
        self.to_string()
    }
}
