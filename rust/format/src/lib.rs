//! Provides the `Format` enum and utility functions for working with document formats

use std::{
    fmt::Display,
    path::{Path, PathBuf},
    str::FromStr,
};

use eyre::{Report, Result};
use serde_with::{DeserializeFromStr, SerializeDisplay};
use strum::EnumIter;

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
#[serde_with(crate = "serde_with")]
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
    // Bibliographic data formats
    Csl,
    Cff,
    Bibtex,
    // Tabular data formats
    Csv,
    Tsv,
    Parquet,
    Arrow,
    // Spreadsheet formats
    Xlsx,
    Xls,
    Ods,
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
    Mov,
    Mp4,
    Ogv,
    WebM,
    Wmv,
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
            Arrow => "Apache Arrow",
            AsciiMath => "AsciiMath",
            Avi => "AVI",
            Bash => "Bash",
            Bibtex => "BibTeX",
            Cbor => "CBOR",
            CborZstd => "CBOR+Zstd",
            Cff => "Citation File Format",
            Csl => "CSL-JSON",
            Css => "CSS",
            Csv => "CSV",
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
            Json5 => "JSON5",
            JsonLd => "JSON-LD",
            JsonZip => "JSON+Zip",
            Koenig => "Koenig JSON",
            Latex => "LaTeX",
            Lexical => "Lexical JSON",
            Llmd => "LLM Markdown",
            Markdown => "Markdown",
            Meca => "Meca",
            Mermaid => "Mermaid",
            Mkv => "Matroska",
            Mov => "QuickTime",
            Mp3 => "MPEG-3",
            Mp4 => "MPEG-4",
            Myst => "MyST Markdown",
            Ods => "OpenDocument Spreadsheet",
            Odt => "OpenDocument Text",
            Ogg => "Ogg Vorbis",
            Ogv => "Ogg Vorbis Video",
            Pandoc => "Pandoc AST",
            Parquet => "Apache Parquet",
            Pdf => "PDF",
            PmcOa => "PubMed Central OA Package",
            Png => "PNG",
            Python => "Python",
            Qmd => "Quarto Markdown",
            R => "R",
            Rhai => "Rhai",
            Rnw => "R+LaTeX",
            Shell => "Shell",
            Smd => "Stencila Markdown",
            Sql => "SQL",
            Svg => "SVG",
            Swb => "Stencila Web Bundle",
            Tailwind => "Tailwind",
            Tex => "TeX",
            Text => "Plain text",
            Toml => "TOML",
            Tsv => "TSV",
            Wav => "WAV",
            WebM => "WebM",
            WebP => "WebP",
            Wmv => "Windows Media Video",
            Xls => "Microsoft Excel XLS",
            Xlsx => "Microsoft Excel XLSX",
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

    /// Is this a binary format?
    pub fn is_binary(&self) -> bool {
        use Format::*;
        matches!(
            self,
            Arrow
                | Cbor
                | CborZstd
                | Docx
                | JsonZip
                | Meca
                | Ods
                | Odt
                | Parquet
                | Pdf
                | PmcOa
                | Xls
                | Xlsx
        ) || self.is_media()
    }

    /// Is this a media format (image, audio or video) ?
    pub fn is_media(&self) -> bool {
        self.is_image() || self.is_audio() || self.is_video()
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
        matches!(self, Avi | Mkv | Mov | Mp4 | Ogv | WebM | Wmv)
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
            "arrow" => Arrow,
            "asciimath" => AsciiMath,
            "avi" => Avi,
            "bash" => Bash,
            "bibtex" | "bib" => Bibtex,
            "cbor" => Cbor,
            "czst" | "cborzstd" | "cbor.zstd" => CborZstd,
            "cff" => Cff,
            "csl" => Csl,
            "css" => Css,
            "csv" => Csv,
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
            "jpeg" | "jpg" => Jpeg,
            "json" => Json,
            "json5" => Json5,
            "jsonld" | "json-ld" => JsonLd,
            "jsonzip" | "json.zip" => JsonZip,
            "koenig" => Koenig,
            "latex" => Latex,
            "lexical" => Lexical,
            "llmd" | "llmmd" => Llmd,
            "markdown" | "md" => Markdown,
            "meca" => Meca,
            "mermaid" => Mermaid,
            "mkv" => Mkv,
            "mov" => Mov,
            "mp3" => Mp3,
            "mp4" => Mp4,
            "myst" => Myst,
            "ods" => Ods,
            "odt" => Odt,
            "ogg" => Ogg,
            "ogv" => Ogv,
            "pandoc" => Pandoc,
            "parquet" => Parquet,
            "pdf" => Pdf,
            "pmcoa" => PmcOa,
            "png" => Png,
            "python" | "py" => Python,
            "qmd" => Qmd,
            "r" => R,
            "rhai" => Rhai,
            "rnw" => Rnw,
            "shell" | "sh" => Shell,
            "smd" => Smd,
            "sql" => Sql,
            "svg" => Svg,
            "swb" => Swb,
            "tailwind" => Tailwind,
            "tex" => Tex,
            "text" | "txt" => Text,
            "toml" => Toml,
            "tsv" | "tab" => Tsv,
            "unknown" => Unknown,
            "wav" => Wav,
            "webm" => WebM,
            "webp" => WebP,
            "wmv" => Wmv,
            "xls" => Xls,
            "xlsx" => Xlsx,
            "yaml" | "yml" => Yaml,
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
            // Only include explicit mappings where Format::from_name would not match
            // the part after the slash, or where there are special cases
            "application/cbor+zstd" => Ok(CborZstd),
            "application/json+zip" => Ok(JsonZip),
            "application/ld+json" => Ok(JsonLd),
            "application/vnd.citationstyles.csl+json" => Ok(Csl),
            "application/vnd.ms-excel" => Ok(Xls),
            "application/vnd.oasis.opendocument.spreadsheet" => Ok(Ods),
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet" => Ok(Xlsx),
            "audio/mp4" => Ok(Aac), // M4A files use audio/mp4 MIME type but are AAC format
            "audio/mpeg" => Ok(Mp3),
            "image/svg+xml" => Ok(Svg),
            "text/jats+xml" => Ok(Jats),
            "text/plain" => Ok(Text),
            "video/quicktime" => Ok(Mov),
            "video/x-msvideo" => Ok(Avi),
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
            Xlsx => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
            Xls => "application/vnd.ms-excel".to_string(),
            Ods => "application/vnd.oasis.opendocument.spreadsheet".to_string(),
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
            Arrow => "arrow",
            AsciiMath => "asciimath",
            Avi => "avi",
            Bash => "bash",
            Bibtex => "bibtex",
            Cbor => "cbor",
            CborZstd => "czst",
            Cff => "cff",
            Csl => "csl",
            Css => "css",
            Csv => "csv",
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
            Json5 => "json5",
            JsonLd => "jsonld",
            JsonZip => "json.zip",
            Koenig => "koenig",
            Latex => "latex",
            Lexical => "lexical",
            Llmd => "llmd",
            Markdown => "md",
            Meca => "meca",
            Mermaid => "mermaid",
            Mkv => "mkv",
            Mov => "mov",
            Mp3 => "mp3",
            Mp4 => "mp4",
            Myst => "myst",
            Ods => "ods",
            Odt => "odt",
            Ogg => "ogg",
            Ogv => "ogv",
            Pandoc => "pandoc",
            Parquet => "parquet",
            Pdf => "pdf",
            PmcOa => "pmcoa",
            Png => "png",
            Python => "python",
            Qmd => "qmd",
            R => "r",
            Rhai => "rhai",
            Rnw => "rnw",
            Shell => "shell",
            Smd => "smd",
            Sql => "sql",
            Svg => "svg",
            Swb => "swb",
            Tailwind => "tailwind",
            Tex => "tex",
            Text => "text",
            Toml => "toml",
            Tsv => "tsv",
            Wav => "wav",
            WebM => "webm",
            WebP => "webp",
            Wmv => "wmv",
            Xls => "xls",
            Xlsx => "xlsx",
            Yaml => "yaml",
            Other(name) => name,
            Unknown => "unknown",
        })
    }
}

