use std::path::PathBuf;

use insta::assert_snapshot;
use stencila_codec::{Codec, EncodeOptions, eyre::Result};
use stencila_codec_latex::LatexCodec;
use stencila_document::Document;

/// Test encoding of citations to LaTeX in both source and render modes
///
/// This tests that Citation and CitationGroup nodes are properly encoded
/// to LaTeX citation commands (\citep, \citet, etc.) based on their citation_mode.
#[tokio::test(flavor = "multi_thread")]
async fn citations() -> Result<()> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/citations/main.tex")
        .canonicalize()?;

    // Decode the LaTeX file (using default coarse decoding)
    let (article, ..) = LatexCodec.from_path(&path, None).await?;

    // Encode without render mode (source mode) - should output citation commands
    let (latex_source, ..) = LatexCodec
        .to_string(
            &article,
            Some(EncodeOptions {
                render: Some(false),
                ..Default::default()
            }),
        )
        .await?;
    assert_snapshot!("citations.source.tex", &latex_source);

    // Open and compile the document to populate citation content via Hayagriva
    let doc = Document::open(&path, None).await?;
    doc.compile().await?;

    // Get the compiled root node
    let compiled_article = doc.root().await;

    // Encode with render mode - should output rendered citation content
    let (latex_render, ..) = LatexCodec
        .to_string(
            &compiled_article,
            Some(EncodeOptions {
                render: Some(true),
                ..Default::default()
            }),
        )
        .await?;
    assert_snapshot!("citations.render.tex", &latex_render);

    Ok(())
}
