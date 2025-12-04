use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::{self, Display},
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::{Args, ValueEnum};
use eyre::{Report, Result, bail};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use smart_default::SmartDefault;
use strum::{Display, EnumIter, EnumMessage, EnumString, IntoEnumIterator};
use tokio::{
    fs::{File, create_dir_all},
    io::{AsyncReadExt, AsyncWriteExt},
};

use stencila_codec_utils::git_info;
use stencila_format::Format;
use stencila_node_strip::StripScope;
use stencila_schema::{Article, Node};

// Re-exports for the convenience of internal crates implementing `Codec`
pub use async_trait::async_trait;
pub use eyre;
pub use stencila_codec_info::*;
pub use stencila_format;
pub use stencila_schema;

mod push;
pub use push::*;

mod references;
pub use references::*;

/// The direction of conversion
pub enum CodecDirection {
    Decode,
    Encode,
}

/// The availability of a codec on the current machine
#[derive(Debug, Display, Clone, Serialize, Deserialize)]
#[strum(serialize_all = "lowercase")]
pub enum CodecAvailability {
    /// Available on this machine
    Available,
    /// Available on this machine but requires installation of external binary
    Installable(String),
    /// Not available on this machine
    Unavailable,
}

/// A codec for decoding/encoding between Stencila Schema nodes and alternative formats
#[async_trait]
pub trait Codec: Sync + Send {
    /// The name of the codec
    ///
    /// Used when listing codecs and to select a codec when the user specifies
    /// the relevant options on the command line e.g. `--to jats-pandoc`.
    /// Should be kebab-cased.
    fn name(&self) -> &str;

    /// Get the availability of the kernel on the current machine
    fn availability(&self) -> CodecAvailability {
        CodecAvailability::Available
    }

    /// Is the kernel available on the current machine
    fn is_available(&self) -> bool {
        matches!(self.availability(), CodecAvailability::Available)
    }

