use eyre::Result;
use stencila_schema::Node;

/// Decode a YAML document to a `Node`
pub fn decode(yaml: &str) -> Result<Node> {
    Ok(serde_yaml::from_str::<Node>(yaml)?)
}
