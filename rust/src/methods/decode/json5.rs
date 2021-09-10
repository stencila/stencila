use crate::methods::coerce::coerce;
use eyre::Result;
use stencila_schema::Node;

/// Decode a JSON5 document to a `Node`
pub fn decode(json: &str) -> Result<Node> {
    coerce(json5::from_str(json)?)
}
