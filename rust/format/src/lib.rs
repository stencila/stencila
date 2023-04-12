use std::path::Path;

use common::{
    clap::{self, ValueEnum},
    eyre::{bail, Result},
    strum::{Display, EnumString},
};

#[derive(Debug, Display, Clone, ValueEnum, EnumString)]
#[strum(serialize_all = "lowercase", crate = "common::strum")]
pub enum Format {
    Json,
    Json5,
    Yaml,
}

impl Format {
    pub fn from_ext(ext: &str) -> Result<Self> {
        Ok(match ext.to_lowercase().as_str() {
            "json" => Format::Json,
            _ => bail!("Unknown extension for format: {ext}"),
        })
    }

    pub fn from_path(path: &Path) -> Result<Self> {
        let ext = path
            .extension()
            .map_or_else(|| path.to_string_lossy(), |ext| ext.to_string_lossy());
        Self::from_ext(&ext)
    }
}
