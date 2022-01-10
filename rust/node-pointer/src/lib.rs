use eyre::{bail, Result};
use node_address::Address;
use node_transform::Transform;
use stencila_schema::{BlockContent, CreativeWorkTypes, InlineContent, Node};

/// Resolve a node [`Address`] or id into a node [`Pointer`]
///
/// The intent is to use the `id` as a fallback if the `address` can not be resolved.
/// However the borrow checker isn't allowing that implementation
pub fn resolve<Type: Pointable>(
    node: &mut Type,
    address: Option<Address>,
    id: Option<String>,
) -> Result<Pointer> {
    if let Some(mut address) = address {
        let pointer = node.resolve(&mut address)?;
        match pointer {
            Pointer::None => bail!("Unable to find node with address `{}`", address),
            _ => Ok(pointer),
        }
    } else if let Some(id) = id {
        let pointer = node.find(&id);
        match pointer {
            Pointer::None => bail!("Unable to find node with id `{}`", id),
            _ => Ok(pointer),
        }
    } else {
        bail!("One of address or node id must be supplied to resolve a node")
    }
}

/// A pointer to a node within the tree of another root node
#[derive(Debug)]
pub enum Pointer<'lt> {
    None,
    Some,
    Inline(&'lt mut InlineContent),
    Block(&'lt mut BlockContent),
    Work(&'lt mut CreativeWorkTypes),
    Node(&'lt mut Node),
}

impl<'lt> Pointer<'lt> {
    pub fn as_inline(&self) -> Option<&InlineContent> {
        match self {
            Pointer::Inline(inline) => Some(inline),
            _ => None,
        }
    }

    pub fn as_inline_mut(&mut self) -> Option<&mut InlineContent> {
        match self {
            Pointer::Inline(inline) => Some(inline),
            _ => None,
        }
    }

    pub fn as_block(&self) -> Option<&BlockContent> {
        match self {
            Pointer::Block(block) => Some(block),
            _ => None,
        }
    }

    pub fn as_block_mut(&mut self) -> Option<&mut BlockContent> {
        match self {
            Pointer::Block(block) => Some(block),
            _ => None,
        }
    }

    pub fn as_work(&self) -> Option<&CreativeWorkTypes> {
        match self {
            Pointer::Work(work) => Some(work),
            _ => None,
        }
    }

    pub fn as_work_mut(&mut self) -> Option<&mut CreativeWorkTypes> {
        match self {
            Pointer::Work(work) => Some(work),
            _ => None,
        }
    }

    pub fn as_node(&self) -> Option<&Node> {
        match self {
            Pointer::Node(node) => Some(node),
            _ => None,
        }
    }

    pub fn as_node_mut(&mut self) -> Option<&mut Node> {
        match self {
            Pointer::Node(node) => Some(node),
            _ => None,
        }
    }

    pub fn to_node(&self) -> Result<Node> {
        Ok(match self {
            Pointer::Inline(node) => node.to_node(),
            Pointer::Block(node) => node.to_node(),
            Pointer::Node(node) => node.to_node(),
            _ => bail!("Invalid node pointer: {:?}", self),
        })
    }
}

/// A trait for document node types that are able to be pointed to
pub trait Pointable {
    /// Resolve an [`Address`] into a node [`Pointer`].
    ///
    /// If the address in empty, and the node is represented in one of the variants of [`Pointer`]
    /// (at the time of writing `Node`, `BlockContent` and `InlineContent`), then it should return
    /// a pointer to itself. Otherwise it should return an "unpointable" type error.
    ///
    /// If the address is not empty then it should be passed on to any child nodes.
    ///
    /// If the address is invalid for the type (e.g. a non-empty address for a leaf node, a name
    /// slot used for a vector) then implementations should return an error.
    ///
    /// The default implementation is only suitable for leaf nodes that are not pointable.
    fn resolve(&mut self, address: &mut Address) -> Result<Pointer> {
        match address.is_empty() {
            true => bail!("Address resolves to a node that can not be pointed to"),
            false => bail!("Address is not empty; does resolve() needs to be overridden?"),
        }
    }

    /// Find a node based on its `id` and return a [`Pointer`] to it.
    ///
    /// This is less efficient than `resolve` (given that it must visit all nodes until one is
    /// found with a matching id). However, it may be necessary to use when an [`Address`] is not available.
    ///
    /// If the node has a matching `id` property then it should return `Pointer::Some` which indicates
    /// that the `id` is matched . This allows the parent type e.g `InlineContent` to populate the
    /// "useable" pointer variants e.g. `Pointer::InlineContent`.
    ///
    /// Otherwise, if the node has children it should call `find` on them and return `Pointer::None` if
    /// no children have a matching `id`.
    ///
    /// The default implementation is only suitable for leaf nodes that do not have an `id` property.
    fn find(&mut self, _id: &str) -> Pointer {
        Pointer::None
    }
}

#[macro_use]
mod generics;

mod blocks;
mod data;
mod inlines;
mod nodes;
mod primitives;
mod works;
