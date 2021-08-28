use super::Options;
use eyre::Result;
use stencila_schema::Node;

/// Encode a `Node` to a JSON document
///
/// Defaults to pretty (indented). Use "compact" theme for non-indented JSON.
pub fn encode(node: &Node, options: Option<Options>) -> Result<String> {
    let Options { theme, .. } = options.unwrap_or_default();
    let json = match theme.as_str() {
        "compact" => serde_json::to_string::<Node>(node)?,
        _ => serde_json::to_string_pretty::<Node>(node)?,
    };
    Ok(json)
}
