//! Most of these tests are trivial but are important given that we use
//! them as the main unit tests for `serde` settings (e.g. untagged, flatten)
//! and other things (e.g. ordering of types in the `Node` enum) which
//! affect serialization and deserialization in the `schema` crate.
//!
//! Other `serde`-based codecs (e.g. `yaml`) do not have as comprehensive unit tests
//! (although they do have round-trip prop tests) because they should work if these tests pass).

use codec::{
    common::tokio,
    schema::{
        shortcuts::{p, text},
        Array, Article, ArticleOptions, Block, Boolean, Date, Emphasis, Inline, Integer,
        IntegerOrString, Node, Null, Number, Object, Paragraph, Primitive, Time,
    },
};
use common_dev::pretty_assertions::assert_eq;

use super::*;

/// Test deserialization of primitive types from JSON
#[test]
fn primitive_types_from_json() -> Result<()> {
    assert_eq!(Null::from_json("null")?, Null {});

    assert_eq!(Boolean::from_json("true")?, true);
    assert_eq!(Boolean::from_json("false")?, false);

    assert_eq!(Integer::from_json("123")?, 123);
    assert_eq!(Integer::from_json("-123")?, -123);

    assert_eq!(Number::from_json("1.23")?, 1.23);
    assert_eq!(Number::from_json("-1.23")?, -1.23);

    assert_eq!(String::from_json(r#""""#)?, String::default());
    assert_eq!(
        String::from_json("\"Hello world\"")?,
        "Hello world".to_string()
    );

    assert_eq!(Array::from_json("[]")?, Array::new());
    assert_eq!(
        Array::from_json(r#"[false, 1, 1.23, "abc"]"#)?,
        Array::from([
            Primitive::Boolean(false),
            Primitive::Integer(1),
            Primitive::Number(1.23),
            Primitive::String("abc".to_string())
        ])
    );

    assert_eq!(Object::from_json("{}")?, Object::default());
    assert_eq!(
        Object::from_json(r#"{"a": 1, "b": [-1], "c": {"d": true}}"#)?,
        Object::from([
            ("a", Primitive::Integer(1)),
            ("b", Primitive::Array(Array::from([Primitive::Integer(-1)]))),
            (
                "c",
                Primitive::Object(Object::from([("d", Primitive::Boolean(true))]))
            )
        ])
    );

    Ok(())
}

/// Test deserialization of `Primitive` enum from JSON
#[test]
fn primitive_enum_from_json() -> Result<()> {
    assert_eq!(Primitive::from_json("null")?, Primitive::Null(Null {}));
    assert_eq!(Primitive::from_json("true")?, Primitive::Boolean(true));
    assert_eq!(Primitive::from_json("123")?, Primitive::Integer(123));
    assert_eq!(Primitive::from_json("1.23")?, Primitive::Number(1.23));
    assert_eq!(
        Primitive::from_json(r#""abc""#)?,
        Primitive::String("abc".to_string())
    );
    assert_eq!(Primitive::from_json("[]")?, Primitive::Array(Array::new()));
    assert_eq!(
        Primitive::from_json("{}")?,
        Primitive::Object(Object::new())
    );

    Ok(())
}

/// Test deserialization of various entity types, including those with `options`
#[test]
fn entity_types_from_json() -> Result<()> {
    assert_eq!(
        Date::from_json(r#"{ "type":"Date", "value": "2022-02-02" }"#)?,
        Date {
            value: "2022-02-02".to_string(),
            ..Default::default()
        }
    );

    assert_eq!(
        Article::from_json(
            r#"{
                    "type": "Article",
                    "content": [
                        {
                            "type": "Paragraph",
                            "content": ["Hello world"]
                        }
                    ],
                    "pageStart": 1,
                    "pageEnd": "MXC"
                }"#
        )?,
        Article {
            content: vec![Block::Paragraph(Paragraph {
                content: vec![Inline::String("Hello world".to_string())],
                ..Default::default()
            })],
            options: Box::new(ArticleOptions {
                page_start: Some(IntegerOrString::Integer(1)),
                page_end: Some(IntegerOrString::String("MXC".to_string())),
                ..Default::default()
            }),
            ..Default::default()
        }
    );

    Ok(())
}

/// Test deserialization of various entity enums from JSON
#[test]
fn entity_enum_from_json() -> Result<()> {
    assert_eq!(
        Inline::from_json(r#""abc""#)?,
        Inline::String("abc".to_string())
    );

    assert_eq!(
        Inline::from_json(r#"{ "type":"Emphasis", "content":[] }"#)?,
        Inline::Emphasis(Emphasis {
            content: vec![],
            ..Default::default()
        })
    );

    assert_eq!(
        Block::from_json(r#"{ "type":"Paragraph", "content":[] }"#)?,
        Block::Paragraph(Paragraph {
            content: vec![],
            ..Default::default()
        })
    );

    assert_eq!(Node::from_json("123")?, Node::Integer(123));

    assert_eq!(
        Node::from_json(r#""abc""#)?,
        Node::String("abc".to_string())
    );

    assert_eq!(
        Node::from_json(r#"{ "type":"Time", "value":"01:02:03" }"#)?,
        Node::Time(Time {
            value: "01:02:03".to_string(),
            ..Default::default()
        })
    );

    Ok(())
}

/// Test of compact option
#[tokio::test]
async fn compact() -> Result<()> {
    let codec = JsonCodec {};

    let doc1 = Node::Article(Article::new(vec![p([text("Hello world")])]));

    let (json, _) = codec
        .to_string(
            &doc1,
            Some(EncodeOptions {
                compact: true,
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

    let doc1 = Node::Article(Article::new(vec![p([text("Hello world")])]));

    let (json, _) = codec
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
  "$schema": "https://stencila.dev/Article.schema.json",
  "@context": "https://stencila.dev/Article.jsonld",
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
