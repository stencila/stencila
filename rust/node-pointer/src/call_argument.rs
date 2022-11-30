use common::eyre::{bail, Result};
use node_address::{invalid_address, Address};
use stencila_schema::CallArgument;

use crate::{Pointable, Pointer, PointerMut};

impl Pointable for CallArgument {
    fn resolve(&self, address: &mut Address) -> Result<Pointer> {
        match address.is_empty() {
            true => Ok(Pointer::CallArgument(self)),
            false => bail!(invalid_address::<Self>("properties are not pointable")),
        }
    }
    fn resolve_mut(&mut self, address: &mut Address) -> Result<PointerMut> {
        match address.is_empty() {
            true => Ok(PointerMut::CallArgument(self)),
            false => bail!(invalid_address::<Self>("properties are not pointable")),
        }
    }

    fn is(&self, id: &str) -> bool {
        if let Some(my_id) = self.id.as_deref() {
            if id == my_id {
                return true;
            }
        }
        false
    }

    fn find(&self, id: &str) -> Pointer {
        match self.is(id) {
            true => Pointer::CallArgument(self),
            false => Pointer::None,
        }
    }
    fn find_mut(&mut self, id: &str) -> PointerMut {
        match self.is(id) {
            true => PointerMut::CallArgument(self),
            false => PointerMut::None,
        }
    }
}
