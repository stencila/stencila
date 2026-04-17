use stencila_codec::{
    Codec, DecodeOptions, EncodeOptions,
    eyre::{Result, eyre},
    stencila_schema::{
        Article, Author, Block, Inline, Node, Paragraph, Person, SuggestionInline, SuggestionType,
        Text,
    },
};
use stencila_codec_pandoc::PandocCodec;

#[tokio::test]
async fn roundtrip_suggestion_inline_attrs() -> Result<()> {
    let codec = PandocCodec;
    let article = Article {
        content: vec![Block::Paragraph(Paragraph::new(vec![
            Inline::Text(Text::from("A ")),
            Inline::SuggestionInline(SuggestionInline {
                suggestion_type: Some(SuggestionType::Insert),
                authors: Some(vec![Author::Person(Person::from("Alice"))]),
                date_published: Some("2024-04-17T12:34:00Z".parse()?),
                content: vec![Inline::Text(Text::from("change"))],
                ..Default::default()
            }),
            Inline::Text(Text::from(".")),
        ]))],
        ..Default::default()
    };

    let node = Node::Article(article.clone());
    let (json, ..) = codec
        .to_string(
            &node,
            Some(EncodeOptions {
                compact: Some(false),
                ..Default::default()
            }),
        )
        .await?;
    let (decoded, ..) = codec
        .from_str(&json, Some(DecodeOptions::default()))
        .await?;

    let Node::Article(decoded) = decoded else {
        return Err(eyre!("expected article"));
    };

    let Some(Block::Paragraph(Paragraph { content, .. })) = decoded.content.first() else {
        return Err(eyre!("expected paragraph"));
    };
    let Some(Inline::SuggestionInline(suggestion)) = content.get(1) else {
        return Err(eyre!("expected suggestion inline"));
    };

    assert_eq!(suggestion.suggestion_type, Some(SuggestionType::Insert));
    assert_eq!(
        suggestion.authors,
        Some(vec![Author::Person(Person::from("Alice"))])
    );
    assert_eq!(
        suggestion.date_published,
        "2024-04-17T12:34:00Z".parse().ok()
    );
    assert_eq!(suggestion.content, vec![Inline::Text(Text::from("change"))]);

    Ok(())
}

#[tokio::test]
async fn roundtrip_replacement_suggestion_inline_date_published() -> Result<()> {
    let codec = PandocCodec;
    let article = Article {
        content: vec![Block::Paragraph(Paragraph::new(vec![
            Inline::Text(Text::from("A ")),
            Inline::SuggestionInline(SuggestionInline {
                suggestion_type: Some(SuggestionType::Replace),
                authors: Some(vec![Author::Person(Person::from("Alice"))]),
                date_published: Some("2024-04-20T12:34:00Z".parse()?),
                original: Some(vec![Inline::Text(Text::from("before"))]),
                content: vec![Inline::Text(Text::from("after"))],
                ..Default::default()
            }),
            Inline::Text(Text::from(".")),
        ]))],
        ..Default::default()
    };

    let node = Node::Article(article.clone());
    let (json, ..) = codec
        .to_string(
            &node,
            Some(EncodeOptions {
                compact: Some(false),
                ..Default::default()
            }),
        )
        .await?;
    let (decoded, ..) = codec
        .from_str(&json, Some(DecodeOptions::default()))
        .await?;

    let Node::Article(decoded) = decoded else {
        return Err(eyre!("expected article"));
    };

    let Some(Block::Paragraph(Paragraph { content, .. })) = decoded.content.first() else {
        return Err(eyre!("expected paragraph"));
    };
    let Some(Inline::SuggestionInline(suggestion)) = content.get(1) else {
        return Err(eyre!("expected suggestion inline"));
    };

    assert_eq!(suggestion.suggestion_type, Some(SuggestionType::Replace));
    assert_eq!(
        suggestion.date_published,
        "2024-04-20T12:34:00Z".parse().ok()
    );
    assert_eq!(
        suggestion.original,
        Some(vec![Inline::Text(Text::from("before"))])
    );
    assert_eq!(suggestion.content, vec![Inline::Text(Text::from("after"))]);
    assert_eq!(suggestion.execution_ended, None);

    Ok(())
}
