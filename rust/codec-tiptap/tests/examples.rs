use pretty_assertions::assert_eq;
use stencila_codec::{
    eyre::{Result, bail},
    Losses,
    stencila_schema::{
        Article, Block, CodeChunk, Emphasis, Heading, Inline, MathInline, Node, Paragraph, Strong,
        Text,
    },
};
use stencila_codec_tiptap::{decode, encode};

const SUPPORTED_CANONICAL_JSON: &str = r#"{"type":"doc","content":[{"type":"heading","attrs":{"level":2},"content":[{"type":"text","text":"Title"}]},{"type":"paragraph","content":[{"type":"text","text":"Hello "},{"type":"text","marks":[{"type":"bold"}],"text":"bold"},{"type":"text","text":" and "},{"type":"text","marks":[{"type":"italic"}],"text":"italic"}]}]}"#;

fn losses_json(losses: &Losses) -> Result<serde_json::Value> {
    Ok(serde_json::to_value(losses)?)
}

#[test]
fn decode_simple_paragraph() -> Result<()> {
    let json = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"Hello"}]}]}"#;

    let (Node::Article(article), info) = decode(json, None)? else {
        bail!("Tiptap should decode to an article")
    };

    assert!(info.losses.is_empty());
    let Block::Paragraph(paragraph) = &article.content[0] else {
        bail!("expected paragraph")
    };
    let Inline::Text(text) = &paragraph.content[0] else {
        bail!("expected text")
    };
    assert_eq!(text.value.to_string(), "Hello");

    Ok(())
}

#[test]
fn decode_heading_levels() -> Result<()> {
    for level in 1..=6 {
        let json = format!(
            r#"{{"type":"doc","content":[{{"type":"heading","attrs":{{"level":{level}}},"content":[{{"type":"text","text":"Heading"}}]}}]}}"#
        );

        let (Node::Article(article), ..) = decode(&json, None)? else {
            bail!("Tiptap should decode to an article")
        };
        let Block::Heading(heading) = &article.content[0] else {
            bail!("expected heading")
        };

        assert_eq!(heading.level, level);
    }

    Ok(())
}

#[test]
fn decode_bold_and_italic_marks() -> Result<()> {
    let json = r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"both","marks":[{"type":"bold"},{"type":"italic"}]}]}]}"#;

    let (Node::Article(article), ..) = decode(json, None)? else {
        bail!("Tiptap should decode to an article")
    };

    let Block::Paragraph(paragraph) = &article.content[0] else {
        bail!("expected paragraph")
    };
    let Inline::Emphasis(Emphasis { content, .. }) = &paragraph.content[0] else {
        bail!("expected emphasis as outer wrapper")
    };
    let Inline::Strong(Strong { content, .. }) = &content[0] else {
        bail!("expected strong as inner wrapper")
    };
    let Inline::Text(text) = &content[0] else {
        bail!("expected text")
    };

    assert_eq!(text.value.to_string(), "both");

    Ok(())
}

#[test]
fn encode_supported_content() -> Result<()> {
    let node = Node::Article(Article {
        content: vec![
            Block::Heading(Heading::new(
                2,
                vec![Inline::Text(Text::new("Title".into()))],
            )),
            Block::Paragraph(Paragraph::new(vec![
                Inline::Text(Text::new("Hello ".into())),
                Inline::Strong(Strong::new(vec![Inline::Text(Text::new("bold".into()))])),
                Inline::Text(Text::new(" and ".into())),
                Inline::Emphasis(Emphasis::new(vec![Inline::Text(Text::new(
                    "italic".into(),
                ))])),
            ])),
        ],
        ..Default::default()
    });

    let (json, info) = encode(&node, None)?;

    assert!(info.losses.is_empty());
    assert_eq!(json, SUPPORTED_CANONICAL_JSON);

    Ok(())
}

#[test]
fn roundtrip_supported_content() -> Result<()> {
    let (node, ..) = decode(SUPPORTED_CANONICAL_JSON, None)?;
    let (json, info) = encode(&node, None)?;

    assert!(info.losses.is_empty());
    assert_eq!(json, SUPPORTED_CANONICAL_JSON);

    Ok(())
}

