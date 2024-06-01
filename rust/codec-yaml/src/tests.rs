use codec::{
    common::tokio,
    schema::shortcuts::{art, p, t},
};

use super::*;

/// Test of standalone option
#[tokio::test]
async fn standalone() -> Result<()> {
    let codec = YamlCodec {};

    let doc1 = art([p([t("Hello world")])]);

    let (yaml, ..) = codec
        .to_string(
            &doc1,
            Some(EncodeOptions {
                standalone: Some(true),
                ..Default::default()
            }),
        )
        .await?;
    assert_eq!(
        yaml,
        r#"$schema: https://stencila.org/Article.schema.json
'@context': https://stencila.org/context.jsonld
type: Article
content:
- type: Paragraph
  content:
  - type: Text
    value:
      string: Hello world
"#
    );

    let (doc2, ..) = codec.from_str(&yaml, None).await?;
    assert_eq!(doc2, doc1);

    Ok(())
}
