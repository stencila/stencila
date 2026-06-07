//! Block-level conversion between Tiptap JSON nodes and Stencila blocks.
//!
//! Native mappings cover common prose and structural blocks. Stencila blocks
//! with fields that native Tiptap nodes cannot represent are preserved as custom
//! opaque `stencilaBlock` nodes, while unsupported native Tiptap blocks are
//! recorded as losses until explicit mappings are added.

use serde_json::Value;
use stencila_codec::stencila_schema::{
    Block, CodeBlock, Heading, HorizontalAlignment, List, ListItem, ListOrder, Paragraph,
    QuoteBlock, Table, TableCell, TableCellType, TableRow, TableRowType, ThematicBreak,
};

use crate::{
    inlines::{inlines_from_tiptap, inlines_to_tiptap},
    shared::TiptapDecodeContext,
    shared::TiptapEncodeContext,
    tiptap::{
        self, BlockNode, BlockquoteNode, BulletListNode, CodeBlockAttrs, CodeBlockNode,
        HeadingAttrs, HeadingNode, HorizontalRuleNode, InlineNode, ListItemNode, OrderedListAttrs,
        OrderedListNode, ParagraphNode, StencilaAttrs, StencilaBlockNode, StencilaInlineNode,
        TableAttrs, TableCell as TiptapTableCell, TableCellAttrs, TableCellNode, TableHeader,
        TableNode, TableRowNode, TaskItemAttrs, TaskItemNode, TaskListNode, TextNode,
    },
};

/// Decode Tiptap block nodes into Stencila blocks.
pub(super) fn blocks_from_tiptap(
    blocks: Vec<BlockNode>,
    context: &mut TiptapDecodeContext,
) -> Vec<Block> {
    blocks
        .into_iter()
        .map(|block| block_from_tiptap(block, context))
        .collect()
}

/// Encode Stencila blocks into Tiptap block nodes.
pub(super) fn blocks_to_tiptap(
    blocks: &[Block],
    context: &mut TiptapEncodeContext,
) -> Vec<BlockNode> {
    blocks
        .iter()
        .map(|block| block_to_tiptap(block, context))
        .collect()
}

fn block_from_tiptap(block: BlockNode, context: &mut TiptapDecodeContext) -> Block {
    match block {
        BlockNode::Blockquote(blockquote) => quote_block_from_tiptap(blockquote, context),
        BlockNode::BulletList(bullet_list) => bullet_list_from_tiptap(bullet_list, context),
        BlockNode::CodeBlock(code_block) => code_block_from_tiptap(code_block, context),
        BlockNode::Heading(heading) => heading_from_tiptap(heading, context),
        BlockNode::HorizontalRule(horizontal_rule) => thematic_break_from_tiptap(horizontal_rule),
        BlockNode::OrderedList(ordered_list) => ordered_list_from_tiptap(ordered_list, context),
        BlockNode::Paragraph(paragraph) => paragraph_from_tiptap(paragraph, context),
        BlockNode::Table(table) => table_from_tiptap(table, context),
        BlockNode::TaskList(task_list) => task_list_from_tiptap(task_list, context),
        BlockNode::StencilaBlock(stencila_block) => {
            block_from_stencila_block(stencila_block, context)
        }
        BlockNode::Unknown(value) => unknown_block_from_tiptap(value, context),
    }
}

fn block_to_tiptap(block: &Block, context: &mut TiptapEncodeContext) -> BlockNode {
    match block {
        Block::CodeBlock(code_block) => code_block_to_tiptap(code_block),
        Block::Heading(heading) => heading_to_tiptap(heading, context),
        Block::List(list) => list_to_tiptap(list, context),
        Block::Paragraph(paragraph) => paragraph_to_tiptap(paragraph, context),
        Block::QuoteBlock(quote) => quote_block_to_tiptap(quote, context),
        Block::Table(table) => table_to_tiptap(table, context),
        Block::ThematicBreak(..) => thematic_break_to_tiptap(),
        _ => opaque_block_to_tiptap(block, context),
    }
}

