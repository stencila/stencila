use super::pandoc;
use eyre::Result;
use stencila_schema::Node;

/// Decode a LaTeX document to a `Node`
pub async fn decode(latex: &str) -> Result<Node> {
    pandoc::decode(latex, "latex", &[]).await
}
