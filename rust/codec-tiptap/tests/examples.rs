use pretty_assertions::assert_eq;
use serde_json::{Value, json};
use stencila_codec::{
    Losses,
    eyre::{Result, bail},
    stencila_schema::{
        Article, Block, CodeBlock, CodeChunk, CodeInline, Emphasis, Heading, Inline, Link, List,
        ListItem, ListOrder, MathInline, Node, Paragraph, QuoteBlock, Strikeout, Strong, Subscript,
        Superscript, Table, TableCell, TableCellType, TableRow, TableRowType, Text, ThematicBreak,
        Underline,
    },
};
use stencila_codec_tiptap::{decode, encode};

fn losses_json(losses: &Losses) -> Result<serde_json::Value> {
    Ok(serde_json::to_value(losses)?)
}

fn encoded_json(json: &str) -> Result<Value> {
    Ok(serde_json::from_str(json)?)
}

fn supported_inline_marks_json() -> Value {
    json!({
        "type": "doc",
        "content": [
            {
                "type": "paragraph",
                "content": [
                    {"type": "text", "marks": [{"type": "bold"}], "text": "bold"},
                    {"type": "text", "text": " "},
                    {"type": "text", "marks": [{"type": "italic"}], "text": "italic"},
                    {"type": "text", "text": " "},
                    {
                        "type": "text",
                        "marks": [{
                            "type": "link",
                            "attrs": {
                                "href": "https://example.com",
                                "title": "Example",
                                "rel": "noopener",
                            },
                        }],
                        "text": "link",
                    },
                    {"type": "text", "text": " "},
                    {"type": "text", "marks": [{"type": "code"}], "text": "code"},
                    {"type": "text", "text": " "},
                    {"type": "text", "marks": [{"type": "strike"}], "text": "strike"},
                    {"type": "text", "text": " "},
                    {"type": "text", "marks": [{"type": "underline"}], "text": "under"},
                    {"type": "text", "text": " "},
                    {"type": "text", "marks": [{"type": "subscript"}], "text": "sub"},
                    {"type": "text", "text": " "},
                    {"type": "text", "marks": [{"type": "superscript"}], "text": "sup"},
                ],
            },
        ],
    })
}

fn inline_mark_attrs_json() -> Value {
    json!({
        "type": "doc",
        "content": [
            {
                "type": "paragraph",
                "content": [
                    {
                        "type": "text",
                        "marks": [{
                            "type": "code",
                            "attrs": {"programmingLanguage": "rust"},
                        }],
                        "text": "let x",
                    },
                    {"type": "text", "text": " "},
                    {
                        "type": "text",
                        "marks": [{
                            "type": "link",
                            "attrs": {"href": "#fig-1", "labelOnly": true},
                        }],
                        "text": "Figure 1",
                    },
                ],
            },
        ],
    })
}

fn stencila_paragraph_json(text: &str) -> Value {
    json!({
        "type": "Paragraph",
        "content": [
            {
                "type": "Text",
                "value": {
                    "string": text,
                },
            },
        ],
    })
}

