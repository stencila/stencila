use std::path::Path;

use common::{
    async_trait::async_trait,
    clap::{self, ValueEnum},
    defaults::Defaults,
    derive_more::{Deref, DerefMut},
    eyre::{bail, eyre, Result},
    itertools::Itertools,
    serde::Serialize,
    strum::Display,
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
    async fn from_str(
        &self,
        _str: &str,
        _options: Option<DecodeOptions>,
    ) -> Result<(Node, Losses)> {
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
    async fn from_file(
        &self,
        file: &mut File,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, Losses)> {
        let mut content = String::new();
        file.read_to_string(&mut content).await?;
        self.from_str(&content, options).await
    }

    /// Decode a Stencila Schema node from a file system path
    #[tracing::instrument(skip(self))]
    async fn from_path(
        &self,
        path: &Path,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, Losses)> {
        if !path.exists() {
            bail!("Path `{}` does not exist", path.display());
        }

        let mut file = File::open(path).await?;
        self.from_file(&mut file, options).await
    }

    /// Encode a Stencila Schema node to a string
    async fn to_string(
        &self,
        _node: &Node,
        _options: Option<EncodeOptions>,
    ) -> Result<(String, Losses)> {
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
    ) -> Result<Losses> {
        let (content, losses) = self.to_string(node, options).await?;
        file.write_all(content.as_bytes()).await?;
        Ok(losses)
    }

    /// Encode a Stencila Schema to a file system path
    #[tracing::instrument(skip(self))]
    async fn to_path(
        &self,
        node: &Node,
        path: &Path,
        options: Option<EncodeOptions>,
    ) -> Result<Losses> {
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
    pub name: String,
    pub status: Status,
    pub supported_formats: Vec<Format>,
    pub supports_from_string: bool,
    pub supports_from_path: bool,
    pub supports_to_string: bool,
    pub supports_to_path: bool,
    pub has_remote_state: bool,
}

/// Decoding options
#[derive(Debug, Defaults, Clone)]
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

    /// The response to take when there are losses in the decoding
    #[def = "LossesResponse::Warn"]
    pub losses: LossesResponse,
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

    /// The response to take when there are losses in the encoding
    #[def = "LossesResponse::Warn"]
    pub losses: LossesResponse,
}

/// The response to take when there are losses in decoding or encoding
#[derive(Debug, Clone, Copy, ValueEnum, Display)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
pub enum LossesResponse {
    /// Ignore the losses; do nothing
    Ignore,
    /// Log losses as spearate log entries with the `TRACE` severity level
    Trace,
    /// Log losses as spearate log entries with the `DEBUG` severity level
    Debug,
    /// Log losses as spearate log entries with the `INFO` severity level
    Info,
    /// Log losses as spearate log entries with the `WARN` severity level
    Warn,
    /// Log losses as spearate log entries with the `ERROR` severity level
    Error,
    /// Abort the current function call by returning a `Err` result with the losses enumerated
    Abort,
}

/// A record of a loss during encoding or decoding
#[derive(Debug)]
pub struct Loss {
    /// The type for which the loss occurred e.g. `Paragraph`
    r#type: String,

    /// The properties for which the loss occurred e.g. `authors`
    ///
    /// If empty, or an asterisk, then loss is assumed to be for all properties of the type.
    properties: String,

    /// A message explaining the loss
    message: String,

    /// A count of the number of times the loss occurred
    count: usize,
}

/// Decoding and encoding losses
#[derive(Debug, Default, Deref, DerefMut)]
pub struct Losses {
    inner: Vec<Loss>,
}

impl Losses {
    /// Create a new, empty set of losses
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a loss
    ///
    /// If the type of loss is already registered then increments the count by one.
    pub fn register(&mut self, r#type: &str, properties: &str, message: &str) {
        for loss in self.iter_mut() {
            if loss.r#type == r#type && loss.properties == properties && loss.message == message {
                loss.count += 1;
                break;
            }
        }

        self.push(Loss {
            r#type: r#type.to_string(),
            properties: properties.to_string(),
            message: message.to_string(),
            count: 1,
        })
    }

    /// Respond to losses according to the `LossesResponse` variant
    pub fn respond(&self, response: LossesResponse) -> Result<()> {
        if self.is_empty() || matches!(response, LossesResponse::Ignore) {
            return Ok(());
        }

        if matches!(response, LossesResponse::Abort) {
            let summary = self
                .iter()
                .map(
                    |Loss {
                         r#type,
                         properties,
                         message,
                         count,
                     }| format!("{type}[{properties}]: {message} ({count})"),
                )
                .join("; ");
            let error = eyre!(summary).wrap_err("Conversion losses occurred");
            return Err(error);
        }

        for Loss {
            r#type,
            properties,
            message,
            count,
        } in self.iter()
        {
            match response {
                LossesResponse::Trace => {
                    tracing::event!(tracing::Level::TRACE, "Conversion losses for {type}[{properties}]: {message} ({count})", type = r#type, properties = properties, message = message, count = count);
                }
                LossesResponse::Debug => {
                    tracing::event!(tracing::Level::DEBUG, "Conversion losses for {type}[{properties}]: {message} ({count})", type = r#type, properties = properties, message = message, count = count);
                }
                LossesResponse::Info => {
                    tracing::event!(tracing::Level::INFO, "Conversion losses for {type}[{properties}]: {message} ({count})", type = r#type, properties = properties, message = message, count = count);
                }
                LossesResponse::Warn => {
                    tracing::event!(tracing::Level::WARN, "Conversion losses for {type}[{properties}]: {message} ({count})", type = r#type, properties = properties, message = message, count = count);
                }
                LossesResponse::Error => {
                    tracing::event!(tracing::Level::ERROR, "Conversion losses for {type}[{properties}]: {message} ({count})", type = r#type, properties = properties, message = message, count = count);
                }
                _ => bail!("Should be unreachable"),
            };
        }

        Ok(())
    }
}
