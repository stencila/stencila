//! These tests are intentionally simple and just test that
//! node types have expected traits e.g. `Clone`, `Serialize` etc.

use maplit::btreemap;
use pretty_assertions::assert_eq;
use serde_json::{json, Result, Value};
use stencila_schema::{
    Article, BlockContent, CodeExpression, CreativeWorkAuthors, CreativeWorkTitle, InlineContent,
    Paragraph, Person, Primitive,
};

#[test]
fn primitives_deserialize() -> Result<()> {
    let null: Primitive = serde_json::from_str("null")?;
    assert!(matches!(null, Primitive::Null));

    let bool: Primitive = serde_json::from_str("true")?;
    assert!(matches!(bool, Primitive::Boolean(_)));

    let bool: Primitive = serde_json::from_str("false")?;
    assert!(matches!(bool, Primitive::Boolean(_)));

    let integer: Primitive = serde_json::from_str("42")?;
    assert!(matches!(integer, Primitive::Integer(_)));

    let number: Primitive = serde_json::from_str("3.14")?;
    assert!(matches!(number, Primitive::Number(_)));

    let string: Primitive = serde_json::from_str("\"str  ing\"")?;
    assert!(matches!(string, Primitive::String(_)));

    let array: Primitive = serde_json::from_str(r#"[null, false, 42, 3.14, "string"]"#)?;
    if let Primitive::Array(array) = array {
        assert!(matches!(array[0], Primitive::Null));
        assert!(matches!(array[1], Primitive::Boolean(false)));
        assert!(matches!(array[2], Primitive::Integer(_)));
        assert!(matches!(array[3], Primitive::Number(_)));
        assert!(matches!(array[4], Primitive::String(_)));
    } else {
        panic!("Not an array!")
    };

    let object: Primitive = serde_json::from_str(
        r#"{
            "a": null,
            "b": false,
            "c": 42,
            "d": 3.14,
            "e": "string"
        }"#,
    )?;
    if let Primitive::Object(object) = object {
        assert!(matches!(object["a"], Primitive::Null));
        assert!(matches!(object["b"], Primitive::Boolean(false)));
        assert!(matches!(object["c"], Primitive::Integer(_)));
        assert!(matches!(object["d"], Primitive::Number(_)));
        assert!(matches!(object["e"], Primitive::String(_)));
    } else {
        panic!("Not an object!")
    };

    Ok(())
}

#[test]
fn primitives_serialize() -> Result<()> {
    let null = Primitive::Null;
    assert_eq!(serde_json::to_string(&null)?, "null");

    let bool = Primitive::Boolean(true);
    assert_eq!(serde_json::to_string(&bool)?, "true");

    let bool = Primitive::Boolean(false);
    assert_eq!(serde_json::to_string(&bool)?, "false");

    let integer = Primitive::Integer(42);
    assert_eq!(serde_json::to_string(&integer)?, "42");

    let number = Primitive::Number(3.14);
    assert_eq!(serde_json::to_string(&number)?, "3.14");

    let string = Primitive::String("string".to_string());
    assert_eq!(serde_json::to_string(&string)?, "\"string\"");

    let array = Primitive::Array(vec![
        Primitive::Null,
        Primitive::Boolean(false),
        Primitive::Integer(42),
        Primitive::Number(3.14),
        Primitive::String("string".to_string()),
    ]);
    assert_eq!(
        serde_json::to_string(&array)?,
        "[null,false,42,3.14,\"string\"]"
    );

    let object = Primitive::Object(btreemap! {
        "a".to_string() => Primitive::Null,
        "b".to_string() => Primitive::Boolean(false),
        "c".to_string() => Primitive::Integer(42),
        "d".to_string() => Primitive::Number(3.14),
        "e".to_string() => Primitive::String("string".to_string())
    });
    assert_eq!(
        serde_json::to_string(&object)?,
        r#"{"a":null,"b":false,"c":42,"d":3.14,"e":"string"}"#
    );

    Ok(())
}

fn article_fixture() -> Article {
    Article {
        title: Some(CreativeWorkTitle::String("The article title".into())),
        authors: Some(vec![CreativeWorkAuthors::Person({
            Person {
                given_names: Some(vec!["Jane".into()]),
                family_names: Some(vec!["Jones".into()]),
                ..Default::default()
            }
        })]),
        content: Some(vec![BlockContent::Paragraph(Paragraph {
            content: vec![
                InlineContent::String("A paragraph with a ".into()),
                InlineContent::CodeExpression(CodeExpression {
                    programming_language: Some("r".into()),
                    text: "2^2".into(),
                    ..Default::default()
                }),
            ],
            ..Default::default()
        })]),
        ..Default::default()
    }
}

#[test]
fn entity_is_clonable() {
    let article1 = article_fixture();
    let _article2 = article1.clone();
}

#[test]
fn entity_is_debuggable() {
    let article = article_fixture();

    assert!(format!("{:?}", article).starts_with("Article {"))
}

#[test]
fn entity_is_serdeable() -> Result<()> {
    let article = article_fixture();
    let json = json!({
      "type": "Article",
      "authors": [
        {
          "type": "Person",
          "familyNames": [
            "Jones"
          ],
          "givenNames": [
            "Jane"
          ]
        }
      ],
      "content": [
        {
          "type": "Paragraph",
          "content": [
            "A paragraph with a ",
            {
              "type": "CodeExpression",
              "text": "2^2",
              "programmingLanguage": "r"
            }
          ]
        }
      ],
      "title": "The article title"
    });

    // Test serialization

    let json_str1 = serde_json::to_string_pretty(&article)?;
    let json_val1: Value = serde_json::from_str(json_str1.as_str())?;
    assert_eq!(json_val1, json);

    // Test deserialization

    let article2: Article = serde_json::from_str(json_str1.as_str())?;
    let json_str2 = serde_json::to_string_pretty(&article2)?;
    let json_val2: Value = serde_json::from_str(json_str2.as_str())?;
    assert_eq!(json_val2, json_val1);

    Ok(())
}
