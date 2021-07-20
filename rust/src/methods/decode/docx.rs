use super::pandoc;
use eyre::Result;
use stencila_schema::Node;

/// Decode a DOCX file to a `Node`
///
/// If the document contains media (e.g images) these will be extracted
/// in to the project cache.
pub async fn decode(input: &str) -> Result<Node> {
    let path = if let Some(path) = input.strip_prefix("file://") {
        path
    } else {
        input
    };

    // TODO: Resolve the project's .stencila dir
    let media = ".stencila/cache".to_string();

    pandoc::decode(
        &["file://", path].concat(),
        pandoc::Options {
            format: "docx".to_string(),
            args: vec!["--extract-media".to_string(), media],
        },
    )
    .await
}
