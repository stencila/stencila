use std::path::Path;

use common::{
    async_trait::async_trait,
    defaults::Defaults,
    eyre::{bail, Result},
    serde::Serialize,
    tokio::{
        fs::{create_dir_all, File},
        io::{AsyncReadExt, AsyncWriteExt},
    },
    tracing,
};
use format::Format;
use schema::Node;

// Rexports for the convienience of internal crates implementing `Codec`
pub use common;
pub use format;
pub use schema;
pub use status;
use status::Status;

/// A codec for decoding/encdoing between Stencila Schema nodes and alternative formats
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

    /// The formats that the codec supports
    ///
    /// Most codecs only support a single format, but multiple formats are
    /// possible.
    fn supported_formats(&self) -> Vec<Format>;

    /// Whether the codec supports decoding from string content
    fn supports_from_string(&self) -> bool {
        true
    }

    /// Whether the codec supports decoding from a file system path
    fn supports_from_path(&self) -> bool {
        true
    }

    /// Whether the codec supports encoding to string content
    fn supports_to_string(&self) -> bool {
        true
    }

    /// Whether the codec supports encoding to a file system path
    fn supports_to_path(&self) -> bool {
        true
    }

    /// Whether the codec uses remote state
    ///
    /// Some formats (e.g. Google Docs) have their canonical state in a remote
    /// location (e.g. Google's servers) and Stencila only "mirrors" them in a local
    /// file containing an id or URL linking the file to the remote state.
    fn has_remote_state(&self) -> bool {
        false
    }

    /// Generate a [`CodecSpec`] for the codec
    fn spec(&self) -> CodecSpec {
        CodecSpec {
            name: self.name().to_string(),
            status: self.status(),
            supported_formats: self.supported_formats(),
            supports_from_string: self.supports_from_string(),
            supports_from_path: self.supports_from_path(),
            supports_to_string: self.supports_to_string(),
            supports_to_path: self.supports_to_path(),
            has_remote_state: self.has_remote_state(),
        }
    }

    /// Decode a Stencila Schema node from a string
    #[allow(clippy::wrong_self_convention)]
    async fn from_str(&self, _str: &str, _options: Option<DecodeOptions>) -> Result<Node> {
        bail!(
            "Decoding from string is not implemented for codec `{}`",
            self.name()
        )
    }

    /// Decode a Stencila Schema node from a file
    ///
    /// This function reads the file as a string and passes that on to `from_str`
    /// for decoding. If working with binary formats, you should override this function
    /// to read the file as bytes instead.
    #[tracing::instrument(skip(self))]
    async fn from_file(&self, file: &mut File, options: Option<DecodeOptions>) -> Result<Node> {
        let mut content = String::new();
        file.read_to_string(&mut content).await?;
        self.from_str(&content, options).await
    }

    /// Decode a Stencila Schema node from a file system path
    #[tracing::instrument(skip(self))]
    async fn from_path(&self, path: &Path, options: Option<DecodeOptions>) -> Result<Node> {
        if !path.exists() {
            bail!("Path `{}` does not exist", path.display());
        }

        let mut file = File::open(path).await?;
        self.from_file(&mut file, options).await
    }

    /// Encode a Stencila Schema node to a string
    async fn to_string(&self, _node: &Node, _options: Option<EncodeOptions>) -> Result<String> {
        bail!(
            "Encoding to a string is not implemented for codec `{}`",
            self.name()
        )
    }

    /// Encode a Stencila Schema to a file
    #[tracing::instrument(skip(self))]
    async fn to_file(
        &self,
        node: &Node,
        file: &mut File,
        options: Option<EncodeOptions>,
    ) -> Result<()> {
        let content = self.to_string(node, options).await?;
        file.write_all(content.as_bytes()).await?;
        Ok(())
    }

    /// Encode a Stencila Schema to a file system path
    #[tracing::instrument(skip(self))]
    async fn to_path(
        &self,
        node: &Node,
        path: &Path,
        options: Option<EncodeOptions>,
    ) -> Result<()> {
        if let Some(parent) = path.parent() {
            create_dir_all(parent).await?;
        }
        let mut file = File::create(path).await?;
        self.to_file(node, &mut file, options).await
    }
}

/// A specification of a codec
///
/// Used to allow user inspection of the capabilities of a codec.
#[derive(Debug, Serialize)]
#[serde(crate = "common::serde")]
pub struct CodecSpec {
    name: String,
    status: Status,
    supported_formats: Vec<Format>,
    supports_from_string: bool,
    supports_from_path: bool,
    supports_to_string: bool,
    supports_to_path: bool,
    has_remote_state: bool,
}

/// Decoding options
#[derive(Debug, Default, Clone)]
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
}

/// Encoding options
#[derive(Debug, Defaults, Clone)]
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

    /// Whether to encode in compact form
    ///
    /// Some formats (e.g HTML and JSON) can be encoded in either compact
    /// or "pretty-printed" (e.g. indented) forms.
    #[def = "false"]
    pub compact: bool,

    /// Whether to strip the id property of nodes when encoding
    #[def = "true"]
    pub strip_id: bool,

    /// Whether to strip the code of executable nodes when encoding
    #[def = "false"]
    pub strip_code: bool,

    /// Whether to strip the derived properties of executable nodes when encoding
    #[def = "false"]
    pub strip_execution: bool,

    /// Whether to strip the outputs of executable nodes when encoding
    #[def = "false"]
    pub strip_outputs: bool,
}
