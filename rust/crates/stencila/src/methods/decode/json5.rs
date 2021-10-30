use eyre::Result;
use node_coerce::coerce;
use stencila_schema::Node;

/// Decode a JSON5 document to a `Node`
pub fn decode(json: &str) -> Result<Node> {
    coerce(json5::from_str(json)?, None)
}
