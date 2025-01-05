use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use common::{
    async_trait::async_trait,
    eyre::{bail, Result},
    serde::{Deserialize, Serialize},
    serde_with::skip_serializing_none,
    smart_default::SmartDefault,
    strum::{Display, IntoEnumIterator},
    tokio::{
        fs::{create_dir_all, File},
        io::{AsyncReadExt, AsyncWriteExt},
    },
    tracing,
};
use format::Format;
use node_strip::StripScope;
use schema::Node;
use status::Status;

// Re-exports for the convenience of internal crates implementing `Codec`
pub use codec_info::*;
pub use common;
pub use format;
pub use schema;
pub use status;

/// The direction of conversion
pub enum CodecDirection {
    Decode,
    Encode,
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

    /// The status of the codec
    ///
    /// Used when listing codecs and to warn users when using a codec that
    /// is not stable.
    fn status(&self) -> Status;

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
    #[tracing::instrument(skip(self))]
    async fn from_path(
        &self,
        path: &Path,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        if !path.exists() {
            bail!("Path `{}` does not exist", path.display());
        }

        let mut file = File::open(path).await?;
        self.from_file(&mut file, options).await
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

    /// Encode a Stencila Schema to a file
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

        Ok(info)
    }

    /// Encode a Stencila Schema to a file system path
    #[tracing::instrument(skip(self, node))]
    async fn to_path(
        &self,
        node: &Node,
        path: &Path,
        options: Option<EncodeOptions>,
    ) -> Result<EncodeInfo> {
        if let Some(parent) = path.parent() {
            create_dir_all(parent).await?;
        }
        let mut file = File::create(path).await?;
        let options = Some(EncodeOptions {
            to_path: Some(path.to_path_buf()),
            ..options.unwrap_or_default()
        });
        self.to_file(node, &mut file, options).await
    }
}

/// The level of support that a codec provides for a format or node type
#[derive(Debug, Default, Display, Serialize)]
#[serde(crate = "common::serde")]
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
#[serde(crate = "common::serde", rename_all = "camelCase")]
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
#[serde(default, rename_all = "kebab-case", crate = "common::serde")]
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

    /// The response to take when there are losses in the decoding
    #[default(_code = "LossesResponse::Warn")]
    pub losses: LossesResponse,

    /// Arguments to passthrough to CLI tools delegated to for decoding (e.g. Pandoc)
    pub passthrough_args: Vec<String>,
}

/// Encoding options
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "kebab-case", crate = "common::serde")]
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

    /// Whether to encode only outputs, and no source, for executable nodes
    ///
    /// When this option is `true`, for executable node types such as `CodeChunk`,
    /// `IncludeBlock`, and `InstructionBlock`, only the outputs from execution
    /// will be rendered, and not the source.
    ///
    /// This option is only supported by some formats.
    pub render: Option<bool>,

    /// Whether to encode as a standalone document
    ///
    /// Unless specified otherwise, this is the default when encoding to a file
    /// (as opposed to a string).
    pub standalone: Option<bool>,

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
    #[default(_code = "LossesResponse::Warn")]
    pub losses: LossesResponse,

    /// Arguments to passthrough to CLI tools delegated to for encoding (e.g. Pandoc)
    pub passthrough_args: Vec<String>,
}
