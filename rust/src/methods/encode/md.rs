use eyre::{bail, Result};
use stencila_schema::Node;

/// Encode a `Node` to Markdown
pub fn encode(_node: &Node) -> Result<String> {
    bail!("TODO")
}
