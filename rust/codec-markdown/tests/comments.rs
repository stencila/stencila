use insta::assert_snapshot;
use stencila_codec::{
    Codec, DecodeOptions,
    eyre::{Result, eyre},
    stencila_schema::{
        Article, ArticleOptions, Author, Block, Comment, DateTime, Inline, Node, Paragraph, Person,
        Text,
    },
};
use stencila_codec_markdown::{MarkdownCodec, decode};

#[test]
fn decode_comment_definition_attrs() -> Result<()> {
    let (node, _) = decode(
        r#"A paragraph with {>>c1}a comment{<<c1}.

[>>c1]{by="Alice; Bob", at="2026-04-16T10:34:00Z"}: This is the first comment.
"#,
        Some(DecodeOptions::default()),
    )?;

    let Node::Article(article) = node else {
        return Err(eyre!("expected Article"));
    };

    let comments = article
        .options
        .comments
        .ok_or_else(|| eyre!("article should have top-level comments"))?;
    let comment = comments
        .first()
        .ok_or_else(|| eyre!("should have first comment"))?;

    assert_eq!(comment.id.as_deref(), Some("c1"));
    assert_eq!(
        comment.authors,
        Some(vec![
            Author::Person(Person::from("Alice")),
            Author::Person(Person::from("Bob")),
        ])
    );
    assert_eq!(
        comment
            .date_published
            .as_ref()
            .map(|date| date.value.as_str()),
        Some("2026-04-16T10:34:00Z")
    );

    Ok(())
}

/// Ensure dotted comment definition IDs are decoded into nested reply comments.
#[test]
fn decode_nested_comment_definitions_as_replies() -> Result<()> {
    let (node, _) = decode(
        r#"{>>0}Hello{<<0}.

[>>0]{by="Arthur Dent", at="2026-04-17T16:38:28Z"}: A comment

[>>0.1]{by="Ford Prefect", at="2026-04-17T16:38:36Z"}: A reply to the comment

[>>0.2]{by="Trillian", at="2026-04-17T16:54:58Z"}: Another reply
"#,
        Some(DecodeOptions::default()),
    )?;

    let Node::Article(article) = node else {
        return Err(eyre!("expected Article"));
    };

    let comments = article
        .options
        .comments
        .ok_or_else(|| eyre!("article should have top-level comments"))?;
    let comment = comments
        .first()
        .ok_or_else(|| eyre!("should have first comment"))?;

    let replies = comment
        .options
        .comments
        .as_ref()
        .ok_or_else(|| eyre!("comment should have replies"))?;

    assert_eq!(replies.len(), 2);
    assert_eq!(replies[0].id.as_deref(), Some("0.1"));
    assert_eq!(replies[1].id.as_deref(), Some("0.2"));
    assert!(replies[0].options.comments.is_none());
    assert!(replies[1].options.comments.is_none());

    Ok(())
}

/// Ensure nested reply comments are encoded using canonical parent-based IDs.
#[tokio::test]
async fn encode_replies_using_canonical_hierarchical_ids() -> Result<()> {
    let codec = MarkdownCodec {};
    let reply = Comment {
        id: Some("1".into()),
        authors: Some(vec![Author::Person(Person::from("Ford Prefect"))]),
        date_published: Some(DateTime::new("2026-04-17T16:38:36Z".to_string())),
        content: vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
            Text::from("A reply to the comment"),
        )]))],
        ..Default::default()
    };

    let comment = Comment {
        id: Some("0".into()),
        authors: Some(vec![Author::Person(Person::from("Arthur Dent"))]),
        date_published: Some(DateTime::new("2026-04-17T16:38:28Z".to_string())),
        content: vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
            Text::from("A comment"),
        )]))],
        options: Box::new(stencila_codec::stencila_schema::CommentOptions {
            comments: Some(vec![reply]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let article = Article {
        content: vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
            Text::from("Hello."),
        )]))],
        options: Box::new(ArticleOptions {
            comments: Some(vec![comment]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let (md, ..) = codec.to_string(&Node::Article(article), None).await?;

    assert_snapshot!(md, @r#"
    Hello.

    [>>0]{by="Arthur Dent", at="2026-04-17T16:38:28Z"}: A comment

    [>>0.1]{by="Ford Prefect", at="2026-04-17T16:38:36Z"}: A reply to the comment
    "#);

    Ok(())
}

/// Ensure nested reply threads are encoded with hierarchical sibling and descendant IDs.
#[tokio::test]
async fn encode_nested_replies_with_hierarchical_ids() -> Result<()> {
    let codec = MarkdownCodec {};
    let nested_reply = Comment {
        authors: Some(vec![Author::Person(Person::from("Zaphod Beeblebrox"))]),
        date_published: Some(DateTime::new("2026-04-17T16:40:00Z".to_string())),
        content: vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
            Text::from("A reply to the reply"),
        )]))],
        ..Default::default()
    };

    let first_reply = Comment {
        authors: Some(vec![Author::Person(Person::from("Ford Prefect"))]),
        date_published: Some(DateTime::new("2026-04-17T16:38:36Z".to_string())),
        content: vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
            Text::from("A reply to the comment"),
        )]))],
        options: Box::new(stencila_codec::stencila_schema::CommentOptions {
            comments: Some(vec![nested_reply]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let second_reply = Comment {
        authors: Some(vec![Author::Person(Person::from("Trillian"))]),
        date_published: Some(DateTime::new("2026-04-17T16:54:58Z".to_string())),
        content: vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
            Text::from("Another reply"),
        )]))],
        ..Default::default()
    };

    let comment = Comment {
        id: Some("0".into()),
        authors: Some(vec![Author::Person(Person::from("Arthur Dent"))]),
        date_published: Some(DateTime::new("2026-04-17T16:38:28Z".to_string())),
        content: vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
            Text::from("A comment"),
        )]))],
        options: Box::new(stencila_codec::stencila_schema::CommentOptions {
            comments: Some(vec![first_reply, second_reply]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let article = Article {
        content: vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
            Text::from("Hello."),
        )]))],
        options: Box::new(ArticleOptions {
            comments: Some(vec![comment]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let (md, ..) = codec.to_string(&Node::Article(article), None).await?;

    assert_snapshot!(md, @r#"
    Hello.

    [>>0]{by="Arthur Dent", at="2026-04-17T16:38:28Z"}: A comment

    [>>0.1]{by="Ford Prefect", at="2026-04-17T16:38:36Z"}: A reply to the comment

    [>>0.1.1]{by="Zaphod Beeblebrox", at="2026-04-17T16:40:00Z"}: A reply to the reply

    [>>0.2]{by="Trillian", at="2026-04-17T16:54:58Z"}: Another reply
    "#);

    Ok(())
}

#[tokio::test]
async fn encode_comment_definition_attrs() -> Result<()> {
    let codec = MarkdownCodec {};
    let article = Article {
        content: vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
            Text::from("A paragraph."),
        )]))],
        options: Box::new(ArticleOptions {
            comments: Some(vec![Comment {
                id: Some("c1".into()),
                authors: Some(vec![
                    Author::Person(Person::from("Alice")),
                    Author::Person(Person::from("Bob")),
                ]),
                date_published: Some(DateTime::new("2026-04-16T10:34:00Z".to_string())),
                content: vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
                    Text::from("This is the first comment."),
                )]))],
                ..Default::default()
            }]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let (md, ..) = codec.to_string(&Node::Article(article), None).await?;

    assert_snapshot!(md, @r#"
    A paragraph.

    [>>c1]{by="Alice; Bob", at="2026-04-16T10:34:00Z"}: This is the first comment.
    "#);

    Ok(())
}