    /// The level of support that the codec provides for decoding from a format
    #[allow(unused)]
    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        CodecSupport::None
    }

    /// The level of support that the codec provides for decoding from each format
    fn supports_from_formats(&self) -> BTreeMap<Format, CodecSupport> {
        Format::iter()
            .filter_map(|format| {
                let support = self.supports_from_format(&format);
                support.is_supported().then_some((format, support))
            })
            .collect()
    }

    /// The level of support that the codec provides for decoding for a [`NodeType`]
    #[allow(unused)]
    fn supports_from_type(&self, node_type: NodeType) -> CodecSupport {
        CodecSupport::None
    }

    /// The level of support that the codec provides for decoding for each [`NodeType`]
    fn supports_from_types(&self) -> BTreeMap<String, CodecSupport> {
        NodeType::iter()
            .filter_map(|node_type| {
                let support = self.supports_from_type(node_type);
                support
                    .is_supported()
                    .then_some((node_type.to_string(), support))
            })
            .collect()
    }

    /// Whether the codec supports decoding from bytes
    fn supports_from_bytes(&self) -> bool {
        false
    }

    /// Whether the codec supports decoding from string content
    fn supports_from_string(&self) -> bool {
        true
    }

    /// Whether the codec supports decoding from a file system path
    fn supports_from_path(&self) -> bool {
        true
    }

    /// The level of support that the codec provides for encoding to a format
    #[allow(unused)]
    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        CodecSupport::None
    }

    /// The level of support that the codec provides for encoding to each format
    fn supports_to_formats(&self) -> BTreeMap<Format, CodecSupport> {
        Format::iter()
            .filter_map(|format| {
                let support = self.supports_to_format(&format);
                support.is_supported().then_some((format, support))
            })
            .collect()
    }

    /// The level of support that the codec provides for encoding for a [`NodeType`]
    #[allow(unused)]
    fn supports_to_type(&self, node_type: NodeType) -> CodecSupport {
        CodecSupport::None
    }

    /// The level of support that the codec provides for encoding for each [`NodeType`]
    fn supports_to_types(&self) -> BTreeMap<String, CodecSupport> {
        NodeType::iter()
            .filter_map(|node_type| {
                let support = self.supports_to_type(node_type);
                support
                    .is_supported()
                    .then_some((node_type.to_string(), support))
            })
            .collect()
    }

    /// Whether the codec supports encoding to bytes
    fn supports_to_bytes(&self) -> bool {
        false
    }

    /// Whether the codec supports encoding to string content
    fn supports_to_string(&self) -> bool {
        true
    }

    /// Whether the codec supports encoding to a file system path
    fn supports_to_path(&self) -> bool {
        true
    }

    /// Get a list of types that the codec has either lossy decoding, or encoding, or both
    fn lossy_types(&self, direction: Option<CodecDirection>) -> Vec<NodeType> {
        let mut types = Vec::new();

        for node_type in NodeType::iter() {
            if (direction.is_none() || matches!(direction, Some(CodecDirection::Decode)))
                && self.supports_from_type(node_type).is_lossy()
                && !types.contains(&node_type)
            {
                types.push(node_type)
            }

            if (direction.is_none() || matches!(direction, Some(CodecDirection::Encode)))
                && self.supports_to_type(node_type).is_lossy()
                && !types.contains(&node_type)
            {
                types.push(node_type)
            }
        }

        types
    }

    /// Get a the default structuring options for the format
    fn structuring_options(&self, _format: &Format) -> StructuringOptions {
        StructuringOptions::none()
    }

    /// Decode a Stencila Schema node from bytes
    #[allow(unused_variables, clippy::wrong_self_convention)]
    async fn from_bytes(
        &self,
        bytes: &[u8],
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        bail!(
            "Decoding from bytes is not implemented for codec `{}`",
            self.name()
        )
    }

    /// Decode a Stencila Schema node from a string
    #[allow(unused_variables, clippy::wrong_self_convention)]
    async fn from_str(
        &self,
        str: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        self.from_bytes(str.as_bytes(), options).await
    }

    /// Decode a Stencila Schema node from a file
    ///
    /// This function reads the file as a string and passes that on to `from_str`
    /// for decoding. If working with binary formats, you should override this function
    /// to read the file as bytes instead.
    #[tracing::instrument(skip(self, file))]
    async fn from_file(
        &self,
        file: &mut File,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        if self.supports_from_bytes() {
            let mut content = Vec::new();
            file.read_to_end(&mut content).await?;
            self.from_bytes(&content, options).await
        } else {
            let mut content = String::new();
            file.read_to_string(&mut content).await?;
            self.from_str(&content, options).await
        }
    }

    /// Decode a Stencila Schema node from a file system path
    ///
    /// May be overridden by codecs to return an unedited version of the document
    /// which may be used to rebase edits
    #[tracing::instrument(skip(self))]
    async fn from_path(
        &self,
        path: &Path,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, Option<Node>, DecodeInfo)> {
        if !path.exists() {
            bail!("Path `{}` does not exist", path.display());
        }

        // Capture info needed to reverse changes unless flag is explicitly false
        let reproducible = options
            .as_ref()
            .and_then(|opts| opts.reproducible)
            .unwrap_or(true);

        let mut file = File::open(path).await?;
        let (mut node, info) = self.from_file(&mut file, options).await?;

        if reproducible && let Node::Article(Article { options, .. }) = &mut node {
            let git_info = git_info(path)?;
            options.repository = git_info.origin;
            options.path = git_info.path;
            options.commit = git_info.commit;
        }

        Ok((node, None, info))
    }

    /// Encode a Stencila Schema node to bytes
    #[allow(unused_variables)]
    async fn to_bytes(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(Vec<u8>, EncodeInfo)> {
        bail!(
            "Encoding to bytes is not implemented for codec `{}`",
            self.name()
        )
    }

    /// Encode a Stencila Schema node to a string
    #[allow(unused_variables)]
    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        self.to_bytes(node, options)
            .await
            .map(|(bytes, info)| (String::from_utf8_lossy(&bytes).to_string(), info))
    }

    /// Encode a Stencila Schema node to a file
    #[tracing::instrument(skip(self, node, file))]
    async fn to_file(
        &self,
        node: &Node,
        file: &mut File,
        options: Option<EncodeOptions>,
    ) -> Result<EncodeInfo> {
        let mut options = options.unwrap_or_default();
        if options.standalone.is_none() {
            options.standalone = Some(true);
        }

        let (content, info) = if self.supports_to_bytes() {
            self.to_bytes(node, Some(options)).await
        } else {
            self.to_string(node, Some(options))
                .await
                .map(|(string, info)| (string.as_bytes().to_vec(), info))
        }?;
        file.write_all(&content).await?;
        file.flush().await?;

        Ok(info)
    }

    /// Encode a Stencila Schema node to a file system path
    #[tracing::instrument(skip(self, node))]
    async fn to_path(
        &self,
        node: &Node,
        path: &Path,
        options: Option<EncodeOptions>,
    ) -> Result<EncodeInfo> {
        let options = EncodeOptions {
            to_path: Some(path.to_path_buf()),
            ..options.unwrap_or_default()
        };

        if let Some(parent) = path.parent() {
            create_dir_all(parent).await?;
        }
        let mut file = File::create(path).await?;
        self.to_file(node, &mut file, Some(options)).await
    }
}

/// The level of support that a codec provides for a format or node type
#[derive(Debug, Default, Display, Serialize)]
pub enum CodecSupport {
    #[default]
    None,
    HighLoss,
    LowLoss,
    NoLoss,
}

impl CodecSupport {
    /// Whether a format or node type is supported
    pub fn is_supported(&self) -> bool {
        !matches!(self, CodecSupport::None)
    }

