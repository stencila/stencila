use eyre::Result;
use node_coerce::coerce;
use stencila_schema::Node;

/// Decode a TOML document to a `Node`
pub fn decode(toml: &str) -> Result<Node> {
    coerce(toml::from_str(toml)?, None)
}
