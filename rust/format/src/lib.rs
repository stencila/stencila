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
    Hash,
    EnumIter,
    SerializeDisplay,
    DeserializeFromStr,
)]
#[strum(crate = "common::strum")]
#[serde_with(crate = "common::serde_with")]
pub enum Format {
    // Grouped and ordered as most appropriate for documentation
    // Markup formats
    Dom,
    Html,
    Jats,
    // Markdown and derivatives
    Markdown, // Commonmark Markdown with GitHub Flavored Markdown extensions (as in the `markdown` crate)
    Smd,
    Qmd,
    Myst,
    Llmd,
    // Typesetting / text formats
    Latex,
    Rnw,
    Pdf,
    Text,
    // Notebook formats
    Ipynb,
    // Word processor formats
    Docx,
    GDocx,
    Odt,
    // Math languages
    AsciiMath,
    Tex,
    // Programming languages
    Bash,
    Shell,
    JavaScript,
    Jinja,
    Python,
    R,
    Rhai,
    // Database query languages
    Cypher,
    Sql,
    DocsQL,
    // Diagramming languages
    Dot,
    Mermaid,
    // Styling languages
    Tailwind,
    Css,
    // Data serialization formats
    Json,
    JsonZip,
    Json5,
    JsonLd,
    Cbor,
    CborZstd,
    Toml,
    Yaml,
    Lexical,
    Koenig,
    Pandoc,
    Csl,
    Cff,
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
    // Directories, bundles and archives
    Directory,
    Swb,
    Meca,
    PmcOa,
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
            AsciiMath => "AsciiMath",
            Avi => "AVI",
            Bash => "Bash",
            Cbor => "CBOR",
            CborZstd => "CBOR+Zstd",
            Cff => "Citation File Format",
            Csl => "CSL-JSON",
            Css => "CSS",
            Cypher => "Cypher",
            Debug => "Debug",
            Directory => "Directory",
            DocsQL => "Stencila DocsQL",
            Docx => "Microsoft Word DOCX",
            Dom => "DOM HTML",
            Dot => "Graphviz DOT",
            Flac => "FLAC",
            GDocx => "Google Docs DOCX",
            Gif => "GIF",
            Html => "HTML",
            Ipynb => "IPYNB",
            Jats => "JATS",
            JavaScript => "JavaScript",
            Jinja => "Jinja",
            Jpeg => "JPEG",
            Json => "JSON",
            JsonZip => "JSON+Zip",
            Json5 => "JSON5",
            JsonLd => "JSON-LD",
            Koenig => "Koenig JSON",
            Latex => "LaTeX",
            Lexical => "Lexical JSON",
            Llmd => "LLM Markdown",
            Markdown => "Markdown",
            Meca => "Meca",
            Mermaid => "Mermaid",
            Mkv => "Matroska",
            Mp3 => "MPEG-3",
            Mp4 => "MPEG-4",
            Myst => "MyST Markdown",
            Rnw => "R+LaTeX",
            Odt => "OpenDocument ODT",
            Ogg => "Ogg Vorbis",
            Ogv => "Ogg Vorbis Video",
            Pandoc => "Pandoc AST",
            PmcOa => "PubMed Central OA Package",
            Pdf => "PDF",
            Png => "PNG",
            Python => "Python",
            Qmd => "Quarto Markdown",
            R => "R",
            Rhai => "Rhai",
            Shell => "Shell",
            Sql => "SQL",
            Smd => "Stencila Markdown",
            Swb => "Stencila Web Bundle",
            Svg => "SVG",
            Tailwind => "Tailwind",
            Tex => "TeX",
            Text => "Plain text",
            Toml => "TOML",
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
            Json | JsonZip | Json5 | JsonLd | Cbor | CborZstd | Yaml => 0,
            Html | Jats | Markdown => 1,
            _ => u8::MAX,
        }
    }

    /// Is this an unknown format?
    pub fn is_unknown(&self) -> bool {
        use Format::*;
        matches!(self, Unknown)
    }

    /// Is this a lossless format for Stencila nodes?
    pub fn is_lossless(&self) -> bool {
        use Format::*;
        matches!(
            self,
            Cbor | CborZstd | Json | Json5 | JsonLd | JsonZip | Yaml
        )
    }

    /// Is this a lossy format for Stencila nodes?
    pub fn is_lossy(&self) -> bool {
        !self.is_lossless()
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

    /// Is this format a flavor of JSON?
    pub fn is_json_flavor(&self) -> bool {
        use Format::*;
        matches!(self, Json | Json5 | JsonLd | Ipynb | Lexical | Koenig)
    }

    /// Is this format a flavor of Markdown?
    pub fn is_markdown_flavor(&self) -> bool {
        use Format::*;
        matches!(self, Markdown | Smd | Myst | Qmd | Llmd)
    }

    /// Is this format a flavor of XML?
    pub fn is_xml_flavor(&self) -> bool {
        use Format::*;
        matches!(self, Jats)
    }

    /// Resolve a [`Format`] from a name for the format
    ///
    /// Also includes kernel names e.g "nodejs" & "quickjs" so that we
    /// can associate those names with a language
    pub fn from_name(name: &str) -> Self {
        use Format::*;
        match name.to_lowercase().trim() {
            "aac" => Aac,
            "asciimath" => AsciiMath,
            "avi" => Avi,
            "bash" => Bash,
            "cbor" => Cbor,
            "cborzstd" | "cbor.zstd" => CborZstd,
            "cff" => Cff,
            "csl" => Csl,
            "css" => Css,
            "cypher" | "cyp" | "cql" => Cypher,
            "debug" => Debug,
            "directory" | "dir" => Directory,
            "docsql" => DocsQL,
            "docx" => Docx,
            "dom" | "dom.html" => Dom,
            "dot" => Dot,
            "flac" => Flac,
            "gdocx" => GDocx,
            "gif" => Gif,
            "html" => Html,
            "ipynb" => Ipynb,
            "jats" | "jats.xml" => Jats,
            "javascript" | "js" | "nodejs" | "quickjs" => JavaScript,
            "jinja" => Jinja,
            "jpeg" => Jpeg,
            "json" => Json,
            "jsonzip" | "json.zip" => JsonZip,
            "json5" => Json5,
            "jsonld" | "json-ld" => JsonLd,
            "koenig" => Koenig,
            "latex" => Latex,
            "lexical" => Lexical,
            "llmd" | "llmmd" => Llmd,
            "markdown" | "md" => Markdown,
            "meca" => Meca,
            "mermaid" => Mermaid,
            "myst" => Myst,
            "mkv" => Mkv,
            "mp3" => Mp3,
            "mp4" => Mp4,
            "rnw" => Rnw,
            "odt" => Odt,
            "ogg" => Ogg,
            "ogv" => Ogv,
            "pandoc" => Pandoc,
            "pmcoa" => PmcOa,
            "png" => Png,
            "pdf" => Pdf,
            "python" | "py" => Python,
            "qmd" => Qmd,
            "r" => R,
            "rhai" => Rhai,
            "shell" | "sh" => Shell,
            "sql" => Sql,
            "smd" => Smd,
            "svg" => Svg,
            "swb" => Swb,
            "tailwind" => Tailwind,
            "tex" => Tex,
            "text" | "txt" => Text,
            "toml" => Toml,
            "wav" => Wav,
            "webm" => WebM,
            "webp" => WebP,
            "yaml" | "yml" => Yaml,
            "unknown" => Unknown,
            _ => Other(name.to_string()),
        }
    }

    /// Resolve a [`Format`] from a file path
    pub fn from_path(path: &Path) -> Self {
        use Format::*;

        if path.is_dir() {
            return Directory;
        }

        // Catch "double extensions" here
        let path_string = path.to_string_lossy();
        for (end, format) in [
            (".cbor.zstd", CborZstd),
            (".dom.html", Dom),
            (".jats.xml", Jats),
            (".json.zip", JsonZip),
        ] {
            if path_string.ends_with(end) {
                return format;
            }
        }

        // Resolve from extension or filename (if no extension)
        let name = match path.extension() {
            Some(ext) => ext,
            None => match path.file_name() {
                Some(name) => name,
                None => path.as_os_str(),
            },
        };
        Self::from_name(&name.to_string_lossy())
    }

    /// Resolve a [`Format`] from a URL
    pub fn from_url<S: AsRef<str>>(string: S) -> Self {
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
            "application/cbor+zstd" => Ok(CborZstd),
            "application/json" => Ok(Json),
            "application/json+zip" => Ok(JsonZip),
            "application/ld+json" => Ok(JsonLd),
            "application/yaml" => Ok(Yaml),
            "application/vnd.citationstyles.csl+json" => Ok(Csl),
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

    /// Resolve a [`Format`] from a HTTP `Content-Type` header
    pub fn from_content_type<S: AsRef<str>>(string: S) -> Result<Self> {
        let content_type = string.as_ref();
        // Strip suffixes like "; charset=utf-8" from content type
        let media_type = content_type
            .split(';')
            .next()
            .unwrap_or(content_type)
            .trim();
        Format::from_media_type(media_type)
    }

    /// Get the media type of the format
    pub fn media_type(&self) -> String {
        // This is home grown implementation avoids depending on the `mime_guess`
        // crate for no other reason that adding a dependency. That may be reviewed in the future.
        use Format::*;
        match self {
            Cbor => "application/cbor".to_string(),
            CborZstd => "application/cbor+zstd".to_string(),
            Json => "application/json".to_string(),
            JsonZip => "application/json+zip".to_string(),
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
        use Format::*;
        match self {
            Jats => "jats.xml".to_string(),
            _ => self.to_string(),
        }
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
            AsciiMath => "asciimath",
            Avi => "avi",
            Bash => "bash",
            Cbor => "cbor",
            CborZstd => "cbor.zstd",
            Cff => "cff",
            Csl => "csl",
            Css => "css",
            Cypher => "cypher",
            Debug => "debug",
            Directory => "directory",
            DocsQL => "docsql",
            Docx => "docx",
            Dom => "dom.html",
            Dot => "dot",
            Flac => "flac",
            GDocx => "gdocx",
            Gif => "gif",
            Html => "html",
            Ipynb => "ipynb",
            Jats => "jats",
            JavaScript => "js",
            Jinja => "jinja",
            Jpeg => "jpeg",
            Json => "json",
            JsonZip => "json.zip",
            Json5 => "json5",
            JsonLd => "jsonld",
            Koenig => "koenig",
            Latex => "latex",
            Lexical => "lexical",
            Llmd => "llmd",
            Markdown => "md",
            Meca => "meca",
            Mermaid => "mermaid",
            Mkv => "mkv",
            Mp3 => "mp3",
            Mp4 => "mp4",
            Myst => "myst",
            Rnw => "rnw",
            Odt => "odt",
            Ogg => "ogg",
            Ogv => "ogv",
            Pandoc => "pandoc",
            PmcOa => "pmcoa",
            Pdf => "pdf",
            Png => "png",
            Python => "python",
            Qmd => "qmd",
            R => "r",
            Rhai => "rhai",
            Shell => "shell",
            Sql => "sql",
            Svg => "svg",
            Smd => "smd",
            Swb => "swb",
            Tailwind => "tailwind",
            Tex => "tex",
            Text => "text",
            Toml => "toml",
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
    fn from_url() {
        assert_eq!(Format::from_url("Python"), Format::Python);
        assert_eq!(Format::from_url("python"), Format::Python);
        assert_eq!(Format::from_url("Py"), Format::Python);
        assert_eq!(Format::from_url("py"), Format::Python);

        assert_eq!(Format::from_url("cborZstd"), Format::CborZstd);
        assert_eq!(Format::from_url("cborzstd"), Format::CborZstd);

        assert_eq!(Format::from_url("mp3"), Format::Mp3);

        assert_eq!(Format::from_url("file.avi"), Format::Avi);

        assert_eq!(Format::from_url("https://example.org/cat.mp4"), Format::Mp4);

        assert_eq!(
            Format::from_url("file.foo"),
            Format::Other("foo".to_string())
        );
        assert_eq!(Format::from_url("foo"), Format::Other("foo".to_string()));
    }

    #[test]
    fn from_content_type() -> Result<()> {
        // Test basic content types
        assert_eq!(Format::from_content_type("application/json")?, Format::Json);
        assert_eq!(Format::from_content_type("text/plain")?, Format::Text);
        assert_eq!(
            Format::from_content_type("text/markdown")?,
            Format::Markdown
        );
        assert_eq!(Format::from_content_type("text/jats+xml")?, Format::Jats);

        // Test content types with charset suffix
        assert_eq!(
            Format::from_content_type("application/json; charset=utf-8")?,
            Format::Json
        );
        assert_eq!(
            Format::from_content_type("text/plain; charset=UTF-8")?,
            Format::Text
        );
        assert_eq!(
            Format::from_content_type("text/html; charset=utf-8")?,
            Format::Html
        );

        // Test content types with multiple parameters
        assert_eq!(
            Format::from_content_type("text/html; charset=utf-8; boundary=something")?,
            Format::Html
        );

        // Test content types with no space after semicolon
        assert_eq!(
            Format::from_content_type("text/html;charset=utf-8")?,
            Format::Html
        );
        assert_eq!(
            Format::from_content_type("application/json;charset=UTF-8")?,
            Format::Json
        );

        // Test edge cases
        assert_eq!(Format::from_content_type("text/html;")?, Format::Html);
        assert_eq!(
            Format::from_content_type("  text/html  ; charset=utf-8")?,
            Format::Html
        );

        // Test unknown content types
        assert_eq!(
            Format::from_content_type("application/octet-stream")?,
            Format::Other("octet-stream".to_string())
        );
        assert_eq!(Format::from_content_type("application/pdf")?, Format::Pdf);

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
