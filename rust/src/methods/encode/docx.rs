use super::pandoc;
use eyre::{bail, Result};
use stencila_schema::Node;

/// Encode a `Node` to a DOCX file
pub async fn encode(node: &Node, output: &str) -> Result<String> {
    let path = if let Some(path) = output.strip_prefix("file://") {
        path
    } else {
        bail!("Can only encode to a file:// output")
    };

    pandoc::encode(node, &["file://", path].concat(), "docx", &[]).await
}
