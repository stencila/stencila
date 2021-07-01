use eyre::Result;
use stencila_schema::Node;

/// Coerce a JSON value to the Stencila schema
///
/// This function is intended to be used when deserializing
/// generic data formats e.g JSON, YAML prior to
/// avoid `serde` validation errors when subsequently deserializing
/// to a `Node`. Places this might be necessary:
///
/// - when decoding JSON, YAML, etc documents
/// - when deserializing the result from delegating a method
///   to a peer or plugin
/// - when decoding the YAML header of a Markdown document
pub fn coerce(value: serde_json::Value) -> Result<Node> {
    // TODO
    Ok(serde_json::from_value(value)?)
}
