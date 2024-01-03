//! Functions for transforming nodes between types
//!
//! These functions implement transformations for `Vec` and other types
//! where, due to Rust's "orphan rule", we can not implement the `From`
//! trait. If possible, prefer to implement `From`, or `Into` in the `implem/*.rs`
//! file for the type, rather than add another function here.

use crate::{Block, CodeBlock, CodeChunk, CodeExpression, CodeInline, Inline, Node, NodeType};

/// Transform a vector of [`Block`]s into a vector of [`Inline`]s
pub fn blocks_to_inlines(blocks: Vec<Block>) -> Vec<Inline> {
    blocks
        .into_iter()
        .flat_map(|block| -> Vec<Inline> { block.into() })
        .collect()
}

/// Transform a vector of [`Inline`]s into a vector of [`Block`]s
pub fn inlines_to_blocks(inlines: Vec<Inline>) -> Vec<Block> {
    vec![inlines.into()]
}

/// Transform a vector of [`Inline`]s into a vector of [`Node`]s
pub fn inlines_to_nodes(inlines: Vec<Inline>) -> Vec<Node> {
    inlines
        .into_iter()
        .map(|inline| -> Node { inline.into() })
        .collect()
}

/// Transform a vector of [`Block`]s into a vector of [`Node`]s
pub fn blocks_to_nodes(blocks: Vec<Block>) -> Vec<Node> {
    blocks
        .into_iter()
        .map(|block| -> Node { block.into() })
        .collect()
}

/// Transform between inline types if possible
pub fn transform_inline(inline: Inline, node_type: NodeType) -> Inline {
    match inline {
        Inline::CodeInline(inner) => match node_type {
            NodeType::CodeExpression => Inline::CodeExpression(CodeExpression {
                code: inner.code,
                programming_language: inner.programming_language,
                ..Default::default()
            }),
            _ => Inline::CodeInline(inner),
        },
        Inline::CodeExpression(inner) => match node_type {
            NodeType::CodeInline => Inline::CodeInline(CodeInline {
                code: inner.code,
                programming_language: inner.programming_language,
                ..Default::default()
            }),
            _ => Inline::CodeExpression(inner),
        },
        _ => inline,
    }
}

/// Transform between block types if possible
pub fn transform_block(block: Block, node_type: NodeType) -> Block {
    match block {
        Block::CodeBlock(inner) => match node_type {
            NodeType::CodeChunk => Block::CodeChunk(CodeChunk {
                code: inner.code,
                programming_language: inner.programming_language,
                ..Default::default()
            }),
            _ => Block::CodeBlock(inner),
        },
        Block::CodeChunk(inner) => match node_type {
            NodeType::CodeBlock => Block::CodeBlock(CodeBlock {
                code: inner.code,
                programming_language: inner.programming_language,
                ..Default::default()
            }),
            _ => Block::CodeChunk(inner),
        },
        _ => block,
    }
}
