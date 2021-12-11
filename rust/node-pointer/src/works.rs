use crate::{Pointable, Pointer};
use eyre::{bail, Result};
use node_address::Address;
use node_dispatch::dispatch_work;
use stencila_schema::*;

impl Pointable for CreativeWorkTypes {
    /// Resolve an [`Address`] into a node [`Pointer`].
    ///
    /// `CreativeWorkTypes` is one of the pointer variants so return a `Pointer::Work` if
    /// the address is empty. Otherwise dispatch to variant.
    fn resolve(&mut self, address: &mut Address) -> Result<Pointer> {
        match address.is_empty() {
            true => Ok(Pointer::Work(self)),
            false => dispatch_work!(self, resolve, address),
        }
    }

    /// Find a node based on its `id` and return a [`Pointer`] to it.
    ///
    /// Dispatch to variant and if it returns `Pointer::Some` then rewrite to `Pointer::Work`
    fn find(&mut self, id: &str) -> Pointer {
        match dispatch_work!(self, find, id) {
            Pointer::Some => Pointer::Work(self),
            _ => Pointer::None,
        }
    }
}

// Implementations for `CreativeWork` structs. Presently none of the descendants of these
// works are made pointable but they may be done in the future.

pointable_struct!(Article);
pointable_struct!(AudioObject);
pointable_struct!(Claim);
pointable_struct!(Collection);
pointable_struct!(Comment);
pointable_struct!(CreativeWork);
pointable_struct!(Datatable);
pointable_struct!(Figure);
pointable_struct!(ImageObject);
pointable_struct!(MediaObject);
pointable_struct!(Periodical);
pointable_struct!(PublicationIssue);
pointable_struct!(PublicationVolume);
pointable_struct!(Review);
pointable_struct!(SoftwareApplication);
pointable_struct!(SoftwareSourceCode);
pointable_struct!(Table);
pointable_struct!(VideoObject);
