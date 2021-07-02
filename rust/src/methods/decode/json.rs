use crate::methods::coerce::coerce;
use eyre::Result;
use stencila_schema::Node;

/// Decode a JSON document to a `Node`
pub fn decode(json: &str) -> Result<Node> {
    coerce(serde_json::from_str(json)?)
}
