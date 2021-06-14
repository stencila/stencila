use crate::utils::schemas;
use eyre::Result;
use once_cell::sync::Lazy;
use schemars::JsonSchema;
use serde::Serialize;
use serde_with::skip_serializing_none;
use std::{collections::HashMap, path::Path};

/// Information about a document format
///
/// Used to determine various application behaviors
/// e.g. not reading binary formats into memory unnecessarily
#[skip_serializing_none]
#[derive(Clone, Debug, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct Format {
    /// The lowercase name of the format e.g. `md`, `docx`, `dockerfile`
    pub name: String,

    /// Whether or not the format should be considered binary
    /// e.g. not to be displayed in a text / code editor
    pub binary: bool,

    /// The type of `CreativeWork` that this format is expected to be.
    /// This will be `None` for data serialization formats such as
    /// JSON or YAML which have no expected type (the actual type is
    /// embedded in the data).
    #[serde(rename = "type")]
    pub type_: Option<String>,
}

impl Format {
    /// Create a new file format
    pub fn new(name: &str, binary: bool, type_: &str) -> Format {
        Format {
            name: name.into(),
            binary,
            type_: if type_.is_empty() {
                None
            } else {
                Some(type_.to_string())
            },
        }
    }

    /// Create the special `unknown` file format
    pub fn unknown() -> Format {
        Format {
            name: "unknown".to_string(),
            binary: true,
            type_: None,
        }
    }
}

/// List of known document formats
#[derive(Serialize)]
#[serde(transparent)]
pub struct Formats {
    /// Document formats keyed by their name
    formats: HashMap<String, Format>,
}

impl Default for Formats {
    fn default() -> Formats {
        let formats = vec![
            // Data serialization formats
            Format::new("json", false, ""),
            Format::new("json5", false, ""),
            Format::new("toml", false, ""),
            Format::new("xml", false, ""),
            Format::new("yaml", false, ""),
            // Code formats
            Format::new("dockerfile", false, ""),
            Format::new("js", false, ""),
            Format::new("makefile", false, ""),
            Format::new("py", false, ""),
            Format::new("r", false, ""),
            Format::new("sh", false, ""),
            Format::new("ts", false, ""),
            // Article formats
            Format::new("docx", true, "Article"),
            Format::new("odt", true, "Article"),
            Format::new("ipynb", false, "Article"),
            Format::new("md", false, "Article"),
            Format::new("rmd", false, "Article"),
            Format::new("tex", false, "Article"),
            Format::new("txt", false, "Article"),
            // Audio formats
            Format::new("flac", true, "AudioObject"),
            Format::new("mp3", true, "AudioObject"),
            Format::new("ogg", true, "AudioObject"),
            // Image formats
            Format::new("gif", true, "ImageObject"),
            Format::new("jpeg", true, "ImageObject"),
            Format::new("jpg", true, "ImageObject"),
            Format::new("png", true, "ImageObject"),
            // Video formats
            Format::new("3gp", true, "VideoObject"),
            Format::new("mp4", true, "VideoObject"),
            Format::new("ogv", true, "VideoObject"),
            Format::new("webm", true, "VideoObject"),
            // Special `unknown` format which defines handling
            // of file types that are not registered above
            Format::unknown(),
        ];

        let formats = formats
            .into_iter()
            .map(|format| (format.name.clone(), format))
            .collect();

        Formats { formats }
    }
}

impl Formats {
    /// Match a format name to a `Format`
    pub fn match_name(&self, name: &str) -> Format {
        match self.formats.get(&name.to_lowercase()) {
            Some(format) => format.clone(),
            None => Format::unknown(),
        }
    }

    /// Match a file path to a `Format`
    pub fn match_path<P: AsRef<Path>>(&self, path: &P) -> Format {
        let path = path.as_ref();
        // Use file extension
        let name = match path.extension() {
            Some(ext) => ext,
            // Fallback to the filename if no extension
            None => match path.file_name() {
                Some(name) => name,
                // Fallback to the provided "path"
                None => path.as_os_str(),
            },
        };

        self.match_name(&name.to_string_lossy().to_string())
    }
}

pub static FORMATS: Lazy<Formats> = Lazy::new(Formats::default);

/// Get JSON Schemas for this modules
pub fn schemas() -> Result<serde_json::Value> {
    let schemas = serde_json::Value::Array(vec![schemas::generate::<Format>()?]);
    Ok(schemas)
}
