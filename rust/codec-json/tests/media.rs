use std::path::Path;
use stencila_codec::{
    Codec, EncodeOptions,
    eyre::Result,
    stencila_schema::{Article, Block, ImageObject, Node},
};
use stencila_codec_json::JsonCodec;

#[tokio::test]
async fn embedding_converts_media_and_leaves_the_node_unchanged() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let bytes = include_bytes!("../../codec-jats/tests/media/extensionless/figure.tif");
    std::fs::write(dir.path().join("figure.tif"), bytes)?;
    std::fs::write(dir.path().join("article.xml"), "<article/>")?;
    let node = Node::Article(Article::new(vec![Block::ImageObject(ImageObject {
        content_url: "figure.tif".into(),
        media_type: Some("image/tiff".into()),
        ..Default::default()
    })]));
    let (embedded, _) = JsonCodec
        .to_string(
            &node,
            Some(EncodeOptions {
                from_path: Some(dir.path().join("article.xml")),
                embed_media: Some(true),
                ..Default::default()
            }),
        )
        .await?;
    assert!(embedded.contains("data:image/png;base64,"));
    let value: serde_json::Value = serde_json::from_str(&embedded)?;
    assert_eq!(value["content"][0]["mediaType"], "image/png");
    assert!(!serde_json::to_string(&node)?.contains("data:image"));
    Ok(())
}

#[tokio::test]
async fn extraction_of_embedded_media_also_preserves_bytes() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let bytes = include_bytes!("../../codec-jats/tests/media/extensionless/figure.tif");
    let encoded = bytes
        .iter()
        .map(|byte| format!("%{byte:02X}"))
        .collect::<String>();
    let node = Node::ImageObject(ImageObject::new(format!("data:image/tiff,{encoded}")));
    let output = dir.path().join("nested/out.json");
    JsonCodec
        .to_path(
            &node,
            &output,
            Some(EncodeOptions {
                from_path: Some(dir.path().join("elsewhere/source.xml")),
                extract_media: Some(dir.path().join("media")),
                ..Default::default()
            }),
        )
        .await?;
    let (node, _, _) = JsonCodec.from_path(&output, None).await?;
    let Node::ImageObject(image) = node else {
        stencila_codec::eyre::bail!("expected image")
    };
    assert_eq!(
        std::fs::read(
            output
                .parent()
                .unwrap_or(Path::new("."))
                .join(image.content_url)
        )?,
        bytes
    );
    Ok(())
}

#[tokio::test]
async fn references_are_preserved_unless_relocation_is_requested() -> Result<()> {
    let dir = tempfile::tempdir()?;
    std::fs::write(dir.path().join("img.tif"), b"bytes")?;
    let absolute = dir.path().join("img.tif").to_string_lossy().to_string();
    let node = Node::Article(Article::new(vec![
        Block::ImageObject(ImageObject::new("img.tif".into())),
        Block::ImageObject(ImageObject::new(absolute.clone())),
    ]));

    // As for `Document::store`: a path elsewhere with no source path or opt-in
    let output = dir.path().join(".stencila/cache/doc.json");
    JsonCodec
        .to_path(
            &node,
            &output,
            Some(EncodeOptions {
                from_path: Some(dir.path().join("doc.smd")),
                ..Default::default()
            }),
        )
        .await?;
    let (decoded, _, _) = JsonCodec.from_path(&output, None).await?;
    let Node::Article(article) = decoded else {
        stencila_codec::eyre::bail!("expected article")
    };
    let urls = article
        .content
        .iter()
        .filter_map(|block| match block {
            Block::ImageObject(image) => Some(image.content_url.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(urls, vec!["img.tif", absolute.as_str()]);
    Ok(())
}
