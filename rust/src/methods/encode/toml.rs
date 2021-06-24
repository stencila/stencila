use eyre::Result;
use stencila_schema::Node;

/// Encode a `Node` to a TOML document
///
/// TOML is not recommended for large complex documents and encoding
/// may fail with the error "values must be emitted before tables".
pub fn encode(node: &Node) -> Result<String> {
    Ok(toml::to_string::<Node>(&node)?)
}
