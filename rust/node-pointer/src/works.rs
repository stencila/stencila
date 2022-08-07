use common::eyre::{bail, Result};
use node_address::Address;
use node_dispatch::dispatch_work;
use stencila_schema::*;

use crate::{Pointable, Pointer, PointerMut, Visitor, VisitorMut};

impl Pointable for CreativeWorkTypes {
    /// Resolve an [`Address`] into a node [`Pointer`].
    ///
    /// `CreativeWorkTypes` is one of the pointer variants so return a `Pointer::Work` if
    /// the address is empty. Otherwise dispatch to variant.
    fn resolve(&self, address: &mut Address) -> Result<Pointer> {
        match address.is_empty() {
            true => Ok(Pointer::Work(self)),
            false => dispatch_work!(self, resolve, address),
        }
    }
    fn resolve_mut(&mut self, address: &mut Address) -> Result<PointerMut> {
        match address.is_empty() {
            true => Ok(PointerMut::Work(self)),
            false => dispatch_work!(self, resolve_mut, address),
        }
    }

    /// Find a node based on its `id` and return a [`Pointer`] to it.
    ///
    /// Dispatch to variant and if it returns `Pointer::Some` then rewrite to `Pointer::Work`
    fn find(&self, id: &str) -> Pointer {
        match dispatch_work!(self, find, id) {
            Pointer::Some => Pointer::Work(self),
            _ => Pointer::None,
        }
    }
    fn find_mut(&mut self, id: &str) -> PointerMut {
        match dispatch_work!(self, find_mut, id) {
            PointerMut::Some => PointerMut::Work(self),
            _ => PointerMut::None,
        }
    }

    /// Walk over a node with a [`Visitor`]
    ///
    /// `CreativeWorkTypes` is one of the visited types so call `visit_work` and,
    /// if it returns `true`, continue walk over variant.
    fn walk(&self, address: Address, visitor: &mut impl Visitor) {
        let cont = visitor.visit_work(&address, self);
        if cont {
            dispatch_work!(self, walk, address, visitor)
        }
    }
    fn walk_mut(&mut self, address: Address, visitor: &mut impl VisitorMut) {
        let cont = visitor.visit_work_mut(&address, self);
        if cont {
            dispatch_work!(self, walk_mut, address, visitor)
        }
    }
}

// Implementations for `CreativeWork` structs. For descendants of these
// works to be pointable they must be within one of the listed properties e.g. `Article.content`.

pointable_struct!(Article, content);
pointable_struct!(AudioObject);
pointable_struct!(Claim, content);
pointable_struct!(Collection, parts);
pointable_struct!(Comment);
pointable_struct!(CreativeWork, content);
pointable_struct!(Figure);
pointable_struct!(File, path);
pointable_struct!(ImageObject);
pointable_struct!(MediaObject);
pointable_struct!(Periodical);
pointable_struct!(PublicationIssue);
pointable_struct!(PublicationVolume);
pointable_struct!(Review);
pointable_struct!(SoftwareApplication);
pointable_struct!(SoftwareSourceCode);
pointable_struct!(Table, caption, rows);
pointable_struct!(VideoObject);

pointable_variants!(
    CreativeWorkContent,
    CreativeWorkContent::String,
    CreativeWorkContent::VecNode
);
