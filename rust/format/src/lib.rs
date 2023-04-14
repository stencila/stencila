//! Provides the `Format` enum and utility functions for working with document formats

use std::path::Path;

use common::{
    clap::{self, ValueEnum},
    eyre::{bail, Result},
    strum::{Display, EnumString},
};

#[derive(Debug, Display, Clone, ValueEnum, EnumString)]
#[strum(serialize_all = "lowercase", crate = "common::strum")]
pub enum Format {
    Jats,
    Json,
    Json5,
    Html,
    Markdown,
    Yaml,
}

impl Format {
    /// Resolve a [`Format`] from a name for the format
    pub fn from_name(ext: &str) -> Result<Self> {
        use Format::*;

        Ok(match ext.to_lowercase().trim() {
            "jats" => Jats,
            "json" => Json,
            "json5" => Json5,
            "html" => Html,
            "md" | "markdown" => Markdown,
            "yaml" => Yaml,
            _ => bail!("Unknown extension for format: {ext}"),
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
}
