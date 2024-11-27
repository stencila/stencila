use std::any::type_name_of_val;

use pandoc_types::definition::{self as pandoc};

use codec::schema::*;

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
        Block::QuoteBlock(block) => quote_block_to_pandoc(block, context),
        Block::RawBlock(block) => raw_block_to_pandoc(block, context),
        Block::StyledBlock(block) => styled_block_to_pandoc(block, context),

        // Block types currently ignored create an empty div and record loss
        // TODO: implement these
        Block::Admonition(..)
        | Block::CallBlock(..)
        | Block::Claim(..)
        | Block::DeleteBlock(..)
        | Block::ForBlock(..)
        | Block::Form(..)
        | Block::IfBlock(..)
        | Block::IncludeBlock(..)
        | Block::InsertBlock(..)
        | Block::InstructionBlock(..)
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
    pandoc::Block::Para(inlines_to_pandoc(&para.content, context))
}

fn paragraph_from_pandoc(inlines: Vec<pandoc::Inline>, context: &mut PandocDecodeContext) -> Block {
    Block::Paragraph(Paragraph::new(inlines_from_pandoc(inlines, context)))
}

fn section_to_pandoc(section: &Section, context: &mut PandocEncodeContext) -> pandoc::Block {
    pandoc::Block::Div(attrs_empty(), blocks_to_pandoc(&section.content, context))
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
    let caption = table
        .caption
        .as_ref()
        .map(|caption| blocks_to_pandoc(&caption, context))
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
        attr: attrs_empty(),
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

    let caption = figure
        .caption
        .as_ref()
        .map(|blocks| blocks_to_pandoc(blocks, context))
        .unwrap_or_default();

    pandoc::Block::Figure(
        attrs,
        pandoc::Caption {
            short: None,
            long: caption,
        },
        blocks_to_pandoc(&figure.content, context),
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

    let caption = (!caption.long.is_empty()).then(|| blocks_from_pandoc(caption.long, context));

    Block::Figure(Figure {
        content,
        label,
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
    let programming_language = get_attr(&attrs, "classes");

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
    if let Some(_outputs) = &code_chunk.outputs {
        // TODO
        pandoc::Block::Div(attrs_empty(), Vec::new())
    } else {
        let classes = code_chunk
            .programming_language
            .as_ref()
            .map_or(Vec::new(), |lang| vec![lang.to_string()]);

        let attrs = pandoc::Attr {
            classes,
            ..Default::default()
        };
        pandoc::Block::CodeBlock(attrs, code_chunk.code.to_string())
    }
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
    Block::StyledBlock(StyledBlock::new(
        attrs.classes.join(" ").into(),
        blocks_from_pandoc(blocks, context),
    ))
}
