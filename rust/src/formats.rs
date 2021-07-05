use crate::utils::schemas;
use defaults::Defaults;
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
#[derive(Clone, Debug, Defaults, JsonSchema, Serialize)]
#[schemars(deny_unknown_fields)]
pub struct Format {
    /// The lowercase name of the format e.g. `md`, `docx`, `dockerfile`
    pub name: String,

    /// Whether or not the format should be considered binary
    /// e.g. not to be displayed in a text / code editor
    pub binary: bool,

    /// Whether or not previews should be generated for files of
    /// this format. e.g. a `.py` is not binary, but should not
    /// necessarily have a preview opened for it.
    pub preview: bool,

    /// The type of `CreativeWork` that this format is expected to be.
    /// This will be `None` for data serialization formats such as
    /// JSON or YAML which have no expected type (the actual type is
    /// embedded in the data).
    #[serde(rename = "type")]
    pub type_: Option<String>,

    /// Any additional extensions (other than it's name) that this format
    /// should match against.
    pub extensions: Vec<String>,
}

impl Format {
    /// Create a new file format
    pub fn new(name: &str, binary: bool, preview: bool, type_: &str) -> Format {
        Format {
            name: name.into(),
            binary,
            preview,
            type_: if type_.is_empty() {
                None
            } else {
                Some(type_.to_string())
            },
            ..Default::default()
        }
    }

    /// Create a new file format with mutliple matching extensions
    pub fn new_extensions(
        name: &str,
        binary: bool,
        preview: bool,
        type_: &str,
        extensions: &[&str],
    ) -> Format {
        let mut format = Format::new(name, binary, preview, type_);
        format.extensions = extensions.iter().map(|s| s.to_string()).collect();
        format
    }

    /// Create the special `directory` format used on `File` objects
    /// that are directories
    pub fn directory() -> Format {
        Format {
            name: "dir".into(),
            binary: true,
            preview: false,
            type_: Some("Collection".into()),
            ..Default::default()
        }
    }

    /// Create the special `unregistered` file format where all we
    /// have is the name e.g. from a file extension
    pub fn unregistered(name: &str) -> Format {
        Format {
            name: name.into(),
            binary: true,
            preview: false,
            type_: None,
            ..Default::default()
        }
    }

    /// Create the special `unknown` file format where we do not
    /// even know the name.
    pub fn unknown() -> Format {
        Format::unregistered("unknown")
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
            // Data serialization formats. These may be used
            // to store documents so `preview: true`.
            Format::new("json", false, true, ""),
            Format::new("json5", false, true, ""),
            Format::new("toml", false, true, ""),
            Format::new("xml", false, true, ""),
            Format::new("yaml", false, true, ""),
            // Code formats
            Format::new("dockerfile", false, false, ""),
            Format::new("js", false, false, ""),
            Format::new("makefile", false, false, ""),
            Format::new("py", false, false, ""),
            Format::new("r", false, false, ""),
            Format::new("sh", false, false, ""),
            Format::new("ts", false, false, ""),
            // Article formats
            Format::new("docx", true, true, "Article"),
            Format::new("html", false, true, "Article"),
            Format::new("ipynb", false, true, "Article"),
            Format::new("md", false, true, "Article"),
            Format::new("odt", true, true, "Article"),
            Format::new("rmd", false, true, "Article"),
            Format::new_extensions("latex", false, true, "Article", &["tex"]),
            Format::new("txt", false, true, "Article"),
            // Audio formats
            Format::new("flac", true, true, "AudioObject"),
            Format::new("mp3", true, true, "AudioObject"),
            Format::new("ogg", true, true, "AudioObject"),
            // Image formats
            Format::new("gif", true, true, "ImageObject"),
            Format::new("jpeg", true, true, "ImageObject"),
            Format::new("jpg", true, true, "ImageObject"),
            Format::new("png", true, true, "ImageObject"),
            // Video formats
            Format::new("3gp", true, true, "VideoObject"),
            Format::new("mp4", true, true, "VideoObject"),
            Format::new("ogv", true, true, "VideoObject"),
            Format::new("webm", true, true, "VideoObject"),
            // Specials
            Format::directory(),
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
            None => Format::unregistered(name),
        }
    }

    /// Match a file path to a `Format`
    pub fn match_path<P: AsRef<Path>>(&self, path: &P) -> Format {
        let path = path.as_ref();

        // Get name from file extension, or filename if no extension
        let name = match path.extension() {
            Some(ext) => ext,
            None => match path.file_name() {
                Some(name) => name,
                // Fallback to the provided "path"
                None => path.as_os_str(),
            },
        };
        let name = name.to_string_lossy().to_string();
        if let Some(format) = self.formats.get(&name.to_lowercase()) {
            return format.clone();
        }

        // Try matching against "extra" extensions
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_string();
            for format in self.formats.values() {
                if format.extensions.contains(&ext) {
                    return format.clone();
                }
            }
        }

        Format::unregistered(&name)
    }
}

pub static FORMATS: Lazy<Formats> = Lazy::new(Formats::default);

/// Get JSON Schemas for this modules
pub fn schemas() -> Result<serde_json::Value> {
    let schemas = serde_json::Value::Array(vec![schemas::generate::<Format>()?]);
    Ok(schemas)
}
