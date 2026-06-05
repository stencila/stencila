//! Block-level conversion between Tiptap JSON nodes and Stencila blocks.
//!
//! The native block mapping is intentionally narrow for the initial codec
//! implementation. Unsupported Stencila blocks are preserved as custom opaque
//! `stencilaBlock` nodes, while unsupported native Tiptap blocks are recorded as
//! losses until explicit mappings are added.

use serde_json::Value;
use stencila_codec::stencila_schema::{Block, Heading, Paragraph};

use crate::{
    inlines::{inlines_from_tiptap, inlines_to_tiptap},
    shared::TiptapDecodeContext,
    shared::TiptapEncodeContext,
    tiptap::{
        self, BlockNode, HeadingAttrs, HeadingNode, ParagraphNode, StencilaAttrs, StencilaBlockNode,
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
        BlockNode::Heading(HeadingNode { attrs, content, .. }) => {
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

        BlockNode::Paragraph(ParagraphNode { content, .. }) => Block::Paragraph(Paragraph {
            content: inlines_from_tiptap(content, context),
            ..Default::default()
        }),

        BlockNode::StencilaBlock(StencilaBlockNode { attrs, .. }) => {
            block_from_stencila_attrs(attrs, context)
        }

        BlockNode::Unknown(value) => {
            context
                .losses
                .add(format!("Unknown ({})", tiptap::value_type(&value)));
            Block::Paragraph(Paragraph::default())
        }
    }
}

fn block_to_tiptap(block: &Block, context: &mut TiptapEncodeContext) -> BlockNode {
    match block {
        Block::Heading(heading) => heading_to_tiptap(heading, context),
        Block::Paragraph(paragraph) => BlockNode::Paragraph(ParagraphNode {
            content: inlines_to_tiptap(&paragraph.content, context),
            ..Default::default()
        }),
        _ => opaque_block_to_tiptap(block, context),
    }
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
