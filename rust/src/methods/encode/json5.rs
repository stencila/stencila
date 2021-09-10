use super::Options;
use eyre::Result;
use stencila_schema::Node;

/// Encode a `Node` to a JSON5 document
///
/// At the time of writing, the `json5` crate actually produces plain
/// old JSON, and does not offer pretty printing (so we use `serde_json` for that).
pub fn encode(node: &Node, options: Option<Options>) -> Result<String> {
    let Options { theme, .. } = options.unwrap_or_default();
    let json = match theme.as_str() {
        "compact" => json5::to_string::<Node>(node)?,
        _ => serde_json::to_string_pretty::<Node>(node)?,
    };
    Ok(json)
}
