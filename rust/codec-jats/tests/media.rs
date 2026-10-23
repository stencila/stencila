use std::path::{Path, PathBuf};

use stencila_codec::{
    Codec, DecodeOptions,
    eyre::Result,
    stencila_schema::{Block, Inline, Node},
};
use stencila_codec_jats::{JatsCodec, decode};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/media")
        .join(name)
}

fn options() -> Option<DecodeOptions> {
    Some(DecodeOptions {
        reproducible: Some(false),
        ..Default::default()
    })
}

#[tokio::test]
async fn loose_pmc_figures_and_supplements() -> Result<()> {
    let path = fixture("PMC5754353.1/PMC5754353.1.xml");
    let (node, _, info) = JatsCodec.from_path(&path, options()).await?;
    let json = serde_json::to_string(&node)?;
    assert!(json.contains("41467_2017_2484_Fig1_HTML.jpg"));
    assert!(json.contains("image/jpeg"));
    assert!(!json.contains("data:image"));
    assert!(
        !info
            .losses
            .iter()
            .any(|(loss, _)| loss.contains("unresolved:"))
    );
    Ok(())
}

#[tokio::test]
async fn extensionless_media_and_missing_references() -> Result<()> {
    let path = fixture("extensionless/article.xml");
    let (node, _, info) = JatsCodec.from_path(&path, options()).await?;
    let json = serde_json::to_string(&node)?;
    for expected in [
        "figure.tif",
        "small.png",
        "audio.wav",
        "audio/wav",
        "movie.mp4",
        "video/mp4",
        "data.csv",
        "attachment.zip",
        "https://example.org/video.mp4",
    ] {
        assert!(json.contains(expected), "missing {expected}: {json}");
    }
    let missing = info
        .losses
        .iter()
        .filter(|(loss, _)| loss.contains("unresolved:"))
        .collect::<Vec<_>>();
    assert_eq!(missing.len(), 2);
    assert!(
        missing
            .iter()
            .any(|(loss, _)| loss.contains("fig[2]/graphic[1]/@xlink:href (unresolved: absent)"))
    );
    assert!(
        missing
            .iter()
            .any(|(loss, _)| loss.contains("p[2]/inline-graphic[1]"))
    );
    Ok(())
}

#[tokio::test]
async fn nih_manuscript_without_files_is_not_an_error() -> Result<()> {
    let (node, _, info) = JatsCodec
        .from_path(&fixture("nih-manuscript/article.xml"), options())
        .await?;
    let Node::Article(article) = node else {
        stencila_codec::eyre::bail!("expected article")
    };
    assert_eq!(
        article
            .content
            .iter()
            .filter(|block| matches!(block, Block::Figure(..)))
            .count(),
        2
    );
    assert_eq!(
        info.losses
            .iter()
            .filter(|(loss, _)| loss.contains("unresolved:"))
            .count(),
        3
    );
    Ok(())
}

#[tokio::test]
async fn exact_file_wins_and_raster_size_breaks_stem_ties() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("article.xml");
    std::fs::write(
        &path,
        r#"<article xmlns:xlink="http://www.w3.org/1999/xlink"><body><fig><graphic xlink:href="figure.jpg"/></fig><fig><graphic xlink:href="figure"/></fig></body></article>"#,
    )?;
    image::RgbImage::new(10, 10).save(dir.path().join("figure.jpg"))?;
    image::RgbImage::new(20, 20).save(dir.path().join("figure.png"))?;
    image::RgbImage::new(10, 10).save(dir.path().join("figure.tif"))?;
    let (node, _, _) = JatsCodec.from_path(&path, options()).await?;
    let json = serde_json::to_string(&node)?;
    assert!(json.contains("figure.jpg"));
    assert!(json.contains("figure.png"));
    assert!(!json.contains("figure.tif"));
    std::fs::write(dir.path().join("figure.svg"), "<svg/>")?;
    let (node, _, _) = JatsCodec.from_path(&path, options()).await?;
    assert!(serde_json::to_string(&node)?.contains("figure.svg"));
    Ok(())
}

#[test]
fn string_decoding_does_not_look_for_files() -> Result<()> {
    let (node, info) = decode(
        r#"<article xmlns:xlink="http://www.w3.org/1999/xlink"><body><p><inline-graphic xlink:href="missing"/></p></body></article>"#,
        None,
    )?;
    assert!(info.losses.is_empty());
    let Node::Article(article) = node else {
        stencila_codec::eyre::bail!("unexpected decoded node")
    };
    let Block::Paragraph(paragraph) = &article.content[0] else {
        stencila_codec::eyre::bail!("unexpected decoded node")
    };
    let Inline::ImageObject(image) = &paragraph.content[0] else {
        stencila_codec::eyre::bail!("unexpected decoded node")
    };
    assert_eq!(image.content_url, "missing");
    Ok(())
}

#[test]
fn supplement_media_type_attributes_infer_work_type() -> Result<()> {
    let (node, info) = decode(
        r#"<article xmlns:xlink="http://www.w3.org/1999/xlink"><body><supplementary-material xlink:href="supp1" mimetype="video" mime-subtype="mp4"/></body></article>"#,
        None,
    )?;
    assert!(info.losses.is_empty());
    let Node::Article(article) = node else {
        stencila_codec::eyre::bail!("unexpected decoded node")
    };
    let Block::Supplement(supplement) = &article.content[0] else {
        stencila_codec::eyre::bail!("unexpected decoded node")
    };
    assert_eq!(
        supplement.work_type,
        Some(stencila_codec::stencila_schema::CreativeWorkType::VideoObject)
    );
    Ok(())
}

#[tokio::test]
async fn supplement_media_type_inferred_from_resolved_file() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("article.xml");
    std::fs::write(
        &path,
        r#"<article xmlns:xlink="http://www.w3.org/1999/xlink"><body><supplementary-material xlink:href="supp1"/></body></article>"#,
    )?;
    std::fs::write(dir.path().join("supp1.wav"), b"RIFF")?;
    let (node, _, _) = JatsCodec.from_path(&path, options()).await?;
    let Node::Article(article) = node else {
        stencila_codec::eyre::bail!("unexpected decoded node")
    };
    let Block::Supplement(supplement) = &article.content[0] else {
        stencila_codec::eyre::bail!("unexpected decoded node")
    };
    assert_eq!(
        supplement.work_type,
        Some(stencila_codec::stencila_schema::CreativeWorkType::AudioObject)
    );
    Ok(())
}

#[test]
fn image_valued_inline_media_keeps_description() -> Result<()> {
    let (node, info) = decode(
        r#"<article xmlns:xlink="http://www.w3.org/1999/xlink"><body><p><inline-media xlink:href="f1.png" mimetype="image"><long-desc>Detailed description</long-desc></inline-media></p></body></article>"#,
        None,
    )?;
    assert!(info.losses.is_empty());
    let Node::Article(article) = node else {
        stencila_codec::eyre::bail!("unexpected decoded node")
    };
    let Block::Paragraph(paragraph) = &article.content[0] else {
        stencila_codec::eyre::bail!("unexpected decoded node")
    };
    let Inline::ImageObject(image) = &paragraph.content[0] else {
        stencila_codec::eyre::bail!("unexpected decoded node")
    };
    assert_eq!(
        image.options.description.as_deref(),
        Some("Detailed description")
    );
    assert_eq!(image.media_type, None);
    Ok(())
}
