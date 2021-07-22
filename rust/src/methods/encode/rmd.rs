use super::md;
use eyre::Result;
use stencila_schema::Node;

/// Encode a `Node` to R Markdown
pub fn encode(node: &Node) -> Result<String> {
    // TODO: Any necessary translations of Markdown to RMarkdown
    md::encode(node)
}
