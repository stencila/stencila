use super::pandoc;
use eyre::Result;
use stencila_schema::Node;

/// Decode a LaTeX document to a `Node`
pub async fn decode(latex: &str) -> Result<Node> {
    pandoc::decode(
        latex,
        pandoc::Options {
            format: "latex".to_string(),
            is_file: false,
            ..Default::default()
        },
    )
    .await
}
