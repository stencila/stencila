use crate::methods::coerce::coerce;
use eyre::Result;
use stencila_schema::Node;

/// Decode a TOML document to a `Node`
pub fn decode(toml: &str) -> Result<Node> {
    coerce(toml::from_str(toml)?)
}
