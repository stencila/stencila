use codec::{
    format::Format,
    schema::{
        shortcuts::p, transforms::blocks_to_inlines, Block, Heading, Inline, Paragraph, QuoteBlock,
        Text,
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
        .map(|block| block_from_lexical(block, context))
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

fn block_from_lexical(block: lexical::BlockNode, context: &mut LexicalDecodeContext) -> Block {
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

    match block {
        lexical::BlockNode::Heading(lexical::HeadingNode { tag, children, .. })
        | lexical::BlockNode::ExtendedHeading(lexical::ExtendedHeadingNode {
            tag, children, ..
        }) => heading_from_lexical(tag, children, context),

        lexical::BlockNode::Paragraph(block) => paragraph_from_lexical(block, context),

        lexical::BlockNode::List(..) => loss!("List"),

        lexical::BlockNode::Quote(lexical::QuoteNode { children, .. })
        | lexical::BlockNode::ExtendedQuote(lexical::ExtendedQuoteNode { children, .. }) => {
            quote_from_lexical(children, context)
        }

        lexical::BlockNode::Unknown(block) => {
            let typename = block
                .get("type")
                .and_then(|value| value.as_str())
                .unwrap_or("unknown");
            loss!(format!("Unknown ({typename})"))
        }
    }
}

fn block_to_lexical(block: &Block, context: &mut LexicalEncodeContext) -> lexical::BlockNode {
    use Block::*;
    match block {
        Heading(block) => heading_to_lexical(block, context),
        Paragraph(block) => paragraph_to_lexical(block, context),
        QuoteBlock(block) => quote_to_lexical(block, context),

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
