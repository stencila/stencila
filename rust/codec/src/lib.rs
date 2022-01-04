use async_trait::async_trait;
use eyre::{bail, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};
use stencila_schema::Node;

// Re-export for the convenience of crates that implement `CodecTrait`
pub use ::async_trait;
pub use eyre;
pub use serde;
pub use stencila_schema;
pub use utils;

/// A specification for codecs
///
/// All codecs, including those implemented in plugins, should provide this
/// specification. Rust implementations return a `Codec` instance from the
/// `spec` function of `CodecTrait`. Plugins provide a JSON or YAML serialization
/// as part of their manifest.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct Codec {
    /// A list of format names (or aliases) that the codec can handle
    pub formats: Vec<String>,

    /// Whether the codec supports decoding from string content
    pub from_string: bool,

    /// Whether the codec supports decoding from a file system path
    pub from_path: bool,

    /// Whether the codec supports encoding to string content
    pub to_string: bool,

    /// Whether the codec supports encoding to a file system path
    pub to_path: bool,

    /// A list of root node types that the codec can encode / decode
    ///
    /// Most codecs usually only handle one root type e.g. `Article`.
    /// Used to provide a list of formats to the user that support
    /// the current document type.
    pub root_types: Vec<String>,

    /// A list of node types that the codec does not support
    ///
    /// Used to provide warnings to the user on potential loss of content
    /// when encoding using the codec.
    pub unsupported_types: Vec<String>,

    /// A list of node properties that the codec does not support
    ///
    /// The format for these strings is `<type>.<property>` e.g. `Article.funders`
    /// Used to provide warnings to the user on potential loss of content
    /// when encoding using the codec.
    pub unsupported_properties: Vec<String>,

    /// The status of the codec e.g. `alpha`, `beta`
    ///
    /// Leave a blank string for stable, production ready codecs.
    pub status: String,
}

/// A trait for codecs
///
/// This trait can be used by Rust implementations of codecs, allowing them to
/// be compiled into the Stencila binaries.
///
/// It defines similar functions to `serde_json` (and other `serde_` crates) for
/// converting nodes to/from strings, files, readers etc.
#[async_trait]
pub trait CodecTrait {
    /// Get the [`Codec`] specification for this implementation
    fn spec() -> Codec;

    /// Decode a document node from a string
    fn from_str(_str: &str, _options: Option<DecodeOptions>) -> Result<Node> {
        bail!("Decoding from string is not implemented for this format")
    }

    /// Decode a document node from a string asynchronously
    async fn from_str_async(str: &str, options: Option<DecodeOptions>) -> Result<Node> {
        Self::from_str(str, options)
    }

    /// Decode a document node from a `BufReader`
    async fn from_buffer<T: Read>(
        reader: &mut BufReader<T>,
        options: Option<DecodeOptions>,
    ) -> Result<Node>
    where
        T: Send + Sync,
    {
        let mut content = String::new();
        reader.read_to_string(&mut content)?;
        Self::from_str(&content, options)
    }

    /// Decode a document node from a file
    ///
    /// This function reads the file as a string and passes that on to `from_str`
    /// for decoding. If working with binary formats, you should override this function
    /// to read the file as bytes instead.
    async fn from_file(file: &mut File, options: Option<DecodeOptions>) -> Result<Node> {
        let mut content = String::new();
        file.read_to_string(&mut content)?;
        Self::from_str(&content, options)
    }

    /// Decode a document node from a file system path
    async fn from_path(path: &Path, options: Option<DecodeOptions>) -> Result<Node> {
        let mut file = File::open(path)?;
        Self::from_file(&mut file, options).await
    }

    /// Encode a document node to a string
    fn to_string(_node: &Node, _options: Option<EncodeOptions>) -> Result<String> {
        bail!("Encoding to string is not implemented for this format")
    }

    /// Encode a document node to a string asynchronously
    async fn to_string_async(node: &Node, options: Option<EncodeOptions>) -> Result<String> {
        Self::to_string(node, options)
    }

    /// Encode a document node to a `BufWriter`
    async fn to_buffer<T: Write>(
        node: &Node,
        writer: &mut BufWriter<T>,
        options: Option<EncodeOptions>,
    ) -> Result<()>
    where
        T: Send + Sync,
    {
        let content = Self::to_string(node, options)?;
        writer.write_all(content.as_bytes())?;
        Ok(())
    }

    /// Encode a document node to a file
    async fn to_file(node: &Node, file: &mut File, options: Option<EncodeOptions>) -> Result<()> {
        let content = Self::to_string(node, options)?;
        file.write_all(content.as_bytes())?;
        Ok(())
    }

    /// Encode a document node to a file system path
    async fn to_path(node: &Node, path: &Path, options: Option<EncodeOptions>) -> Result<()> {
        let mut file = File::open(path)?;
        Self::to_file(node, &mut file, options).await
    }
}

/// Decoding options
///
/// Decoding functions (including those in plugins) are encouraged to respect these options
/// but are not required to.
#[derive(Clone)]
pub struct DecodeOptions {
    /// The format of the input to be decoded
    ///
    /// Most codecs only decode one format. However, for those that handle multiple
    /// format it may be necessary to specify this option.
    pub format: Option<String>,
}

impl Default for DecodeOptions {
    fn default() -> Self {
        Self { format: None }
    }
}

/// Encoding options
///
/// Encoding functions (including those in plugins) are encouraged to respect these options
/// but are not required to. Indeed, some options do not apply for some formats.
/// For example, a PDF is always `standalone` (so if that option is set to `false` is it will be ignored).
/// Futhermore, some combinations of options are ineffectual e.g. a `theme` when `standalone: false`
#[derive(Clone)]
pub struct EncodeOptions {
    /// Whether to encode in compact form.
    ///
    /// Some formats (e.g HTML and JSON) can be encoded in either compact
    /// or "pretty-printed" ie.e. indented forms.
    pub compact: bool,

    /// Whether to ensure that the encoded document is standalone.
    ///
    /// Some formats (e.g. Markdown, DOCX) are always standalone, others
    /// can be frangments, or standalong documents (e.g HTML).
    pub standalone: bool,

    /// Whether to bundle local media files into the encoded document
    ///
    /// Some formats (e.g. DOCX, PDF) always bundle. For HTML,
    /// bundling means including media as data URIs rather than
    /// links to files.
    pub bundle: bool,

    /// The theme to apply to the encoded document
    ///
    /// Only applies to some formats (e.g. HTML, PDF, PNG).
    pub theme: String,

    /// The format to encode to
    ///
    /// Most codecs only encode to one format. However, for those that handle multiple
    /// formats it may be necessary to specify this option.
    pub format: Option<String>,
}

impl Default for EncodeOptions {
    fn default() -> Self {
        Self {
            compact: true,
            standalone: false,
            bundle: false,
            theme: "stencila".to_string(),
            format: None,
        }
    }
}
