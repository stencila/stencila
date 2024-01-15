use codec::{
    common::{
        eyre::{bail, Result},
        serde_json::{self, json},
        tokio,
    },
    schema::{
        shortcuts::{art, p, t},
        Article, Block, Inline,
    },
    Codec, EncodeOptions,
};
use common_dev::pretty_assertions::assert_eq;

use codec_json::JsonCodec;

/// Test serialization and deserialization of a high level creative work type
///
/// Mainly for testing correct ser/de of options including non-core options
/// in the `options` property.
#[test]
fn article() -> Result<()> {
    let value = json!({
        "type": "Article",
        "content": []
    });
    let article: Article = serde_json::from_value(value.clone())?;
    assert!(article.content.is_empty());
    assert!(article.id.is_none());
    assert!(article.options.alternate_names.is_none());
    assert_eq!(serde_json::to_value(&article)?, value);

    let value = json!({
        "type": "Article",
        "content": [{
            "type": "Paragraph",
            "content": [
                {
                    "type": "Text",
                    "value": "Some text"
                }
            ]
        }],
        "id": "An id",
        "alternateNames": ["Another name"]
    });

    let article: Article = serde_json::from_value(value.clone())?;
    match &article.content[0] {
        Block::Paragraph(para) => match &para.content[0] {
            Inline::Text(text) => assert_eq!(text.value.as_str(), "Some text"),
            _ => bail!("Unexpected inline type"),
        },
        _ => bail!("Unexpected block type"),
    }
    assert_eq!(article.id, Some("An id".to_string()));
    assert_eq!(
        article.options.alternate_names,
        Some(vec!["Another name".to_string()])
    );
    assert_eq!(serde_json::to_value(&article)?, value);

    Ok(())
}

/// Test of compact option
#[tokio::test]
async fn compact() -> Result<()> {
    let codec = JsonCodec {};

    let doc1 = art([p([t("Hello world")])]);

    let (json, ..) = codec
        .to_string(
            &doc1,
            Some(EncodeOptions {
                compact: Some(true),
                ..Default::default()
            }),
        )
        .await?;
    assert_eq!(
        json,
        r#"{"type":"Article","content":[{"type":"Paragraph","content":[{"type":"Text","value":"Hello world"}]}]}"#
    );

    let (doc2, _) = codec.from_str(&json, None).await?;
    assert_eq!(doc2, doc1);

    Ok(())
}

/// Test of standalone option
#[tokio::test]
async fn standalone() -> Result<()> {
    let codec = JsonCodec {};

    let doc1 = art([p([t("Hello world")])]);

    let (json, ..) = codec
        .to_string(
            &doc1,
            Some(EncodeOptions {
                standalone: Some(true),
                ..Default::default()
            }),
        )
        .await?;
    assert_eq!(
        json,
        r#"{
  "$schema": "https://stencila.org/Article.schema.json",
  "@context": "https://stencila.org/context.jsonld",
  "type": "Article",
  "content": [
    {
      "type": "Paragraph",
      "content": [
        {
          "type": "Text",
          "value": "Hello world"
        }
      ]
    }
  ]
}"#
    );

    let (doc2, _) = codec.from_str(&json, None).await?;
    assert_eq!(doc2, doc1);

    Ok(())
}
