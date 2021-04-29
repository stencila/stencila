use crate::{encode::encode, nodes::Node, write::write};
use eyre::Result;
use std::path::Path;

/// Export a node to a URL (including a `file://` or `string://` URL)
///
/// # Arguments
///
/// - `node`: The node to export
/// - `output`: The URL to export to
/// - `format`: The format to export the node to.
///             Defaults to the URL's file extension. Falling back to JSON.
pub fn export(node: Node, output: &str, format: Option<String>) -> Result<()> {
    let format = format.unwrap_or_else(|| match Path::new(output).extension() {
        Some(ext) => ext.to_string_lossy().into(),
        None => "json".to_string(),
    });
    let content = encode(node, &format)?;
    write(&content, output)
}