    /// Whether there is any loss for a format or node type
    pub fn is_lossy(&self) -> bool {
        !matches!(self, CodecSupport::NoLoss)
    }
}

/// Specifications for a codec
///
/// Currently used only for outputs and display.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodecSpecification {
    name: String,
    from: Vec<String>,
    to: Vec<String>,
}

impl From<&dyn Codec> for CodecSpecification {
    fn from(codec: &dyn Codec) -> Self {
        Self {
            name: codec.name().to_string(),
            from: codec
                .supports_from_formats()
                .keys()
                .map(|format| format.to_string())
                .collect(),
            to: codec
                .supports_to_formats()
                .keys()
                .map(|format| format.to_string())
                .collect(),
        }
    }
}

/// Decoding options
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct DecodeOptions {
    /// The name of the codec to use for decoding
    ///
    /// If not supplied then the format will be used to choose a codec.
    pub codec: Option<String>,

    /// The format to be decode from
    ///
    /// Most codecs only decode one format. However, for those that handle multiple
    /// format it may be necessary to specify this option.
    pub format: Option<Format>,

    /// The media type to decode from
    ///
    /// In some cases (e.g. when decoding content from a HTTP response) the
    /// IANA Media Type (MIME) will be known or need to be specified.
    pub media_type: Option<String>,

    /// The pages to include
    ///
    /// Used by some codecs for page-based formats e.g. PDF
    pub include_pages: Option<Vec<PageSelector>>,

    /// The pages to exclude
    pub exclude_pages: Option<Vec<PageSelector>>,

    /// Whether to embed media files as data URIs
    ///
    /// When enabled, external media files (images, audio, video) referenced in the document
    /// will be converted to data URIs and embedded directly in the document. This creates
    /// a self-contained document but may increase memory usage.
    pub embed_media: Option<bool>,

    /// Whether to embed supplement files as embedded supplemental works
    ///
    /// When enabled, supplemental files (e.g. images, audio, video, CSV, DOCX)
    /// referenced in the document will be decoded and embedded in the document
    /// as the `work` property of the `Supplement`. This creates a
    /// self-contained document but may increase memory usage.
    pub embed_supplements: Option<bool>,

    /// Scopes defining which properties of nodes should be stripped before decoding
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub strip_scopes: Vec<StripScope>,

    /// A list of node types to strip before decoding
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub strip_types: Vec<String>,

    /// A list of node properties to strip before decoding
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub strip_props: Vec<String>,

    /// Decode in strict mode for the format
    pub strict: Option<bool>,

    /// Decode coarsely
    ///
    /// Codecs that support this option will only decode certain node types
    /// (usually executable block types) and put the content between those nodes
    /// into `RawBlock`s of the given format. Useful for formats such as LaTeX
    /// where the codec does not fully decoding all elements.
    pub coarse: Option<bool>,

    /// Decode such that changes in the encoded document can be applied back to the source
    ///
    /// Usually defaults to `true` when decoding from a path, but can be explicitly set
    /// to `false` if `source` and `commit` properties should not be populated.
    pub reproducible: Option<bool>,

    /// Reconstitute nodes from a cache
    ///
    /// Only supported by some codecs and only if `--link` was used when encoding
    /// the document.
    pub cache: Option<PathBuf>,

    /// Ignore any existing artifacts created during decoding of the input
    ///
    /// When `true`, ignores cached artifacts and forces re-processing. Useful when you want
    /// to repeat downloads or intermediate processing (e.g., to get updated data or retry
    /// failed OCR).
    pub ignore_artifacts: Option<bool>,

    /// Do not create any artifacts during decoding
    ///
    /// By default, artifacts (cached intermediate files) are created in the closest `.stencila`
    /// directory and reused when decoding the same input. Artifacts include large file downloads,
    /// intermediate conversion results (e.g., OCR output from images, extracted media files),
    /// and processed data that's expensive to regenerate. Setting this to `true` prevents
    /// creating new artifacts, though existing ones may still be used unless `ignore_artifacts`
    /// is also set.
    pub no_artifacts: Option<bool>,

    /// Automatically create `Island` nodes by wrapping elements in the decoded document
    ///
    /// Only supported by some codecs. The interpretation of these strings is dependent on
    /// the decoding codec. For example, the LaTeX codec will wrap environments with matching
    /// names in an island.
    pub island_wrap: Vec<String>,

    /// The style to apply to automatically created `Island` nodes
    ///
    /// Only supported by some codecs and only if `--island-wrap` is used.
    pub island_style: Option<String>,

    /// The response to take when there are losses in the decoding
    #[default(_code = "LossesResponse::Warn")]
    pub losses: LossesResponse,

    /// The tool to delegate to for decoding (e.g. `pandoc`)
    pub tool: Option<String>,

    /// Additional arguments to pass through to the tool delegated to for decoding
    pub tool_args: Vec<String>,

    /// Options for structuring the decode node
    pub structuring_options: StructuringOptions,
}

