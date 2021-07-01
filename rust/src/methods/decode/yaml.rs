use eyre::Result;
use stencila_schema::Node;

use crate::methods::coerce::coerce;

/// Decode a YAML document to a `Node`
pub fn decode(yaml: &str) -> Result<Node> {
    Ok(coerce(serde_yaml::from_str(yaml)?)?)
}
