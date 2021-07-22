use super::pandoc;
use eyre::Result;
use stencila_schema::Node;

/// Encode a `Node` to a LaTeX string
pub async fn encode(node: &Node) -> Result<String> {
    pandoc::encode(node, "string://", "latex", &[]).await
}
