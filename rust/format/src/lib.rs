//! Provides the `Format` enum and utility functions for working with document formats

use std::path::Path;

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
    // Development focussed formats
    Debug
}

impl Format {
    /// Get the name of a format to use when displayed
    pub fn name(&self) -> &str {
        use Format::*;
        match self {
            Debug => "Debug",
            Html => "HTML",
            Jats => "JATS",
            Json => "JSON",
            Json5 => "JSON5",
            Markdown => "Markdown",
            Text => "Plain text",
            Yaml => "YAML",
        }
    }

    /// Resolve a [`Format`] from a name for the format
    pub fn from_name(ext: &str) -> Result<Self> {
        use Format::*;

        Ok(match ext.to_lowercase().trim() {
            "debug" => Debug,
            "html" => Html,
            "jats" => Jats,
            "json" => Json,
            "json5" => Json5,
            "md" | "markdown" => Markdown,
            "text" | "txt" => Text,
            "yaml" | "yml" => Yaml,
            _ => bail!("No format matching file name extension `{ext}`"),
        })
    }

    /// Resolve a [`Format`] from a file path
    pub fn from_path(path: &Path) -> Result<Self> {
        let name = match path.extension() {
            Some(ext) => ext,
            None => match path.file_name() {
                Some(name) => name,
                None => path.as_os_str(),
            },
        };

        Self::from_name(&name.to_string_lossy())
    }

    /// Get the default file name extension for a format
    pub fn get_extension(&self) -> String {
        self.to_string()
    }
}
