use eyre::Result;
use node_coerce::coerce;
use stencila_schema::Node;

/// Decode a YAML document to a `Node`
pub fn decode(yaml: &str) -> Result<Node> {
    coerce(serde_yaml::from_str(yaml)?, None)
}
