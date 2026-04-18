use std::collections::HashMap;

use pandoc_types::definition::{Attr, Block as PandocBlock, Inline as PandocInline, Pandoc};
use stencila_codec::{
    Codec, EncodeOptions,
    stencila_format::Format,
    stencila_schema::{
        Article, ArticleOptions, Author, Block, Boundary, Comment, CommentOptions, DateTime,
        Inline, Node, Paragraph, Person, Text,
    },
};
use stencila_codec_pandoc::{PandocCodec, root_to_pandoc};

fn span(attrs: Attr, inlines: Vec<PandocInline>) -> PandocInline {
    PandocInline::Span(attrs, inlines)
}

fn collect_spans<'a>(
    inlines: &'a [PandocInline],
    spans: &mut Vec<(&'a Attr, &'a Vec<PandocInline>)>,
) {
    for inline in inlines {
        if let PandocInline::Span(attrs, nested) = inline {
            spans.push((attrs, nested));
            collect_spans(nested, spans);
        }
    }
}

fn attr_value<'a>(attrs: &'a Attr, name: &str) -> Option<&'a str> {
    attrs
        .attributes
        .iter()
        .find(|(attr_name, _)| attr_name == name)
        .map(|(_, value)| value.as_str())
}

#[tokio::test]
async fn encode_docx_replies_as_anchored_pandoc_comments() {
    let node = Node::Article(Article {
        content: vec![Block::Paragraph(Paragraph {
            content: vec![
                Inline::Boundary(Boundary {
                    id: Some("comment-0-start".into()),
                    ..Default::default()
                }),
                Inline::Text(Text::from("Hello")),
                Inline::Boundary(Boundary {
                    id: Some("comment-0-end".into()),
                    ..Default::default()
                }),
                Inline::Text(Text::from(".")),
            ],
            ..Default::default()
        })],
        options: Box::new(ArticleOptions {
            comments: Some(vec![Comment {
                id: Some("comment-a".into()),
                authors: Some(vec![Author::Person(Person::from("nokome"))]),
                date_published: Some(DateTime::new("2026-04-18T11:14:01Z".to_string())),
                content: vec![Block::Paragraph(Paragraph {
                    content: vec![Inline::Text(Text::from("Comment"))],
                    ..Default::default()
                })],
                options: Box::new(CommentOptions {
                    comments: Some(vec![
                        Comment {
                            id: Some("reply-a".into()),
                            authors: Some(vec![Author::Person(Person::from("nokome"))]),
                            date_published: Some(DateTime::new("2026-04-18T11:14:10Z".to_string())),
                            content: vec![Block::Paragraph(Paragraph {
                                content: vec![Inline::Text(Text::from("Reply to comment"))],
                                ..Default::default()
                            })],
                            options: Box::new(CommentOptions {
                                parent_item: Some("comment-a".into()),
                                ..Default::default()
                            }),
                            ..Default::default()
                        },
                        Comment {
                            id: Some("reply-b".into()),
                            authors: Some(vec![Author::Person(Person::from("nokome"))]),
                            date_published: Some(DateTime::new("2026-04-18T11:14:18Z".to_string())),
                            content: vec![Block::Paragraph(Paragraph {
                                content: vec![Inline::Text(Text::from("Another reply to comment"))],
                                ..Default::default()
                            })],
                            options: Box::new(CommentOptions {
                                parent_item: Some("comment-a".into()),
                                ..Default::default()
                            }),
                            ..Default::default()
                        },
                        Comment {
                            id: Some("reply-c".into()),
                            authors: Some(vec![Author::Person(Person::from("nokome"))]),
                            date_published: Some(DateTime::new("2026-04-18T11:14:30Z".to_string())),
                            content: vec![Block::Paragraph(Paragraph {
                                content: vec![Inline::Text(Text::from(
                                    "A third reply to the comment",
                                ))],
                                ..Default::default()
                            })],
                            options: Box::new(CommentOptions {
                                parent_item: Some("comment-a".into()),
                                ..Default::default()
                            }),
                            ..Default::default()
                        },
                    ]),
                    start_location: Some("#comment-0-start".into()),
                    end_location: Some("#comment-0-end".into()),
                    ..Default::default()
                }),
                ..Default::default()
            }]),
            ..Default::default()
        }),
        ..Default::default()
    });

    let (pandoc, _) = root_to_pandoc(
        &node,
        Format::Docx,
        &Some(EncodeOptions {
            format: Some(Format::Docx),
            ..Default::default()
        }),
    )
    .expect("encode pandoc");
    let Some(PandocBlock::Para(inlines)) = pandoc.blocks.first() else {
        panic!("expected first paragraph");
    };

    let mut spans = Vec::new();
    collect_spans(inlines, &mut spans);

    let top_level_start_ids: Vec<&str> = inlines
        .iter()
        .take_while(|inline| {
            matches!(inline, PandocInline::Span(attrs, _) if attrs.classes.iter().any(|class| class == "comment-start"))
        })
        .map(|inline| match inline {
            PandocInline::Span(attrs, _) => attr_value(attrs, "id").expect("comment start id"),
            _ => unreachable!(),
        })
        .collect();
    assert_eq!(top_level_start_ids, vec!["0", "1", "2", "3"]);
    assert!(matches!(inlines.get(4), Some(PandocInline::Str(text)) if text == "Hello"));

    let start_ids: Vec<&str> = spans
        .iter()
        .filter(|(attrs, _)| attrs.classes.iter().any(|class| class == "comment-start"))
        .filter_map(|(attrs, _)| attr_value(attrs, "id"))
        .collect();
    assert_eq!(start_ids, vec!["0", "1", "2", "3"]);

    let reply_start = spans
        .iter()
        .find(|(attrs, _)| {
            attrs.classes.iter().any(|class| class == "comment-start")
                && attr_value(attrs, "id") == Some("1")
        })
        .expect("reply start span");
    assert_eq!(attr_value(reply_start.0, "parent"), Some("0"));

    let Some(PandocInline::Span(top_level_end_attrs, top_level_end_inlines)) = inlines.get(5)
    else {
        panic!("expected top-level end span");
    };
    assert!(
        top_level_end_attrs
            .classes
            .iter()
            .any(|class| class == "comment-end")
    );
    assert_eq!(attr_value(top_level_end_attrs, "id"), Some("0"));

    let reply_end_ids: Vec<&str> = top_level_end_inlines
        .iter()
        .map(|inline| match inline {
            PandocInline::Span(attrs, _) => attr_value(attrs, "id").expect("comment end id"),
            _ => panic!("expected nested comment end span"),
        })
        .collect();
    assert_eq!(reply_end_ids, vec!["1", "2", "3"]);
    assert!(matches!(inlines.get(6), Some(PandocInline::Str(text)) if text == "."));
}

