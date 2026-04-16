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

[>>c1]{by=\"Alice; Bob\", at=\"2026-04-16T10:34:00Z\"}: This is the first comment.
"#);

    Ok(())
}
