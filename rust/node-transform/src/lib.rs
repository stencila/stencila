//! Transform a Stencila document node to another node type
//!
//! This crate provides a `Transform` trait and implements it for:
//!
//! - [`InlineContent`] and `Vec<InlineContent>`
//! - [`BlockContent`] and `Vec<BlockContent>`
//! - [`Node`] and `Vec<Node>`
//!
//! The trait has methods that allow for transformation between these types:
//!
//! - `to_inline`, `to_inlines`: transform to a [`InlineContent`] or `Vec<InlineContent>`
//! - `to_block`, `to_blocks`: transform to a [`BlockContent`] or `Vec<BlockContent>`
//! - `to_node`, `to_nodes`: transform to a [`Node`] or `Vec<Node>`
//!
//! In addition, the `to_static_inline` and `to_static_block` reduce dynamic, executable
//! document nodes to their static content. This is an intentionally lossy transformation.
//! For example, a `CodeExpresssion`, `to_static_inline` will only represent the `output`
//! of the node.

use stencila_schema::{BlockContent, InlineContent, Node};

pub trait Transform {
    /// Is a node an `InlineContent` variant e.g. a `Node:Strong`
    fn is_inline(&self) -> bool {
        false
    }

    /// Transform a value to a `InlineContent` variant
    fn to_inline(&self) -> InlineContent;

    /// Transform a value to a vector of `InlineContent` variants
    ///
    /// The default implementation simply calls `to_inline` and wraps
    /// the result in a vector.
    fn to_inlines(&self) -> Vec<InlineContent> {
        vec![self.to_inline()]
    }

    /// Transform a value to a static `InlineContent` variants

    /// Is a node a `BlockContent` variant e.g. a `Node:CodeChunk`
    fn is_block(&self) -> bool {
        false
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

    /// Transform a value to a vector of static inline content
    fn to_static_inlines(&self) -> Vec<InlineContent> {
        vec![self.to_inline()]
    }

    /// Transform a value to a vector of static block content
    fn to_static_blocks(&self) -> Vec<BlockContent> {
        vec![self.to_block()]
    }

    /// Transform a value to a vector of static nodes
    fn to_static_nodes(&self) -> Vec<Node> {
        vec![self.to_node()]
    }
}

mod blocks;
mod inlines;
mod nodes;
mod primitives;
