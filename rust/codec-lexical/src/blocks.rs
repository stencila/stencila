use codec::{
    common::{serde_json::from_value, tracing},
    format::Format,
    schema::{
        shortcuts::{art, p, t},
        transforms::blocks_to_inlines,
        AudioObject, Block, CodeBlock, File, Heading, ImageObject, Inline, List, ListItem,
        ListOrder, MathBlock, Paragraph, QuoteBlock, RawBlock, Table, Text, ThematicBreak,
        VideoObject,
    },
};
use codec_text::to_text;

use crate::{
    inlines::{inlines_from_lexical, inlines_to_lexical},
    lexical::{self, AudioNode, FileNode, VideoNode},
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
        lexical::BlockNode::LineBreak(..) => loss!("LineBreak"),

        lexical::BlockNode::Quote(lexical::QuoteNode { children, .. })
        | lexical::BlockNode::ExtendedQuote(lexical::ExtendedQuoteNode { children, .. })
        | lexical::BlockNode::Aside(lexical::AsideNode { children, .. }) => {
            quote_from_lexical(children, context)
        }

        lexical::BlockNode::Image(image) => image_from_lexical(image, context),
        lexical::BlockNode::Audio(audio) => audio_from_lexical(audio, context),
        lexical::BlockNode::Video(video) => video_from_lexical(video, context),

        lexical::BlockNode::File(file) => file_from_lexical(file, context),

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
        MathBlock(math_block) => math_block_to_lexical(math_block),
        Table(table) => table_to_lexical(table, context),
        ImageObject(image) => image_to_lexical(image, context),
        AudioObject(audio) => audio_to_lexical(audio),
        VideoObject(video) => video_to_lexical(video),
        File(file) => file_to_lexical(file),
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
    // If the paragraph only has an image, then create a Lexical image, rather than a paragraph
    if let (1, Some(Inline::ImageObject(image))) =
        (paragraph.content.len(), paragraph.content.first())
    {
        return image_to_lexical(image, context);
    }

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
        .map(|child| {
            ListItem {
                content: child
                    .children
                    .into_iter()
                    .fold(Vec::new(), |mut blocks, block| {
                        // split on linebreaks
                        match block {
                            lexical::BlockNode::LineBreak(..) => {
                                blocks.push(Vec::new());
                            }
                            _ => {
                                if blocks.is_empty() {
                                    blocks.push(Vec::new());
                                }
                                match blocks.last_mut() {
                                    Some(vec) => vec.push(block),
                                    None => blocks.push(Vec::new()),
                                };
                            }
                        }
                        blocks
                    })
                    .into_iter()
                    .flat_map(|blocks| {
                        blocks
                            .into_iter()
                            .flat_map(|block| {
                                if let lexical::BlockNode::Unknown(value) = block {
                                    let inlines = if let Ok(text_node) =
                                        from_value::<lexical::TextNode>(value.clone())
                                    {
                                        vec![lexical::InlineNode::Text(text_node)]
                                    } else if let Ok(extended_text) =
                                        from_value::<lexical::ExtendedTextNode>(value.clone())
                                    {
                                        vec![lexical::InlineNode::ExtendedText(extended_text)]
                                    } else if let Ok(link) =
                                        from_value::<lexical::LinkNode>(value.clone())
                                    {
                                        vec![lexical::InlineNode::Link(link)]
                                    } else if let Ok(hashtag) =
                                        from_value::<lexical::HashTagNode>(value.clone())
                                    {
                                        vec![lexical::InlineNode::HashTag(hashtag)]
                                    } else {
                                        vec![lexical::InlineNode::Unknown(value)]
                                    };

                                    vec![p(inlines_from_lexical(inlines, context))]
                                } else {
                                    blocks_from_lexical(vec![block], context)
                                }
                            })
                            .collect::<Vec<Block>>()
                    })
                    // wrap each split in a paragraph
                    .collect(),
                ..Default::default()
            }
        })
        .collect();

    let order = match list.list_type {
        lexical::ListType::Number => ListOrder::Ascending,
        lexical::ListType::Bullet => ListOrder::Unordered,
    };

    Block::List(List {
        order,
        items,
        ..Default::default()
    })
}

