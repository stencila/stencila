use eyre::Result;
use stencila_schema::Node;

/// Decode a `Node` from YAML
pub fn decode(yaml: &str) -> Result<Node> {
    Ok(serde_yaml::from_str::<Node>(yaml)?)
}
