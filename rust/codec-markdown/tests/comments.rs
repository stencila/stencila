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

/// Ensure nested reply comments preserve explicit IDs and emit parent attributes.
#[tokio::test]
async fn encode_replies_preserve_explicit_ids() -> Result<()> {
    let codec = MarkdownCodec {};
    let reply = Comment {
        id: Some("r1".into()),
        authors: Some(vec![Author::Person(Person::from("Ford Prefect"))]),
        date_published: Some(DateTime::new("2026-04-17T16:38:36Z".to_string())),
        content: vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
            Text::from("A reply to the comment"),
        )]))],
        ..Default::default()
    };

    let comment = Comment {
        id: Some("c1".into()),
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

    [>>c1]{by="Arthur Dent", at="2026-04-17T16:38:28Z"}: A comment

    [>>r1]{by="Ford Prefect", at="2026-04-17T16:38:36Z", parent="c1"}: A reply to the comment
    "#);

    Ok(())
}

#[test]
fn decode_replies_using_explicit_parent_attrs() -> Result<()> {
    let (node, _) = decode(
        r#"{>>c1}Hello{<<c1}.

[>>c1]{by="Arthur Dent", at="2026-04-17T16:38:28Z"}: A comment

[>>r1]{by="Ford Prefect", at="2026-04-17T16:38:36Z", parent="c1"}: A reply to the comment

[>>r2]{by="Zaphod Beeblebrox", at="2026-04-17T16:40:00Z", parent="r1"}: A reply to the reply
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

    assert_eq!(replies.len(), 1);
    assert_eq!(replies[0].id.as_deref(), Some("r1"));
    assert_eq!(replies[0].options.parent_item.as_deref(), Some("c1"));

    let nested = replies[0]
        .options
        .comments
        .as_ref()
        .ok_or_else(|| eyre!("reply should have nested reply"))?;
    assert_eq!(nested.len(), 1);
    assert_eq!(nested[0].id.as_deref(), Some("r2"));
    assert_eq!(nested[0].options.parent_item.as_deref(), Some("r1"));

    Ok(())
}

/// Ensure nested reply threads use explicit parent attrs without relying on dotted IDs.
#[tokio::test]
async fn encode_nested_replies_with_explicit_parent_attrs() -> Result<()> {
    let codec = MarkdownCodec {};
    let nested_reply = Comment {
        id: Some("r2".into()),
        authors: Some(vec![Author::Person(Person::from("Zaphod Beeblebrox"))]),
        date_published: Some(DateTime::new("2026-04-17T16:40:00Z".to_string())),
        content: vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
            Text::from("A reply to the reply"),
        )]))],
        ..Default::default()
    };

    let first_reply = Comment {
        id: Some("r1".into()),
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
        id: Some("r3".into()),
        authors: Some(vec![Author::Person(Person::from("Trillian"))]),
        date_published: Some(DateTime::new("2026-04-17T16:54:58Z".to_string())),
        content: vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
            Text::from("Another reply"),
        )]))],
        ..Default::default()
    };

    let comment = Comment {
        id: Some("c1".into()),
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

    [>>c1]{by="Arthur Dent", at="2026-04-17T16:38:28Z"}: A comment

    [>>r1]{by="Ford Prefect", at="2026-04-17T16:38:36Z", parent="c1"}: A reply to the comment

    [>>r2]{by="Zaphod Beeblebrox", at="2026-04-17T16:40:00Z", parent="r1"}: A reply to the reply

    [>>r3]{by="Trillian", at="2026-04-17T16:54:58Z", parent="c1"}: Another reply
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
