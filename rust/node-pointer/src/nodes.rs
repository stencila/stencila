use crate::{Pointable, Pointer};
use eyre::Result;
use node_address::Address;
use node_dispatch::dispatch_node;
use stencila_schema::*;

impl Pointable for Node {
    /// Resolve an [`Address`] into a node [`Pointer`].
    ///
    /// `Node` is one of the pointer variants so return a `Pointer::Node` if
    /// the address is empty. Otherwise dispatch to variant.
    fn resolve(&mut self, address: &mut Address) -> Result<Pointer> {
        match address.is_empty() {
            true => Ok(Pointer::Node(self)),
            false => dispatch_node!(self, Ok(Pointer::None), resolve, address),
        }
    }

    /// Find a node based on its `id` and return a [`Pointer`] to it.
    ///
    /// Dispatch to variant and if it returns `Pointer::Some` then rewrite to `Pointer::Node`
    fn find(&mut self, id: &str) -> Pointer {
        match dispatch_node!(self, Pointer::None, find, id) {
            Pointer::Some => Pointer::Node(self),
            _ => Pointer::None,
        }
    }
}
