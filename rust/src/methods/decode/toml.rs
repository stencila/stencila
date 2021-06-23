use eyre::Result;
use stencila_schema::Node;

/// Decode a `Node` from TOML
pub fn decode(toml: &str) -> Result<Node> {
    Ok(toml::from_str::<Node>(toml)?)
}
