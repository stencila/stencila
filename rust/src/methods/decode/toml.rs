use eyre::Result;
use stencila_schema::Node;

/// Decode a TOML document to a `Node`
pub fn decode(toml: &str) -> Result<Node> {
    Ok(toml::from_str::<Node>(toml)?)
}
