use std::path::Path;

use codec::{common::eyre::Result, stencila_schema::Node, EncodeOptions};

/// Encode a Stencila `Node` as a Microsoft Word `docx` that can then be uploaded
/// as a Google Doc.
pub(crate) async fn encode(node: &Node, path: &Path, options: Option<EncodeOptions>) -> Result<()> {
    let mut options = options.unwrap_or_default();
    options.rpng_content = true;
    options.rpng_link = true;

    codec_pandoc::encode(node, Some(path), "docx", &[], Some(options)).await?;

    Ok(())
}