fn supported_block_nodes_json() -> Value {
    json!({
        "type": "doc",
        "content": [
            {
                "type": "blockquote",
                "content": [
                    {
                        "type": "paragraph",
                        "content": [{"type": "text", "text": "Quoted"}],
                    },
                ],
            },
            {
                "type": "bulletList",
                "content": [
                    {
                        "type": "listItem",
                        "content": [
                            {
                                "type": "paragraph",
                                "content": [{"type": "text", "text": "Bullet"}],
                            },
                        ],
                    },
                ],
            },
            {
                "type": "orderedList",
                "attrs": {
                    "start": 3,
                    "type": null,
                },
                "content": [
                    {
                        "type": "listItem",
                        "content": [
                            {
                                "type": "paragraph",
                                "content": [{"type": "text", "text": "Third"}],
                            },
                        ],
                    },
                    {
                        "type": "listItem",
                        "content": [
                            {
                                "type": "paragraph",
                                "content": [{"type": "text", "text": "Fourth"}],
                            },
                        ],
                    },
                ],
            },
            {
                "type": "codeBlock",
                "attrs": {
                    "id": "code-1",
                    "isDemo": true,
                    "language": "rust",
                },
                "content": [{"type": "text", "text": "fn main() {}"}],
            },
            {"type": "horizontalRule"},
            {
                "type": "table",
                "attrs": {
                    "id": "table-1",
                    "label": "Table 1",
                    "labelAutomatically": true,
                    "caption": [stencila_paragraph_json("Caption")],
                    "notes": [stencila_paragraph_json("Note")],
                },
                "content": [
                    {
                        "type": "tableRow",
                        "content": [
                            {
                                "type": "tableHeader",
                                "attrs": {
                                    "align": null,
                                    "colspan": 1,
                                    "rowspan": 1,
                                    "colwidth": null,
                                },
                                "content": [
                                    {
                                        "type": "paragraph",
                                        "content": [{"type": "text", "text": "Head"}],
                                    },
                                ],
                            },
                        ],
                    },
                    {
                        "type": "tableRow",
                        "content": [
                            {
                                "type": "tableCell",
                                "attrs": {
                                    "align": null,
                                    "colspan": 1,
                                    "rowspan": 1,
                                    "colwidth": null,
                                },
                                "content": [
                                    {
                                        "type": "paragraph",
                                        "content": [{"type": "text", "text": "Data"}],
                                    },
                                ],
                            },
                        ],
                    },
                ],
            },
        ],
    })
}

