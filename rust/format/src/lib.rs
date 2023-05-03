//! Provides the `Format` enum and utility functions for working with document formats

use std::path::Path;

use common::{
    clap::{self, ValueEnum},
    eyre::{bail, Result},
    serde::Serialize,
    strum::{Display, EnumString},
};

#[derive(Debug, Display, Clone, Copy, PartialEq, ValueEnum, EnumString, Serialize)]
#[strum(
    serialize_all = "lowercase",
    ascii_case_insensitive,
    crate = "common::strum"
)]
#[serde(rename_all = "lowercase", crate = "common::serde")]
pub enum Format {
    Debug,
    Jats,
    Json,
    Json5,
    Html,
    Md,
    Ron,
    Yaml,
}

impl Format {
    /// Resolve a [`Format`] from a name for the format
    pub fn from_name(ext: &str) -> Result<Self> {
        use Format::*;

        Ok(match ext.to_lowercase().trim() {
            "debug" => Debug,
            "jats" => Jats,
            "json" => Json,
            "json5" => Json5,
            "html" => Html,
            "md" | "markdown" => Md,
            "ron" => Ron,
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
