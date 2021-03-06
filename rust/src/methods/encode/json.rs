use eyre::Result;
use stencila_schema::Node;

/// Encode a `Node` to a JSON document
pub fn encode(node: &Node) -> Result<String> {
    Ok(serde_json::to_string_pretty::<Node>(&node)?)
}