fn task_list_json() -> Value {
    json!({
        "type": "doc",
        "content": [
            {
                "type": "taskList",
                "content": [
                    {
                        "type": "taskItem",
                        "attrs": {"checked": true},
                        "content": [
                            {
                                "type": "paragraph",
                                "content": [{"type": "text", "text": "Done"}],
                            },
                        ],
                    },
                    {
                        "type": "taskItem",
                        "attrs": {"checked": false},
                        "content": [
                            {
                                "type": "paragraph",
                                "content": [{"type": "text", "text": "Todo"}],
                            },
                        ],
                    },
                ],
            },
        ],
    })
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
fn decode_common_block_nodes() -> Result<()> {
    let fixture = supported_block_nodes_json();
    let (Node::Article(article), info) = decode(&fixture.to_string(), None)? else {
        bail!("Tiptap should decode to an article")
    };

    assert!(info.losses.is_empty());

    let Block::QuoteBlock(quote) = &article.content[0] else {
        bail!("expected quote block")
    };
    let Block::Paragraph(paragraph) = &quote.content[0] else {
        bail!("expected quoted paragraph")
    };
    let Inline::Text(text) = &paragraph.content[0] else {
        bail!("expected quoted text")
    };
    assert_eq!(text.value.to_string(), "Quoted");

    let Block::List(list) = &article.content[1] else {
        bail!("expected bullet list")
    };
    assert_eq!(list.order, ListOrder::Unordered);
    assert_eq!(list.items.len(), 1);

    let Block::List(list) = &article.content[2] else {
        bail!("expected ordered list")
    };
    assert_eq!(list.order, ListOrder::Ascending);
    assert_eq!(list.items[0].position, Some(3));
    assert_eq!(list.items[1].position, Some(4));

    let Block::CodeBlock(code_block) = &article.content[3] else {
        bail!("expected code block")
    };
    assert_eq!(code_block.id.as_deref(), Some("code-1"));
    assert_eq!(code_block.code.to_string(), "fn main() {}");
    assert_eq!(code_block.programming_language.as_deref(), Some("rust"));
    assert_eq!(code_block.is_demo, Some(true));

    let Block::ThematicBreak(..) = &article.content[4] else {
        bail!("expected thematic break")
    };

    let Block::Table(table) = &article.content[5] else {
        bail!("expected table")
    };
    assert_eq!(table.id.as_deref(), Some("table-1"));
    assert_eq!(table.label.as_deref(), Some("Table 1"));
    assert_eq!(table.label_automatically, Some(true));
    assert_eq!(
        serde_json::to_value(&table.caption)?,
        json!([stencila_paragraph_json("Caption")])
    );
    assert_eq!(
        serde_json::to_value(&table.notes)?,
        json!([stencila_paragraph_json("Note")])
    );
    assert_eq!(table.rows.len(), 2);
    assert_eq!(table.rows[0].row_type, Some(TableRowType::HeaderRow));
    assert_eq!(
        table.rows[0].cells[0].cell_type,
        Some(TableCellType::HeaderCell)
    );

    Ok(())
}

#[test]
fn decode_task_list_items() -> Result<()> {
    let fixture = task_list_json();
    let (Node::Article(article), info) = decode(&fixture.to_string(), None)? else {
        bail!("Tiptap should decode to an article")
    };

    assert!(info.losses.is_empty());

    let Block::List(list) = &article.content[0] else {
        bail!("expected task list")
    };
    assert_eq!(list.order, ListOrder::Unordered);
    assert_eq!(list.items[0].is_checked, Some(true));
    assert_eq!(list.items[1].is_checked, Some(false));

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
fn decode_common_inline_marks() -> Result<()> {
    let fixture = supported_inline_marks_json();
    let (Node::Article(article), info) = decode(&fixture.to_string(), None)? else {
        bail!("Tiptap should decode to an article")
    };

    assert!(info.losses.is_empty());

    let Block::Paragraph(paragraph) = &article.content[0] else {
        bail!("expected paragraph")
    };

    let Inline::Strong(Strong { content, .. }) = &paragraph.content[0] else {
        bail!("expected strong")
    };
    let Inline::Text(text) = &content[0] else {
        bail!("expected strong text")
    };
    assert_eq!(text.value.to_string(), "bold");

    let Inline::Emphasis(Emphasis { content, .. }) = &paragraph.content[2] else {
        bail!("expected emphasis")
    };
    let Inline::Text(text) = &content[0] else {
        bail!("expected emphasis text")
    };
    assert_eq!(text.value.to_string(), "italic");

    let Inline::Link(Link {
        content,
        target,
        title,
        rel,
        ..
    }) = &paragraph.content[4]
    else {
        bail!("expected link")
    };
    assert_eq!(target.to_string(), "https://example.com");
    assert_eq!(title.as_deref(), Some("Example"));
    assert_eq!(rel.as_deref(), Some("noopener"));
    let Inline::Text(text) = &content[0] else {
        bail!("expected link text")
    };
    assert_eq!(text.value.to_string(), "link");

    let Inline::CodeInline(code) = &paragraph.content[6] else {
        bail!("expected inline code")
    };
    assert_eq!(code.code.to_string(), "code");

    let Inline::Strikeout(Strikeout { content, .. }) = &paragraph.content[8] else {
        bail!("expected strikeout")
    };
    let Inline::Text(text) = &content[0] else {
        bail!("expected strikeout text")
    };
    assert_eq!(text.value.to_string(), "strike");

    let Inline::Underline(Underline { content, .. }) = &paragraph.content[10] else {
        bail!("expected underline")
    };
    let Inline::Text(text) = &content[0] else {
        bail!("expected underline text")
    };
    assert_eq!(text.value.to_string(), "under");

    let Inline::Subscript(Subscript { content, .. }) = &paragraph.content[12] else {
        bail!("expected subscript")
    };
    let Inline::Text(text) = &content[0] else {
        bail!("expected subscript text")
    };
    assert_eq!(text.value.to_string(), "sub");

    let Inline::Superscript(Superscript { content, .. }) = &paragraph.content[14] else {
        bail!("expected superscript")
    };
    let Inline::Text(text) = &content[0] else {
        bail!("expected superscript text")
    };
    assert_eq!(text.value.to_string(), "sup");

    Ok(())
}

#[test]
fn decode_inline_mark_attrs() -> Result<()> {
    let fixture = inline_mark_attrs_json();
    let (Node::Article(article), info) = decode(&fixture.to_string(), None)? else {
        bail!("Tiptap should decode to an article")
    };

    assert!(info.losses.is_empty());

    let Block::Paragraph(paragraph) = &article.content[0] else {
        bail!("expected paragraph")
    };

    let Inline::CodeInline(code) = &paragraph.content[0] else {
        bail!("expected inline code")
    };
    assert_eq!(code.code.to_string(), "let x");
    assert_eq!(code.programming_language.as_deref(), Some("rust"));

    let Inline::Link(link) = &paragraph.content[2] else {
        bail!("expected link")
    };
    assert_eq!(link.target.to_string(), "#fig-1");
    assert_eq!(link.label_only, Some(true));

    Ok(())
}

#[test]
fn records_losses_for_unsupported_known_mark_attrs() -> Result<()> {
    let fixture = json!({
        "type": "doc",
        "content": [
            {
                "type": "paragraph",
                "content": [
                    {
                        "type": "text",
                        "marks": [{
                            "type": "link",
                            "attrs": {
                                "href": "https://example.com",
                                "target": "_blank",
                                "class": "external",
                            },
                        }],
                        "text": "external link",
                    },
                ],
            },
        ],
    });

    let (Node::Article(article), info) = decode(&fixture.to_string(), None)? else {
        bail!("Tiptap should decode to an article")
    };

    assert_eq!(
        losses_json(&info.losses)?,
        json!({
            "Link.class": 1,
            "Link.target": 1,
        })
    );

    let Block::Paragraph(paragraph) = &article.content[0] else {
        bail!("expected paragraph")
    };
    let Inline::Link(link) = &paragraph.content[0] else {
        bail!("expected link")
    };
    assert_eq!(link.target.to_string(), "https://example.com");

    Ok(())
}

#[test]
fn encode_heading() -> Result<()> {
    let node = Node::Article(Article {
        content: vec![Block::Heading(Heading::new(
            2,
            vec![Inline::Text(Text::new("Title".into()))],
        ))],
        ..Default::default()
    });

    let (json, info) = encode(&node, None)?;

    assert!(info.losses.is_empty());
    assert_eq!(
        encoded_json(&json)?,
        json!({
            "type": "doc",
            "content": [
                {
                    "type": "heading",
                    "attrs": {"level": 2},
                    "content": [{"type": "text", "text": "Title"}],
                },
            ],
        })
    );

    Ok(())
}

#[test]
fn encode_common_block_nodes() -> Result<()> {
    let mut third = ListItem::new(vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
        Text::new("Third".into()),
    )]))]);
    third.position = Some(3);
    let mut fourth = ListItem::new(vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
        Text::new("Fourth".into()),
    )]))]);
    fourth.position = Some(4);
    let table = {
        let mut table = Table::new(vec![
            TableRow {
                cells: vec![TableCell {
                    cell_type: Some(TableCellType::HeaderCell),
                    content: vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
                        Text::new("Head".into()),
                    )]))],
                    ..Default::default()
                }],
                row_type: Some(TableRowType::HeaderRow),
                ..Default::default()
            },
            TableRow::new(vec![TableCell::new(vec![Block::Paragraph(
                Paragraph::new(vec![Inline::Text(Text::new("Data".into()))]),
            )])]),
        ]);
        table.id = Some("table-1".into());
        table.label = Some("Table 1".into());
        table.label_automatically = Some(true);
        table.caption = Some(vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
            Text::new("Caption".into()),
        )]))]);
        table.notes = Some(vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
            Text::new("Note".into()),
        )]))]);
        table
    };

    let node = Node::Article(Article {
        content: vec![
            Block::QuoteBlock(QuoteBlock::new(vec![Block::Paragraph(Paragraph::new(
                vec![Inline::Text(Text::new("Quoted".into()))],
            ))])),
            Block::List(List::new(
                vec![ListItem::new(vec![Block::Paragraph(Paragraph::new(vec![
                    Inline::Text(Text::new("Bullet".into())),
                ]))])],
                ListOrder::Unordered,
            )),
            Block::List(List::new(vec![third, fourth], ListOrder::Ascending)),
            Block::CodeBlock(CodeBlock {
                id: Some("code-1".into()),
                code: "fn main() {}".into(),
                programming_language: Some("rust".into()),
                is_demo: Some(true),
                ..Default::default()
            }),
            Block::ThematicBreak(ThematicBreak::new()),
            Block::Table(table),
        ],
        ..Default::default()
    });

    let (json, info) = encode(&node, None)?;

    assert!(info.losses.is_empty());
    assert_eq!(encoded_json(&json)?, supported_block_nodes_json());

    Ok(())
}

