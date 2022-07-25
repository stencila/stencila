use std::path::Path;

use codec::{
    common::{eyre::Result, tempfile::tempdir, tracing},
    stencila_schema::{Article, Node},
    EncodeOptions,
};
use provider_gdrive::{GoogleDriveProvider, ProviderTrait};

/// Encode a Stencila `Node` as a Google Doc
///
/// The Google Docs API v1 [`documents.create` method](https://developers.google.com/docs/api/reference/rest/v1/documents/create)
/// ignores any provided content (i.e. only empty documents can be created). However, the Google Drive API has a
/// [`files.create` method](https://developers.google.com/drive/api/v3/reference/files/create) which allows a new Google Doc
/// to be created by uploading a Microsoft Word file.
///
/// Therefore, this function 'encodes' a new `.gdoc` file by:
///
/// 1. creating a temporary `.docx` file
/// 2. creating a new Google Doc by uploading the `.docx` file
/// 3. pulling the new Google Doc to a `.gdoc` file at `path`
///
/// Step three is not entirely necessary (since the Google Doc is available online) but provides
/// consistent UX when users are using Stencila to convert between file formats.
pub(crate) async fn encode(node: &Node, path: &Path, options: Option<EncodeOptions>) -> Result<()> {
    // Encode to docx
    let tempdir = tempdir()?;
    let docx = tempdir.path().join("temp.docx");
    codec_pandoc::encode(
        node,
        Some(&docx),
        "docx",
        &[],
        Some(EncodeOptions {
            rpng_types: vec![
                "CodeExpression".to_string(),
                "CodeChunk".to_string(),
                "MathBlock".to_string(),
                "MathFragment".to_string(),
            ],
            rpng_text: true,
            rpng_link: true,
            ..options.unwrap_or_default()
        }),
    )
    .await?;

    // Create new Google Doc from docx
    let gdoc = GoogleDriveProvider::push(&Node::Article(Article::default()), &docx, None).await?;
    if let Node::Article(Article { url: Some(url), .. }) = &gdoc {
        tracing::info!("Successfully created Google Doc: {}", url);
    }

    // Pull the new Google Doc
    GoogleDriveProvider::pull(&gdoc, path, None).await?;

    Ok(())
}
