use std::any::type_name_of_val;

use pandoc_types::definition as pandoc;

use codec::schema::*;

use crate::{
    inlines::{inlines_from_pandoc, inlines_to_pandoc},
    shared::{attrs_empty, PandocDecodeContext, PandocEncodeContext},
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
    attrs: pandoc::Attr,
    inlines: Vec<pandoc::Inline>,
    context: &mut PandocDecodeContext,
) -> Block {
    Block::Heading(Heading {
        level: level as i64,
        content: inlines_from_pandoc(inlines, context),
        ..Default::default()
    })
}

fn paragraph_to_pandoc(para: &Paragraph, context: &mut PandocEncodeContext) -> pandoc::Block {
    pandoc::Block::Para(inlines_to_pandoc(&para.content, context))
}

fn paragraph_from_pandoc(inlines: Vec<pandoc::Inline>, context: &mut PandocDecodeContext) -> Block {
    Block::Paragraph(Paragraph {
        content: inlines_from_pandoc(inlines, context),
        ..Default::default()
    })
}

fn section_to_pandoc(section: &Section, context: &mut PandocEncodeContext) -> pandoc::Block {
    todo!()
}

fn list_to_pandoc(list: &List, context: &mut PandocEncodeContext) -> pandoc::Block {
    todo!()
}

fn list_from_pandoc(
    list_order: ListOrder,
    blocks: Vec<Vec<pandoc::Block>>,
    context: &mut PandocDecodeContext,
) -> Block {
    todo!()
}

fn table_to_pandoc(table: &Table, context: &mut PandocEncodeContext) -> pandoc::Block {
    todo!()
}

fn table_from_pandoc(table: pandoc::Table, context: &mut PandocDecodeContext) -> Block {
    todo!()
}

fn figure_to_pandoc(figure: &Figure, context: &mut PandocEncodeContext) -> pandoc::Block {
    todo!()
}

fn figure_from_pandoc(
    attrs: pandoc::Attr,
    caption: pandoc::Caption,
    content: Vec<pandoc::Block>,
    context: &mut PandocDecodeContext,
) -> Block {
    todo!()
}

fn code_block_to_pandoc(
    code_block: &CodeBlock,
    context: &mut PandocEncodeContext,
) -> pandoc::Block {
    todo!()
}

fn code_block_from_pandoc(
    attrs: pandoc::Attr,
    code: String,
    context: &mut PandocDecodeContext,
) -> Block {
    todo!()
}

fn code_chunk_to_pandoc(
    code_chunk: &CodeChunk,
    context: &mut PandocEncodeContext,
) -> pandoc::Block {
    todo!()
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
                ..Default::default()
            }));
        }
    }

    None
}

fn quote_block_to_pandoc(block: &QuoteBlock, context: &mut PandocEncodeContext) -> pandoc::Block {
    todo!()
}

fn quote_block_from_pandoc(blocks: Vec<pandoc::Block>, context: &mut PandocDecodeContext) -> Block {
    todo!()
}

fn raw_block_to_pandoc(block: &RawBlock, context: &mut PandocEncodeContext) -> pandoc::Block {
    todo!()
}

fn raw_block_from_pandoc(
    format: pandoc::Format,
    code: String,
    context: &mut PandocDecodeContext,
) -> Block {
    todo!()
}

fn styled_block_to_pandoc(block: &StyledBlock, context: &mut PandocEncodeContext) -> pandoc::Block {
    todo!()
}

fn styled_block_from_pandoc(
    attrs: pandoc::Attr,
    blocks: Vec<pandoc::Block>,
    context: &mut PandocDecodeContext,
) -> Block {
    todo!()
}