impl DecodeOptions {
    /// Set `tool` and `tool_args` properties
    pub fn with_tool(self, tool: Option<String>, tool_args: Vec<String>) -> Self {
        Self {
            tool,
            tool_args,
            ..self
        }
    }
}

/// Encoding options
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case")]
pub struct EncodeOptions {
    /// The name of the codec to use for encoding
    ///
    /// If not supplied then the format will be used to choose a codec.
    pub codec: Option<String>,

    /// The format to encode to
    ///
    /// Most codecs only encode to one format. However, for those that handle multiple
    /// formats it may be necessary to specify this option.
    pub format: Option<Format>,

    /// Encode the outputs, rather than the source, of executable nodes
    pub render: Option<bool>,

    /// Highlight the rendered outputs of executable nodes
    pub highlight: Option<bool>,

    /// Encode such that changes in the encoded document can be applied back to its source
    pub reproducible: Option<bool>,

    /// The template document to use
    ///
    /// Only supported by some formats (e.g. DOCX).
    pub template: Option<PathBuf>,

    /// Whether to encode as a standalone document
    ///
    /// Unless specified otherwise, this is the default when encoding to a file
    /// (as opposed to a string).
    pub standalone: Option<bool>,

    /// The CSS theme to use when encoding to HTML and HTML-derived formats
    ///
    /// Use this option to specify the theme for HTML and HTML-derived (e.g.
    /// PDF) formats.
    pub theme: Option<String>,

    /// The document view to use when encoding to HTML and HTML-derived formats
    ///
    /// Stencila provides alternatives views of documents providing alternative
    /// ways of interacting with a document (e.g. "dynamic", "static", "none").
    pub view: Option<String>,

    /// Whether to embed media files as data URIs
    ///
    /// When enabled, external media files (images, audio, video) referenced in the document
    /// will be converted to data URIs and embedded directly in the output. This creates
    /// a self-contained document but may increase file size significantly.
    /// Should not be used together with `extract_media`.
    pub embed_media: Option<bool>,

    /// Path to extract embedded media to
    ///
    /// When provided, any data URIs in the document will be extracted to files
    /// in the specified directory, and the references will be updated to point
    /// to these external files. This reduces document size but creates external dependencies.
    /// Should not be used together with `embed_media`.
    pub extract_media: Option<PathBuf>,

    /// Path to collect local media to
    ///
    /// When provided, any local media files referred to in the document will be
    /// copied into the specified directory, and the references will be updated
    /// to point to these copies. This is necessary, for example, when pushing a
    /// document to Stencila Sites so that it can be deployed independently of
    /// the local file system layout.
    pub collect_media: Option<PathBuf>,

    /// Whether to embed supplement files as embedded supplemental works
    ///
    /// When enabled, supplemental files (e.g. images, audio, video, CSV, DOCX)
    /// referenced in the document will be decoded and embedded in the document
    /// as the `work` property of the `Supplement`. This creates a
    /// self-contained document but may increase file size significantly. Should
    /// not be used together with `extract_supplements`.
    pub embed_supplements: Option<bool>,

    /// Path to extract embedded supplemental works to
    ///
    /// When provided, any supplemental works embedded in `Supplement` nodes (in
    /// the `work` property) will be extracted to files in the specified
    /// directory. Supplements are saved as `supplement-<N>.cbor.zstd` files
    /// with the specified directory. This reduces document size but creates
    /// external dependencies. Should not be used together with
    /// `embed_supplements`.
    pub extract_supplements: Option<PathBuf>,

    /// The type and name of alternate files
    ///
    /// A codec may encode a document in several formats by delegating to other codecs.
    /// This option allows the list of alternative encodings to be passed to each
    /// delegate codec so that links can be made between them.
    ///
    /// A vector of (media type, relative path) tuples.
    pub alternates: Option<Vec<(String, String)>>,

    /// Whether to encode in compact form
    ///
    /// Some formats (e.g HTML and JSON) can be encoded in either compact
    /// or "pretty-printed" (e.g. indented) forms. If not specified, the default
    /// for the format will be used.
    pub compact: Option<bool>,

    /// The path of the document being encoded from
    ///
    /// Used by some codecs to resolve any relative paths in the document
    /// (e.g. in the `content_url` property of `MediaObject`s)
    pub from_path: Option<PathBuf>,

    /// The path of the file being encoded to
    ///
    /// Used by some codecs to create sidecar files or folders. Note that
    /// the default implementation of `Codec::to_path` will set this option and any
    /// overrides should do the same.
    pub to_path: Option<PathBuf>,

    /// Recursively encode the content of `IncludeBlock`s to their source file
    pub recurse: Option<bool>,

    /// The base URL of the file being encoded to
    ///
    /// Used by some codecs when it is necessary to create absolute URLs.
    pub base_url: Option<String>,

