use std::collections::HashMap;

use pandoc_types::definition::{Attr, Block as PandocBlock, Inline as PandocInline, Pandoc};
use stencila_codec::{Codec, stencila_schema::Node};
use stencila_codec_pandoc::PandocCodec;

fn span(attrs: Attr, inlines: Vec<PandocInline>) -> PandocInline {
    PandocInline::Span(attrs, inlines)
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

    assert_eq!(
        root.options.start_location.as_deref(),
        Some("#comment-0-start")
    );
    assert_eq!(root.options.end_location.as_deref(), Some("#comment-0-end"));

    assert!(replies.iter().all(
        |reply| reply.options.start_location.is_none() && reply.options.end_location.is_none()
    ));
}