#[tokio::test]
async fn encode_docx_replies_skip_reserved_top_level_ids() {
    let node = Node::Article(Article {
        content: vec![Block::Paragraph(Paragraph {
            content: vec![
                Inline::Boundary(Boundary {
                    id: Some("comment-0-start".into()),
                    ..Default::default()
                }),
                Inline::Text(Text::from("Hello")),
                Inline::Boundary(Boundary {
                    id: Some("comment-0-end".into()),
                    ..Default::default()
                }),
                Inline::Text(Text::from(" ")),
                Inline::Boundary(Boundary {
                    id: Some("comment-1-start".into()),
                    ..Default::default()
                }),
                Inline::Text(Text::from("world")),
                Inline::Boundary(Boundary {
                    id: Some("comment-1-end".into()),
                    ..Default::default()
                }),
            ],
            ..Default::default()
        })],
        options: Box::new(ArticleOptions {
            comments: Some(vec![
                Comment {
                    id: Some("comment-a".into()),
                    content: vec![Block::Paragraph(Paragraph {
                        content: vec![Inline::Text(Text::from("Comment"))],
                        ..Default::default()
                    })],
                    options: Box::new(CommentOptions {
                        comments: Some(vec![Comment {
                            id: Some("reply-a".into()),
                            content: vec![Block::Paragraph(Paragraph {
                                content: vec![Inline::Text(Text::from("Reply"))],
                                ..Default::default()
                            })],
                            options: Box::new(CommentOptions {
                                parent_item: Some("comment-a".into()),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }]),
                        start_location: Some("#comment-0-start".into()),
                        end_location: Some("#comment-0-end".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                Comment {
                    id: Some("comment-b".into()),
                    content: vec![Block::Paragraph(Paragraph {
                        content: vec![Inline::Text(Text::from("Second comment"))],
                        ..Default::default()
                    })],
                    options: Box::new(CommentOptions {
                        start_location: Some("#comment-1-start".into()),
                        end_location: Some("#comment-1-end".into()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ]),
            ..Default::default()
        }),
        ..Default::default()
    });

    let (pandoc, _) = root_to_pandoc(
        &node,
        Format::Docx,
        &Some(EncodeOptions {
            format: Some(Format::Docx),
            ..Default::default()
        }),
    )
    .expect("encode pandoc");
    let Some(PandocBlock::Para(inlines)) = pandoc.blocks.first() else {
        panic!("expected first paragraph");
    };

    let mut spans = Vec::new();
    collect_spans(inlines, &mut spans);

    let mut start_ids: Vec<usize> = spans
        .iter()
        .filter(|(attrs, _)| attrs.classes.iter().any(|class| class == "comment-start"))
        .filter_map(|(attrs, _)| attrs.attributes.iter().find(|(name, _)| name == "id"))
        .map(|(_, value)| value.parse().expect("numeric comment id"))
        .collect();
    start_ids.sort_unstable();
    assert_eq!(start_ids, vec![0, 1, 2]);

    let reply_start = spans
        .iter()
        .find(|(attrs, _)| {
            attrs.classes.iter().any(|class| class == "comment-start")
                && attrs
                    .attributes
                    .iter()
                    .any(|(name, value)| name == "parent" && value == "0")
        })
        .expect("reply start span");
    assert!(
        reply_start
            .0
            .attributes
            .iter()
            .any(|(name, value)| name == "id" && value == "2")
    );
}

#[tokio::test]
async fn decode_docx_reply_chain_with_second_top_level_comment() {
    let pandoc = Pandoc {
        meta: HashMap::new(),
        blocks: vec![PandocBlock::Para(vec![
            span(
                attrs(
                    &["comment-start"],
                    &[
                        ("id", "0"),
                        ("author", "nokome"),
                        ("date", "2026-04-17T16:38:28Z"),
                    ],
                ),
                vec![
                    PandocInline::Str("A".into()),
                    PandocInline::Space,
                    PandocInline::Str("comment".into()),
                ],
            ),
            PandocInline::Str("Hello".into()),
            span(
                attrs(&["comment-start"], &[("id", "4"), ("author", "nokome")]),
                vec![
                    PandocInline::Str("Another".into()),
                    PandocInline::Space,
                    PandocInline::Str("comment".into()),
                ],
            ),
            span(
                attrs(&["comment-end"], &[("id", "0")]),
                vec![span(
                    attrs(&["comment-end"], &[("id", "1")]),
                    vec![span(
                        attrs(&["comment-end"], &[("id", "2")]),
                        vec![span(attrs(&["comment-end"], &[("id", "3")]), vec![])],
                    )],
                )],
            ),
            PandocInline::Str(".".into()),
            span(attrs(&["comment-end"], &[("id", "4")]), vec![]),
        ])],
    };

    let (node, _) = PandocCodec
        .from_str(
            &serde_json::to_string(&pandoc).expect("serialize pandoc json"),
            None,
        )
        .await
        .expect("decode pandoc json");

    let Node::Article(article) = node else {
        panic!("expected article");
    };

    let comments = article.options.comments.expect("article comments");
    assert_eq!(comments.len(), 2);

    let root = &comments[0];
    let replies = root.options.comments.as_ref().expect("root replies");
    assert_eq!(replies.len(), 3);
    assert!(replies.iter().all(|reply| reply.options.comments.is_none()));

    assert_eq!(replies[0].id.as_deref(), Some("1"));
    assert_eq!(replies[1].id.as_deref(), Some("2"));
    assert_eq!(replies[2].id.as_deref(), Some("3"));
    assert!(
        replies
            .iter()
            .all(|reply| reply.options.parent_item.as_deref() == Some("0"))
    );

    let second_root = &comments[1];
    assert_eq!(second_root.id.as_deref(), Some("4"));
    assert_eq!(second_root.options.parent_item.as_deref(), None);
    assert!(second_root.options.comments.is_none());
}

fn attrs(classes: &[&str], attributes: &[(&str, &str)]) -> Attr {
    Attr {
        identifier: String::new(),
        classes: classes.iter().map(|class| (*class).to_string()).collect(),
        attributes: attributes
            .iter()
            .map(|(name, value)| ((*name).to_string(), (*value).to_string()))
            .collect(),
    }
}

#[tokio::test]
async fn decode_docx_reply_chain_as_sibling_replies() {
    let pandoc = Pandoc {
        meta: HashMap::new(),
        blocks: vec![PandocBlock::Para(vec![
            span(
                attrs(
                    &["comment-start"],
                    &[
                        ("id", "0"),
                        ("author", "nokome"),
                        ("date", "2026-04-17T16:38:28Z"),
                    ],
                ),
                vec![
                    PandocInline::Str("A".into()),
                    PandocInline::Space,
                    PandocInline::Str("comment".into()),
                ],
            ),
            span(
                attrs(
                    &["comment-start"],
                    &[
                        ("id", "1"),
                        ("author", "nokome"),
                        ("date", "2026-04-17T16:38:36Z"),
                    ],
                ),
                vec![
                    PandocInline::Str("A".into()),
                    PandocInline::Space,
                    PandocInline::Str("reply".into()),
                    PandocInline::Space,
                    PandocInline::Str("to".into()),
                    PandocInline::Space,
                    PandocInline::Str("the".into()),
                    PandocInline::Space,
                    PandocInline::Str("comment".into()),
                ],
            ),
            span(
                attrs(
                    &["comment-start"],
                    &[
                        ("id", "2"),
                        ("author", "nokome"),
                        ("date", "2026-04-17T16:54:58Z"),
                    ],
                ),
                vec![
                    PandocInline::Str("Another".into()),
                    PandocInline::Space,
                    PandocInline::Str("reply".into()),
                ],
            ),
            span(
                attrs(
                    &["comment-start"],
                    &[
                        ("id", "3"),
                        ("author", "nokome"),
                        ("date", "2026-04-17T18:31:33Z"),
                    ],
                ),
                vec![
                    PandocInline::Str("Yet".into()),
                    PandocInline::Space,
                    PandocInline::Str("another".into()),
                    PandocInline::Space,
                    PandocInline::Str("reply".into()),
                ],
            ),
            PandocInline::Str("Hello".into()),
            span(
                attrs(&["comment-end"], &[("id", "0")]),
                vec![span(
                    attrs(&["comment-end"], &[("id", "1")]),
                    vec![span(
                        attrs(&["comment-end"], &[("id", "2")]),
                        vec![span(attrs(&["comment-end"], &[("id", "3")]), vec![])],
                    )],
                )],
            ),
            PandocInline::Str(".".into()),
        ])],
    };

    let (node, _) = PandocCodec
        .from_str(
            &serde_json::to_string(&pandoc).expect("serialize pandoc json"),
            None,
        )
        .await
        .expect("decode pandoc json");

    let Node::Article(article) = node else {
        panic!("expected article");
    };

    let comments = article.options.comments.expect("article comments");
    assert_eq!(comments.len(), 1);

    let root = &comments[0];
    let replies = root.options.comments.as_ref().expect("root replies");
    assert_eq!(replies.len(), 3);
    assert!(replies.iter().all(|reply| reply.options.comments.is_none()));

    assert_eq!(replies[0].id.as_deref(), Some("1"));
    assert_eq!(replies[1].id.as_deref(), Some("2"));
    assert_eq!(replies[2].id.as_deref(), Some("3"));
    assert!(
        replies
            .iter()
            .all(|reply| reply.options.parent_item.as_deref() == Some("0"))
    );

    assert_eq!(
        root.options.start_location.as_deref(),
        Some("#comment-0-start")
    );
    assert_eq!(root.options.end_location.as_deref(), Some("#comment-0-end"));

    assert!(replies.iter().all(
        |reply| reply.options.start_location.is_none() && reply.options.end_location.is_none()
    ));
}
