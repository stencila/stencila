use codec::{
    format::Format,
    schema::{
        shortcuts::{art, p},
        transforms::blocks_to_inlines,
        Block, CodeBlock, Heading, Inline, Paragraph, QuoteBlock, Table, Text, ThematicBreak,
    },
};

use crate::{
    inlines::{inlines_from_lexical, inlines_to_lexical},
    lexical,
    shared::{LexicalDecodeContext, LexicalEncodeContext},
};

pub(super) fn blocks_from_lexical(
    blocks: Vec<lexical::BlockNode>,
    context: &mut LexicalDecodeContext,
) -> Vec<Block> {
    blocks
        .into_iter()
        .flat_map(|block| block_from_lexical(block, context))
        .collect()
}

pub(super) fn blocks_to_lexical(
    blocks: &[Block],
    context: &mut LexicalEncodeContext,
) -> Vec<lexical::BlockNode> {
    blocks
        .iter()
        .map(|block| block_to_lexical(block, context))
        .collect()
}

fn block_from_lexical(block: lexical::BlockNode, context: &mut LexicalDecodeContext) -> Vec<Block> {
    // Macro to indicate type that has not yet been implemented
    macro_rules! loss {
        ($name:expr) => {{
            context.losses.add($name);
            Block::Paragraph(Paragraph {
                content: vec![Inline::Text(Text::from(format!("LOST {}", $name)))],
                ..Default::default()
            })
        }};
    }

    vec![match block {
        lexical::BlockNode::Heading(lexical::HeadingNode { tag, children, .. })
        | lexical::BlockNode::ExtendedHeading(lexical::ExtendedHeadingNode {
            tag, children, ..
        }) => heading_from_lexical(tag, children, context),

        lexical::BlockNode::Paragraph(paragraph) => paragraph_from_lexical(paragraph, context),

        lexical::BlockNode::List(..) => loss!("List"),

        lexical::BlockNode::Quote(lexical::QuoteNode { children, .. })
        | lexical::BlockNode::ExtendedQuote(lexical::ExtendedQuoteNode { children, .. }) => {
            quote_from_lexical(children, context)
        }

        lexical::BlockNode::CodeBlock(code_block) => code_block_from_lexical(code_block, context),
        lexical::BlockNode::Markdown(block) => return markdown_from_lexical(block, context),
        lexical::BlockNode::HorizontalRule(..) => thematic_break_from_lexical(),

        lexical::BlockNode::Unknown(block) => {
            let typename = block
                .get("type")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown");
            loss!(format!("Unknown ({typename})"))
        }
    }]
}

fn block_to_lexical(block: &Block, context: &mut LexicalEncodeContext) -> lexical::BlockNode {
    use Block::*;
    match block {
        Heading(heading) => heading_to_lexical(heading, context),
        Paragraph(paragraph) => paragraph_to_lexical(paragraph, context),
        QuoteBlock(quote) => quote_to_lexical(quote, context),
        CodeBlock(code_block) => code_block_to_lexical(code_block, context),
        Table(table) => table_to_lexical(table, context),
        ThematicBreak(..) => thematic_break_to_lexical(),

        _ => {
            context.losses.add(block.node_type().to_string());
            lexical::BlockNode::Paragraph(lexical::ParagraphNode::default())
        }
    }
}

fn heading_from_lexical(
    tag: lexical::HeadingTagType,
    children: Vec<lexical::InlineNode>,
    context: &mut LexicalDecodeContext,
) -> Block {
    use lexical::HeadingTagType::*;
    let level = match tag {
        H1 => 1,
        H2 => 2,
        H3 => 3,
        H4 => 4,
        H5 => 5,
        H6 => 6,
    };

    let content = inlines_from_lexical(children, context);

    Block::Heading(Heading {
        level,
        content,
        ..Default::default()
    })
}

