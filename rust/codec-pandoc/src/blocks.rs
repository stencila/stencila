use std::{any::type_name_of_val, str::FromStr};

use pandoc_types::definition::{self as pandoc};

use stencila_codec::{stencila_format::Format, stencila_schema::*};
use stencila_codec_png::to_png_data_uri;
use stencila_codec_text_trait::to_text;

use crate::{
    inlines::{
        image_to_pandoc, inline_to_pandoc, inlines_from_pandoc, inlines_to_pandoc,
        string_from_pandoc_inlines,
    },
    shared::{
        PandocDecodeContext, PandocEncodeContext, attrs_attributes, attrs_classes, attrs_empty,
        get_attr,
    },
};

pub(super) fn blocks_to_pandoc(
    property: NodeProperty,
    blocks: &[Block],
    context: &mut PandocEncodeContext,
) -> Vec<pandoc::Block> {
    context.within_property(property, |context| {
        blocks
            .iter()
            .enumerate()
            .flat_map(|(index, block)| {
                context.within_index(index, |context| block_to_pandoc(block, context))
            })
            .collect()
    })
}

pub(super) fn blocks_from_pandoc(
    blocks: Vec<pandoc::Block>,
    context: &mut PandocDecodeContext,
) -> Vec<Block> {
    blocks
        .into_iter()
        .map(|block| block_from_pandoc(block, context))
        .collect()
}

pub fn block_to_pandoc(block: &Block, context: &mut PandocEncodeContext) -> Vec<pandoc::Block> {
    let block = match block {
        // Structure
        Block::Heading(para) => heading_to_pandoc(para, context),
        Block::Paragraph(para) => paragraph_to_pandoc(para, context),
        Block::Section(section) => section_to_pandoc(section, context),
        Block::ThematicBreak(..) => pandoc::Block::HorizontalRule,

        // Lists
        Block::List(list) => list_to_pandoc(list, context),

        // Tables and Figures
        Block::Table(table) => table_to_pandoc(table, context),
        Block::Figure(figure) => figure_to_pandoc(figure, context),

        // Media
        Block::AudioObject(..) | Block::ImageObject(..) | Block::VideoObject(..) => {
            media_block_to_pandoc(block.clone().into(), context)
        }

        // Code and math
        Block::CodeBlock(block) => code_block_to_pandoc(block, context),
        Block::CodeChunk(chunk) => return code_chunk_to_pandoc(chunk, context),
        Block::MathBlock(block) => math_block_to_pandoc(block, context),

        // Other
        Block::Admonition(block) => admonition_to_pandoc(block, context),
        Block::AppendixBreak(block) => appendix_break_to_pandoc(block, context),
        Block::Claim(block) => claim_to_pandoc(block, context),
        Block::CallBlock(block) => call_block_to_pandoc(block, context),
        Block::ChatMessage(block) => chat_message_to_pandoc(block, context),
        Block::IfBlock(block) => if_block_to_pandoc(block, context),
        Block::IncludeBlock(block) => include_block_to_pandoc(block, context),
        Block::InstructionBlock(block) => instruction_block_to_pandoc(block, context),
        Block::Excerpt(block) => excerpt_to_pandoc(block, context),
        Block::ForBlock(block) => for_block_to_pandoc(block, context),
        Block::QuoteBlock(block) => quote_block_to_pandoc(block, context),
        Block::RawBlock(block) => raw_block_to_pandoc(block, context),
        Block::StyledBlock(block) => styled_block_to_pandoc(block, context),

        // Block types currently ignored create an empty div and record loss
        // TODO: implement these
        _ => {
            context.losses.add(block.node_type().to_string());
            pandoc::Block::Div(attrs_empty(), Vec::new())
        }
    };
    vec![block]
}

#[rustfmt::skip]
pub fn block_from_pandoc(block: pandoc::Block, context: &mut PandocDecodeContext) -> Block {
    if let Some(block) = math_block_from_pandoc(&block, context) {
        return block
    }

    match block {
        // Structure
        pandoc::Block::Header(level, attrs, inlines) =>  heading_from_pandoc(level, attrs, inlines, context),
        pandoc::Block::Para(inlines) | pandoc::Block::Plain(inlines)=> paragraph_from_pandoc(inlines, context),
        pandoc::Block::HorizontalRule => Block::ThematicBreak(ThematicBreak::new()),

        // Lists
        pandoc::Block::BulletList(items) => list_from_pandoc(ListOrder::Unordered, items, context),
        pandoc::Block::OrderedList(_list_attrs, items) => list_from_pandoc(ListOrder::Ascending, items, context),

        // Tables and Figures
        pandoc::Block::Table(table) => table_from_pandoc(table, context),
        pandoc::Block::Figure(attrs, caption, content) => figure_from_pandoc(attrs, caption, content, context),

        // Code
        pandoc::Block::CodeBlock(attrs, code) => code_block_from_pandoc(attrs, code, context),

        // Other
        pandoc::Block::BlockQuote(blocks) => quote_block_from_pandoc(blocks, context),
        pandoc::Block::RawBlock(format, string) => raw_block_from_pandoc(format, string, context), 
        pandoc::Block::Div(attrs, blocks) => div_from_pandoc(attrs, blocks, context),
        
        // Block types currently ignored create an empty section and record loss
        // TODO: implement these
        pandoc::Block::DefinitionList(..)
        | pandoc::Block::LineBlock(..)
        | pandoc::Block::Null => {
            context.losses.add(type_name_of_val(&block));
            Block::Section(Section::default())
        }
    }
}

fn heading_to_pandoc(heading: &Heading, context: &mut PandocEncodeContext) -> pandoc::Block {
    let mut inlines = inlines_to_pandoc(NodeProperty::Content, &heading.content, context);

    if context.render
        && (matches!(heading.label_type, Some(LabelType::AppendixLabel)) || heading.label.is_some())
    {
        let mut prefix = if matches!(heading.label_type, Some(LabelType::AppendixLabel)) {
            "Appendix ".into()
        } else {
            String::new()
        };

        if let Some(label) = &heading.label {
            prefix.push_str(label);
            prefix.push(' ');
        }

        inlines.insert(0, context.output_span(pandoc::Inline::Str(prefix)));
    }

    pandoc::Block::Header(heading.level as i32, attrs_empty(), inlines)
}

