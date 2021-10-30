//! Defines the codec trait for decoding/encoding nodes

use eyre::{bail, Result};
use stencila_schema::Node;

// Export for use by other crates
pub use eyre;
pub use stencila_schema;

/// A codec for for decoding/encoding nodes
///
/// Defines similar functions to `serde_json` (and other `serde` crates) for
/// converting nodes to/from strings, files, readers etc.
pub trait Codec {
    fn from_str(_str: &str) -> Result<Node> {
        bail!("Decoding is not implemented for this format")
    }

    fn to_string(_node: &Node, _options: Option<EncodeOptions>) -> Result<String> {
        bail!("Encoding is not implemented for this format")
    }
}

/// Common encoding options
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
}

impl Default for EncodeOptions {
    fn default() -> Self {
        Self {
            compact: true,
            standalone: false,
            bundle: false,
            theme: "stencila".to_string(),
        }
    }
}