fn heading_from_tiptap(heading: HeadingNode, context: &mut TiptapDecodeContext) -> Block {
    let HeadingNode { attrs, content, .. } = heading;
    let level = if matches!(attrs.level, 1..=6) {
        attrs.level.into()
    } else {
        context
            .losses
            .add(format!("Heading.level ({})", attrs.level));
        1
    };

    Block::Heading(Heading {
        level,
        content: inlines_from_tiptap(content, context),
        ..Default::default()
    })
}

fn heading_to_tiptap(heading: &Heading, context: &mut TiptapEncodeContext) -> BlockNode {
    let content = inlines_to_tiptap(&heading.content, context);

    match heading.level {
        0 => BlockNode::Paragraph(ParagraphNode {
            content,
            ..Default::default()
        }),
        1..=6 => BlockNode::Heading(HeadingNode {
            attrs: HeadingAttrs {
                level: heading.level as u8,
            },
            content,
            r#type: Default::default(),
        }),
        level => {
            context.losses.add(format!("Heading.level ({level})"));
            BlockNode::Heading(HeadingNode {
                attrs: HeadingAttrs { level: 6 },
                content,
                r#type: Default::default(),
            })
        }
    }
}

fn paragraph_from_tiptap(paragraph: ParagraphNode, context: &mut TiptapDecodeContext) -> Block {
    Block::Paragraph(Paragraph {
        content: inlines_from_tiptap(paragraph.content, context),
        ..Default::default()
    })
}

fn paragraph_to_tiptap(paragraph: &Paragraph, context: &mut TiptapEncodeContext) -> BlockNode {
    BlockNode::Paragraph(ParagraphNode {
        content: inlines_to_tiptap(&paragraph.content, context),
        ..Default::default()
    })
}

fn quote_block_from_tiptap(blockquote: BlockquoteNode, context: &mut TiptapDecodeContext) -> Block {
    Block::QuoteBlock(QuoteBlock::new(blocks_from_tiptap(
        blockquote.content,
        context,
    )))
}

fn quote_block_to_tiptap(quote: &QuoteBlock, context: &mut TiptapEncodeContext) -> BlockNode {
    BlockNode::Blockquote(BlockquoteNode {
        content: blocks_to_tiptap(&quote.content, context),
        r#type: Default::default(),
    })
}

fn code_block_from_tiptap(code_block: CodeBlockNode, context: &mut TiptapDecodeContext) -> Block {
    let CodeBlockNode { attrs, content, .. } = code_block;
    let mut code = String::new();

    for inline in content {
        code_block_inline_from_tiptap(inline, &mut code, context);
    }

    Block::CodeBlock(CodeBlock {
        id: attrs.id,
        code: code.into(),
        programming_language: attrs.language,
        is_demo: attrs.is_demo,
        ..Default::default()
    })
}

fn code_block_inline_from_tiptap(
    inline: InlineNode,
    code: &mut String,
    context: &mut TiptapDecodeContext,
) {
    match inline {
        InlineNode::Text(text) => code_block_text_from_tiptap(text, code, context),
        InlineNode::StencilaInline(stencila_inline) => {
            code_block_stencila_inline_from_tiptap(stencila_inline, context)
        }
        InlineNode::Unknown(value) => unknown_code_block_inline_from_tiptap(value, context),
    }
}

fn code_block_text_from_tiptap(
    text_node: TextNode,
    code: &mut String,
    context: &mut TiptapDecodeContext,
) {
    if !text_node.marks.is_empty() {
        context.losses.add("CodeBlock.marks");
    }
    code.push_str(&text_node.text);
}

fn code_block_stencila_inline_from_tiptap(
    _stencila_inline: StencilaInlineNode,
    context: &mut TiptapDecodeContext,
) {
    context.losses.add("CodeBlock.stencilaInline");
}

fn unknown_code_block_inline_from_tiptap(value: Value, context: &mut TiptapDecodeContext) {
    context
        .losses
        .add(format!("Unknown ({})", tiptap::value_type(&value)));
}

fn code_block_to_tiptap(code_block: &CodeBlock) -> BlockNode {
    let code = code_block.code.to_string();
    let content = if code.is_empty() {
        Vec::new()
    } else {
        vec![InlineNode::Text(TextNode {
            text: code,
            marks: Vec::new(),
            r#type: Default::default(),
        })]
    };

    BlockNode::CodeBlock(CodeBlockNode {
        attrs: CodeBlockAttrs {
            language: code_block.programming_language.clone(),
            id: code_block.id.clone(),
            is_demo: code_block.is_demo,
        },
        content,
        r#type: Default::default(),
    })
}

