use common::eyre::{bail, Result};
use schema::{Article, Block, Inline};

/// Test deserialization and deserialization of a high level creative work type
///
/// Mainly for testing correct ser/de of options including non-core options
/// in the `options` property.
#[test]
fn article() -> Result<()> {
    use common::serde_json::{self, json};

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
            Inline::Text(text) => assert_eq!(text.value.0, "Some text"),
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
