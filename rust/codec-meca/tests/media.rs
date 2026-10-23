use std::path::Path;
use stencila_codec::{
    Codec, DecodeOptions,
    eyre::{OptionExt, Result},
    stencila_schema::{Block, Node},
};
use stencila_codec_meca::MecaCodec;

#[tokio::test]
async fn referenced_originals_outlive_archive_extraction() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let archive = dir.path().join("article.meca");
    std::fs::write(&archive, include_bytes!("fixtures/originals.meca"))?;
    let (node, _, info) = MecaCodec
        .from_path(
            &archive,
            Some(DecodeOptions {
                embed_media: Some(false),
                embed_supplements: Some(false),
                reproducible: Some(false),
                ..Default::default()
            }),
        )
        .await?;
    assert!(info.losses.is_empty());
    let Node::Article(article) = node else {
        stencila_codec::eyre::bail!("expected article")
    };
    let Block::Figure(figure) = &article.content[0] else {
        stencila_codec::eyre::bail!("expected figure")
    };
    let Block::ImageObject(image) = &figure.content[0] else {
        stencila_codec::eyre::bail!("expected image")
    };
    assert_eq!(image.media_type.as_deref(), Some("image/tiff"));
    assert!(image.content_url.starts_with("article.meca.media/"));
    assert_eq!(
        std::fs::read(dir.path().join(&image.content_url))?,
        include_bytes!("../../codec-jats/tests/media/extensionless/figure.tif")
    );
    let Block::Supplement(supplement) = &article.content[1] else {
        stencila_codec::eyre::bail!("expected supplement")
    };
    assert!(supplement.options.work.is_none());
    let target = supplement.target.as_deref().ok_or_eyre("missing target")?;
    assert!(target.starts_with("article.meca.supplements/"));
    assert_eq!(std::fs::read(dir.path().join(target))?, b"x,y\n1,2\n");
    Ok(())
}

#[tokio::test]
async fn default_and_explicit_embedding_are_independent() -> Result<()> {
    let archive = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/originals.meca");
    // Use a temporary copy because references create durable side directories.
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("article.meca");
    std::fs::copy(archive, &path)?;
    for (media, supplements) in [
        (None, None),
        (Some(true), Some(false)),
        (Some(false), Some(true)),
    ] {
        let (node, _, _) = MecaCodec
            .from_path(
                &path,
                Some(DecodeOptions {
                    embed_media: media,
                    embed_supplements: supplements,
                    reproducible: Some(false),
                    ..Default::default()
                }),
            )
            .await?;
        let json = serde_json::to_string(&node)?;
        assert_eq!(
            json.contains("data:image/png;base64,"),
            media.unwrap_or(true)
        );
        let Node::Article(article) = node else {
            stencila_codec::eyre::bail!("expected article")
        };
        let Block::Supplement(supplement) = &article.content[1] else {
            stencila_codec::eyre::bail!("expected supplement")
        };
        assert_eq!(
            supplement.options.work.is_some(),
            supplements.unwrap_or(true)
        );
    }
    Ok(())
}