#[test]
fn encode_task_list_items() -> Result<()> {
    let checked = {
        let mut item = ListItem::new(vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
            Text::new("Done".into()),
        )]))]);
        item.is_checked = Some(true);
        item
    };
    let unchecked = {
        let mut item = ListItem::new(vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
            Text::new("Todo".into()),
        )]))]);
        item.is_checked = Some(false);
        item
    };
    let unset = ListItem::new(vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
        Text::new("Maybe".into()),
    )]))]);

    let node = Node::Article(Article {
        content: vec![Block::List(List::new(
            vec![checked, unchecked, unset],
            ListOrder::Unordered,
        ))],
        ..Default::default()
    });

    let (json, info) = encode(&node, None)?;

    assert!(info.losses.is_empty());
    assert_eq!(
        encoded_json(&json)?,
        json!({
            "type": "doc",
            "content": [
                {
                    "type": "taskList",
                    "content": [
                        {
                            "type": "taskItem",
                            "attrs": {"checked": true},
                            "content": [
                                {
                                    "type": "paragraph",
                                    "content": [{"type": "text", "text": "Done"}],
                                },
                            ],
                        },
                        {
                            "type": "taskItem",
                            "attrs": {"checked": false},
                            "content": [
                                {
                                    "type": "paragraph",
                                    "content": [{"type": "text", "text": "Todo"}],
                                },
                            ],
                        },
                        {
                            "type": "taskItem",
                            "attrs": {"checked": false},
                            "content": [
                                {
                                    "type": "paragraph",
                                    "content": [{"type": "text", "text": "Maybe"}],
                                },
                            ],
                        },
                    ],
                },
            ],
        })
    );

    Ok(())
}

