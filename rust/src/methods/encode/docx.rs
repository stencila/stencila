use super::pandoc;
use eyre::Result;
use stencila_schema::Node;

/// Encode a `Node` to a DOCX file
pub async fn encode(node: &Node, output: &str) -> Result<String> {
    let path = if let Some(path) = output.strip_prefix("file://") {
        path
    } else {
        output
    };

    pandoc::encode(
        node,
        &["file://", path].concat(),
        pandoc::Options {
            format: "docx".to_string(),
            ..Default::default()
        },
    )
    .await
}