fn heading_from_pandoc(
    level: i32,
    _attrs: pandoc::Attr,
    mut inlines: Vec<pandoc::Inline>,
    context: &mut PandocDecodeContext,
) -> Block {
    if matches!(context.format, Format::Docx | Format::Odt) {
        // For word processor formats, strip the appendix label that may have been
        // added during rendering. We can not rely on the custom-style=Output still
        // being on the label so we need to do this (including Strong etc if output was highlighted)...
        if let Some(pandoc::Inline::Str(first)) = inlines.first() {
            if first == "Appendix" {
                inlines.remove(0);
                if matches!(inlines.first(), Some(pandoc::Inline::Space)) {
                    inlines.remove(0);
                }
                if let Some(pandoc::Inline::Str(next)) = inlines.first()
                    && next.len() == 1
                {
                    inlines.remove(0);
                }
                if matches!(inlines.first(), Some(pandoc::Inline::Space)) {
                    inlines.remove(0);
                }
            }
        } else if let Some(
            pandoc::Inline::Emph(inners)
            | pandoc::Inline::Strong(inners)
            | pandoc::Inline::Underline(inners),
        ) = inlines.first_mut()
        {
            if let Some(pandoc::Inline::Str(first)) = inners.first()
                && first == "Appendix"
            {
                inners.remove(0);
                if matches!(inners.first(), Some(pandoc::Inline::Space)) {
                    inners.remove(0);
                }
                if let Some(pandoc::Inline::Str(next)) = inners.first()
                    && next.len() == 1
                {
                    inners.remove(0);
                }
                if matches!(inners.first(), Some(pandoc::Inline::Space)) {
                    inners.remove(0);
                }
            }
            if inners.is_empty() {
                inlines.remove(0);
            }
            while matches!(inlines.first(), Some(pandoc::Inline::Space)) {
                inlines.remove(0);
            }
        }
    }

    Block::Heading(Heading::new(
        level as i64,
        inlines_from_pandoc(inlines, context),
    ))
}

fn paragraph_to_pandoc(para: &Paragraph, context: &mut PandocEncodeContext) -> pandoc::Block {
    let inlines = inlines_to_pandoc(NodeProperty::Content, &para.content, context);

    // Do the equivalent of Pandoc's `implicit_figures` default extension https://pandoc.org/MANUAL.html#extension-implicit_figures
    if let (true, Some(pandoc::Inline::Image(_, caption, _))) =
        (inlines.len() == 1, inlines.first())
        && !caption.is_empty()
    {
        return pandoc::Block::Figure(
            attrs_empty(),
            pandoc::Caption {
                short: None,
                long: vec![pandoc::Block::Plain(caption.clone())],
            },
            vec![pandoc::Block::Plain(inlines)],
        );
    }

    if context.paragraph_to_plain {
        pandoc::Block::Plain(inlines)
    } else {
        pandoc::Block::Para(inlines)
    }
}

fn paragraph_from_pandoc(inlines: Vec<pandoc::Inline>, context: &mut PandocDecodeContext) -> Block {
    let mut inlines = inlines_from_pandoc(inlines, context);

    if let (1, Some(Inline::ImageObject(..) | Inline::AudioObject(..) | Inline::VideoObject(..))) =
        (inlines.len(), inlines.first())
    {
        match inlines.swap_remove(0) {
            Inline::AudioObject(obj) => Block::AudioObject(obj),
            Inline::ImageObject(obj) => Block::ImageObject(obj),
            Inline::VideoObject(obj) => Block::VideoObject(obj),
            inline => Block::Paragraph(Paragraph::new(vec![inline])),
        }
    } else {
        Block::Paragraph(Paragraph::new(inlines))
    }
}

fn section_to_pandoc(section: &Section, context: &mut PandocEncodeContext) -> pandoc::Block {
    let section_type = match &section.section_type {
        Some(section_type) => section_type.to_string().to_lowercase(),
        None => "".to_string(),
    };
    let class = ["section-", &section_type].concat();

    // Handle reproducible rendering for Iteration sections
    if context.render
        && context.reproducible
        && matches!(section.section_type, Some(SectionType::Iteration))
    {
        let number = match context.node_path.back() {
            Some(NodeSlot::Index(index)) => format!(" #{}", index + 1),
            _ => String::new(),
        };

        let mut blocks = Vec::new();

        blocks.push(pandoc::Block::Para(vec![context.reproducible_link(
            NodeType::Section,
            section,
            Some(NodePosition::Begin),
            pandoc::Inline::Str(format!("[Begin Iteration{number}]")),
        )]));

        blocks.append(&mut blocks_to_pandoc(
            NodeProperty::Content,
            &section.content,
            context,
        ));

        blocks.push(pandoc::Block::Para(vec![context.reproducible_link(
            NodeType::Section,
            section,
            Some(NodePosition::End),
            pandoc::Inline::Str(format!("[End Iteration{number}]")),
        )]));

        return pandoc::Block::Div(attrs_empty(), blocks);
    }

    let attrs = pandoc::Attr {
        classes: vec![class],
        ..attrs_empty()
    };
    pandoc::Block::Div(
        attrs,
        blocks_to_pandoc(NodeProperty::Content, &section.content, context),
    )
}

fn list_to_pandoc(list: &List, context: &mut PandocEncodeContext) -> pandoc::Block {
    let items = list
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            context.within_index(index, |context| {
                blocks_to_pandoc(NodeProperty::Content, &item.content, context)
            })
        })
        .collect();

    match &list.order {
        ListOrder::Ascending => pandoc::Block::OrderedList(
            pandoc::ListAttributes {
                start_number: 1,
                style: pandoc::ListNumberStyle::Decimal,
                delim: pandoc::ListNumberDelim::DefaultDelim,
            },
            items,
        ),
        _ => pandoc::Block::BulletList(items),
    }
}

fn list_from_pandoc(
    order: ListOrder,
    items: Vec<Vec<pandoc::Block>>,
    context: &mut PandocDecodeContext,
) -> Block {
    Block::List(List::new(
        items
            .into_iter()
            .map(|blocks| ListItem::new(blocks_from_pandoc(blocks, context)))
            .collect(),
        order,
    ))
}