#[cfg(test)]
mod test {
    use strum::IntoEnumIterator;

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
    fn from_media_type() -> Result<()> {
        // Test cases: (mime_type, expected_format)
        let test_cases = vec![
            // Explicit mappings where Format::from_name wouldn't match
            ("application/cbor+zstd", Format::CborZstd),
            ("application/json+zip", Format::JsonZip),
            ("application/ld+json", Format::JsonLd),
            ("application/vnd.citationstyles.csl+json", Format::Csl),
            ("application/vnd.ms-excel", Format::Xls),
            (
                "application/vnd.oasis.opendocument.spreadsheet",
                Format::Ods,
            ),
            (
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
                Format::Xlsx,
            ),
            ("audio/mp4", Format::Aac),
            ("audio/mpeg", Format::Mp3),
            ("image/jpeg", Format::Jpeg),
            ("image/svg+xml", Format::Svg),
            ("text/jats+xml", Format::Jats),
            ("text/plain", Format::Text),
            ("video/quicktime", Format::Mov),
            ("video/x-msvideo", Format::Avi),
            // Fallback cases that should work via Format::from_name
            ("application/cbor", Format::Cbor),
            ("application/json", Format::Json),
            ("application/yaml", Format::Yaml),
            ("text/markdown", Format::Markdown),
            ("text/html", Format::Html),
            ("image/png", Format::Png),
            ("image/gif", Format::Gif),
            ("image/webp", Format::WebP),
            ("audio/wav", Format::Wav),
            ("audio/ogg", Format::Ogg),
            ("audio/aac", Format::Aac),
            ("audio/flac", Format::Flac),
            ("video/mp4", Format::Mp4),
            ("video/webm", Format::WebM),
            ("video/avi", Format::Avi),
        ];

        for (mime_type, expected) in test_cases {
            let result = Format::from_media_type(mime_type)?;
            assert_eq!(
                result, expected,
                "Failed for mime type '{}': expected {:?}, got {:?}",
                mime_type, expected, result
            );
        }

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