    /// Scopes defining which properties of nodes should be stripped before encoding
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub strip_scopes: Vec<StripScope>,

    /// A list of node types to strip before encoding
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub strip_types: Vec<String>,

    /// A list of node properties to strip before encoding
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub strip_props: Vec<String>,

    /// The response to take when there are losses in the encoding
    #[default(_code = "LossesResponse::Debug")]
    pub losses: LossesResponse,

    /// The tool to delegate to for encoding (e.g. `pandoc`)
    pub tool: Option<String>,

    /// Additional arguments to pass through to the tool delegated to for encoding
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tool_args: Vec<String>,
}

impl EncodeOptions {
    /// Set `tool` and `tool_args` properties
    pub fn with_tool(self, tool: Option<String>, tool_args: Vec<String>) -> Self {
        Self {
            tool,
            tool_args,
            ..self
        }
    }
}

/// A selector for pages in a document
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PageSelector {
    Single(usize),       // N
    Range(usize, usize), // N-M
    From(usize),         // N-
    To(usize),           // -M
    Odd,
    Even,
}

impl FromStr for PageSelector {
    type Err = Report;

    fn from_str(raw: &str) -> Result<Self> {
        let s = raw.trim();
        if s.is_empty() {
            bail!("Empty page selector");
        }
        match s.to_ascii_lowercase().as_str() {
            "odd" => return Ok(PageSelector::Odd),
            "even" => return Ok(PageSelector::Even),
            _ => {}
        }

        fn check(n: usize, raw: &str) -> Result<()> {
            if n == 0 {
                bail!("page numbers are 1-based; got 0 in '{raw}'")
            } else {
                Ok(())
            }
        }

        // -M
        if let Some(rest) = s.strip_prefix('-') {
            let end = rest.parse()?;
            check(end, s)?;
            return Ok(PageSelector::To(end));
        }

        // N-
        if let Some(rest) = s.strip_suffix('-') {
            let start = rest.parse()?;
            check(start, s)?;
            return Ok(PageSelector::From(start));
        }

        // N-M
        if let Some((a, b)) = s.split_once('-') {
            let start = a.parse()?;
            let end = b.parse()?;
            check(start, s)?;
            check(end, s)?;
            if start > end {
                bail!("invalid range '{s}': start > end");
            }
            return Ok(PageSelector::Range(start, end));
        }

        // N
        let n = s.parse()?;
        check(n, s)?;
        Ok(PageSelector::Single(n))
    }
}

impl PageSelector {
    /// Expand one selector to concrete page numbers (1-based, clamped to total_pages).
    fn resolve(&self, total_pages: usize) -> BTreeSet<usize> {
        let mut out = BTreeSet::new();
        match *self {
            PageSelector::Single(n) => {
                if n <= total_pages {
                    out.insert(n);
                }
            }
            PageSelector::Range(a, b) => {
                if a <= total_pages {
                    for p in a..=b.min(total_pages) {
                        out.insert(p);
                    }
                }
            }
            PageSelector::From(a) => {
                if a <= total_pages {
                    for p in a..=total_pages {
                        out.insert(p);
                    }
                }
            }
            PageSelector::To(b) => {
                for p in 1..=b.min(total_pages) {
                    out.insert(p);
                }
            }
            PageSelector::Odd => {
                for p in (1..=total_pages).step_by(2) {
                    out.insert(p);
                }
            }
            PageSelector::Even => {
                for p in (2..=total_pages).step_by(2) {
                    out.insert(p);
                }
            }
        }
        out
    }

    /// Expand many selectors and union them.
    fn resolve_all(specs: &[PageSelector], total_pages: usize) -> BTreeSet<usize> {
        let mut acc = BTreeSet::new();
        for sel in specs {
            acc.extend(sel.resolve(total_pages));
        }
        acc
    }

    /// Compute a final page set taking into account inclusions and exclusions
    pub fn resolve_include_exclude(
        include: Option<&[PageSelector]>,
        exclude: Option<&[PageSelector]>,
        total_pages: usize,
    ) -> BTreeSet<usize> {
        let mut keep: BTreeSet<usize> = if let Some(specs) = include {
            PageSelector::resolve_all(specs, total_pages)
        } else {
            (1..=total_pages).collect()
        };

        if let Some(ex) = exclude {
            for p in PageSelector::resolve_all(ex, total_pages) {
                keep.remove(&p);
            }
        }
        keep
    }
}

