use eyre::Result;
use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag};
use stencila_schema::{
    Article, BlockContent, CodeBlock, CodeFragment, Delete, Emphasis, Heading, InlineContent, Link,
    Node, Paragraph, Strong, ThematicBreak,
};

/// Decode a `Node` from Markdown
pub fn decode(markdown: &str) -> Result<Node> {
    let parser = Parser::new_ext(markdown, Options::all());

    let mut inline_content: Vec<InlineContent> = Vec::new();
    let mut inline_content_marks: Vec<usize> = Vec::new();

    let mut text = String::new();

    let mut block_content: Vec<BlockContent> = Vec::new();

    for event in parser {
        match event {
            Event::Start(tag) => match tag {
                Tag::Heading(_) => inline_content.clear(),
                Tag::Paragraph => inline_content.clear(),
                Tag::CodeBlock(_) => (),
                Tag::Emphasis => inline_content_marks.push(inline_content.len()),
                Tag::Strong => inline_content_marks.push(inline_content.len()),
                Tag::Strikethrough => inline_content_marks.push(inline_content.len()),
                Tag::Link(_, _, _) => inline_content_marks.push(inline_content.len()),
                _ => println!("Start {:?}", tag),
            },
            Event::End(tag) => match tag {
                Tag::Heading(depth) => block_content.push(BlockContent::Heading(Heading {
                    depth: Some(Box::new(depth as i64)),
                    content: inline_content.clone(),
                    ..Default::default()
                })),
                Tag::Paragraph => block_content.push(BlockContent::Paragraph(Paragraph {
                    content: inline_content.clone(),
                    ..Default::default()
                })),
                Tag::CodeBlock(kind) => block_content.push(BlockContent::CodeBlock(CodeBlock {
                    text: text.clone(),
                    programming_language: match kind {
                        CodeBlockKind::Fenced(lang) => Some(Box::new(lang.to_string())),
                        _ => None,
                    },
                    ..Default::default()
                })),
                Tag::Emphasis => {
                    let n = inline_content_marks
                        .pop()
                        .expect("Unbalanced start and end");
                    let content = inline_content.split_off(n);
                    inline_content.push(InlineContent::Emphasis(Emphasis {
                        content,
                        ..Default::default()
                    }))
                }
                Tag::Strong => {
                    let n = inline_content_marks
                        .pop()
                        .expect("Unbalanced start and end");
                    let content = inline_content.split_off(n);
                    inline_content.push(InlineContent::Strong(Strong {
                        content,
                        ..Default::default()
                    }))
                }
                Tag::Strikethrough => {
                    let n = inline_content_marks
                        .pop()
                        .expect("Unbalanced start and end");
                    let content = inline_content.split_off(n);
                    inline_content.push(InlineContent::Delete(Delete {
                        content,
                        ..Default::default()
                    }))
                }
                Tag::Link(_link_type, url, title) => {
                    let n = inline_content_marks
                        .pop()
                        .expect("Unbalanced start and end");
                    let content = inline_content.split_off(n);
                    inline_content.push(InlineContent::Link(Link {
                        content,
                        target: url.to_string(),
                        title: Some(Box::new(title.to_string())),
                        ..Default::default()
                    }))
                }
                _ => println!(
                    "End {:?} {:?} {:?}",
                    tag, inline_content, inline_content_marks
                ),
            },
            Event::Text(value) => {
                inline_content.push(InlineContent::String(value.to_string()));
                text = value.to_string()
            }
            Event::Code(value) => {
                inline_content.push(InlineContent::CodeFragment(CodeFragment {
                    text: value.to_string(),
                    ..Default::default()
                }));
            }
            Event::Html(value) => {
                println!("Html {}", value);
            }
            Event::FootnoteReference(value) => {
                println!("FootnoteReference {}", value);
            }
            Event::SoftBreak => {
                println!("SoftBreak");
            }
            Event::HardBreak => {
                println!("HardBreak");
            }
            Event::Rule => block_content.push(BlockContent::ThematicBreak(ThematicBreak {
                ..Default::default()
            })),
            Event::TaskListMarker(value) => {
                println!("TaskListMarker {}", value);
            }
        };
    }

    let article = Article {
        content: Some(block_content),
        ..Default::default()
    };
    Ok(Node::Article(article))
}
