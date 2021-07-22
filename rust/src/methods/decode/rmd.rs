use super::md;
use eyre::Result;
use stencila_schema::Node;

/// Decode a R Markdown document to a `Node`
pub fn decode(input: &str) -> Result<Node> {
    // TODO: Any necessary translations before parsing as Markdown
    md::decode(input)
}