/// A document structuring operation
#[derive(
    Debug,
    Display,
    Clone,
    Copy,
    PartialEq,
    Eq,
    ValueEnum,
    EnumIter,
    EnumString,
    EnumMessage,
    Serialize,
    Deserialize,
)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum StructuringOperation {
    /// No structuring operations
    ///
    /// Special value to disable all structuring operations. Use this when you
    /// want to completely bypass document structuring and preserve the input
    /// format exactly as provided.
    None_,

    /// All structuring operations
    ///
    /// Special value to enable all available structuring operations.
    All,

    /// Extract keywords from the "Keywords" section
    ///
    /// Detects headings with text "Keywords" or "Key words" and extracts
    /// the following paragraph content as document keywords, splitting on
    /// commas and trimming whitespace.
    SectionsToKeywords,

    /// Extract abstract from the "Abstract" section
    ///
    /// Identifies sections with "Abstract" headings and extracts their content
    /// as the document abstract, removing the section heading and preserving
    /// the paragraph structure for document metadata.
    SectionsToAbstract,

    /// Extract references from "References" section
    ///
    /// Identifies sections with "References", "Bibliography", or similar
    /// headings and extracts their content as structured reference metadata.
    /// Removes the section from document body and processes individual
    /// references for proper citation linking and bibliography generation.
    SectionsToReferences,

    /// Extract document title from the first heading
    ///
    /// To be extracted as a title, the heading must have no numbering, be a
    /// level 1 or 2 heading, not be a recognized section type (e.g. "Abstract"),
    /// and cannot be after the first primary heading (e.g. "Introduction").
    HeadingsToTitle,

    /// Extract document title from the very first level 1 heading
    ///
    /// A more conservative version of HeadingsToTitle that only extracts the title
    /// if it is the very first block in the document content and is explicitly
    /// level 1. This prevents false positives and ensures only clearly intended
    /// title headings are extracted.
    Heading1ToTitle,

    /// Decrement all heading levels by 1 (H2→H1, H3→H2, etc.)
    ///
    /// Normalizes document structure when the original H1 was extracted as
    /// title by `heading1-to-title`. This ensures that content headings start
    /// at level 1, maintaining proper hierarchical structure for documents that
    /// originally used H1 for title and H2+ for main sections.
    HeadingsDecrement,

    /// Ensure that all "primary" headings have level 1
    ///
    /// Normalizes document structure by forcing primary section headings
    /// (Introduction, Methods, Results, Discussion, Conclusions, etc.) to level
    /// 1, regardless of their original heading levels.
    HeadingsPrimaryLevel1,

    /// Create a section for each heading
    ///
    /// Wraps each heading and its following content in structured section elements,
    /// creating a hierarchical document structure that improves semantic meaning
    /// and enables better navigation and processing.
    HeadingsToSections,

    /// Transform headings to paragraphs if appropriate
    ///
    /// Word processor formats can produce "fake" headings when content has a
    /// heading style applied but is manually formatted as normal body text.
    /// Converts headings to paragraphs when they exhibit paragraph-like traits:
    /// longer than 80 characters, not in all caps, not title case, or ending
    /// with sentence punctuation (. ! ?).
    HeadingsToParagraphs,

    /// Extract keywords from paragraphs starting with "Keywords"
    ///
    /// Detects paragraphs beginning with "Keywords:", "KEYWORDS:", "Key words:"
    /// or similar patterns and extracts the remaining text as document keywords,
    /// splitting on commas for structured metadata.
    ParagraphsToKeywords,

    /// Transform paragraphs to headings if appropriate
    ///
    /// Word processors and OCR sometimes incorrectly format headings as paragraphs
    /// with bold/strong formatting. Converts paragraphs to headings when they
    /// contain only a single bold element, are shorter than 80 characters, and
    /// don't end with sentence punctuation. Heading level is determined by context.
    ParagraphsToHeadings,

    /// Split paragraphs into individual sentences
    ///
    /// Analyzes text content within paragraphs and splits them into separate
    /// sentence elements based on punctuation patterns. Recognizes sentence
    /// boundaries at periods, exclamation marks, and question marks when
    /// followed by whitespace. Preserves non-text inline elements (like links
    /// or emphasis) within their appropriate sentences. Does not split on
    /// abbreviations like "Mr." or "Dr." when not followed by whitespace.
    ParagraphsToSentences,

    /// Combine an image with a figure caption before or after it
    ///
    /// A heading or paragraph is treated as a figure caption if it starts with
    /// "Figure" or "Fig." (case insensitive) followed by a number or letter-number
    /// combination, and the remaining text starts with an uppercase letter or
    /// punctuation. This excludes references like "Figure 2 shows that..." while
    /// including captions like "Figure 1. Plot of results".
    FiguresWithCaptions,

    /// Combine a table caption with the following table or datatable
    ///
    /// A heading or paragraph is treated as a table caption if it starts with
    /// "Table" (case insensitive) followed by a number or letter-number combination,
    /// and the remaining text starts with an uppercase letter or punctuation.
    /// This excludes references like "Table 2 shows that..." while including
    /// captions like "Table 1. Summary of results".
    TablesWithCaptions,

    /// Convert table images to table rows using OCR
    ///
    /// Uses AI models to perform OCR on images within table blocks that have
    /// no rows but contain images. The extracted table data is parsed and
    /// converted to structured TableRow elements, enabling proper table
    /// processing and display of image-based tabular content.
    TableImagesToRows,

    /// Transform tables into datatables if possible
    ///
    /// Converts tables to typed datatables when they meet strict uniformity
    /// requirements: consistent row lengths, simple text-only cells, and no
    /// column/row spans. This enables type inference and validation for
    /// structured tabular data while preserving complex tables as-is.
    TablesToDatatables,

    /// Unwrap media objects from paragraphs to block level
    ///
    /// Documents decoded from word processors can have paragraphs containing
    /// only media objects (image, audio, or video), typically due to complex
    /// layouts in DOCX files. This operation extracts those media objects from
    /// their containing paragraph, promoting them to block-level elements and
    /// removing the now-empty paragraph.
    UnwrapMediaObjects,

    /// Unwrap quote blocks containing more than two child blocks
    ///
    /// Long `QuoteBlock` nodes are often created when decoding word processor
    /// documents because Pandoc treats indented text as block quotes. This
    /// operation unwraps quote blocks that contain more than two blocks,
    /// promoting their child blocks to the parent level and removing the
    /// quote block wrapper. This helps normalize documents where indentation
    /// was used for layout rather than quotation semantics.
    UnwrapQuoteBlocks,

    /// Convert text to structured citations
    ///
    /// Detects citation patterns like "(Smith 2023)" or "[1]" in text and
    /// converts them to structured citation elements. Supports multiple
    /// citation styles including author-year, numeric bracketed, parenthetic,
    /// and superscripted formats. Citation style is auto-detected or can be
    /// specified explicitly in structuring options.
    TextToCitations,

    /// Convert URL text to structured links
    ///
    /// Detects plain text URLs in content and converts them to proper
    /// link elements with href attributes. Handles common URL patterns
    /// including http/https protocols and improves document accessibility
    /// by creating clickable links from plain text references.
    TextToLinks,

    /// Convert math to structured citations
    ///
    /// Converts superscript citations in math notation (often from OCR) to
    /// structured citation elements. This handles cases where citations appear
    /// as superscripted numbers in mathematical expressions that should be
    /// converted to proper citation references for consistent document
    /// structure and linking.
    MathToCitations,

    /// Convert math images to TeX code using OCR
    ///
    /// Uses AI models to perform OCR on images within math blocks and inline
    /// math that have empty code but contain images. The extracted mathematical
    /// expressions are converted to TeX notation, enabling proper rendering
    /// and processing of image-based mathematical content.
    MathImagesToTex,

    /// Convert links to citations
    ///
    /// Converts anchor links that point to reference IDs (e.g., href="#ref-1")
    /// to proper Citation nodes. Only converts links whose targets match
    /// existing reference IDs, preserving the link content as citation content.
    LinksToCitations,

    /// Normalize citation formatting and grouping
    ///
    /// Removes parentheses and square brackets around citations, groups
    /// adjacent citations into citation groups, handles commas and semicolons
    /// between citations by grouping them, handles simple citation ranges
    /// (e.g., dash between two numeric citations), extracts citations from
    /// superscripts, and sets appropriate citation modes: Parenthetical for
    /// citations that had brackets/parentheses removed, Narrative for other
    /// standalone citations, and None for citations within citation groups.
    NormalizeCitations,

    /// Remove content before the first primary heading
    ///
    /// In scholarly articles, author bylines and affiliations usually occur
    /// between the title and the abstract or introduction . This operation
    /// cleans up document structure by removing such content. Be aware that if
    /// no primary heading or section exists in the document that all content
    /// will be removed.
    RemovePrePrimary,

    /// Remove front matter that duplicates article metadata
    ///
    /// Uses edit distance to identify headings and paragraphs that duplicate
    /// metadata already collected (title, DOI, authors, affiliations, keywords, etc.).
    /// More precise than RemovePrePrimary as it only removes actual duplicates
    /// rather than all content before first primary section. Uses normalized
    /// Damerau-Levenshtein distance with a threshold of 0.7 (70% similarity) to
    /// detect duplicates while allowing for minor OCR errors or formatting differences.
    RemoveFrontmatterDuplicates,

    /// Remove empty headings
    ///
    /// A heading is considered empty if it has no content after any numbering
    /// prefix is removed.
    RemoveEmptyHeadings,

    /// Remove empty tables and datatables
    ///
    /// A table/datatable is considered empty if it contains no rows/columns and
    /// has no caption or notes.
    RemoveEmptyTables,

    /// Remove empty lists
    ///
    /// A list is considered empty if it contains no items or all items are
    /// empty (contain no content or only whitespace).
    RemoveEmptyLists,

    /// Remove empty paragraphs
    ///
    /// A paragraph is considered empty if it contains no content or only
    /// whitespace-only text nodes.
    RemoveEmptyParagraphs,

    /// Remove empty text
    ///
    /// Text nodes that contain only whitespace characters are removed from
    /// inline content.
    RemoveEmptyText,
}

