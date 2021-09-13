use stencila_schema::{BlockContent, InlineContent, Node};

pub trait Transform {
    /// Transform a value to a `InlineContent` variant
    fn to_inline(&self) -> InlineContent;

    /// Transform a value to a vector of `InlineContent` variants
    fn to_inlines(&self) -> Vec<InlineContent> {
        vec![self.to_inline()]
    }

    /// Transform a value to a `BlockContent` variant
    fn to_block(&self) -> BlockContent;

    /// Transform a value to a vector of `BlockContent` variants
    fn to_blocks(&self) -> Vec<BlockContent> {
        vec![self.to_block()]
    }

    /// Transform a value to a `Node` variant
    fn to_node(&self) -> Node;

    /// Transform a value to a vector of `Node` variants
    fn to_nodes(&self) -> Vec<Node> {
        vec![self.to_node()]
    }
}

mod blocks;
mod inlines;
mod nodes;
