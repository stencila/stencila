use std::{any::type_name_of_val, str::FromStr};

use pandoc_types::definition::{self as pandoc};

use codec::schema::*;
use codec_text_trait::to_text;

use crate::{
    inlines::{inlines_from_pandoc, inlines_to_pandoc},
    shared::{
        attrs_attributes, attrs_classes, attrs_empty, get_attr, PandocDecodeContext,
        PandocEncodeContext,
    },
};

pub(super) fn blocks_to_pandoc(
    blocks: &[Block],
    context: &mut PandocEncodeContext,
) -> Vec<pandoc::Block> {
    blocks
        .iter()
        .map(|block| block_to_pandoc(block, context))
        .collect()
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

pub fn block_to_pandoc(block: &Block, context: &mut PandocEncodeContext) -> pandoc::Block {
    match block {
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

        // Code and math
        Block::CodeBlock(block) => code_block_to_pandoc(block, context),
        Block::CodeChunk(chunk) => code_chunk_to_pandoc(chunk, context),
        Block::MathBlock(block) => math_block_to_pandoc(block, context),

        // Other
        Block::Admonition(block) => admonition_to_pandoc(block, context),
        Block::Claim(block) => claim_to_pandoc(block, context),
        Block::CallBlock(block) => call_block_to_pandoc(block, context),
        Block::ChatMessage(block) => chat_message_to_pandoc(block, context),
        Block::IfBlock(block) => if_block_to_pandoc(block, context),
        Block::IncludeBlock(block) => include_block_to_pandoc(block, context),
        Block::InstructionBlock(block) => instruction_block_to_pandoc(block, context),
        Block::ForBlock(block) => for_block_to_pandoc(block, context),
        Block::QuoteBlock(block) => quote_block_to_pandoc(block, context),
        Block::RawBlock(block) => raw_block_to_pandoc(block, context),
        Block::StyledBlock(block) => styled_block_to_pandoc(block, context),

        // Block types currently ignored create an empty div and record loss
        // TODO: implement these
        Block::Chat(..)
        | Block::DeleteBlock(..)
        | Block::Form(..)
        | Block::InsertBlock(..)
        | Block::ModifyBlock(..)
        | Block::PromptBlock(..)
        | Block::ReplaceBlock(..)
        | Block::SuggestionBlock(..)
        | Block::Walkthrough(..) => {
            context.losses.add(block.node_type().to_string());
            pandoc::Block::Div(attrs_empty(), Vec::new())
        }
    }
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
        pandoc::Block::Div(attrs, blocks) => styled_block_from_pandoc(attrs, blocks, context),
        
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
    pandoc::Block::Header(
        heading.level as i32,
        attrs_empty(),
        inlines_to_pandoc(&heading.content, context),
    )
}

fn heading_from_pandoc(
    level: i32,
    _attrs: pandoc::Attr,
    inlines: Vec<pandoc::Inline>,
    context: &mut PandocDecodeContext,
) -> Block {
    Block::Heading(Heading::new(
        level as i64,
        inlines_from_pandoc(inlines, context),
    ))
}

fn paragraph_to_pandoc(para: &Paragraph, context: &mut PandocEncodeContext) -> pandoc::Block {
    let inlines = inlines_to_pandoc(&para.content, context);

    // Do the equivalent of Pandoc's `implicit_figures` default extension https://pandoc.org/MANUAL.html#extension-implicit_figures
    if let (true, Some(pandoc::Inline::Image(_, caption, _))) =
        (inlines.len() == 1, inlines.first())
    {
        if !caption.is_empty() {
            return pandoc::Block::Figure(
                attrs_empty(),
                pandoc::Caption {
                    short: None,
                    long: vec![pandoc::Block::Plain(caption.clone())],
                },
                vec![pandoc::Block::Plain(inlines)],
            );
        }
    }

    if context.paragraph_to_plain {
        pandoc::Block::Plain(inlines)
    } else {
        pandoc::Block::Para(inlines)
    }
}

fn paragraph_from_pandoc(inlines: Vec<pandoc::Inline>, context: &mut PandocDecodeContext) -> Block {
    Block::Paragraph(Paragraph::new(inlines_from_pandoc(inlines, context)))
}

fn section_to_pandoc(section: &Section, context: &mut PandocEncodeContext) -> pandoc::Block {
    let section_type = match &section.section_type {
        Some(section_type) => section_type.to_string().to_lowercase(),
        None => "".to_string(),
    };
    let class = ["section-", &section_type].concat();

    let attrs = pandoc::Attr {
        classes: vec![class],
        // attributes,
        ..attrs_empty()
    };
    pandoc::Block::Div(attrs, blocks_to_pandoc(&section.content, context))
}

fn list_to_pandoc(list: &List, context: &mut PandocEncodeContext) -> pandoc::Block {
    let items = list
        .items
        .iter()
        .map(|item| blocks_to_pandoc(&item.content, context))
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
        .map(|caption| blocks_to_pandoc(caption, context))
        .unwrap_or_default();

    let mut head = vec![];
    let mut body = vec![];
    let mut foot = vec![];
    let mut cols = 0;
    for row in &table.rows {
        if row.cells.len() > cols {
            cols = row.cells.len();
        }
        let cells = row
            .cells
            .iter()
            .map(|cell| pandoc::Cell {
                attr: attrs_empty(),
                align: pandoc::Alignment::AlignDefault,
                row_span: 1,
                col_span: 1,
                content: blocks_to_pandoc(&cell.content, context),
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

fn table_from_pandoc(table: pandoc::Table, context: &mut PandocDecodeContext) -> Block {
    let label = get_attr(&table.attr, "label");
    let label_automatically = label.is_some().then_some(false);

    let caption = blocks_from_pandoc(table.caption.long, context);
    let caption = match caption.is_empty() {
        true => None,
        false => Some(caption),
    };

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
        .map(|blocks| blocks_to_pandoc(blocks, context))
        .unwrap_or_default();

    let blocks = blocks_to_pandoc(&figure.content, context);

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

    if attrs.classes.iter().any(|class| class == "exec") {
        if programming_language.as_deref() == Some("exec") {
            programming_language = None;
        }

        let execution_mode = attrs
            .classes
            .iter()
            .find_map(|class| ExecutionMode::from_str(class).ok());

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
            execution_mode,
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
    code_chunk: &CodeChunk,
    _context: &mut PandocEncodeContext,
) -> pandoc::Block {
    // TODO: Handle outputs

    // If no outputs, then encode as a code block

    let mut classes = code_chunk
        .programming_language
        .as_ref()
        .map_or(Vec::new(), |lang| vec![lang.to_string()]);
    classes.push("exec".into());
    if let Some(mode) = &code_chunk.execution_mode {
        classes.push(mode.to_string().to_lowercase())
    }

    let mut attributes = Vec::new();
    if let Some(label_type) = &code_chunk.label_type {
        attributes.push(("label-type".into(), label_type.to_string()));
    }
    if let Some(label) = &code_chunk.label {
        attributes.push(("label".into(), label.into()));
    }
    if let Some(caption) = &code_chunk.caption {
        attributes.push(("caption".into(), to_text(caption)));
    }

    let attrs = pandoc::Attr {
        classes,
        attributes,
        ..Default::default()
    };
    pandoc::Block::CodeBlock(attrs, code_chunk.code.to_string())
}

fn math_block_to_pandoc(
    math_block: &MathBlock,
    context: &mut PandocEncodeContext,
) -> pandoc::Block {
    if let Some(lang) = &math_block.math_language {
        if lang != "tex" {
            context.losses.add("MathBlock.mathLanguage");
        }
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
    if let pandoc::Block::Para(inlines) = block {
        if let (1, Some(pandoc::Inline::Math(pandoc::MathType::DisplayMath, code))) =
            (inlines.len(), inlines.first())
        {
            return Some(Block::MathBlock(MathBlock {
                code: code.into(),
                math_language: Some("tex".to_string()),
                ..Default::default()
            }));
        }
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

    pandoc::Block::Div(attrs, blocks_to_pandoc(&claim.content, context))
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

    pandoc::Block::Div(attrs, blocks_to_pandoc(content, context))
}

fn chat_message_to_pandoc(
    message: &ChatMessage,
    context: &mut PandocEncodeContext,
) -> pandoc::Block {
    let attrs = pandoc::Attr {
        classes: vec!["chat-message".into()],
        ..attrs_empty()
    };

    let blocks = blocks_to_pandoc(&message.content, context);

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

    pandoc::Block::Div(attrs, blocks_to_pandoc(&admon.content, context))
}

fn if_block_to_pandoc(block: &IfBlock, context: &mut PandocEncodeContext) -> pandoc::Block {
    let clauses_block = &block.clauses;
    let mut clauses = Vec::new();

    for clause in clauses_block.iter() {
        let mut attributes = vec![("code".into(), clause.code.to_string())];
        if let Some(lang) = &clause.programming_language {
            attributes.push(("lang".into(), lang.clone()));
        }

        let attrs = pandoc::Attr {
            classes: vec!["if-clause".into()],
            attributes,
            ..attrs_empty()
        };

        clauses.push(pandoc::Block::Div(
            attrs,
            blocks_to_pandoc(&clause.content, context),
        ))
    }
    let attrs = pandoc::Attr {
        classes: vec!["if".into()],
        ..attrs_empty()
    };

    pandoc::Block::Div(attrs, clauses)
}

fn include_block_to_pandoc(
    block: &IncludeBlock,
    context: &mut PandocEncodeContext,
) -> pandoc::Block {
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

    pandoc::Block::Div(attrs, blocks_to_pandoc(content, context))
}

fn instruction_block_to_pandoc(
    block: &InstructionBlock,
    context: &mut PandocEncodeContext,
) -> pandoc::Block {
    let mut attributes = vec![(
        "type".into(),
        block.instruction_type.to_string().to_lowercase(),
    )];
    if let Some(message) = &block.message {
        if let Some(MessagePart::Text(Text { value, .. })) = message.parts.first() {
            attributes.push(("message".into(), value.to_string()));
        } else {
            context.losses.add("InstructionBlock.message.parts")
        }
    }
    if let Some(mode) = &block.execution_mode {
        attributes.push(("mode".into(), mode.to_string()));
    }
    if let Some(prompt) = &block.prompt {
        attributes.push(("prompt".into(), prompt.to_string()));
    }
    if let Some(active_suggestion) = &block.active_suggestion {
        attributes.push(("active_suggestion".into(), active_suggestion.to_string()));
    }
    let attrs = pandoc::Attr {
        classes: vec!["instruction".into()],
        attributes,
        ..attrs_empty()
    };
    let content = &block.content.clone().unwrap_or_default();

    pandoc::Block::Div(attrs, blocks_to_pandoc(content, context))
}

fn for_block_to_pandoc(block: &ForBlock, context: &mut PandocEncodeContext) -> pandoc::Block {
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

    pandoc::Block::Div(attrs, blocks_to_pandoc(&block.content, context))
}

fn quote_block_to_pandoc(block: &QuoteBlock, context: &mut PandocEncodeContext) -> pandoc::Block {
    if block.cite.is_some() {
        context.losses.add("QuoteBlock.cite");
    }

    pandoc::Block::BlockQuote(blocks_to_pandoc(&block.content, context))
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
        blocks_to_pandoc(&block.content, context),
    )
}

fn styled_block_from_pandoc(
    attrs: pandoc::Attr,
    blocks: Vec<pandoc::Block>,
    context: &mut PandocDecodeContext,
) -> Block {
    let content = blocks_from_pandoc(blocks, context);

    if let Some(admon_type) = attrs
        .classes
        .iter()
        .find_map(|class| class.strip_prefix("callout-"))
    {
        if let Ok(admonition_type) = AdmonitionType::from_str(admon_type) {
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
        }
    };

    if let Some(claim_type) = attrs
        .classes
        .iter()
        .find_map(|class| class.strip_prefix("claim-"))
    {
        if let Ok(claim_type) = ClaimType::from_str(claim_type) {
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
        }
    };

    if attrs.classes.iter().any(|class| class == "if-clause") {
        let code = attrs
            .attributes
            .iter()
            .find_map(|(name, value)| (name == "code").then_some(value.clone()))
            .unwrap_or_default()
            .into();

        let programming_language = attrs
            .attributes
            .into_iter()
            .find_map(|(name, value)| (name == "lang").then_some(value));
        return Block::IfBlock(IfBlock {
            clauses: vec![IfBlockClause {
                code,
                programming_language,
                content,
                ..Default::default()
            }],
            ..Default::default()
        });
    }

    if attrs.classes.iter().any(|class| class == "call") {
        let mut source = String::new();
        let mut arguments = Vec::new();
        let mut select = None;
        let mut media_type = None;
        let mut execution_mode = None;
        for (name, value) in attrs.attributes {
            if name == "source" {
                source = value;
            } else if name == "select" {
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
        let content = (!content.is_empty()).then_some(content);

        return Block::CallBlock(CallBlock {
            execution_mode,
            source,
            media_type,
            select,
            content,
            arguments,
            ..Default::default()
        });
    }

    if attrs.classes.iter().any(|class| class == "chat-message") {
        return Block::ChatMessage(ChatMessage {
            content,
            ..Default::default()
        });
    }

    if attrs.classes.iter().any(|class| class == "if") {
        let mut clauses = Vec::new();
        for block in content {
            // will never not be StyledBlock unless other error
            let clause = match block {
                Block::IfBlock(IfBlock { clauses, .. }) => clauses[0].clone(),
                _ => IfBlockClause {
                    ..Default::default()
                },
            };
            clauses.push(clause);
        }

        return Block::IfBlock(IfBlock {
            clauses,
            ..Default::default()
        });
    };

    if attrs.classes.iter().any(|class| class == "include") {
        let select = attrs
            .attributes
            .iter()
            .find_map(|(name, value)| (name == "select").then_some(value.clone()))
            .unwrap_or_default();
        let source = attrs
            .attributes
            .iter()
            .find_map(|(name, value)| (name == "source").then_some(value.clone()))
            .unwrap_or_default();
        let execution_mode = attrs
            .attributes
            .iter()
            .find_map(|(name, value)| (name == "mode").then_some(value.clone()))
            .unwrap_or_default();
        let media_type = attrs
            .attributes
            .into_iter()
            .find_map(|(name, value)| (name == "media").then_some(value));
        let execution_mode = ExecutionMode::from_str(&execution_mode).ok();
        let select = match select.as_str() {
            "" => None,
            _ => Some(select),
        };
        let content = match content[..] {
            [] => None,
            _ => Some(content),
        };

        return Block::IncludeBlock(IncludeBlock {
            execution_mode,
            source,
            media_type,
            select,
            content,
            ..Default::default()
        });
    }

    if attrs.classes.iter().any(|class| class == "instruction") {
        let execution_mode = attrs.attributes.iter().find_map(|(name, value)| {
            (name == "mode").then_some(ExecutionMode::from_str(value).unwrap_or_default())
        });
        let message = attrs.attributes.iter().find_map(|(name, value)| {
            (name == "message").then_some(InstructionMessage {
                parts: vec![MessagePart::Text(value.into())],
                ..Default::default()
            })
        });
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
            .find_map(|(name, value)| (name == "prompt").then_some(value.clone()));
        let active_suggestion = attrs.attributes.iter().find_map(|(name, value)| {
            (name == "active_suggestion").then_some(value.clone().parse().unwrap_or_default())
        });
        let content = (!content.is_empty()).then_some(content);

        return Block::InstructionBlock(InstructionBlock {
            execution_mode,
            prompt,
            instruction_type,
            active_suggestion,
            message,
            content,
            ..Default::default()
        });
    }

    if attrs.classes.iter().any(|class| class == "for") {
        let variable = attrs
            .attributes
            .iter()
            .find_map(|(name, value)| (name == "variable").then_some(value.clone()))
            .unwrap_or_default();

        let code = attrs
            .attributes
            .iter()
            .find_map(|(name, value)| (name == "code").then_some(value.clone()))
            .unwrap_or_default()
            .into();

        let programming_language = attrs
            .attributes
            .into_iter()
            .find_map(|(name, value)| (name == "lang").then_some(value));

        return Block::ForBlock(ForBlock {
            variable,
            code,
            programming_language,
            content,
            ..Default::default()
        });
    };

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
