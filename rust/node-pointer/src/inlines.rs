use crate::{Pointable, Pointer, PointerMut};
use eyre::{bail, Result};
use node_address::Address;
use node_dispatch::dispatch_inline;
use stencila_schema::*;

impl Pointable for InlineContent {
    /// Resolve an [`Address`] into a node [`Pointer`].
    ///
    /// `InlineContent` is one of the pointer variants so return a `Pointer::Inline` if
    /// the address is empty. Otherwise dispatch to variant.
    fn resolve(&self, address: &mut Address) -> Result<Pointer> {
        match address.is_empty() {
            true => Ok(Pointer::Inline(self)),
            false => dispatch_inline!(self, resolve, address),
        }
    }
    fn resolve_mut(&mut self, address: &mut Address) -> Result<PointerMut> {
        match address.is_empty() {
            true => Ok(PointerMut::Inline(self)),
            false => dispatch_inline!(self, resolve_mut, address),
        }
    }

    /// Find a node based on its `id` and return a [`Pointer`] to it.
    ///
    /// Dispatch to variant and if it returns `Pointer::Some` then rewrite to `Pointer::Inline`
    fn find(&self, id: &str) -> Pointer {
        let pointer = dispatch_inline!(self, find, id);
        match pointer {
            Pointer::Some => Pointer::Inline(self),
            _ => Pointer::None,
        }
    }
    fn find_mut(&mut self, id: &str) -> PointerMut {
        let pointer = dispatch_inline!(self, find_mut, id);
        match pointer {
            PointerMut::Some => PointerMut::Inline(self),
            _ => PointerMut::None,
        }
    }
}

// Implementations for `InlineContent` structs (usually only properties that are content or `Node`s)

pointable_struct!(AudioObjectSimple);
pointable_struct!(Cite, content);
pointable_struct!(CiteGroup, items);
pointable_struct!(CodeExpression);
pointable_struct!(CodeFragment);
pointable_struct!(Delete, content);
pointable_struct!(Emphasis, content);
pointable_struct!(ImageObjectSimple);
pointable_struct!(Link, content);
pointable_struct!(MathFragment);
pointable_struct!(NontextualAnnotation, content);
pointable_struct!(Note, content);
pointable_struct!(Parameter);
pointable_struct!(Quote, content);
pointable_struct!(Strong, content);
pointable_struct!(Subscript, content);
pointable_struct!(Superscript, content);
pointable_struct!(VideoObjectSimple);
