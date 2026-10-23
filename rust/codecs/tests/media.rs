//! Rebasing of media and supplement references for exports

use std::path::Path;

use stencila_codec::{
    eyre::{Result, bail, eyre},
    stencila_format::Format,
    stencila_schema::{Article, Block, ImageObject, Node, Supplement},
};
use stencila_codecs::{EncodeOptions, from_path, to_path, to_string};

const BYTES: &[u8] = include_bytes!("../../codec-jats/tests/media/extensionless/figure.tif");

/// A source directory with an image and a supplement, and a node referring to them
fn source(dir: &Path) -> Result<(Node, EncodeOptions)> {
    let source = dir.join("source");
    std::fs::create_dir(&source)?;
    std::fs::write(source.join("figure.tif"), BYTES)?;
    std::fs::write(source.join("data.csv"), "x,y\n1,2\n")?;
    std::fs::write(source.join("article.xml"), "<article/>")?;
    let node = Node::Article(Article::new(vec![
        Block::ImageObject(ImageObject {
            content_url: "figure.tif".into(),
            media_type: Some("image/tiff".into()),
            ..Default::default()
        }),
        Block::Supplement(Supplement {
            target: Some("data.csv".into()),
            ..Default::default()
        }),
    ]));
    let options = EncodeOptions {
        from_path: Some(source.join("article.xml")),
        relocate_media: Some(true),
        ..Default::default()
    };
    Ok((node, options))
}

fn references(node: &Node) -> Result<(String, String)> {
    let Node::Article(article) = node else {
        bail!("expected article")
    };
    let Block::ImageObject(image) = &article.content[0] else {
        bail!("expected image")
    };
    let Block::Supplement(supplement) = &article.content[1] else {
        bail!("expected supplement")
    };
    Ok((
        image.content_url.clone(),
        supplement
            .target
            .clone()
            .ok_or_else(|| eyre!("no target"))?,
    ))
}

#[tokio::test]
async fn json_references_follow_output_and_copies_preserve_bytes() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let (node, options) = source(dir.path())?;
    let output = dir.path().join("elsewhere/out.json");

    to_path(&node, &output, Some(options.clone())).await?;
    let json = std::fs::read_to_string(&output)?;
    assert!(json.contains("../source/figure.tif"));
    assert!(json.contains("../source/data.csv"));

    to_path(
        &node,
        &output,
        Some(EncodeOptions {
            extract_media: Some(dir.path().join("originals")),
            extract_supplements: Some(dir.path().join("supplements")),
            ..options.clone()
        }),
    )
    .await?;
    let (image, target) = references(&from_path(&output, None).await?)?;
    let base = output.parent().unwrap_or(Path::new("."));
    assert!(image.starts_with("../originals/"));
    assert_eq!(std::fs::read(base.join(image))?, BYTES);
    assert_eq!(std::fs::read(base.join(target))?, b"x,y\n1,2\n");

    let embedded = to_string(
        &node,
        Some(EncodeOptions {
            format: Some(Format::Json),
            embed_media: Some(true),
            ..options
        }),
    )
    .await?;
    assert!(embedded.contains("data:image/png;base64,"));
    assert!(embedded.contains("data.csv"));
    Ok(())
}

#[tokio::test]
async fn markdown_references_follow_output() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let (node, options) = source(dir.path())?;
    let output = dir.path().join("elsewhere/out.md");
    to_path(&node, &output, Some(options)).await?;
    let markdown = std::fs::read_to_string(&output)?;
    assert!(markdown.contains("](../source/figure.tif)"), "{markdown}");
    assert!(markdown.contains("](../source/data.csv)"), "{markdown}");
    Ok(())
}

#[tokio::test]
async fn references_are_kept_without_opt_in_or_for_bundling_formats() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let (node, options) = source(dir.path())?;

    let output = dir.path().join("elsewhere/out.json");
    to_path(
        &node,
        &output,
        Some(EncodeOptions {
            relocate_media: None,
            ..options.clone()
        }),
    )
    .await?;
    let (image, target) = references(&from_path(&output, None).await?)?;
    assert_eq!(
        (image.as_str(), target.as_str()),
        ("figure.tif", "data.csv")
    );

    assert!(!stencila_codecs::keeps_media_references(&Format::Docx));
    assert!(!stencila_codecs::keeps_media_references(&Format::Pdf));
    Ok(())
}
