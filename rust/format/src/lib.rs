//! Provides the `Format` enum and utility functions for working with document formats

use std::{
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};

use common::{
    eyre::{Report, Result},
    serde_with::{DeserializeFromStr, SerializeDisplay},
    strum::EnumIter,
};

#[derive(
    Debug,
    Default,
    Clone,
    PartialOrd,
    PartialEq,
    Eq,
    Ord,
    EnumIter,
    SerializeDisplay,
    DeserializeFromStr,
)]
#[strum(crate = "common::strum")]
#[serde_with(crate = "common::serde_with")]
pub enum Format {
    // Grouped and ordered as most appropriate for documentation
    // CRDTs
    Article,
    // Markup formats
    Dom,
    Html,
    Jats,
    // Text formats
    Markdown,
    Text,
    // Math languages
    AsciiMath,
    Latex,
    Tex,
    // Programming languages
    Bash,
    Shell,
    JavaScript,
    Python,
    R,
    Rhai,
    // Data serialization formats
    Json,
    Json5,
    JsonLd,
    Cbor,
    CborZst,
    Yaml,
    // Image formats
    Gif,
    Jpeg,
    Png,
    Svg,
    WebP,
    // Audio formats
    Aac,
    Flac,
    Mp3,
    Ogg,
    Wav,
    // Video formats
    Avi,
    Mkv,
    Mp4,
    Ogv,
    WebM,
    // Directory
    Directory,
    // Development focussed formats
    Debug,
    // Other arbitrary format, not listed above
    Other(String),
    // Unknown format
    #[default]
    Unknown,
}

impl Format {
    /// Get the name of a format to use when displayed
    pub fn name(&self) -> &str {
        use Format::*;
        match self {
            Aac => "AAC",
            Article => "Stencila Article",
            AsciiMath => "AsciiMath",
            Avi => "AVI",
            Bash => "Bash",
            Cbor => "CBOR",
            CborZst => "CBOR+Zstandard",
            Debug => "Debug",
            Directory => "Directory",
            Dom => "DOM HTML",
            Flac => "FLAC",
            Gif => "GIF",
            Html => "HTML",
            Jats => "JATS",
            JavaScript => "JavaScript",
            Jpeg => "JPEG",
            Json => "JSON",
            Json5 => "JSON5",
            JsonLd => "JSON-LD",
            Latex => "LaTeX",
            Markdown => "Markdown",
            Mkv => "Matroska",
            Mp3 => "MPEG-3",
            Mp4 => "MPEG-4",
            Ogg => "Ogg Vorbis",
            Ogv => "Ogg Vorbis Video",
            Png => "PNG",
            Python => "Python",
            R => "R",
            Rhai => "Rhai",
            Shell => "Shell",
            Svg => "SVG",
            Tex => "TeX",
            Text => "Plain text",
            Wav => "WAV",
            WebM => "WebM",
            WebP => "WebP",
            Yaml => "YAML",
            Other(name) => name,
            Unknown => "Unknown",
        }
    }

    /// Get the rank of the preference for the format
    ///
    /// A lower rank indicates a higher preference. Used when resolving a
    /// URL path to a file when there is more than one file that matches the path.
    pub fn rank(&self) -> u8 {
        use Format::*;
        match self {
            Json | Json5 | JsonLd | Cbor | CborZst | Yaml => 0,
            Html | Jats | Markdown => 1,
            _ => u8::MAX,
        }
    }

    /// Is this an unknown format?
    pub fn is_unknown(&self) -> bool {
        use Format::*;
        matches!(self, Unknown)
    }