fn thematic_break_from_tiptap(_horizontal_rule: HorizontalRuleNode) -> Block {
    Block::ThematicBreak(ThematicBreak::new())
}

fn thematic_break_to_tiptap() -> BlockNode {
    BlockNode::HorizontalRule(HorizontalRuleNode::default())
}

fn bullet_list_from_tiptap(
    bullet_list: BulletListNode,
    context: &mut TiptapDecodeContext,
) -> Block {
    list_from_tiptap(bullet_list.content, ListOrder::Unordered, None, context)
}

fn ordered_list_from_tiptap(
    ordered_list: OrderedListNode,
    context: &mut TiptapDecodeContext,
) -> Block {
    if ordered_list.attrs.r#type.is_some() {
        context.losses.add("OrderedList.type");
    }
    for attr in ordered_list.attrs.extra.keys() {
        context.losses.add(format!("OrderedList.{attr}"));
    }

    list_from_tiptap(
        ordered_list.content,
        ListOrder::Ascending,
        Some(ordered_list.attrs.start),
        context,
    )
}

fn list_from_tiptap(
    items: Vec<ListItemNode>,
    order: ListOrder,
    start: Option<u64>,
    context: &mut TiptapDecodeContext,
) -> Block {
    Block::List(List::new(
        items
            .into_iter()
            .enumerate()
            .map(|(index, item)| list_item_from_tiptap(item, start, index, context))
            .collect(),
        order,
    ))
}

fn list_to_tiptap(list: &List, context: &mut TiptapEncodeContext) -> BlockNode {
    if list.items.iter().any(|item| item.is_checked.is_some()) {
        return task_list_to_tiptap(list, context);
    }

    let content = list
        .items
        .iter()
        .map(|item| ListItemNode {
            content: blocks_to_tiptap(&item.content, context),
            r#type: Default::default(),
        })
        .collect();

    match list.order {
        ListOrder::Unordered => BlockNode::BulletList(BulletListNode {
            content,
            r#type: Default::default(),
        }),
        ListOrder::Ascending => BlockNode::OrderedList(OrderedListNode {
            attrs: OrderedListAttrs {
                start: ordered_list_start(&list.items).unwrap_or(1),
                ..Default::default()
            },
            content,
            r#type: Default::default(),
        }),
        ListOrder::Descending => opaque_block_to_tiptap(&Block::List(list.clone()), context),
    }
}

fn task_list_from_tiptap(task_list: TaskListNode, context: &mut TiptapDecodeContext) -> Block {
    Block::List(List::new(
        task_list
            .content
            .into_iter()
            .map(|item| task_item_from_tiptap(item, context))
            .collect(),
        ListOrder::Unordered,
    ))
}

fn task_list_to_tiptap(list: &List, context: &mut TiptapEncodeContext) -> BlockNode {
    if list.order != ListOrder::Unordered {
        context.losses.add("List.order");
    }

    BlockNode::TaskList(TaskListNode {
        content: list
            .items
            .iter()
            .map(|item| TaskItemNode {
                attrs: TaskItemAttrs {
                    checked: item.is_checked.unwrap_or(false),
                },
                content: blocks_to_tiptap(&item.content, context),
                r#type: Default::default(),
            })
            .collect(),
        r#type: Default::default(),
    })
}

fn list_item_from_tiptap(
    item: ListItemNode,
    start: Option<u64>,
    index: usize,
    context: &mut TiptapDecodeContext,
) -> ListItem {
    let mut item = ListItem::new(blocks_from_tiptap(item.content, context));

    if let Some(start) = start
        && start != 1
    {
        match start
            .checked_add(index as u64)
            .and_then(|position| i64::try_from(position).ok())
        {
            Some(position) => item.position = Some(position),
            None => context.losses.add("ListItem.position"),
        }
    }

    item
}

fn task_item_from_tiptap(item: TaskItemNode, context: &mut TiptapDecodeContext) -> ListItem {
    let mut list_item = ListItem::new(blocks_from_tiptap(item.content, context));
    list_item.is_checked = Some(item.attrs.checked);

    list_item
}