#[test]
fn encode_ordered_checked_list_records_order_loss() -> Result<()> {
    let mut item = ListItem::new(vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
        Text::new("Ordered task".into()),
    )]))]);
    item.is_checked = Some(true);

    let node = Node::Article(Article {
        content: vec![Block::List(List::new(vec![item], ListOrder::Ascending))],
        ..Default::default()
    });

    let (json, info) = encode(&node, None)?;

    assert_eq!(
        losses_json(&info.losses)?,
        serde_json::json!({
            "List.order": 1
        })
    );
    assert_eq!(
        encoded_json(&json)?,
        json!({
            "type": "doc",
            "content": [
                {
                    "type": "taskList",
                    "content": [
                        {
                            "type": "taskItem",
                            "attrs": {"checked": true},
                            "content": [
                                {
                                    "type": "paragraph",
                                    "content": [{"type": "text", "text": "Ordered task"}],
                                },
                            ],
                        },
                    ],
                },
            ],
        })
    );

    Ok(())
}

#[test]
fn encode_common_inline_marks() -> Result<()> {
    let node = Node::Article(Article {
        content: vec![Block::Paragraph(Paragraph::new(vec![
            Inline::Strong(Strong::new(vec![Inline::Text(Text::new("bold".into()))])),
            Inline::Text(Text::new(" ".into())),
            Inline::Emphasis(Emphasis::new(vec![Inline::Text(Text::new(
                "italic".into(),
            ))])),
            Inline::Text(Text::new(" ".into())),
            Inline::Link(Link {
                content: vec![Inline::Text(Text::new("link".into()))],
                target: "https://example.com".into(),
                title: Some("Example".into()),
                rel: Some("noopener".into()),
                ..Default::default()
            }),
            Inline::Text(Text::new(" ".into())),
            Inline::CodeInline(CodeInline::new("code".into())),
            Inline::Text(Text::new(" ".into())),
            Inline::Strikeout(Strikeout::new(vec![Inline::Text(Text::new(
                "strike".into(),
            ))])),
            Inline::Text(Text::new(" ".into())),
            Inline::Underline(Underline::new(vec![Inline::Text(Text::new(
                "under".into(),
            ))])),
            Inline::Text(Text::new(" ".into())),
            Inline::Subscript(Subscript::new(vec![Inline::Text(Text::new("sub".into()))])),
            Inline::Text(Text::new(" ".into())),
            Inline::Superscript(Superscript::new(vec![Inline::Text(Text::new(
                "sup".into(),
            ))])),
        ]))],
        ..Default::default()
    });

    let (json, info) = encode(&node, None)?;

    assert!(info.losses.is_empty());
    assert_eq!(encoded_json(&json)?, supported_inline_marks_json());

    Ok(())
}