/// Options for document structuring
#[derive(Debug, Default, Clone, Args, Serialize, Deserialize)]
pub struct StructuringOptions {
    /// Structuring operations to include (comma-separated)
    ///
    /// If not specified will default to those appropriate for the input format.
    /// Generally, less structuring is done for formats that are already well
    /// structured (e.g. JATS XML). Use 'all' for all operations, 'none' for no
    /// operations. Example: heading-to-title,section-to-abstract
    #[arg(
        long = "include-structuring",
        alias = "structuring",
        help_heading = "Structuring Options",
        value_delimiter = ','
    )]
    pub include_ops: Vec<StructuringOperation>,

    /// Structuring operations to exclude (comma-separated)
    ///
    /// Defaults to empty. Use this to prevent operations used by default for
    /// the input format. Use 'all' to exclude all operations, 'none' to exclude
    /// nothing. Example: remove-empty-text,remove-empty-paragraphs
    #[arg(
        long = "exclude-structuring",
        help_heading = "Structuring Options",
        value_delimiter = ','
    )]
    pub exclude_ops: Vec<StructuringOperation>,

    /// The citation style to assume for text-to-citation structuring.
    ///
    /// If not specified, will be determined automatically based on whether references
    /// are numbered and the relative frequency of detected styles within text.
    /// Only relevant if the `text-to-citations` operation is enabled.
    #[arg(long, help_heading = "Structuring Options")]
    pub citation_style: Option<CitationStyle>,
}

