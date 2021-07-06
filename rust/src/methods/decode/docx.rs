use super::pandoc;
use eyre::Result;
use stencila_schema::Node;

/// Decode a DOCX file to a `Node`
pub async fn decode(docx: &str) -> Result<Node> {
    pandoc::decode(
        docx,
        pandoc::Options {
            format: "docx".to_string(),
            is_file: true,
        },
    )
    .await
}