    /// Is this a document store format?
    pub fn is_store(&self) -> bool {
        use Format::*;
        matches!(self, Article)
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
    pub fn from_name(name: &str) -> Self {
        use Format::*;
        match name.to_lowercase().trim() {
            "aac" => Aac,
            "avi" => Avi,
            "bash" => Bash,
            "cbor" => Cbor,
            "cborzst" | "cbor.zstd" => CborZst,
            "debug" => Debug,
            "directory" | "dir" => Directory,
            "dom" | "dom.html" => Dom,
            "flac" => Flac,
            "gif" => Gif,
            "html" => Html,
            "jats" | "jats.xml" => Jats,
            "javascript" | "js" => JavaScript,
            "jpeg" => Jpeg,
            "json" => Json,
            "json5" => Json5,
            "jsonld" | "json-ld" => JsonLd,
            "latex" => Latex,
            "markdown" | "md" => Markdown,
            "mkv" => Mkv,
            "mp3" => Mp3,
            "mp4" => Mp4,
            "ogg" => Ogg,
            "ogv" => Ogv,
            "png" => Png,
            "python" | "py" => Python,
            "r" => R,
            "rhai" => Rhai,
            "shell" | "sh" => Shell,
            "sta" => Article,
            "svg" => Svg,
            "tex" => Tex,
            "text" | "txt" => Text,
            "wav" => Wav,
            "webm" => WebM,
            "webp" => WebP,
            "yaml" | "yml" => Yaml,
            "unknown" => Unknown,
            _ => Other(name.to_string()),
        }
    }

    /// Resolve a [`Format`] from a file path
    pub fn from_path(path: &Path) -> Result<Self> {
        if path.is_dir() {
            return Ok(Format::Directory);
        }

        // Catch "double extensions" here
        let path_string = path.to_string_lossy();
        if path_string.ends_with(".dom.html") {
            return Ok(Format::Dom);
        }
        if path_string.ends_with(".jats.xml") {
            return Ok(Format::Jats);
        }
        if path_string.ends_with(".cbor.zst") {
            return Ok(Format::CborZst);
        }

        let name = match path.extension() {
            Some(ext) => ext,
            None => match path.file_name() {
                Some(name) => name,
                None => path.as_os_str(),
            },
        };

        Ok(Self::from_name(&name.to_string_lossy()))
    }

    /// Resolve a [`Format`] from a URL
    pub fn from_url<S: AsRef<str>>(string: S) -> Result<Self> {
        Self::from_path(&PathBuf::from(string.as_ref()))
    }

    /// Resolve a [`Format`] from a IANA media type
    ///
    /// See https://www.iana.org/assignments/media-types/media-types.xhtml
    pub fn from_media_type<S: AsRef<str>>(string: S) -> Result<Self> {
        // This is home grown implementation avoids depending on the `mime_guess`
        // crate for no other reason that adding a dependency. That may be reviewed in the future.
        let media_type = string.as_ref();

        use Format::*;
        match media_type {
            "application/cbor" => Ok(Cbor),
            "application/cbor+zstd" => Ok(CborZst),
            "application/json" => Ok(Json),
            "application/ld+json" => Ok(JsonLd),
            "application/yaml" => Ok(Yaml),
            "text/jats+xml" => Ok(Jats),
            "text/markdown" => Ok(Markdown),
            "text/plain" => Ok(Text),
            _ => {
                let name = if let Some((.., name)) = media_type.split_once('/') {
                    name
                } else {
                    media_type
                };
                Ok(Self::from_name(name))
            }
        }
    }

    /// Get the media type of the format
    pub fn media_type(&self) -> String {
        // This is home grown implementation avoids depending on the `mime_guess`
        // crate for no other reason that adding a dependency. That may be reviewed in the future.
        use Format::*;
        match self {
            Cbor => "application/cbor".to_string(),
            CborZst => "application/cbor+zstd".to_string(),
            Json => "application/json".to_string(),
            JsonLd => "application/ld+json".to_string(),
            Yaml => "application/yaml".to_string(),
            Jats => "text/jats+xml".to_string(),
            Markdown => "text/markdown".to_string(),
            Text => "text/plain".to_string(),
            _ => {
                if self.is_audio() {
                    format!("audio/{}", self.extension())
                } else if self.is_image() {
                    format!("image/{}", self.extension())
                } else if self.is_video() {
                    format!("video/{}", self.extension())
                } else {
                    format!("text/{}", self.extension())
                }
            }
        }
    }

    /// Get the default file name extension for a format
    pub fn extension(&self) -> String {
        self.to_string()
    }
}

impl FromStr for Format {
    type Err = Report;

    fn from_str(name: &str) -> Result<Self> {
        Ok(Format::from_name(name))
    }
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Format::*;
        f.write_str(match self {
            Aac => "aac",
            Article => "sta",
            AsciiMath => "asciimath",
            Avi => "avi",
            Bash => "bash",
            Cbor => "cbor",
            CborZst => "cbor.zstd",
            Debug => "debug",
            Directory => "directory",
            Dom => "dom.html",
            Flac => "flac",
            Gif => "gif",
            Html => "html",
            Jats => "jats",
            JavaScript => "js",
            Jpeg => "jpeg",
            Json => "json",
            Json5 => "json5",
            JsonLd => "jsonld",
            Latex => "latex",
            Markdown => "markdown",
            Mkv => "mkv",
            Mp3 => "mp3",
            Mp4 => "mp4",
            Ogg => "ogg",
            Ogv => "ogv",
            Png => "png",
            Python => "python",
            R => "r",
            Rhai => "rhai",
            Shell => "shell",
            Svg => "svg",
            Tex => "tex",
            Text => "text",
            Wav => "wav",
            WebM => "webm",
            WebP => "webp",
            Yaml => "yaml",
            Other(name) => name,
            Unknown => "unknown",
        })
    }
}

#[cfg(test)]
mod test {
    use common::strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn from_url() -> Result<()> {
        assert_eq!(Format::from_url("Python")?, Format::Python);
        assert_eq!(Format::from_url("python")?, Format::Python);
        assert_eq!(Format::from_url("Py")?, Format::Python);
        assert_eq!(Format::from_url("py")?, Format::Python);

        assert_eq!(Format::from_url("cborZst")?, Format::CborZst);
        assert_eq!(Format::from_url("cborzst")?, Format::CborZst);

        assert_eq!(Format::from_url("mp3")?, Format::Mp3);

        assert_eq!(Format::from_url("file.avi")?, Format::Avi);

        assert_eq!(
            Format::from_url("https://example.org/cat.mp4")?,
            Format::Mp4
        );

        assert_eq!(
            Format::from_url("file.foo")?,
            Format::Other("foo".to_string())
        );
        assert_eq!(Format::from_url("foo")?, Format::Other("foo".to_string()));

        Ok(())
    }

    #[test]
    fn roundtrip() -> Result<()> {
        for format in Format::iter() {
            assert_eq!(format, Format::from_str(&format.to_string())?)
        }

        Ok(())
    }
}
