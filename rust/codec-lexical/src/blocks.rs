use codec::{
    format::Format,
    schema::{
        shortcuts::{art, p, t},
        transforms::blocks_to_inlines,
        Block, CodeBlock, Heading, ImageObject, Inline, List, ListItem, Paragraph, QuoteBlock,
        RawBlock, Table, Text, ThematicBreak,
    },
};
use codec_text::to_text;

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

        lexical::BlockNode::List(list) => list_from_lexical(list, context),

        lexical::BlockNode::Quote(lexical::QuoteNode { children, .. })
        | lexical::BlockNode::ExtendedQuote(lexical::ExtendedQuoteNode { children, .. }) => {
            quote_from_lexical(children, context)
        }

        lexical::BlockNode::Image(image) => image_from_lexical(image, context),

        lexical::BlockNode::CodeBlock(code_block) => code_block_from_lexical(code_block, context),
        lexical::BlockNode::Markdown(block) => return markdown_from_lexical(block, context),
        lexical::BlockNode::Html(block) => html_from_lexical(block, context),

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
        List(list) => list_to_lexical(list, context),
        QuoteBlock(quote) => quote_to_lexical(quote, context),
        CodeBlock(code_block) => code_block_to_lexical(code_block, context),
        Table(table) => table_to_lexical(table, context),
        ImageObject(image) => image_to_lexical(image, context),
        RawBlock(block) => raw_block_to_lexical(block, context),
        ThematicBreak(..) => thematic_break_to_lexical(),
        _ => block_to_lexical_default(block),
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

fn list_from_lexical(list: lexical::ListNode, context: &mut LexicalDecodeContext) -> Block {
    let items = list
        .children
        .into_iter()
        .map(|child| ListItem::new(vec![p(inlines_from_lexical(child.children, context))]))
        .collect();

    Block::List(List {
        items,
        ..Default::default()
    })
}

fn list_to_lexical(list: &List, context: &mut LexicalEncodeContext) -> lexical::BlockNode {
    let children = list
        .items
        .clone()
        .into_iter()
        .map(|item| list_item_to_lexical(item.content.first().unwrap(), context))
        .collect();

    lexical::BlockNode::List(lexical::ListNode {
        children,
        ..Default::default()
    })
}

fn list_item_to_lexical(
    block: &Block,
    context: &mut LexicalEncodeContext,
) -> lexical::ListItemNode {
    if let Block::Paragraph(Paragraph { content, .. }) = block {
        lexical::ListItemNode {
            children: inlines_to_lexical(content, context),
            ..Default::default()
        }
    } else {
        lexical::ListItemNode {
            ..Default::default()
        }
    }
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

fn image_from_lexical(image: lexical::ImageNode, context: &mut LexicalDecodeContext) -> Block {
    // Currently, Stencila does not support captions on code blocks
    if image.width.is_some() {
        context.losses.add_prop(&image, "width");
    }
    if image.height.is_some() {
        context.losses.add_prop(&image, "height");
    }
    if image.alt.is_some() {
        context.losses.add_prop(&image, "alt");
    }
    if image.card_width.is_some() {
        context.losses.add_prop(&image, "cardWidth");
    }
    if image.href.is_some() {
        context.losses.add_prop(&image, "href");
    }

    // Captions from Ghost are wrapped in HTML e.g. "<span style=\"white-space: pre-wrap;\">Image caption</span>"
    // So we are currently ignoring them, until we work out best way to parse them.
    if image.caption.is_some() {
        context.losses.add_prop(&image, "caption");
    }

    let title = image
        .title
        .and_then(|title| (!title.is_empty()).then_some(vec![t(title)]));

    Block::ImageObject(ImageObject {
        content_url: image.src,
        title,
        ..Default::default()
    })
}

fn image_to_lexical(
    image: &ImageObject,
    _context: &mut LexicalEncodeContext,
) -> lexical::BlockNode {
    let src = image.content_url.clone();
    let title = image.title.as_ref().map(to_text);

    lexical::BlockNode::Image(lexical::ImageNode {
        src,
        title,
        ..Default::default()
    })
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

fn html_from_lexical(block: lexical::HtmlNode, context: &mut LexicalDecodeContext) -> Block {
    // Currently, Stencila does not Ghost's HTML visibility options
    if block.visibility.is_some() {
        context.losses.add_prop(&block, "visibility");
    }

    Block::RawBlock(RawBlock {
        format: "html".into(),
        content: block.html.into(),
        ..Default::default()
    })
}

fn raw_block_to_lexical(
    block: &RawBlock,
    context: &mut LexicalEncodeContext,
) -> lexical::BlockNode {
    let format = Format::from_name(&block.format);

    match format {
        Format::Markdown => lexical::BlockNode::Markdown(lexical::MarkdownNode {
            markdown: block.content.to_string(),
            ..Default::default()
        }),
        Format::Html | Format::Svg => lexical::BlockNode::Html(lexical::HtmlNode {
            html: block.content.to_string(),
            ..Default::default()
        }),
        _ => {
            // Record loss for other formats and return an empty HTML block
            context.losses.add("RawBlock");
            lexical::BlockNode::Html(lexical::HtmlNode {
                html: String::new(),
                ..Default::default()
            })
        }
    }
}

fn thematic_break_from_lexical() -> Block {
    Block::ThematicBreak(ThematicBreak::new())
}

fn thematic_break_to_lexical() -> lexical::BlockNode {
    lexical::BlockNode::HorizontalRule(lexical::HorizontalRuleNode::default())
}

fn block_to_lexical_default(block: &Block) -> lexical::BlockNode {
    // Default for Stencila block is to encode to DOM HTML and wrap
    // in a Koenig HTML card

    let html = codec_dom::encode(block);

    lexical::BlockNode::Html(lexical::HtmlNode {
        html,
        ..Default::default()
    })
}
