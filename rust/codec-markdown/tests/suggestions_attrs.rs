use stencila_codec::{
    Codec, DecodeOptions,
    eyre::{Result, eyre},
    stencila_schema::{
        Article, ArticleOptions, Author, Block, DateTime, Inline, Node, Paragraph, Person,
        SuggestionBlock, SuggestionInline, SuggestionType, Text,
    },
};
use stencila_codec_markdown::{MarkdownCodec, decode};

#[test]
fn decode_suggestion_inline_attrs() -> Result<()> {
    let (node, _) = decode(
        r#"A {++change++}{id="suggestion-inline", by="Alice", at="2026-04-16T10:34:00Z"}."#,
        Some(DecodeOptions::default()),
    )?;

    let Node::Article(article) = node else {
        return Err(eyre!("expected Article"));
    };

    let Block::Paragraph(paragraph) = &article.content[0] else {
        return Err(eyre!("expected paragraph"));
    };

    let Inline::SuggestionInline(suggestion) = &paragraph.content[1] else {
        return Err(eyre!("expected suggestion inline"));
    };

    assert_eq!(suggestion.id.as_deref(), Some("suggestion-inline"));
    assert_eq!(
        suggestion.authors,
        Some(vec![Author::Person(Person::from("Alice"))])
    );
    assert_eq!(
        suggestion.date_published,
        Some(DateTime::new("2026-04-16T10:34:00Z".to_string()))
    );

    Ok(())
}

#[test]
fn decode_suggestion_block_attrs() -> Result<()> {
    let (node, _) = decode(
        ":++ {id=\"suggestion-block\" by=\"Alice\" at=\"2026-04-16T10:34:00Z\"}\n\nA block change.\n\n:++\n",
        Some(DecodeOptions::default()),
    )?;

    let Node::Article(article) = node else {
        return Err(eyre!("expected Article"));
    };

    let Block::SuggestionBlock(suggestion) = &article.content[0] else {
        return Err(eyre!("expected suggestion block"));
    };

    assert_eq!(suggestion.id.as_deref(), Some("suggestion-block"));
    assert_eq!(
        suggestion.authors,
        Some(vec![Author::Person(Person::from("Alice"))])
    );
    assert_eq!(
        suggestion.date_published,
        Some(DateTime::new("2026-04-16T10:34:00Z".to_string()))
    );

    Ok(())
}

#[tokio::test]
async fn encode_suggestion_inline_attrs() -> Result<()> {
    let codec = MarkdownCodec {};
    let article = Article {
        content: vec![Block::Paragraph(Paragraph::new(vec![
            Inline::Text(Text::from("A ")),
            Inline::SuggestionInline(SuggestionInline {
                id: Some("suggestion-inline".into()),
                suggestion_type: Some(SuggestionType::Insert),
                authors: Some(vec![Author::Person(Person::from("Alice"))]),
                date_published: Some(DateTime::new("2024-04-17T10:14:00+00:00".to_string())),
                content: vec![Inline::Text(Text::from("change"))],
                ..Default::default()
            }),
            Inline::Text(Text::from(".")),
        ]))],
        ..Default::default()
    };

    let (md, ..) = codec.to_string(&Node::Article(article), None).await?;
    assert!(md.contains(r#"{++change++}{id="suggestion-inline", by="Alice""#));
    assert!(md.contains("at=\"2024-04-17T10:14:00+00:00\""));

    Ok(())
}

#[tokio::test]
async fn encode_suggestion_block_attrs() -> Result<()> {
    let codec = MarkdownCodec {};
    let article = Article {
        content: vec![Block::SuggestionBlock(SuggestionBlock {
            id: Some("suggestion-block".into()),
            suggestion_type: Some(SuggestionType::Insert),
            authors: Some(vec![Author::Person(Person::from("Alice"))]),
            date_published: Some(DateTime::new("2024-04-17T10:14:00+00:00".to_string())),
            content: vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
                Text::from("A block change."),
            )]))],
            ..Default::default()
        })],
        options: Box::new(ArticleOptions::default()),
        ..Default::default()
    };

    let (md, ..) = codec.to_string(&Node::Article(article), None).await?;
    assert!(md.contains(r#":++ {id="suggestion-block", by="Alice""#));
    assert!(md.contains("at=\"2024-04-17T10:14:00+00:00\""));

    Ok(())
}
