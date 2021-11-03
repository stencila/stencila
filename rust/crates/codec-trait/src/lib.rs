//! Defines the codec trait for decoding/encoding nodes

use async_trait::async_trait;
use eyre::{bail, Result};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
};
use stencila_schema::Node;

// Export for use by other crates
pub use ::async_trait;
pub use eyre;
pub use stencila_schema;

/// A codec for decoding and encoding Stencila document nodes to one or more formats
///
/// This trait defines similar functions to `serde_json` (and other `serde` crates) for
/// converting nodes to/from strings, files, readers etc.
#[async_trait]
pub trait Codec {
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
    async fn from_path<T: AsRef<Path>>(path: &T, options: Option<DecodeOptions>) -> Result<Node>
    where
        T: Send + Sync,
    {
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
    async fn to_path<T: AsRef<Path>>(
        node: &Node,
        path: &T,
        options: Option<EncodeOptions>,
    ) -> Result<()>
    where
        T: Send + Sync,
    {
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
