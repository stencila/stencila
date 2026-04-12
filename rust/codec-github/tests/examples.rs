use std::{fs::read_to_string, path::PathBuf};

use glob::glob;

use insta::assert_json_snapshot;
use stencila_codec::{
    Codec,
    eyre::{OptionExt, Result},
    stencila_format::Format,
};
use stencila_codec_github::{GithubCodec, export_pull_request};
use stencila_codec_markdown::MarkdownCodec;
use stencila_codec_markdown_trait::{
    MarkdownCodec as _, MarkdownEncodeContext, MarkdownEncodeMode,
};

/// Decode GitHub API responses into Stencila schema nodes
#[tokio::test]
async fn api_responses() -> Result<()> {
    let pattern = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/examples")
        .canonicalize()?
        .to_string_lossy()
        .to_string()
        + "/**/*.json";

    for path in glob(&pattern)?.flatten() {
        let id = path
            .file_name()
            .map(|name| name.to_string_lossy())
            .and_then(|name| name.strip_suffix(".json").map(String::from))
            .ok_or_eyre("should have .json suffix")?;

        let json = read_to_string(&path)?;
        let (node, ..) = GithubCodec.from_str(&json, None).await?;

        assert_json_snapshot!(id, node);
    }

    Ok(())
}

/// Export review comments and suggestions from Stencila Markdown fixtures.
#[tokio::test]
async fn pull_request_exports() -> Result<()> {
    let pattern = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/examples")
        .canonicalize()?
        .to_string_lossy()
        .to_string()
        + "/**/*.smd";

    let mut fixture_count = 0usize;

    for path in glob(&pattern)?.flatten() {
        fixture_count += 1;

        let id = path
            .file_name()
            .map(|name| name.to_string_lossy())
            .and_then(|name| {
                name.strip_suffix(".smd")
                    .or_else(|| name.strip_suffix(".md"))
                    .map(String::from)
            })
            .ok_or_eyre("should have .smd or .md suffix")?;

        let source = read_to_string(&path)?;
        let (node, ..) = MarkdownCodec.from_str(&source, None).await?;

        let mut context =
            MarkdownEncodeContext::new(Some(Format::Smd), Some(MarkdownEncodeMode::Clean));
        node.to_markdown(&mut context);

        let review =
            export_pull_request(&node, &context.content, Format::Smd, Some(&context.mapping))?;

        assert_json_snapshot!(id, review);
    }

    assert!(
        fixture_count > 0,
        "expected at least one review export fixture"
    );

    Ok(())
}