fn table_to_pandoc(table: &Table, context: &mut PandocEncodeContext) -> pandoc::Block {
    let attrs = if let Some(label) = &table.label {
        attrs_attributes(vec![("label".into(), label.into())])
    } else {
        attrs_empty()
    };

    let caption = table
        .caption
        .as_ref()
        .map(|caption| blocks_to_pandoc(NodeProperty::Caption, caption, context))
        .unwrap_or_default();

    let mut head = vec![];
    let mut body = vec![];
    let mut foot = vec![];
    let mut cols = 0;
    for (index, row) in table.rows.iter().enumerate() {
        context.within_index(index, |context| {
            if row.cells.len() > cols {
                cols = row.cells.len();
            }
            let cells = row
                .cells
                .iter()
                .enumerate()
                .map(|(index, cell)| {
                    context.within_index(index, |context| table_cell_to_pandoc(cell, context))
                })
                .collect();
            let pandoc_row = pandoc::Row {
                attr: attrs_empty(),
                cells,
            };
            match row.row_type {
                Some(TableRowType::HeaderRow) => head.push(pandoc_row),
                Some(TableRowType::FooterRow) => foot.push(pandoc_row),
                _ => body.push(pandoc_row),
            }
        })
    }

    let colspecs = (0..cols)
        .map(|_| pandoc::ColSpec {
            ..Default::default()
        })
        .collect();

    pandoc::Block::Table(pandoc::Table {
        attr: attrs,
        caption: pandoc::Caption {
            short: None,
            long: caption,
        },
        colspecs,
        head: pandoc::TableHead {
            attr: attrs_empty(),
            rows: head,
        },
        bodies: vec![pandoc::TableBody {
            attr: attrs_empty(),
            row_head_columns: 1,
            head: vec![],
            body,
        }],
        foot: pandoc::TableFoot {
            attr: attrs_empty(),
            rows: foot,
        },
    })
}

fn table_cell_to_pandoc(cell: &TableCell, context: &mut PandocEncodeContext) -> pandoc::Cell {
    let align = match cell.horizontal_alignment {
        Some(HorizontalAlignment::AlignLeft) => pandoc::Alignment::AlignLeft,
        Some(HorizontalAlignment::AlignCenter) => pandoc::Alignment::AlignCenter,
        Some(HorizontalAlignment::AlignRight) => pandoc::Alignment::AlignRight,
        _ => pandoc::Alignment::AlignDefault,
    };

    pandoc::Cell {
        attr: attrs_empty(),
        align,
        row_span: 1,
        col_span: 1,
        content: blocks_to_pandoc(NodeProperty::Content, &cell.content, context),
    }
}

fn table_from_pandoc(table: pandoc::Table, context: &mut PandocDecodeContext) -> Block {
    let label = get_attr(&table.attr, "label");
    let label_automatically = label.is_some().then_some(false);

    // Remove any leading table label, semicolon and space. These can be added
    // by software such as Word or Libre Office when editing DOCX or ODT but
    // Stencila treats "Table X" as being separate to the caption itself.
    let mut caption = table.caption.long;
    if let Some(pandoc::Block::Para(para)) = caption.first_mut() {
        if let Some(pandoc::Inline::Str(str)) = para.first()
            && str.starts_with("Table")
            && str.ends_with(":")
        {
            para.remove(0);
        }
        if let Some(pandoc::Inline::Str(str)) = para.first()
            && str == ":"
        {
            para.remove(0);
        }
        if matches!(para.first(), Some(pandoc::Inline::Space)) {
            para.remove(0);
        }
    }

    let caption = blocks_from_pandoc(caption, context);
    let caption = (!caption.is_empty()).then_some(caption);

    let head: Vec<TableRow> = table
        .head
        .rows
        .into_iter()
        .map(|row| table_row_from_pandoc(row, context, Some(TableRowType::HeaderRow)))
        .collect();

    let body: Vec<TableRow> = table
        .bodies
        .into_iter()
        .flat_map(|body| {
            let intermediate_head: Vec<TableRow> = body
                .head
                .into_iter()
                .map(|row| table_row_from_pandoc(row, context, Some(TableRowType::HeaderRow)))
                .collect();
            let intermediate_body: Vec<TableRow> = body
                .body
                .into_iter()
                .map(|row| table_row_from_pandoc(row, context, None))
                .collect();
            [intermediate_head, intermediate_body].concat()
        })
        .collect();

    let foot: Vec<TableRow> = table
        .foot
        .rows
        .into_iter()
        .map(|row| table_row_from_pandoc(row, context, Some(TableRowType::FooterRow)))
        .collect();

    let rows = [head, body, foot].concat();

    Block::Table(Table {
        rows,
        label,
        label_automatically,
        caption,
        ..Default::default()
    })
}

fn table_row_from_pandoc(
    row: pandoc::Row,
    context: &mut PandocDecodeContext,
    row_type: Option<TableRowType>,
) -> TableRow {
    let cells = row
        .cells
        .into_iter()
        .map(|cell| table_cell_from_pandoc(cell, context))
        .collect();

    TableRow {
        cells,
        row_type,
        ..Default::default()
    }
}