fn list_to_lexical(list: &List, context: &mut LexicalEncodeContext) -> lexical::BlockNode {
    if list
        .items
        .iter()
        .flat_map(|item| &item.content)
        .any(|block| matches!(block, Block::List(List { .. })))
    {
        let markdown = match codec_markdown::encode(&art([Block::List(list.clone())]), None) {
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
    } else {
        let children = list
            .items
            .iter()
            .map(|item| list_item_to_lexical(item, context))
            .collect();

        let list_type = match list.order {
            ListOrder::Ascending | ListOrder::Descending => lexical::ListType::Number,
            ListOrder::Unordered => lexical::ListType::Bullet,
        };

        lexical::BlockNode::List(lexical::ListNode {
            list_type,
            children,
            ..Default::default()
        })
    }
}

fn list_item_to_lexical(
    list_item: &ListItem,
    context: &mut LexicalEncodeContext,
) -> lexical::ListItemNode {
    let mut children = Vec::new();

    for (i, block) in list_item.content.clone().into_iter().enumerate() {
        if i != 0 {
            children.push(lexical::BlockNode::LineBreak(lexical::LineBreakNode {
                ..Default::default()
            }));
        }
        children.push(block_to_lexical(&block, context));
    }

    lexical::ListItemNode {
        children,
        ..Default::default()
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

fn math_block_to_lexical(math: &MathBlock) -> lexical::BlockNode {
    // If the math is LaTeX then represent as Markdown paragraph so the
    // user can edit it and KaTeX (if the theme has it) can render it.
    // Otherwise, if there is MathML (e.g. compiled from AsciiMath) then
    // represent as a HTML block so it can at least be rendered.

    if let Some(lang) = &math.math_language {
        let lang = lang.to_lowercase();
        if !(lang == "tex" || lang == "latex") {
            if math.options.mathml.is_some() {
                // We could just put the MathML in the HTML block but
                // by encoding using DOM HTML we get that plus the original
                // language and source code which allows us to convert back
                // in the future.
                let html = codec_dom::encode(math);
                return lexical::BlockNode::Html(lexical::HtmlNode {
                    html,
                    ..Default::default()
                });
            }

            tracing::warn!("Math written in `{lang}` may not render correctly")
        }
    }

    lexical::BlockNode::Paragraph(lexical::ParagraphNode {
        children: vec![lexical::InlineNode::Text(lexical::TextNode {
            text: ["$$ ", &math.code, " $$"].concat(),
            ..Default::default()
        })],
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

fn audio_from_lexical(audio: lexical::AudioNode, context: &mut LexicalDecodeContext) -> Block {
    context.losses.add_prop(&audio, "duration");

    Block::AudioObject(AudioObject {
        id: audio.title,
        content_url: audio.src,
        media_type: audio.mime_type,
        ..Default::default()
    })
}

fn audio_to_lexical(audio: &AudioObject) -> lexical::BlockNode {
    lexical::BlockNode::Audio(AudioNode {
        title: audio.id.clone(),
        src: audio.content_url.clone(),
        mime_type: audio.media_type.clone(),
        ..Default::default()
    })
}

fn video_from_lexical(video: lexical::VideoNode, context: &mut LexicalDecodeContext) -> Block {
    if video.file_name.is_some() {
        context.losses.add_prop(&video, "file_name");
    }
    if video.width.is_some() {
        context.losses.add_prop(&video, "width");
    }
    if video.height.is_some() {
        context.losses.add_prop(&video, "height");
    }
    if video.duration.is_some() {
        context.losses.add_prop(&video, "duration");
    }
    if video.thumbnail_src.is_some() {
        context.losses.add_prop(&video, "thumbnail_src");
    }
    if video.custom_thumbnail_src.is_some() {
        context.losses.add_prop(&video, "custom_thumbnail_src");
    }
    if video.thumbnail_width.is_some() {
        context.losses.add_prop(&video, "thumbnail_width");
    }
    if video.thumbnail_height.is_some() {
        context.losses.add_prop(&video, "thumbnail_height");
    }
    if video.card_width.is_some() {
        context.losses.add_prop(&video, "card_width");
    }
    if video.r#loop.is_some() {
        context.losses.add_prop(&video, "loop");
    }

    Block::VideoObject(VideoObject {
        content_url: video.src,
        media_type: video.mime_type,
        ..Default::default()
    })
}

fn video_to_lexical(video: &VideoObject) -> lexical::BlockNode {
    lexical::BlockNode::Video(VideoNode {
        src: video.content_url.clone(),
        mime_type: video.media_type.clone(),
        ..Default::default()
    })
}

fn file_from_lexical(file: lexical::FileNode, context: &mut LexicalDecodeContext) -> Block {
    if file.file_title.is_some() {
        context.losses.add_prop(&file, "file_title");
    }
    if file.file_caption.is_some() {
        context.losses.add_prop(&file, "file_caption");
    }

    Block::File(File {
        name: file.file_name,
        path: file.src,
        size: file.file_size,
        ..Default::default()
    })
}

fn file_to_lexical(file: &File) -> lexical::BlockNode {
    lexical::BlockNode::File(FileNode {
        file_name: file.name.clone(),
        src: file.path.clone(),
        file_size: file.size,
        ..Default::default()
    })
}
