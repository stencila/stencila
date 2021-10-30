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

    fn to_string(_node: &Node) -> Result<String> {
        bail!("Encoding is not implemented for this format")
    }
}
