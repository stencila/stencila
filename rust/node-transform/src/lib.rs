//! Transform a Stencila document node to another node type
//!
//! It can be useful to transform document nodes between types.
//! For example, Stencila documents use a content model where nodes
//! are categorized into `BlockContent` and `InlineContent`.
//! To ensure that content conforms to the Stencila schema, it may be necessary
//! to transform between inline content and block content types and vice versa.
//! It is also sometimes necessary to transform from the general `Node` type to
//! the more specialized `InlineContent`, `BlockContent`, or `CreativeWorkTypes`.

use stencila_schema::{BlockContent, InlineContent, Node};

pub trait Transform {
    /// Transform a value to a `InlineContent` variant
    fn to_inline(&self) -> InlineContent;

    /// Transform a value to a vector of `InlineContent` variants
    ///
    /// The default implementation simply calls `to_inline` and wraps
    /// the result in a vector.
    fn to_inlines(&self) -> Vec<InlineContent> {
        vec![self.to_inline()]
    }

    /// Transform a value to a `BlockContent` variant
    fn to_block(&self) -> BlockContent;

    /// Transform a value to a vector of `BlockContent` variants
    ///
    /// The default implementation simply calls `to_block` and wraps
    /// the result in a vector.
    fn to_blocks(&self) -> Vec<BlockContent> {
        vec![self.to_block()]
    }

    /// Transform a value to a `Node` variant
    fn to_node(&self) -> Node;

    /// Transform a value to a vector of `Node` variants
    ///
    /// The default implementation simply calls `to_node` and wraps
    /// the result in a vector.
    fn to_nodes(&self) -> Vec<Node> {
        vec![self.to_node()]
    }
}

mod blocks;
mod inlines;
mod nodes;
