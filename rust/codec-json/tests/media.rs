use std::path::Path;
use stencila_codec::{
    Codec, EncodeOptions,
    eyre::Result,
    stencila_schema::{Article, Block, ImageObject, Node, Supplement},
};
use stencila_codec_json::JsonCodec;

#[tokio::test]
async fn json_references_follow_output_and_copies_preserve_original_bytes() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let source = dir.path().join("source");
    std::fs::create_dir(&source)?;
    std::fs::write(source.join("article.xml"), "<article/>")?;
    let bytes = include_bytes!("../../codec-jats/tests/media/extensionless/figure.tif");
    std::fs::write(source.join("figure.tif"), bytes)?;
    std::fs::write(source.join("data.csv"), "x,y\n1,2\n")?;
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
    let output = dir.path().join("elsewhere/out.json");
    let opts = EncodeOptions {
        from_path: Some(source.join("article.xml")),
        relocate_media: Some(true),
        ..Default::default()
    };
    JsonCodec
        .to_path(&node, &output, Some(opts.clone()))
        .await?;
    let json = std::fs::read_to_string(&output)?;
    assert!(json.contains("../source/figure.tif"));
    assert!(json.contains("../source/data.csv"));
    assert!(!json.contains("data:image"));

    JsonCodec
        .to_path(
            &node,
            &output,
            Some(EncodeOptions {
                extract_media: Some(dir.path().join("originals")),
                extract_supplements: Some(dir.path().join("supplements")),
                ..opts.clone()
            }),
        )
        .await?;
    let json = std::fs::read_to_string(&output)?;
    let (copied, _) = JsonCodec.from_str(&json, None).await?;
    let Node::Article(article) = copied else {
        stencila_codec::eyre::bail!("expected article")
    };
    let Block::ImageObject(image) = &article.content[0] else {
        stencila_codec::eyre::bail!("expected image")
    };
    let copied_path = output
        .parent()
        .unwrap_or(Path::new("."))
        .join(&image.content_url);
    assert_eq!(std::fs::read(copied_path)?, bytes);
    let Block::Supplement(supplement) = &article.content[1] else {
        stencila_codec::eyre::bail!("expected supplement")
    };
    let target = supplement
        .target
        .as_deref()
        .ok_or_else(|| stencila_codec::eyre::eyre!("target missing"))?;
    assert_eq!(
        std::fs::read(output.parent().unwrap_or(Path::new(".")).join(target))?,
        b"x,y\n1,2\n"
    );

    let (embedded, _) = JsonCodec
        .to_string(
            &node,
            Some(EncodeOptions {
                embed_media: Some(true),
                ..opts
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
