use eyre::{bail, Result};
use stencila_schema::Node;

/// Decode a HTML document to a `Node`
pub fn decode(_html: &str) -> Result<Node> {
    bail!("TODO")
}
