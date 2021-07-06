use std::path::PathBuf;

use super::pandoc;
use eyre::Result;
use stencila_schema::Node;

/// Decode a DOCX file to a `Node`
///
/// If the document contains media (e.g images) these will be extracted
/// to a sibling folder with `.media` extension. This folder needs to be a sibling
/// to ensure it is not outside of the project (for security reasons). This does mean
/// however that the folder gets polluted unnecessarily with this folder.
pub async fn decode(input: &str) -> Result<Node> {
    let media = PathBuf::from(input).canonicalize()?.display().to_string() + ".media";
    pandoc::decode(
        input,
        pandoc::Options {
            format: "docx".to_string(),
            is_file: true,
            args: vec![format!("--extract-media={media}", media = media)],
        },
    )
    .await
}
