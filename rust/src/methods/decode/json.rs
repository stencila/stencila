use eyre::Result;
use stencila_schema::Node;

/// Decode a JSON document to a `Node`
pub fn decode(json: &str) -> Result<Node> {
    Ok(serde_json::from_str::<Node>(json)?)
}
