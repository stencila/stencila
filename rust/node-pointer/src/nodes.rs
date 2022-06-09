use common::eyre::Result;
use node_address::Address;
use node_dispatch::dispatch_node;
use stencila_schema::*;

use crate::{Pointable, Pointer, PointerMut, Visitor, VisitorMut};

impl Pointable for Node {
    /// Resolve an [`Address`] into a node [`Pointer`].
    ///
    /// `Node` is one of the pointer variants so return a `Pointer::Node` if
    /// the address is empty. Otherwise dispatch to variant.
    fn resolve(&self, address: &mut Address) -> Result<Pointer> {
        match address.is_empty() {
            true => Ok(Pointer::Node(self)),
            false => dispatch_node!(self, Ok(Pointer::None), resolve, address),
        }
    }
    fn resolve_mut(&mut self, address: &mut Address) -> Result<PointerMut> {
        match address.is_empty() {
            true => Ok(PointerMut::Node(self)),
            false => dispatch_node!(self, Ok(PointerMut::None), resolve_mut, address),
        }
    }

    /// Find a node based on its `id` and return a [`Pointer`] to it.
    ///
    /// Dispatch to variant and if it returns `Pointer::Some` then rewrite to `Pointer::Node`
    fn find(&self, id: &str) -> Pointer {
        match dispatch_node!(self, Pointer::None, find, id) {
            Pointer::Some => Pointer::Node(self),
            _ => Pointer::None,
        }
    }
    fn find_mut(&mut self, id: &str) -> PointerMut {
        match dispatch_node!(self, PointerMut::None, find_mut, id) {
            PointerMut::Some => PointerMut::Node(self),
            _ => PointerMut::None,
        }
    }

    /// Walk over a node with a [`Visitor`]
    ///
    /// `Node` is one of the visited types so call `visit_block` and,
    /// if it returns `true`, continue walk over variant.
    fn walk(&self, address: Address, visitor: &mut impl Visitor) {
        let cont = visitor.visit_node(&address, self);
        if cont {
            dispatch_node!(self, (), walk, address, visitor)
        }
    }
    fn walk_mut(&mut self, address: Address, visitor: &mut impl VisitorMut) {
        let cont = visitor.visit_node_mut(&address, self);
        if cont {
            dispatch_node!(self, (), walk_mut, address, visitor)
        }
    }
}
