use crate::{Pointable, Pointer};
use eyre::{bail, Result};
use node_address::{invalid_slot_index, invalid_slot_variant, Address, Slot};
use std::ops::DerefMut;

/// Generate a `impl Pointable` for a `struct`
macro_rules! pointable_struct {
    ($type:ty $(, $field:ident )*) => {
        impl Pointable for $type {
            /// Resolve an [`Address`] into a node [`Pointer`].
            ///
            /// Delegate to child fields, erroring if address is invalid.
            fn resolve(&mut self, address: &mut Address) -> Result<Pointer> {
                use node_address::{invalid_address, invalid_slot_variant, invalid_slot_name, Slot};

                match address.pop_front() {
                    Some(Slot::Name(name)) => match name.as_str() {
                        $(
                            stringify!($field) => self.$field.resolve(address),
                        )*
                        _ => bail!(invalid_slot_name::<Self>(&name)),
                    },
                    Some(slot) => bail!(invalid_slot_variant::<Self>(slot)),
                    None => bail!(invalid_address::<Self>("address is empty")),
                }
            }

            /// Find a node based on its `id` and return a [`Pointer`] to it.
            ///
            /// If the `id` matches `self.id` then return `Pointer::Some`. Otherwise, delegate to
            /// child fields, returning `Pointer::None` if not found there.
            fn find(&mut self, id: &str) -> Pointer {
                if let Some(my_id) = self.id.as_ref() {
                    if id == **my_id {
                        return Pointer::Some
                    }
                }

                $(
                    let pointer = self.$field.find(id);
                    match pointer {
                        Pointer::None => (),
                        _ => return pointer
                    }
                )*

                Pointer::None
            }
        }
    };
}

/// Generate a `impl Pointable` for an `enum` having variants of different types.
macro_rules! pointable_variants {
    ($type:ty $(, $variant:path )*) => {
        impl Pointable for $type {
            fn resolve(&mut self, address: &mut Address) -> Result<Pointer> {
                match self {
                    $(
                        $variant(me) => me.resolve(address),
                    )*
                    #[allow(unreachable_patterns)]
                    _ => bail!("Unhandled variant `{}` of `{}`", self.as_ref(), std::any::type_name::<Self>())
                }
            }

            fn find(&mut self, id: &str) -> Pointer {
                match self {
                    $(
                        $variant(me) => me.find(id),
                    )*
                    #[allow(unreachable_patterns)]
                    _ => Pointer::None
                }
            }
        }
    };
}

impl<Type: Pointable> Pointable for Option<Type> {
    /// Resolve an [`Address`] into a node [`Pointer`].
    ///
    /// Delegate to value, if any.
    fn resolve(&mut self, address: &mut Address) -> Result<Pointer> {
        match self {
            Some(me) => me.resolve(address),
            None => Ok(Pointer::None),
        }
    }

    /// Find a node based on its `id` and return a [`Pointer`] to it.
    ///
    /// Delegate to value, if any.
    fn find(&mut self, id: &str) -> Pointer {
        match self {
            Some(me) => me.find(id),
            None => Pointer::None,
        }
    }
}

impl<Type: Pointable> Pointable for Box<Type> {
    /// Resolve an [`Address`] into a node [`Pointer`].
    ///
    /// Delegate to boxed value.
    fn resolve(&mut self, address: &mut Address) -> Result<Pointer> {
        self.deref_mut().resolve(address)
    }

    /// Find a node based on its `id` and return a [`Pointer`] to it.
    ///
    /// Delegate to boxed value.
    fn find(&mut self, id: &str) -> Pointer {
        self.deref_mut().find(id)
    }
}

impl<Type: Pointable> Pointable for Vec<Type> {
    /// Resolve an [`Address`] into a node [`Pointer`].
    ///
    /// Delegate to child items, erroring if address is invalid.
    fn resolve(&mut self, address: &mut Address) -> Result<Pointer> {
        match address.pop_front() {
            Some(Slot::Index(index)) => match self.get_mut(index) {
                Some(item) => item.resolve(address),
                None => bail!(invalid_slot_index::<Self>(index)),
            },
            Some(slot) => bail!(invalid_slot_variant::<Self>(slot)),
            None => bail!(
                "Address resolves to a node that can not be pointed to: {}",
                address
            ),
        }
    }

    /// Find a node based on its `id` and return a [`Pointer`] to it.
    ///
    /// Delegate to child items and return `Pointer::None` if not found.
    fn find(&mut self, id: &str) -> Pointer {
        for item in self {
            let pointer = item.find(id);
            match pointer {
                Pointer::None => continue,
                _ => return pointer,
            }
        }
        Pointer::None
    }
}
