use std::path::PathBuf;

use codec::{
    common::{
        eyre::Result,
        indexmap::IndexMap,
        serde_json::{self, json},
        tokio,
    },
    schema::{Article, ArticleOptions, Node, Object, Primitive},
    Codec,
};
use common_dev::pretty_assertions::assert_eq;

use crate::DocxCodec;

#[tokio::test]
async fn roundtrip_custom_properties() -> Result<()> {
    let article = Node::Article(Article {
        options: Box::new(ArticleOptions {
            extra: Some(Object(IndexMap::from([
                ("boolean".into(), Primitive::Boolean(true)),
                ("integer".into(), Primitive::Integer(123)),
                ("number".into(), Primitive::Number(1.23)),
                ("string".into(), Primitive::String("Hello world".into())),
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

    let path = PathBuf::from("temp.docx");

    DocxCodec.to_path(&article, &path, None).await?;

    let (round_tripped, ..) = DocxCodec.from_path(&path, None).await?;

    assert_eq!(round_tripped, article);

    Ok(())
}