fn table_cell_from_pandoc(cell: pandoc::Cell, context: &mut PandocDecodeContext) -> TableCell {
    let content = blocks_from_pandoc(cell.content, context);

    let horizontal_alignment = match cell.align {
        pandoc::Alignment::AlignLeft => Some(HorizontalAlignment::AlignLeft),
        pandoc::Alignment::AlignCenter => Some(HorizontalAlignment::AlignCenter),
        pandoc::Alignment::AlignRight => Some(HorizontalAlignment::AlignRight),
        _ => None,
    };

    let row_span = match cell.row_span {
        1 => None,
        value => Some(value as i64),
    };
    let column_span = match cell.col_span {
        1 => None,
        value => Some(value as i64),
    };

    TableCell {
        content,
        horizontal_alignment,
        options: Box::new(TableCellOptions {
            row_span,
            column_span,
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn figure_to_pandoc(figure: &Figure, context: &mut PandocEncodeContext) -> pandoc::Block {
    let attrs = if let Some(label) = &figure.label {
        attrs_attributes(vec![("label".into(), label.into())])
    } else {
        attrs_empty()
    };

    context.paragraph_to_plain = true;

    let caption = figure
        .caption
        .as_ref()
        .map(|blocks| blocks_to_pandoc(NodeProperty::Caption, blocks, context))
        .unwrap_or_default();

    let blocks = blocks_to_pandoc(NodeProperty::Content, &figure.content, context);

    context.paragraph_to_plain = false;

    pandoc::Block::Figure(
        attrs,
        pandoc::Caption {
            short: None,
            long: caption,
        },
        blocks,
    )
}

fn figure_from_pandoc(
    attrs: pandoc::Attr,
    caption: pandoc::Caption,
    content: Vec<pandoc::Block>,
    context: &mut PandocDecodeContext,
) -> Block {
    let content = blocks_from_pandoc(content, context);

    let label = get_attr(&attrs, "label");
    let label_automatically = label.is_some().then_some(false);

    let caption = (!caption.long.is_empty()).then(|| blocks_from_pandoc(caption.long, context));

    Block::Figure(Figure {
        content,
        label,
        label_automatically,
        caption,
        ..Default::default()
    })
}

fn media_block_to_pandoc(media_inline: Inline, context: &mut PandocEncodeContext) -> pandoc::Block {
    pandoc::Block::Para(vec![inline_to_pandoc(&media_inline, context)])
}

fn code_block_to_pandoc(
    code_block: &CodeBlock,
    _context: &mut PandocEncodeContext,
) -> pandoc::Block {
    let classes = code_block
        .programming_language
        .as_ref()
        .map_or(Vec::new(), |lang| vec![lang.to_string()]);

    let attrs = pandoc::Attr {
        classes,
        ..Default::default()
    };

    pandoc::Block::CodeBlock(attrs, code_block.code.to_string())
}

fn code_block_from_pandoc(
    attrs: pandoc::Attr,
    code: String,
    _context: &mut PandocDecodeContext,
) -> Block {
    let mut programming_language = attrs.classes.first().cloned();

    let is_executable = attrs.classes.iter().any(|class| class == "exec")
        || attrs.attributes.iter().any(|(name, ..)| name == "exec");

    if is_executable {
        if programming_language.as_deref() == Some("exec") {
            programming_language = None;
        }

        let is_echoed = attrs.attributes.iter().find_map(|(name, value)| {
            (name == "echo")
                .then_some(["true", "yes", "1"].contains(&value.to_lowercase().as_str()))
        });

        let is_hidden = attrs.attributes.iter().find_map(|(name, value)| {
            (name == "hide")
                .then_some(["true", "yes", "1"].contains(&value.to_lowercase().as_str()))
        });

        let execution_mode = attrs
            .classes
            .iter()
            .find_map(|class| ExecutionMode::from_str(class).ok())
            .or_else(|| {
                attrs
                    .attributes
                    .iter()
                    .find_map(|(name, ..)| ExecutionMode::from_str(name).ok())
            });

        let execution_bounds = attrs
            .classes
            .iter()
            .find_map(|class| ExecutionBounds::from_str(class).ok())
            .or_else(|| {
                attrs
                    .attributes
                    .iter()
                    .find_map(|(name, ..)| ExecutionBounds::from_str(name).ok())
            });

        let label_type =
            get_attr(&attrs, "label-type").and_then(|value| LabelType::from_str(&value).ok());
        let label = get_attr(&attrs, "label");
        let label_automatically = label.is_some().then_some(false);
        let caption = get_attr(&attrs, "caption").map(|caption| {
            vec![Block::Paragraph(Paragraph::new(vec![Inline::Text(
                caption.into(),
            )]))]
        });

        return Block::CodeChunk(CodeChunk {
            programming_language,
            is_echoed,
            is_hidden,
            execution_mode,
            execution_bounds,
            label_type,
            label,
            label_automatically,
            caption,
            code: code.into(),
            ..Default::default()
        });
    }

    Block::CodeBlock(CodeBlock {
        programming_language,
        code: code.into(),
        ..Default::default()
    })
}

fn code_chunk_to_pandoc(
    chunk: &CodeChunk,
    context: &mut PandocEncodeContext,
) -> Vec<pandoc::Block> {
    if context.render {
        let Some(outputs) = &chunk.outputs else {
            return if context.reproducible {
                // If no outputs and reproducible then just a repro-link
                let link = context.reproducible_link(
                    NodeType::CodeChunk,
                    chunk,
                    None,
                    // Previously, this was labelled "[Code Chunk: no outputs]" but this is confusing when,
                    // for instance, the chunk is within the the content of the for loop
                    // and has no content, simply because it has not been executed.
                    pandoc::Inline::Str("[Code Chunk]".into()),
                );
                vec![pandoc::Block::Para(vec![link])]
            } else {
                // If no outputs and not reproducible then nothing
                Vec::new()
            };
        };

        // TODO: handle multiple outputs, this just takes first
        let inline = if let Some(output) = outputs.first() {
            match output {
                Node::ImageObject(image) => {
                    // Only use image_to_pandoc if the image has a "image/..."
                    // media type or a content url with an image media type. For
                    // other images (e.g. plotly, html map) use
                    // `to_png_data_uri` to create an image object

                    let has_image_media_type = image
                        .media_type
                        .as_ref()
                        .is_some_and(|media_type| media_type.starts_with("image/"));

                    let has_image_content_url = {
                        let url_lower = image.content_url.to_lowercase();
                        url_lower.contains("data:image/")
                            || url_lower.ends_with(".png")
                            || url_lower.ends_with(".jpg")
                            || url_lower.ends_with(".jpeg")
                            || url_lower.ends_with(".gif")
                            || url_lower.ends_with(".webp")
                            || url_lower.ends_with(".svg")
                    };

                    if has_image_media_type || has_image_content_url {
                        image_to_pandoc(image, context)
                    } else {
                        render_output_to_pandoc(output, context)
                    }
                }
                // Some other
                _ => render_output_to_pandoc(output, context),
            }
        } else {
            pandoc::Inline::Str("".into())
        };

        let inline = if context.reproducible {
            context.reproducible_link(NodeType::CodeChunk, chunk, None, inline)
        } else {
            context.output_span(inline)
        };

        return vec![pandoc::Block::Para(vec![inline])];
    }

    // If not render mode, and if these formats, then encode nothing
    if matches!(context.format, Format::Docx | Format::Odt) {
        return Vec::new();
    }

    // Otherwise, render a static code block..

    let mut classes = chunk
        .programming_language
        .as_ref()
        .map_or(Vec::new(), |lang| vec![lang.to_string()]);
    classes.push("exec".into());
    if let Some(mode) = &chunk.execution_mode {
        classes.push(mode.to_string().to_lowercase())
    }

    let mut attributes = Vec::new();
    if let Some(label_type) = &chunk.label_type {
        attributes.push(("label-type".into(), label_type.to_string()));
    }
    if let Some(label) = &chunk.label {
        attributes.push(("label".into(), label.into()));
    }
    if let Some(caption) = &chunk.caption {
        attributes.push(("caption".into(), to_text(caption).trim().into()));
    }

    let attrs = pandoc::Attr {
        classes,
        attributes,
        ..Default::default()
    };
    vec![pandoc::Block::CodeBlock(attrs, chunk.code.to_string())]
}

fn render_output_to_pandoc(output: &Node, context: &mut PandocEncodeContext) -> pandoc::Inline {
    match to_png_data_uri(output) {
        Ok(data_uri) => {
            let image = ImageObject {
                content_url: data_uri,
                media_type: Some("image/png".to_string()),
                ..Default::default()
            };
            image_to_pandoc(&image, context)
        }
        Err(error) => {
            tracing::error!("While encoding output to data URI: {error}");
            pandoc::Inline::Str(to_text(output))
        }
    }
}

fn math_block_to_pandoc(
    math_block: &MathBlock,
    context: &mut PandocEncodeContext,
) -> pandoc::Block {
    if let Some(lang) = &math_block.math_language
        && lang != "tex"
    {
        context.losses.add("MathBlock.mathLanguage");
    }

    pandoc::Block::Para(vec![pandoc::Inline::Math(
        pandoc::MathType::DisplayMath,
        math_block.code.to_string(),
    )])
}

fn math_block_from_pandoc(
    block: &pandoc::Block,
    _context: &mut PandocDecodeContext,
) -> Option<Block> {
    if let pandoc::Block::Para(inlines) = block
        && let (1, Some(pandoc::Inline::Math(pandoc::MathType::DisplayMath, code))) =
            (inlines.len(), inlines.first())
    {
        return Some(Block::MathBlock(MathBlock {
            code: code.into(),
            math_language: Some("tex".to_string()),
            ..Default::default()
        }));
    }

    None
}

fn claim_to_pandoc(claim: &Claim, context: &mut PandocEncodeContext) -> pandoc::Block {
    let class = ["claim-", &claim.claim_type.to_string().to_lowercase()].concat();

    let mut attributes = Vec::new();
    if let Some(label) = &claim.label {
        attributes.push(("label".into(), label.clone()));
    }

    let attrs = pandoc::Attr {
        classes: vec![class],
        attributes,
        ..attrs_empty()
    };

    pandoc::Block::Div(
        attrs,
        blocks_to_pandoc(NodeProperty::Content, &claim.content, context),
    )
}

fn call_block_to_pandoc(block: &CallBlock, context: &mut PandocEncodeContext) -> pandoc::Block {
    let mut attributes = vec![("source".into(), block.source.clone())];
    let classes = vec!["call".into()];
    if let Some(media) = &block.media_type {
        attributes.push(("media".into(), media.to_string().clone()));
    }
    if let Some(select) = &block.select {
        attributes.push(("select".into(), select.to_string().clone()))
    }
    if let Some(mode) = &block.execution_mode {
        let mode = mode.to_string();
        attributes.push(("mode".into(), mode))
    }
    let arguments = &block.arguments;
    for argument in arguments {
        let name = &argument.name;
        let name = name.to_string();
        let code = &argument.code;
        let code = code.to_string();
        attributes.push((["arg-", &name].concat(), code));
    }

    let attrs = pandoc::Attr {
        classes,
        attributes,
        ..attrs_empty()
    };
    let content = &block.content.clone().unwrap_or_default();

    pandoc::Block::Div(
        attrs,
        blocks_to_pandoc(NodeProperty::Content, content, context),
    )
}

fn call_block_from_pandoc(
    attrs: pandoc::Attr,
    mut blocks: Vec<pandoc::Block>,
    context: &mut PandocDecodeContext,
) -> Block {
    let mut source = attrs
        .attributes
        .iter()
        .find_map(|(name, value)| (name == "source").then_some(value.clone()));
    if source.is_none()
        && context.format == Format::Latex
        && !blocks.is_empty()
        && let Some(pandoc::Block::Para(inlines)) = blocks.get_mut(0)
    {
        if let Some(pandoc::Inline::Span(..)) = inlines.first()
            && let pandoc::Inline::Span(.., inlines) = inlines.remove(0)
        {
            source = Some(string_from_pandoc_inlines(inlines))
        }
        if let Some(pandoc::Inline::SoftBreak) = inlines.first() {
            inlines.remove(0);
        }
    }
    let source = source.unwrap_or_default();

    let mut arguments = Vec::new();
    let mut select = None;
    let mut media_type = None;
    let mut execution_mode = None;
    for (name, value) in attrs.attributes {
        if name == "select" {
            select = Some(value);
        } else if name == "media" {
            media_type = Some(value);
        } else if name == "mode" {
            execution_mode = ExecutionMode::from_str(&value).ok();
        } else if let Some(name) = name.strip_prefix("arg-") {
            arguments.push(CallArgument {
                name: name.to_string(),
                code: value.into(),
                ..Default::default()
            })
        }
    }

    let content = blocks_from_pandoc(blocks, context);
    let content = (!content.is_empty()).then_some(content);

    Block::CallBlock(CallBlock {
        execution_mode,
        source,
        media_type,
        select,
        content,
        arguments,
        ..Default::default()
    })
}

fn chat_message_to_pandoc(
    message: &ChatMessage,
    context: &mut PandocEncodeContext,
) -> pandoc::Block {
    let attrs = pandoc::Attr {
        classes: vec!["chat-message".into()],
        ..attrs_empty()
    };

    let blocks = blocks_to_pandoc(NodeProperty::Content, &message.content, context);

    pandoc::Block::Div(attrs, blocks)
}

fn admonition_to_pandoc(admon: &Admonition, context: &mut PandocEncodeContext) -> pandoc::Block {
    let class = [
        "callout-",
        &admon.admonition_type.to_string().to_lowercase(),
    ]
    .concat();

    let mut attributes = Vec::new();
    if let Some(title) = &admon.title {
        attributes.push(("title".into(), to_text(title)));
    }
    if let Some(is_folded) = &admon.is_folded {
        attributes.push(("collapse".into(), is_folded.to_string()));
    }

    let attrs = pandoc::Attr {
        classes: vec![class],
        attributes,
        ..attrs_empty()
    };

    pandoc::Block::Div(
        attrs,
        blocks_to_pandoc(NodeProperty::Content, &admon.content, context),
    )
}

fn appendix_break_to_pandoc(
    _appendix: &AppendixBreak,
    context: &mut PandocEncodeContext,
) -> pandoc::Block {
    pandoc::Block::Div(
        attrs_classes(vec!["appendix-break".into()]),
        blocks_to_pandoc(NodeProperty::Content, &[], context),
    )
}

fn appendix_break_from_pandoc(
    _attrs: pandoc::Attr,
    _blocks: Vec<pandoc::Block>,
    _context: &mut PandocDecodeContext,
) -> Block {
    Block::AppendixBreak(AppendixBreak::default())
}

fn if_block_to_pandoc(block: &IfBlock, context: &mut PandocEncodeContext) -> pandoc::Block {
    if context.render {
        // In render mode, only render the active clause's content
        if let Some(active_clause) = block
            .clauses
            .iter()
            .find(|clause| clause.is_active == Some(true))
        {
            let mut blocks = Vec::new();

            if context.reproducible {
                blocks.push(pandoc::Block::Para(vec![context.reproducible_link(
                    NodeType::IfBlock,
                    block,
                    Some(NodePosition::Begin),
                    pandoc::Inline::Str("[Begin If Block]".to_string()),
                )]));
            }

            blocks.append(&mut blocks_to_pandoc(
                NodeProperty::Content,
                &active_clause.content,
                context,
            ));

            if context.reproducible {
                blocks.push(pandoc::Block::Para(vec![context.reproducible_link(
                    NodeType::IfBlock,
                    block,
                    Some(NodePosition::End),
                    pandoc::Inline::Str("[End If Block]".to_string()),
                )]));
            }

            return pandoc::Block::Div(attrs_empty(), blocks);
        } else {
            // No active clause, return empty div or a placeholder
            return if context.reproducible {
                pandoc::Block::Para(vec![context.reproducible_link(
                    NodeType::IfBlock,
                    block,
                    None,
                    pandoc::Inline::Str("[If Block: no active clause]".to_string()),
                )])
            } else {
                pandoc::Block::Div(attrs_empty(), Vec::new())
            };
        }
    }

    // Non-render mode: encode all clauses with metadata
    let attrs = pandoc::Attr {
        classes: vec!["if".into()],
        ..attrs_empty()
    };

    let clauses = block
        .clauses
        .iter()
        .enumerate()
        .map(|(index, clause)| {
            context.within_index(index, |context| {
                let mut attributes = vec![("code".into(), clause.code.to_string())];
                if let Some(lang) = &clause.programming_language {
                    attributes.push(("lang".into(), lang.clone()));
                }

                let attrs = pandoc::Attr {
                    classes: vec!["if-clause".into()],
                    attributes,
                    ..attrs_empty()
                };

                pandoc::Block::Div(
                    attrs,
                    blocks_to_pandoc(NodeProperty::Content, &clause.content, context),
                )
            })
        })
        .collect();

    pandoc::Block::Div(attrs, clauses)
}

fn if_block_from_pandoc(
    attrs: pandoc::Attr,
    blocks: Vec<pandoc::Block>,
    context: &mut PandocDecodeContext,
) -> Block {
    // Iterate over children and determine if each is an `IfBlockClause`
    let mut clauses = Vec::new();
    for block in blocks.iter() {
        if let pandoc::Block::Div(attrs, blocks) = block
            && attrs.classes.iter().any(|class| {
                class == "if-clause" || class == "if" || class == "elif" || class == "else"
            })
        {
            let clause = if_block_clause_from_pandoc(attrs.clone(), blocks.clone(), context);
            clauses.push(clause);
        }
    }

    if clauses.is_empty() {
        let clause = if_block_clause_from_pandoc(attrs, blocks, context);

        Block::IfBlock(IfBlock {
            clauses: vec![clause],
            ..Default::default()
        })
    } else {
        Block::IfBlock(IfBlock {
            clauses,
            ..Default::default()
        })
    }
}

fn if_block_clause_from_pandoc(
    attrs: pandoc::Attr,
    mut blocks: Vec<pandoc::Block>,
    context: &mut PandocDecodeContext,
) -> IfBlockClause {
    let mut code = None;
    let mut lang = None;
    for (name, value) in attrs.attributes {
        if name == "code" {
            code = Some(value);
        } else if name == "lang" {
            lang = Some(value);
        }
    }

    if code.is_none()
        && context.format == Format::Latex
        && !blocks.is_empty()
        && let Some(pandoc::Block::Para(inlines)) = blocks.get_mut(0)
    {
        if let Some(pandoc::Inline::Span(..)) = inlines.first()
            && let pandoc::Inline::Span(.., inlines) = inlines.remove(0)
        {
            code = Some(string_from_pandoc_inlines(inlines))
        }
        if let Some(pandoc::Inline::SoftBreak) = inlines.first() {
            inlines.remove(0);
        }
    }
    let code = code.unwrap_or_default().into();

    let content = blocks_from_pandoc(blocks, context);

    IfBlockClause {
        code,
        content,
        programming_language: lang,
        ..Default::default()
    }
}

fn include_block_to_pandoc(
    block: &IncludeBlock,
    context: &mut PandocEncodeContext,
) -> pandoc::Block {
    if context.render {
        let source = &block.source;

        if let Some(content) = &block.content {
            let mut blocks = Vec::new();

            if context.reproducible {
                blocks.push(pandoc::Block::Para(vec![context.reproducible_link(
                    NodeType::IncludeBlock,
                    block,
                    Some(NodePosition::Begin),
                    pandoc::Inline::Str(format!("[Begin {source}]")),
                )]));
            }

            blocks.append(&mut blocks_to_pandoc(
                NodeProperty::Content,
                content,
                context,
            ));

            if context.reproducible {
                blocks.push(pandoc::Block::Para(vec![context.reproducible_link(
                    NodeType::IncludeBlock,
                    block,
                    Some(NodePosition::End),
                    pandoc::Inline::Str(format!("[End {source}]")),
                )]));
            }

            return pandoc::Block::Div(attrs_empty(), blocks);
        } else {
            return pandoc::Block::Para(vec![context.reproducible_link(
                NodeType::IncludeBlock,
                block,
                None,
                pandoc::Inline::Str(format!("[Include {source}]")),
            )]);
        }
    }

    // If LaTeX, encode as an input command
    if matches!(context.format, Format::Latex | Format::Rnw) {
        return pandoc::Block::RawBlock(
            pandoc::Format("latex".into()),
            ["\\input{", block.source.trim_end_matches(".tex"), "}"].concat(),
        );
    }

    // Otherwise encode as a div with class "include"
    let mut attributes = vec![("source".into(), block.source.clone())];
    if let Some(media) = &block.media_type {
        attributes.push(("media".into(), media.to_string().clone()));
    }
    if let Some(select) = &block.select {
        attributes.push(("select".into(), select.to_string().clone()))
    }
    if let Some(mode) = &block.execution_mode {
        let mode = mode.to_string().to_lowercase();
        attributes.push(("mode".into(), mode))
    }

    let attrs = pandoc::Attr {
        classes: vec!["include".into()],
        attributes,
        ..attrs_empty()
    };
    let content = &block.content.clone().unwrap_or_default();

    pandoc::Block::Div(
        attrs,
        blocks_to_pandoc(NodeProperty::Content, content, context),
    )
}

fn include_block_from_pandoc(
    attrs: pandoc::Attr,
    mut blocks: Vec<pandoc::Block>,
    context: &mut PandocDecodeContext,
) -> Block {
    let mut source = attrs
        .attributes
        .iter()
        .find_map(|(name, value)| (name == "source").then_some(value.clone()));
    if source.is_none()
        && context.format == Format::Latex
        && !blocks.is_empty()
        && let Some(pandoc::Block::Para(inlines)) = blocks.get_mut(0)
    {
        if let Some(pandoc::Inline::Span(..)) = inlines.first()
            && let pandoc::Inline::Span(.., inlines) = inlines.remove(0)
        {
            source = Some(string_from_pandoc_inlines(inlines))
        }
        if let Some(pandoc::Inline::SoftBreak) = inlines.first() {
            inlines.remove(0);
        }
    }
    let mut source = source.unwrap_or_default();

    let mut select = None;
    let mut media_type = None;
    let mut execution_mode = None;
    for (name, value) in attrs.attributes {
        if name == "select" {
            select = Some(value);
        } else if name == "media" {
            media_type = Some(value);
        } else if name == "mode" {
            execution_mode = ExecutionMode::from_str(&value).ok();
        }
    }

    let content = if source.is_empty() && matches!(context.format, Format::Latex) {
        // Content is the path of the source
        source = blocks
            .pop()
            .map(|node| match node {
                pandoc::Block::Para(inlines) => string_from_pandoc_inlines(inlines),
                _ => String::new(),
            })
            .unwrap_or_default();
        None
    } else {
        let content = blocks_from_pandoc(blocks, context);
        (!content.is_empty()).then_some(content)
    };

    Block::IncludeBlock(IncludeBlock {
        execution_mode,
        source,
        media_type,
        select,
        content,
        ..Default::default()
    })
}

fn instruction_block_to_pandoc(
    block: &InstructionBlock,
    context: &mut PandocEncodeContext,
) -> pandoc::Block {
    let mut attributes = vec![(
        "type".into(),
        block.instruction_type.to_string().to_lowercase(),
    )];

    if let Some(prompt) = &block.prompt.target {
        attributes.push(("prompt".into(), prompt.to_string()));
    }

    if let Some(MessagePart::Text(Text { value, .. })) = &block.message.parts.first() {
        attributes.push(("message".into(), value.to_string()));
    } else {
        context.losses.add("InstructionBlock.message.parts")
    }

    if let Some(mode) = &block.execution_mode {
        attributes.push(("execution-mode".into(), mode.to_string()));
    }

    if let Some(active_suggestion) = &block.active_suggestion {
        attributes.push(("active-suggestion".into(), active_suggestion.to_string()));
    }

    let attrs = pandoc::Attr {
        classes: vec!["instruction".into()],
        attributes,
        ..attrs_empty()
    };

    let content = &block.content.clone().unwrap_or_default();

    pandoc::Block::Div(
        attrs,
        blocks_to_pandoc(NodeProperty::Content, content, context),
    )
}

fn excerpt_to_pandoc(block: &Excerpt, context: &mut PandocEncodeContext) -> pandoc::Block {
    let attrs = pandoc::Attr {
        classes: vec!["excerpt".into()],
        // TODO: encode source to attributes or some other how
        ..attrs_empty()
    };

    pandoc::Block::Div(
        attrs,
        blocks_to_pandoc(NodeProperty::Content, &block.content, context),
    )
}

fn for_block_to_pandoc(block: &ForBlock, context: &mut PandocEncodeContext) -> pandoc::Block {
    if context.render {
        let mut blocks = Vec::new();

        if context.reproducible {
            blocks.push(pandoc::Block::Para(vec![context.reproducible_link(
                NodeType::ForBlock,
                block,
                Some(NodePosition::Begin),
                pandoc::Inline::Str("[Begin For Block]".to_string()),
            )]));

            // Include content before iterations so that it can be edited
            blocks.append(&mut blocks_to_pandoc(
                NodeProperty::Content,
                &block.content,
                context,
            ));
        }

        // Include all iterations
        if let Some(iterations) = &block.iterations {
            blocks.append(&mut blocks_to_pandoc(
                NodeProperty::Iterations,
                iterations,
                context,
            ));
        }

        if context.reproducible {
            blocks.push(pandoc::Block::Para(vec![context.reproducible_link(
                NodeType::ForBlock,
                block,
                Some(NodePosition::End),
                pandoc::Inline::Str("[End For Block]".to_string()),
            )]));
        }

        return pandoc::Block::Div(attrs_empty(), blocks);
    }

    // Non-render mode: encode with metadata
    let mut attributes = vec![
        ("variable".into(), block.variable.clone()),
        ("code".into(), block.code.to_string()),
    ];
    if let Some(lang) = &block.programming_language {
        attributes.push(("lang".into(), lang.clone()));
    }

    let attrs = pandoc::Attr {
        classes: vec!["for".into()],
        attributes,
        ..attrs_empty()
    };

    pandoc::Block::Div(
        attrs,
        blocks_to_pandoc(NodeProperty::Content, &block.content, context),
    )
}

fn for_block_from_pandoc(
    attrs: pandoc::Attr,
    mut blocks: Vec<pandoc::Block>,
    context: &mut PandocDecodeContext,
) -> Block {
    let mut variable = attrs
        .attributes
        .iter()
        .find_map(|(name, value)| (name == "variable").then_some(value.clone()));

    let mut code = attrs
        .attributes
        .iter()
        .find_map(|(name, value)| (name == "code").then_some(value.clone()));

    // If variable and code are none, then likely this is from LaTeX and so try getting them
    // from the first two spans of the first paragraph (the LaTex environment args that Pandoc
    // does not handle)
    if variable.is_none()
        && code.is_none()
        && context.format == Format::Latex
        && let Some(pandoc::Block::Para(inlines)) = blocks.get_mut(0)
    {
        if let Some(pandoc::Inline::Span(..)) = inlines.first()
            && let pandoc::Inline::Span(.., inlines) = inlines.remove(0)
        {
            variable = Some(string_from_pandoc_inlines(inlines));
        }
        if let Some(pandoc::Inline::Span(..)) = inlines.first()
            && let pandoc::Inline::Span(.., inlines) = inlines.remove(0)
        {
            code = Some(string_from_pandoc_inlines(inlines));
        }
        if let Some(pandoc::Inline::SoftBreak) = inlines.first() {
            inlines.remove(0);
        }
    }

    // If still none, then just make them empty strings
    let variable = variable.unwrap_or_default();
    let code = code.unwrap_or_default().into();

    let programming_language = attrs
        .attributes
        .into_iter()
        .find_map(|(name, value)| (name == "lang").then_some(value));

    let content = blocks_from_pandoc(blocks, context);

    Block::ForBlock(ForBlock {
        variable,
        code,
        programming_language,
        content,
        ..Default::default()
    })
}

fn quote_block_to_pandoc(block: &QuoteBlock, context: &mut PandocEncodeContext) -> pandoc::Block {
    if block.source.is_some() {
        context.losses.add("QuoteBlock.source");
    }

    pandoc::Block::BlockQuote(blocks_to_pandoc(
        NodeProperty::Content,
        &block.content,
        context,
    ))
}

fn quote_block_from_pandoc(blocks: Vec<pandoc::Block>, context: &mut PandocDecodeContext) -> Block {
    Block::QuoteBlock(QuoteBlock::new(blocks_from_pandoc(blocks, context)))
}

fn raw_block_to_pandoc(block: &RawBlock, _context: &mut PandocEncodeContext) -> pandoc::Block {
    pandoc::Block::RawBlock(
        pandoc::Format(block.format.clone()),
        block.content.to_string(),
    )
}

fn raw_block_from_pandoc(
    format: pandoc::Format,
    code: String,
    _context: &mut PandocDecodeContext,
) -> Block {
    Block::RawBlock(RawBlock::new(format.0, code.into()))
}

fn styled_block_to_pandoc(block: &StyledBlock, context: &mut PandocEncodeContext) -> pandoc::Block {
    if block.style_language.is_some() {
        context.losses.add("StyledBlock.styleLanguage");
    }

    let classes = block.code.split(' ').map(String::from).collect();

    pandoc::Block::Div(
        attrs_classes(classes),
        blocks_to_pandoc(NodeProperty::Content, &block.content, context),
    )
}

fn div_from_pandoc(
    attrs: pandoc::Attr,
    blocks: Vec<pandoc::Block>,
    context: &mut PandocDecodeContext,
) -> Block {
    let classes = &attrs.classes;

    if classes
        .iter()
        .any(|class| class == "appendix" || class == "appendix-break")
    {
        return appendix_break_from_pandoc(attrs, blocks, context);
    };

    if classes.iter().any(|class| class == "include") {
        return include_block_from_pandoc(attrs, blocks, context);
    };

    if classes.iter().any(|class| class == "call") {
        return call_block_from_pandoc(attrs, blocks, context);
    };

    if classes
        .iter()
        .any(|class| class == "if" || class == "if-block" || class == "ifblock")
    {
        return if_block_from_pandoc(attrs, blocks, context);
    };

    if classes.iter().any(|class| class == "for") {
        return for_block_from_pandoc(attrs, blocks, context);
    };

    let content = blocks_from_pandoc(blocks, context);

    if let Some(admon_type) = attrs
        .classes
        .iter()
        .find_map(|class| class.strip_prefix("callout-"))
        && let Ok(admonition_type) = AdmonitionType::from_str(admon_type)
    {
        let title = attrs
            .attributes
            .iter()
            .find_map(|(name, value)| (name == "title").then_some(value))
            .map(|title| vec![Inline::Text(Text::from(title))]);

        let is_folded = attrs
            .attributes
            .iter()
            .find_map(|(name, value)| (name == "collapse").then_some(value))
            .and_then(|is_folded| match is_folded.to_lowercase().as_str() {
                "true" | "yes" => Some(true),
                "false" | "no" => Some(false),
                _ => None,
            });

        return Block::Admonition(Admonition {
            admonition_type,
            title,
            is_folded,
            content,
            ..Default::default()
        });
    };

    if let Some(claim_type) = attrs
        .classes
        .iter()
        .find_map(|class| class.strip_prefix("claim-"))
        && let Ok(claim_type) = ClaimType::from_str(claim_type)
    {
        let label = attrs
            .attributes
            .into_iter()
            .find_map(|(name, value)| (name == "label").then_some(value));

        return Block::Claim(Claim {
            claim_type,
            label,
            content,
            ..Default::default()
        });
    };

    if attrs.classes.iter().any(|class| class == "chat-message") {
        return Block::ChatMessage(ChatMessage {
            content,
            ..Default::default()
        });
    }

    if attrs.classes.iter().any(|class| class == "instruction") {
        let instruction_type = attrs
            .attributes
            .iter()
            .find_map(|(name, value)| {
                (name == "type").then_some(InstructionType::from_str(value).unwrap_or_default())
            })
            .unwrap_or_default();

        let prompt = attrs
            .attributes
            .iter()
            .find_map(|(name, value)| {
                (name == "prompt").then_some(PromptBlock {
                    target: Some(value.into()),
                    ..Default::default()
                })
            })
            .unwrap_or_default();

        let message = attrs
            .attributes
            .iter()
            .find_map(|(name, value)| {
                (name == "message").then_some(InstructionMessage {
                    parts: vec![MessagePart::Text(value.into())],
                    ..Default::default()
                })
            })
            .unwrap_or_default();

        let execution_mode = attrs.attributes.iter().find_map(|(name, value)| {
            (name == "execution-mode").then_some(ExecutionMode::from_str(value).unwrap_or_default())
        });

        let active_suggestion = attrs.attributes.iter().find_map(|(name, value)| {
            (name == "active-suggestion").then_some(value.clone().parse().unwrap_or_default())
        });

        let content = (!content.is_empty()).then_some(content);

        return Block::InstructionBlock(InstructionBlock {
            instruction_type,
            prompt,
            message,
            execution_mode,
            active_suggestion,
            content,
            ..Default::default()
        });
    }

    if let Some(section_type) = attrs
        .classes
        .iter()
        .find_map(|class| class.strip_prefix("section-"))
    {
        let section_type = SectionType::from_str(section_type).ok();
        return Block::Section(Section {
            section_type,
            content,

            ..Default::default()
        });
    }

    Block::StyledBlock(StyledBlock::new(attrs.classes.join(" ").into(), content))
}