#[test]
fn encode_inline_mark_attrs() -> Result<()> {
    let node = Node::Article(Article {
        content: vec![Block::Paragraph(Paragraph::new(vec![
            Inline::CodeInline(CodeInline {
                code: "let x".into(),
                programming_language: Some("rust".into()),
                ..Default::default()
            }),
            Inline::Text(Text::new(" ".into())),
            Inline::Link(Link {
                content: vec![Inline::Text(Text::new("Figure 1".into()))],
                target: "#fig-1".into(),
                label_only: Some(true),
                ..Default::default()
            }),
        ]))],
        ..Default::default()
    });

    let (json, info) = encode(&node, None)?;

    assert!(info.losses.is_empty());
    assert_eq!(encoded_json(&json)?, inline_mark_attrs_json());

    Ok(())
}

#[test]
fn roundtrip_common_block_nodes() -> Result<()> {
    let fixture = supported_block_nodes_json();
    let (node, decode_info) = decode(&fixture.to_string(), None)?;
    let (json, encode_info) = encode(&node, None)?;

    assert!(decode_info.losses.is_empty());
    assert!(encode_info.losses.is_empty());
    assert_eq!(encoded_json(&json)?, fixture);

    Ok(())
}

#[test]
fn roundtrip_task_list_items() -> Result<()> {
    let fixture = task_list_json();
    let (node, decode_info) = decode(&fixture.to_string(), None)?;
    let (json, encode_info) = encode(&node, None)?;

    assert!(decode_info.losses.is_empty());
    assert!(encode_info.losses.is_empty());
    assert_eq!(encoded_json(&json)?, fixture);

    Ok(())
}

#[test]
fn roundtrip_common_inline_marks() -> Result<()> {
    let fixture = supported_inline_marks_json();
    let (node, decode_info) = decode(&fixture.to_string(), None)?;
    let (json, encode_info) = encode(&node, None)?;

    assert!(decode_info.losses.is_empty());
    assert!(encode_info.losses.is_empty());
    assert_eq!(encoded_json(&json)?, fixture);

    Ok(())
}

#[test]
fn roundtrip_inline_mark_attrs() -> Result<()> {
    let fixture = inline_mark_attrs_json();
    let (node, decode_info) = decode(&fixture.to_string(), None)?;
    let (json, encode_info) = encode(&node, None)?;

    assert!(decode_info.losses.is_empty());
    assert!(encode_info.losses.is_empty());
    assert_eq!(encoded_json(&json)?, fixture);

    Ok(())
}

#[test]
fn preserve_table_attrs() -> Result<()> {
    let mut table = Table::new(vec![TableRow::new(vec![TableCell::new(vec![
        Block::Paragraph(Paragraph::new(vec![Inline::Text(Text::new("Cell".into()))])),
    ])])]);
    table.id = Some("table-1".into());
    table.label = Some("Table 1".into());
    table.label_automatically = Some(true);
    table.caption = Some(vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
        Text::new("Caption".into()),
    )]))]);
    table.notes = Some(vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
        Text::new("Note".into()),
    )]))]);
    let original = serde_json::to_string(&Block::Table(table.clone()))?;

    let node = Node::Article(Article {
        content: vec![Block::Table(table)],
        ..Default::default()
    });

    let (json, ..) = encode(&node, None)?;
    assert!(json.contains(r#""type":"table""#));
    assert!(json.contains(r#""caption""#));
    assert!(json.contains(r#""notes""#));

    let (Node::Article(article), ..) = decode(&json, None)? else {
        bail!("Tiptap should decode to an article")
    };
    assert_eq!(serde_json::to_string(&article.content[0])?, original);

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
    let json = r#"{"type":"doc","content":[{"type":"unknownBlock"},{"type":"paragraph","content":[{"type":"text","text":"highlighted","marks":[{"type":"highlight","attrs":{"color":"yellow"}}]}]}]}"#;

    let (_, info) = decode(json, None)?;

    assert_eq!(
        losses_json(&info.losses)?,
        serde_json::json!({
            "Unknown (unknownBlock)": 1,
            "Unknown mark (highlight)": 1
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
