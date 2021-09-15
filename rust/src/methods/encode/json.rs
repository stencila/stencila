use super::Options;
use eyre::Result;
use stencila_schema::Node;

/// Encode a `Node` to a JSON document
///
/// Defaults to pretty (indented). Use `compact: true` option for no indentation.
pub fn encode(node: &Node, options: Option<Options>) -> Result<String> {
    let compact = options.map_or_else(|| false, |options| options.compact);
    let json = match compact {
        true => serde_json::to_string::<Node>(node)?,
        false => serde_json::to_string_pretty::<Node>(node)?,
    };
    Ok(json)
}
