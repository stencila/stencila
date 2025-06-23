use codec::{
    common::{
        eyre::Result,
        indexmap::IndexMap,
        serde_json::{self, json},
        tempfile::tempdir,
        tokio,
    },
    schema::{
        shortcuts::{cc, ce, p, t},
        Article, ArticleOptions, Node, Object, Primitive,
    },
    Codec, EncodeOptions,
};
use common_dev::pretty_assertions::assert_eq;

use crate::DocxCodec;

#[tokio::test]
async fn roundtrip_basic() -> Result<()> {
    let article = Node::Article(Article {
        content: vec![
            // Content including nodes that are reconstituted (e.g. code chunk)
            // These paragraphs with code expressions in different positions are regression tests
            p([ce("0", None::<&str>)]),
            p([t("Before "), ce("1 + 2", Some("r"))]),
            p([t("Before "), ce("3 + 4", Some("python")), t(" after.")]),
            cc("plot(data)", Some("r")),
        ],
        options: Box::new(ArticleOptions {
            // Things that are placed in custom properties
            source: Some("source".into()),
            commit: Some("commit".into()),
            extra: Some(Object(IndexMap::from([
                ("boolean".into(), Primitive::Boolean(true)),
                ("integer".into(), Primitive::Integer(123)),
                ("number".into(), Primitive::Number(1.23)),
                ("string".into(), Primitive::String("a string".into())),
                ("array".into(), serde_json::from_value(json!([1, 2, 3]))?),
                (
                    "object".into(),
                    serde_json::from_value(json!({"a": 1, "b": 2}))?,
                ),
            ]))),
            ..Default::default()
        }),
        ..Default::default()
    });

    let temp_dir = tempdir()?;
    let path = temp_dir.path().join("temp.docx");

    DocxCodec
        .to_path(
            &article,
            &path,
            Some(EncodeOptions {
                reproducible: Some(true),
                ..Default::default()
            }),
        )
        .await?;

    let (round_tripped, ..) = DocxCodec.from_path(&path, None).await?;

    assert_eq!(round_tripped, article);

    Ok(())
}