fn heading_to_lexical(heading: &Heading, context: &mut LexicalEncodeContext) -> lexical::BlockNode {
    use lexical::HeadingTagType::*;
    let tag = match heading.level {
        1 => H1,
        2 => H2,
        3 => H3,
        4 => H4,
        5 => H5,
        _ => H6,
    };

    let children = inlines_to_lexical(&heading.content, context);

    match context.format {
        Format::Koenig => lexical::BlockNode::ExtendedHeading(lexical::ExtendedHeadingNode {
            tag,
            children,
            ..Default::default()
        }),
        _ => lexical::BlockNode::Heading(lexical::HeadingNode {
            tag,
            children,
            ..Default::default()
        }),
    }
}

fn paragraph_from_lexical(
    paragraph: lexical::ParagraphNode,
    context: &mut LexicalDecodeContext,
) -> Block {
    let content = inlines_from_lexical(paragraph.children, context);

    Block::Paragraph(Paragraph {
        content,
        ..Default::default()
    })
}

fn paragraph_to_lexical(
    paragraph: &Paragraph,
    context: &mut LexicalEncodeContext,
) -> lexical::BlockNode {
    let children = inlines_to_lexical(&paragraph.content, context);

    lexical::BlockNode::Paragraph(lexical::ParagraphNode {
        children,
        ..Default::default()
    })
}

fn quote_from_lexical(
    children: Vec<lexical::InlineNode>,
    context: &mut LexicalDecodeContext,
) -> Block {
    let content = vec![p(inlines_from_lexical(children, context))];

    Block::QuoteBlock(QuoteBlock {
        content,
        ..Default::default()
    })
}

fn quote_to_lexical(quote: &QuoteBlock, context: &mut LexicalEncodeContext) -> lexical::BlockNode {
    let inlines = blocks_to_inlines(quote.content.clone());
    let children = inlines_to_lexical(&inlines, context);

    match context.format {
        Format::Koenig => lexical::BlockNode::ExtendedQuote(lexical::ExtendedQuoteNode {
            children,
            ..Default::default()
        }),
        _ => lexical::BlockNode::Quote(lexical::QuoteNode {
            children,
            ..Default::default()
        }),
    }
}

fn code_block_from_lexical(
    code_block: lexical::CodeBlockNode,
    context: &mut LexicalDecodeContext,
) -> Block {
    // Currently, Stencila does not support captions on code blocks
    if code_block.caption.is_some() {
        context.losses.add_prop(&code_block, "caption");
    }

    Block::CodeBlock(CodeBlock {
        code: code_block.code.into(),
        programming_language: code_block.language,
        ..Default::default()
    })
}

fn code_block_to_lexical(
    code_block: &CodeBlock,
    _context: &mut LexicalEncodeContext,
) -> lexical::BlockNode {
    lexical::BlockNode::CodeBlock(lexical::CodeBlockNode {
        code: code_block.code.to_string(),
        language: code_block.programming_language.clone(),
        ..Default::default()
    })
}

fn markdown_from_lexical(
    block: lexical::MarkdownNode,
    context: &mut LexicalDecodeContext,
) -> Vec<Block> {
    match codec_markdown::decode(&block.markdown, None).and_then(|(node, ..)| node.try_into()) {
        Ok(blocks) => blocks,
        Err(error) => {
            // If decoding or transform fails (should very, rarely if at all)
            // record loss and return empty vector
            context.losses.add(format!("Markdown: {error}"));
            Vec::new()
        }
    }
}

fn table_to_lexical(table: &Table, context: &mut LexicalEncodeContext) -> lexical::BlockNode {
    let markdown = match codec_markdown::encode(&art([Block::Table(table.clone())]), None) {
        Ok((md, ..)) => md,
        Err(error) => {
            // If encoding fails (should very, rarely if at all)
            // record loss and return empty string
            context.losses.add(format!("Markdown: {error}"));
            String::new()
        }
    };

    lexical::BlockNode::Markdown(lexical::MarkdownNode {
        markdown,
        ..Default::default()
    })
}

fn thematic_break_from_lexical() -> Block {
    Block::ThematicBreak(ThematicBreak::new())
}

fn thematic_break_to_lexical() -> lexical::BlockNode {
    lexical::BlockNode::HorizontalRule(lexical::HorizontalRuleNode::default())
}