fn ordered_list_start(items: &[ListItem]) -> Option<u64> {
    let mut positions = items.iter().map(|item| item.position);
    let first = positions.next().flatten();

    match first {
        Some(start) if start > 0 => {
            let start = u64::try_from(start).ok()?;
            items
                .iter()
                .enumerate()
                .all(|(index, item)| {
                    item.position
                        .and_then(|position| u64::try_from(position).ok())
                        == start.checked_add(index as u64)
                })
                .then_some(start)
        }
        Some(..) => None,
        None => items
            .iter()
            .all(|item| item.position.is_none())
            .then_some(1),
    }
}

fn table_from_tiptap(table: TableNode, context: &mut TiptapDecodeContext) -> Block {
    let TableNode { attrs, content, .. } = table;
    let mut table = Table::new(
        content
            .into_iter()
            .map(|row| table_row_from_tiptap(row, context))
            .collect(),
    );

    table.id = attrs.id;
    table.label = attrs.label;
    table.label_automatically = attrs.label_automatically;
    table.caption = attrs.caption;
    table.notes = attrs.notes;

    Block::Table(table)
}

fn table_to_tiptap(table: &Table, context: &mut TiptapEncodeContext) -> BlockNode {
    BlockNode::Table(TableNode {
        attrs: TableAttrs {
            id: table.id.clone(),
            label: table.label.clone(),
            label_automatically: table.label_automatically,
            caption: table.caption.clone(),
            notes: table.notes.clone(),
        },
        content: table
            .rows
            .iter()
            .map(|row| table_row_to_tiptap(row, context))
            .collect(),
        r#type: Default::default(),
    })
}

fn table_row_from_tiptap(row: TableRowNode, context: &mut TiptapDecodeContext) -> TableRow {
    let cells: Vec<TableCell> = row
        .content
        .into_iter()
        .map(|cell| table_cell_from_tiptap(cell, context))
        .collect();
    let row_type = if !cells.is_empty()
        && cells
            .iter()
            .all(|cell| matches!(cell.cell_type, Some(TableCellType::HeaderCell)))
    {
        Some(TableRowType::HeaderRow)
    } else {
        None
    };

    TableRow {
        cells,
        row_type,
        ..Default::default()
    }
}

fn table_row_to_tiptap(row: &TableRow, context: &mut TiptapEncodeContext) -> TableRowNode {
    let is_header_row = matches!(row.row_type, Some(TableRowType::HeaderRow));

    TableRowNode {
        content: row
            .cells
            .iter()
            .map(|cell| table_cell_to_tiptap(cell, is_header_row, context))
            .collect(),
        r#type: Default::default(),
    }
}

fn table_cell_from_tiptap(cell: TableCellNode, context: &mut TiptapDecodeContext) -> TableCell {
    match cell {
        TableCellNode::TableCell(cell) => table_data_cell_from_tiptap(cell, context),
        TableCellNode::TableHeader(header) => table_header_from_tiptap(header, context),
        TableCellNode::Unknown(value) => unknown_table_cell_from_tiptap(value, context),
    }
}

fn table_data_cell_from_tiptap(
    cell: TiptapTableCell,
    context: &mut TiptapDecodeContext,
) -> TableCell {
    table_cell_from_parts(cell.attrs, cell.content, None, context)
}

fn table_header_from_tiptap(header: TableHeader, context: &mut TiptapDecodeContext) -> TableCell {
    table_cell_from_parts(
        header.attrs,
        header.content,
        Some(TableCellType::HeaderCell),
        context,
    )
}

fn unknown_table_cell_from_tiptap(value: Value, context: &mut TiptapDecodeContext) -> TableCell {
    context
        .losses
        .add(format!("Unknown ({})", tiptap::value_type(&value)));
    TableCell::new(vec![Block::Paragraph(Paragraph::default())])
}

