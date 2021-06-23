use eyre::Result;
use stencila_schema::Node;

/// Encode a `Node` to YAML
pub fn encode(node: &Node) -> Result<String> {
    Ok(serde_yaml::to_string::<Node>(&node)?)
}
