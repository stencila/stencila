use eyre::Result;
use once_cell::sync::Lazy;
use schemars::{schema_for, JsonSchema};
use serde::Serialize;
use serde_json::json;
use serde_with::skip_serializing_none;
use std::{collections::HashMap, path::Path};

/// A list of known formats
#[rustfmt::skip]
pub static FORMATS: Lazy<Formats> = Lazy::new(|| {
    let formats = vec![
        // Data serialization formats. These may be used
        // to store documents so `preview: true`.
        Format::new("json", false, true, FormatNodeType::Unknown, &[]),
        Format::new("json5", false, true, FormatNodeType::Unknown, &[]),
        Format::new("toml", false, true, FormatNodeType::Unknown, &[]),
        Format::new("xml", false, true, FormatNodeType::Unknown, &[]),
        Format::new("yaml", false, true, FormatNodeType::Unknown, &[]),

        // Code formats
        Format::new("dockerfile", false, false, FormatNodeType::SoftwareSourceCode, &[]),
        Format::new("js", false, false, FormatNodeType::SoftwareSourceCode, &["javascript"]),
        Format::new("makefile", false, false, FormatNodeType::SoftwareSourceCode, &[]),
        Format::new("py", false, false, FormatNodeType::SoftwareSourceCode, &["python"]),
        Format::new("r", false, false, FormatNodeType::SoftwareSourceCode, &[]),
        Format::new("sh", false, false, FormatNodeType::SoftwareSourceCode, &["shell"]),
        Format::new("ts", false, false, FormatNodeType::SoftwareSourceCode, &["typescript"]),

        // Article formats
        Format::new("docx", true, true, FormatNodeType::Article, &[]),
        Format::new("html", false, true, FormatNodeType::Article, &[]),
        Format::new("ipynb", false, true, FormatNodeType::Article, &[]),
        Format::new("md", false, true, FormatNodeType::Article, &[]),
        Format::new("odt", true, true, FormatNodeType::Article, &[]),
        Format::new("rmd", false, true, FormatNodeType::Article, &[]),
        Format::new("latex", false, true, FormatNodeType::Article, &["tex"]),

        // Audio formats
        Format::new("flac", true, true, FormatNodeType::AudioObject, &[]),
        Format::new("mp3", true, true, FormatNodeType::AudioObject, &[]),
        Format::new("ogg", true, true, FormatNodeType::AudioObject, &[]),

        // Image formats
        Format::new("gif", true, true, FormatNodeType::ImageObject, &[]),
        Format::new("jpg", true, true, FormatNodeType::ImageObject, &["jpeg"]),
        Format::new("png", true, true, FormatNodeType::ImageObject, &[]),
        Format::new("rpng", true, true, FormatNodeType::ImageObject, &[]),

        // Video formats
        Format::new("3gp", true, true, FormatNodeType::VideoObject, &[]),
        Format::new("mp4", true, true, FormatNodeType::VideoObject, &[]),
        Format::new("ogv", true, true, FormatNodeType::VideoObject, &[]),
        Format::new("webm", true, true, FormatNodeType::VideoObject, &[]),

        // Other
        Format::new("txt", false, false, FormatNodeType::Unknown, &[]),

        // Specials
        Format::directory(),
    ];

    let formats = formats
        .into_iter()
        .map(|format| (format.name.clone(), format))
        .collect();

    Formats { formats }
});

/// The type of format as a schema `Node` type
#[derive(Clone, Debug, JsonSchema, Serialize)]
pub enum FormatNodeType {
    Article,
    AudioObject,
    ImageObject,
    VideoObject,
    SoftwareSourceCode,
    Unknown,
}

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

    /// Whether HTML previews are normally supported for documents of
    /// this format. See also `Document.previewable` which indicates whether
    /// a HTML preview is supported for a particular document.
    pub preview: bool,

    /// The kind of format
    pub node_type: FormatNodeType,

    /// Any additional extensions (other than it's name) that this format
    /// should match against.
    pub aliases: Vec<String>,

    /// Whether or not this is a known format (ie.e. not automatically created)
    pub known: bool,
}

impl Format {
    /// Create a new file format
    pub fn new(
        name: &str,
        binary: bool,
        preview: bool,
        kind: FormatNodeType,
        extensions: &[&str],
    ) -> Format {
        Format {
            known: true,
            name: name.into(),
            binary,
            preview,
            node_type: kind,
            aliases: extensions.iter().map(|s| s.to_string()).collect(),
        }
    }

    /// Create the special `directory` format used on `File` objects
    /// that are directories
    pub fn directory() -> Format {
        Format {
            known: true,
            name: "dir".into(),
            binary: true,
            preview: false,
            node_type: FormatNodeType::Unknown,
            aliases: Vec::new(),
        }
    }

    /// Create the special `unknown` file format where all we
    /// have is the name e.g. from a file extension.
    pub fn unknown(name: &str) -> Format {
        Format {
            known: false,
            name: name.into(),
            // Set binary to false so that any unregistered format
            // will be at least shown in editor...
            binary: false,
            // ..but not have a preview
            preview: false,
            node_type: FormatNodeType::Unknown,
            aliases: Vec::new(),
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

impl Formats {
    /// Match a format name to a `Format`
    pub fn match_name(&self, name: &str) -> Format {
        match self.formats.get(&name.to_lowercase()) {
            Some(format) => format.clone(),
            None => {
                for format in self.formats.values() {
                    let name = name.to_string();
                    if format.aliases.contains(&name) {
                        return format.clone();
                    }
                }
                Format::unknown(name)
            }
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

        // Match that name
        self.match_name(&name.to_string_lossy().to_string())
    }
}

/// Get JSON Schemas for this module
pub fn schemas() -> Result<serde_json::Value> {
    Ok(json!([schema_for!(Format)]))
}