fn table_cell_to_tiptap(
    cell: &TableCell,
    is_header_row: bool,
    context: &mut TiptapEncodeContext,
) -> TableCellNode {
    let attrs = TableCellAttrs {
        colspan: cell.options.column_span.unwrap_or(1) as u64,
        rowspan: cell.options.row_span.unwrap_or(1) as u64,
        align: horizontal_alignment_to_tiptap(cell.options.horizontal_alignment),
        ..Default::default()
    };
    let content = blocks_to_tiptap(&cell.content, context);

    if is_header_row || matches!(cell.cell_type, Some(TableCellType::HeaderCell)) {
        TableCellNode::TableHeader(TableHeader {
            attrs,
            content,
            r#type: Default::default(),
        })
    } else {
        TableCellNode::TableCell(TiptapTableCell {
            attrs,
            content,
            r#type: Default::default(),
        })
    }
}

fn table_cell_from_parts(
    attrs: TableCellAttrs,
    content: Vec<BlockNode>,
    cell_type: Option<TableCellType>,
    context: &mut TiptapDecodeContext,
) -> TableCell {
    if attrs.colwidth.is_some() {
        context.losses.add("TableCell.colwidth");
    }
    for attr in attrs.extra.keys() {
        context.losses.add(format!("TableCell.{attr}"));
    }

    let mut cell = TableCell::new(blocks_from_tiptap(content, context));
    cell.cell_type = cell_type;
    cell.options.horizontal_alignment = horizontal_alignment_from_tiptap(attrs.align, context);

    if attrs.colspan > 1 {
        match i64::try_from(attrs.colspan) {
            Ok(span) => cell.options.column_span = Some(span),
            Err(..) => context.losses.add("TableCell.colspan"),
        }
    }
    if attrs.rowspan > 1 {
        match i64::try_from(attrs.rowspan) {
            Ok(span) => cell.options.row_span = Some(span),
            Err(..) => context.losses.add("TableCell.rowspan"),
        }
    }

    cell
}

fn horizontal_alignment_from_tiptap(
    align: Option<String>,
    context: &mut TiptapDecodeContext,
) -> Option<HorizontalAlignment> {
    match align.as_deref() {
        Some("left") => Some(HorizontalAlignment::AlignLeft),
        Some("right") => Some(HorizontalAlignment::AlignRight),
        Some("center") => Some(HorizontalAlignment::AlignCenter),
        Some(..) => {
            context.losses.add("TableCell.align");
            None
        }
        None => None,
    }
}

fn horizontal_alignment_to_tiptap(align: Option<HorizontalAlignment>) -> Option<String> {
    match align {
        Some(HorizontalAlignment::AlignLeft) => Some("left".into()),
        Some(HorizontalAlignment::AlignRight) => Some("right".into()),
        Some(HorizontalAlignment::AlignCenter) => Some("center".into()),
        Some(HorizontalAlignment::AlignJustify | HorizontalAlignment::AlignCharacter) | None => {
            None
        }
    }
}

fn opaque_block_to_tiptap(block: &Block, context: &mut TiptapEncodeContext) -> BlockNode {
    match serde_json::to_value(block) {
        Ok(node) => BlockNode::StencilaBlock(StencilaBlockNode {
            attrs: StencilaAttrs {
                node_type: block.node_type().to_string(),
                node,
            },
            r#type: Default::default(),
        }),
        Err(error) => {
            context
                .losses
                .add(format!("{}: {error}", block.node_type()));
            BlockNode::Paragraph(ParagraphNode::default())
        }
    }
}

fn block_from_stencila_block(block: StencilaBlockNode, context: &mut TiptapDecodeContext) -> Block {
    block_from_stencila_attrs(block.attrs, context)
}

fn unknown_block_from_tiptap(value: Value, context: &mut TiptapDecodeContext) -> Block {
    context
        .losses
        .add(format!("Unknown ({})", tiptap::value_type(&value)));
    Block::Paragraph(Paragraph::default())
}

fn block_from_stencila_attrs(attrs: StencilaAttrs, context: &mut TiptapDecodeContext) -> Block {
    match serde_json::from_value::<Block>(attrs.node) {
        Ok(block) => {
            let node_type = block.node_type().to_string();
            if node_type != attrs.node_type {
                context.losses.add(format!(
                    "StencilaBlock.nodeType (expected {}, got {node_type})",
                    attrs.node_type
                ));
            }
            block
        }
        Err(error) => {
            context.losses.add(format!("{}: {error}", attrs.node_type));
            Block::Paragraph(Paragraph::default())
        }
    }
}

#[allow(dead_code)]
fn _assert_value_preserves_order(_: Value) {}