#[test]
fn encode_empty_parent_like_tiptap() -> Result<()> {
    let node = Node::Article(Article {
        content: vec![Block::Paragraph(Paragraph::default())],
        ..Default::default()
    });

    let (json, info) = encode(&node, None)?;

    assert!(info.losses.is_empty());
    assert_eq!(json, r#"{"type":"doc","content":[{"type":"paragraph"}]}"#);

    Ok(())
}

#[test]
fn encode_heading_level_zero_as_paragraph() -> Result<()> {
    let node = Node::Article(Article {
        content: vec![Block::Heading(Heading::new(
            0,
            vec![Inline::Text(Text::new("No level".into()))],
        ))],
        ..Default::default()
    });

    let (json, ..) = encode(&node, None)?;

    assert_eq!(
        json,
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","text":"No level"}]}]}"#
    );

    Ok(())
}

#[test]
fn encode_merges_adjacent_text_with_same_marks() -> Result<()> {
    let node = Node::Article(Article {
        content: vec![Block::Paragraph(Paragraph::new(vec![Inline::Strong(
            Strong::new(vec![
                Inline::Text(Text::new("a".into())),
                Inline::Text(Text::new("".into())),
                Inline::Text(Text::new("b".into())),
            ]),
        )]))],
        ..Default::default()
    });

    let (json, ..) = encode(&node, None)?;

    assert_eq!(
        json,
        r#"{"type":"doc","content":[{"type":"paragraph","content":[{"type":"text","marks":[{"type":"bold"}],"text":"ab"}]}]}"#
    );

    Ok(())
}

#[test]
fn records_losses_for_unknown_tiptap_nodes_and_marks() -> Result<()> {
    let json = r#"{"type":"doc","content":[{"type":"unknownBlock"},{"type":"paragraph","content":[{"type":"text","text":"linked","marks":[{"type":"link","attrs":{"href":"https://example.com"}}]}]}]}"#;

    let (_, info) = decode(json, None)?;

    assert_eq!(
        losses_json(&info.losses)?,
        serde_json::json!({
            "Unknown (unknownBlock)": 1,
            "Unknown mark (link)": 1
        })
    );

    Ok(())
}

#[test]
fn preserve_unsupported_block_opaque_payload() -> Result<()> {
    let code_chunk = Block::CodeChunk(CodeChunk {
        code: "print('hello')".into(),
        programming_language: Some("python".into()),
        ..Default::default()
    });
    let original = serde_json::to_string(&code_chunk)?;

    let node = Node::Article(Article {
        content: vec![
            Block::Paragraph(Paragraph::new(vec![Inline::Text(Text::new(
                "before".into(),
            ))])),
            code_chunk,
            Block::Paragraph(Paragraph::new(vec![Inline::Text(Text::new(
                "after".into(),
            ))])),
        ],
        ..Default::default()
    });

    let (json, ..) = encode(&node, None)?;
    assert!(json.contains(r#""type":"stencilaBlock""#));

    let (Node::Article(article), ..) = decode(&json, None)? else {
        bail!("Tiptap should decode to an article")
    };
    assert_eq!(serde_json::to_string(&article.content[1])?, original);

    let edited = json.replace("after", "changed");
    let (Node::Article(article), ..) = decode(&edited, None)? else {
        bail!("Tiptap should decode to an article")
    };
    assert_eq!(serde_json::to_string(&article.content[1])?, original);

    Ok(())
}

#[test]
fn records_loss_for_stencila_block_node_type_mismatch() -> Result<()> {
    let code_chunk = Block::CodeChunk(CodeChunk {
        code: "print('hello')".into(),
        programming_language: Some("python".into()),
        ..Default::default()
    });
    let node = Node::Article(Article {
        content: vec![code_chunk],
        ..Default::default()
    });

    let (json, ..) = encode(&node, None)?;
    let json = json.replace(r#""nodeType":"CodeChunk""#, r#""nodeType":"Paragraph""#);
    let (_, info) = decode(&json, None)?;

    assert_eq!(
        losses_json(&info.losses)?,
        serde_json::json!({
            "StencilaBlock.nodeType (expected Paragraph, got CodeChunk)": 1
        })
    );

    Ok(())
}

#[test]
fn records_loss_for_stencila_inline_node_type_mismatch() -> Result<()> {
    let math = Inline::MathInline(MathInline::new("x + y".into()));
    let node = Node::Article(Article {
        content: vec![Block::Paragraph(Paragraph::new(vec![math]))],
        ..Default::default()
    });

    let (json, ..) = encode(&node, None)?;
    let json = json.replace(r#""nodeType":"MathInline""#, r#""nodeType":"Text""#);
    let (_, info) = decode(&json, None)?;

    assert_eq!(
        losses_json(&info.losses)?,
        serde_json::json!({
            "StencilaInline.nodeType (expected Text, got MathInline)": 1
        })
    );

    Ok(())
}

#[test]
fn preserve_unsupported_inline_opaque_payload() -> Result<()> {
    let math = Inline::MathInline(MathInline::new("x + y".into()));
    let original = serde_json::to_string(&math)?;

    let node = Node::Article(Article {
        content: vec![Block::Paragraph(Paragraph::new(vec![
            Inline::Text(Text::new("before ".into())),
            math,
            Inline::Text(Text::new(" after".into())),
        ]))],
        ..Default::default()
    });

    let (json, ..) = encode(&node, None)?;
    assert!(json.contains(r#""type":"stencilaInline""#));

    let (Node::Article(article), ..) = decode(&json, None)? else {
        bail!("Tiptap should decode to an article")
    };
    let Block::Paragraph(paragraph) = &article.content[0] else {
        bail!("expected paragraph")
    };

    assert_eq!(serde_json::to_string(&paragraph.content[1])?, original);

    Ok(())
}