impl Display for StructuringOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let includes = self
            .include_ops
            .iter()
            .map(|op| op.to_string())
            .collect::<Vec<_>>()
            .join(",");
        write!(f, "{includes}")?;

        if !self.exclude_ops.is_empty() {
            let excludes = self
                .exclude_ops
                .iter()
                .map(|op| op.to_string())
                .collect::<Vec<_>>()
                .join(",");
            write!(f, "(excl. {excludes})")?;
        }

        Ok(())
    }
}

impl StructuringOptions {
    /// Create a new set of structuring options which includes no operations
    pub fn none() -> Self {
        Self {
            include_ops: vec![StructuringOperation::None_],
            exclude_ops: Vec::new(),
            citation_style: None,
        }
    }

    /// Create a new set of structuring options which includes all operations
    pub fn all() -> Self {
        Self {
            include_ops: vec![StructuringOperation::All],
            exclude_ops: Vec::new(),
            citation_style: None,
        }
    }

    /// Create a new set of structuring options by specifying the list operations to include and exclude
    pub fn new<I1, I2>(include_ops: I1, exclude_ops: I2) -> Self
    where
        I1: IntoIterator<Item = StructuringOperation>,
        I2: IntoIterator<Item = StructuringOperation>,
    {
        Self {
            include_ops: include_ops.into_iter().collect(),
            exclude_ops: exclude_ops.into_iter().collect(),
            citation_style: None,
        }
    }

    /// Merge a set of structuring options into the current options
    ///
    /// Sets any options that are empty or `None` to the value of the other.
    pub fn merge(&mut self, other: Self) -> &Self {
        if self.include_ops.is_empty() {
            self.include_ops = other.include_ops;
        }

        if self.exclude_ops.is_empty() {
            self.exclude_ops = other.exclude_ops;
        }

        if self.citation_style.is_none() {
            self.citation_style = other.citation_style;
        }

        self
    }

    /// Whether any structuring operations should be performed
    pub fn should_perform_any(&self) -> bool {
        use StructuringOperation::*;

        !(self.include_ops.is_empty()
            || self.include_ops.contains(&None_)
            || self.exclude_ops.contains(&All))
    }

    /// Whether a structuring operation should be performed
    pub fn should_perform(&self, op: StructuringOperation) -> bool {
        use StructuringOperation::*;

        if self.include_ops.contains(&None_) {
            return false;
        }

        if self.exclude_ops.contains(&All) {
            return false;
        }

        (self.include_ops.contains(&op) || self.include_ops.contains(&All))
            && !(self.exclude_ops.contains(&op) || self.exclude_ops.contains(&All))
    }
}

/// Citation style options for in-text citations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Display, ValueEnum, Serialize, Deserialize)]
#[strum(serialize_all = "kebab-case")]
pub enum CitationStyle {
    /// Author-year citations like (Smith, 2023)
    AuthorYear,

    /// Bracketed numeric citations like [1]
    BracketedNumeric,

    /// Parenthetic numeric citations like (1)
    ParentheticNumeric,

    /// Superscripted numeric citations like ¹
    SuperscriptedNumeric,
}

impl CitationStyle {
    pub fn is_numeric(&self) -> bool {
        matches!(
            self,
            Self::BracketedNumeric | Self::ParentheticNumeric | Self::SuperscriptedNumeric
        )
    }
}
