//! Most of these tests are trivial but are important given that we use
//! them as the main unit tests for `serde` settings (e.g. untagged, flatten)
//! and other things (e.g. ordering of types in the `Node` enum) which
//! affect serialization and deserialization in the `schema` crate.
//!
//! Other `serde`-based codecs (e.g. `yaml`) do not have as comprehensive unit tests
//! (although they do have round-trip prop tests) because they should work if these tests pass).

use codec::{
    common::{eyre::Result, serde_json::json},
    schema::{
        shortcuts::{p, t},
        Array, Article, ArticleOptions, Block, Boolean, Date, Emphasis, Inline, Integer,
        IntegerOrString, Node, Null, Number, Object, Paragraph, Person, Primitive, ThematicBreak,
        Time,
    },
};
use common_dev::pretty_assertions::assert_eq;

use codec_json::r#trait::JsonCodec;

/// Test deserialization of primitive types from JSON
#[test]
fn primitive_types() -> Result<()> {
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
fn primitive_enum() -> Result<()> {
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
fn entity_types() -> Result<()> {
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
                            "content": [
                                {
                                    "type": "Text",
                                    "value": "Hello world"
                                }
                            ]
                        }
                    ],
                    "pageStart": 1,
                    "pageEnd": "MXC"
                }"#
        )?,
        Article {
            content: vec![p([t("Hello world")])],
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
fn entity_enum() -> Result<()> {
    assert_eq!(
        Inline::from_json(r#"{ "type":"Text", "value":"abc" }"#)?,
        t("abc")
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

/// Test deserialization with aliases and single values for properties
#[test]
fn property_aliases() -> Result<()> {
    let article = Article::from_json_value(json!({
        "type": "Article",
        "keyword": "one",
        "alternate-name": "alt-name",
        "date": { "type": "Date", "value": "2010" },
        "content": { "type": "ThematicBreak"}
    }))?;

    assert_eq!(article.keywords, Some(vec!["one".to_string()]));
    assert_eq!(
        article.options.alternate_names,
        Some(vec!["alt-name".to_string()])
    );
    assert_eq!(article.date_published, Some(Date::new("2010".to_string())));
    assert_eq!(
        article.content,
        vec![Block::ThematicBreak(ThematicBreak::new())]
    );

    Ok(())
}

/// Test deserialization from comma and space separated strings or array
#[test]
fn csv_and_ssv_or_array() -> Result<()> {
    let person1 = Person::from_json_value(json!({
        "type": "Person",
        "givenNames": "One Two",
        "familyNames": "Tahi Rua",
        "emails": "one@example.com, two@example.org"
    }))?;

    let person2 = Person::from_json_value(json!({
        "type": "Person",
        "givenNames": ["One", "Two"],
        "familyNames": ["Tahi", "Rua"],
        "emails": ["one@example.com", "two@example.org"]
    }))?;

    assert_eq!(person1, person2);

    Ok(())
}

/// Test deserialization of dates from string
#[test]
fn date() -> Result<()> {
    let article1 = Article::from_json_value(json!({
        "type": "Article",
        "datePublished": "2022",
        "dateCreated": "2022-02",
        "dateModified": "2022-02-22",
        "dateAccepted": "22 February 2022",
        "dateReceived": "Feb 22, 2022",
        "content": []
    }))?;

    let article2 = Article::from_json_value(json!({
        "type": "Article",
        "datePublished": {
            "type": "Date",
            "value": "2022"
        },
        "dateCreated": {
            "type": "Date",
            "value": "2022-02"
        },
        "dateModified": {
            "type": "Date",
            "value": "2022-02-22"
        },
        "dateAccepted": {
            "type": "Date",
            "value": "2022-02-22"
        },
        "dateReceived": {
            "type": "Date",
            "value": "2022-02-22"
        },
        "content": []
    }))?;

    assert_eq!(article1, article2);

    Ok(())
}

/// Test deserialization of person from string
/// Note: this also tests aliases for properties
#[test]
fn person() -> Result<()> {
    let article1 = Article::from_json_value(json!({
        "type": "Article",
        "author": "Dr Alice Andrews MD <alice@example.org> (https://example.org/alice)",
        "content": []
    }))?;

    let article2 = Article::from_json_value(json!({
        "type": "Article",
        "authors": {
            "type": "Person",
            "firstName": "Alice",
            "surname": "Andrews",
            "honorificPrefix": "Dr.",
            "honorificSuffix": "MD",
            "email": "alice@example.org",
            "url": "https://example.org/alice"
        },
        "content": []
    }))?;

    assert_eq!(article1, article2);

    Ok(())
}
