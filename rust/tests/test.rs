//! These tests are intentionally simple and just test that
//! node types have expected traits e.g. `Clone`, `Serialize` etc.

use pretty_assertions::assert_eq;
use serde_json::{json, Result, Value};
use stencila_schema::{
    Article, BlockContent, CodeExpression, CreativeWorkAuthors, CreativeWorkTitle, InlineContent,
    Paragraph, Person,
};

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
fn is_clonable() {
    let article1 = article_fixture();
    let _article2 = article1.clone();
}

#[test]
fn is_debugable() {
    let article = article_fixture();

    assert!(format!("{:?}", article).starts_with("Article {"))
}

#[test]
fn is_serdeable() -> Result<()> {
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
