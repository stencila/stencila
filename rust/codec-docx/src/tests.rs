use codec::{
    Codec, EncodeOptions,
    eyre::Result,
    schema::{
        Article, ArticleOptions, Node, Object, Primitive,
        shortcuts::{cc, ce, p, t},
    },
};
use indexmap::IndexMap;
use serde_json::{self, json};
use tempfile::tempdir;
use common_dev::pretty_assertions::assert_eq;
use version::STENCILA_VERSION;

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
            repository: Some("repository".into()),
            path: Some("path".into()),
            commit: Some("commit".into()),
            extra: Some(Object(IndexMap::from([
                (
                    "generator".into(),
                    Primitive::String(format!("Stencila {STENCILA_VERSION}")),
                ),
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

    let (mut round_tripped, ..) = DocxCodec.from_path(&path, None).await?;

    // Strip the encoding options inserted into extra
    if let Node::Article(Article { options, .. }) = &mut round_tripped
        && let Some(extra) = &mut options.extra
    {
        extra.swap_remove("encoding");
    };

    assert_eq!(round_tripped, article);

    Ok(())
}
