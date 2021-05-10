use pretty_assertions::assert_eq;
use serde_json::{json, Result, Value};
use stencila_schema::{
    Article, ArticleAuthors, ArticleTitle, BlockContent, CodeExpression, InlineContent, Paragraph,
    Person,
};

#[test]
fn article() -> Result<()> {
    let article: Article = Default::default();
    assert!(article.title.is_none());

    let article = Article {
        title: Some(ArticleTitle::String("The article title".into())),
        authors: Some(vec![ArticleAuthors::Person({
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
    };
    assert!(!article.title.is_none());
    assert!(!article.authors.is_none());

    let json_expected = json!({
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
    assert_eq!(json_val1, json_expected);

    // Test deserialization

    let article2: Article = serde_json::from_str(json_str1.as_str())?;
    let json_str2 = serde_json::to_string_pretty(&article2)?;
    let json_val2: Value = serde_json::from_str(json_str2.as_str())?;
    assert_eq!(json_val2, json_val1);

    Ok(())
}
