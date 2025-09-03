use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
    str::FromStr,
};

use eyre::{Report, Result, bail};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use smart_default::SmartDefault;
use strum::{Display, IntoEnumIterator};
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

    /// Whether the codec uses remote state
    ///
    /// Some formats (e.g. Google Docs) have their canonical state in a remote
    /// location (e.g. Google's servers) and Stencila only "mirrors" them in a local
    /// file containing an id or URL linking the file to the remote state.
    fn has_remote_state(&self) -> bool {
        false
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

    /// The theme to use when encoding
    ///
    /// Use this option to specify the theme form HTML and HTML-based formats (e.g. PDF).
    pub theme: Option<String>,

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
