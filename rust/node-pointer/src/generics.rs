use crate::{Pointable, Pointer, PointerMut};
use eyre::{bail, Result};
use node_address::{invalid_slot_index, invalid_slot_variant, Address, Slot};
use std::ops::{Deref, DerefMut};

/// Generate a `impl Pointable` for a `struct`
macro_rules! pointable_struct {
    ($type:ty $(, $field:ident )*) => {
        impl Pointable for $type {
            /// Resolve an [`Address`] into a node [`Pointer`].
            ///
            /// Delegate to child fields, erroring if address is invalid.
            fn resolve(&self, address: &mut Address) -> Result<Pointer> {
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
            fn resolve_mut(&mut self, address: &mut Address) -> Result<PointerMut> {
                use node_address::{invalid_address, invalid_slot_variant, invalid_slot_name, Slot};

                match address.pop_front() {
                    Some(Slot::Name(name)) => match name.as_str() {
                        $(
                            stringify!($field) => self.$field.resolve_mut(address),
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
            fn find(&self, id: &str) -> Pointer {
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
            fn find_mut(&mut self, id: &str) -> PointerMut {
                if let Some(my_id) = self.id.as_ref() {
                    if id == **my_id {
                        return PointerMut::Some
                    }
                }

                $(
                    let pointer = self.$field.find_mut(id);
                    match pointer {
                        PointerMut::None => (),
                        _ => return pointer
                    }
                )*

                PointerMut::None
            }
        }
    };
}

/// Generate a `impl Pointable` for an `enum` having variants of different types.
macro_rules! pointable_variants {
    ($type:ty $(, $variant:path )*) => {
        impl Pointable for $type {
            fn resolve(& self, address: &mut Address) -> Result<Pointer> {
                match self {
                    $(
                        $variant(me) => me.resolve(address),
                    )*
                    #[allow(unreachable_patterns)]
                    _ => bail!("Unhandled variant `{}` of `{}` in `Pointable::resolve`", self.as_ref(), std::any::type_name::<Self>())
                }
            }
            fn resolve_mut(&mut self, address: &mut Address) -> Result<PointerMut> {
                match self {
                    $(
                        $variant(me) => me.resolve_mut(address),
                    )*
                    #[allow(unreachable_patterns)]
                    _ => bail!("Unhandled variant `{}` of `{}` in `Pointable::resolve`", self.as_ref(), std::any::type_name::<Self>())
                }
            }

            fn find(& self, id: &str) -> Pointer {
                match self {
                    $(
                        $variant(me) => me.find(id),
                    )*
                    #[allow(unreachable_patterns)]
                    _ => Pointer::None
                }
            }
            fn find_mut(&mut self, id: &str) -> PointerMut {
                match self {
                    $(
                        $variant(me) => me.find_mut(id),
                    )*
                    #[allow(unreachable_patterns)]
                    _ => PointerMut::None
                }
            }
        }
    };
}

impl<Type: Pointable> Pointable for Option<Type> {
    /// Resolve an [`Address`] into a node [`Pointer`].
    ///
    /// Delegate to value, if any.
    fn resolve(&self, address: &mut Address) -> Result<Pointer> {
        match self {
            Some(me) => me.resolve(address),
            None => Ok(Pointer::None),
        }
    }
    fn resolve_mut(&mut self, address: &mut Address) -> Result<PointerMut> {
        match self {
            Some(me) => me.resolve_mut(address),
            None => Ok(PointerMut::None),
        }
    }

    /// Find a node based on its `id` and return a [`Pointer`] to it.
    ///
    /// Delegate to value, if any.
    fn find(&self, id: &str) -> Pointer {
        match self {
            Some(me) => me.find(id),
            None => Pointer::None,
        }
    }
    fn find_mut(&mut self, id: &str) -> PointerMut {
        match self {
            Some(me) => me.find_mut(id),
            None => PointerMut::None,
        }
    }
}

impl<Type: Pointable> Pointable for Box<Type> {
    /// Resolve an [`Address`] into a node [`Pointer`].
    ///
    /// Delegate to boxed value.
    fn resolve(&self, address: &mut Address) -> Result<Pointer> {
        self.deref().resolve(address)
    }
    fn resolve_mut(&mut self, address: &mut Address) -> Result<PointerMut> {
        self.deref_mut().resolve_mut(address)
    }

    /// Find a node based on its `id` and return a [`Pointer`] to it.
    ///
    /// Delegate to boxed value.
    fn find(&self, id: &str) -> Pointer {
        self.deref().find(id)
    }
    fn find_mut(&mut self, id: &str) -> PointerMut {
        self.deref_mut().find_mut(id)
    }
}

impl<Type: Pointable> Pointable for Vec<Type> {
    /// Resolve an [`Address`] into a node [`Pointer`].
    ///
    /// Delegate to child items, erroring if address is invalid.
    fn resolve(&self, address: &mut Address) -> Result<Pointer> {
        match address.pop_front() {
            Some(Slot::Index(index)) => match self.get(index) {
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
    fn resolve_mut(&mut self, address: &mut Address) -> Result<PointerMut> {
        match address.pop_front() {
            Some(Slot::Index(index)) => match self.get_mut(index) {
                Some(item) => item.resolve_mut(address),
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
    fn find(&self, id: &str) -> Pointer {
        for item in self {
            let pointer = item.find(id);
            match pointer {
                Pointer::None => continue,
                _ => return pointer,
            }
        }
        Pointer::None
    }
    fn find_mut(&mut self, id: &str) -> PointerMut {
        for item in self {
            let pointer = item.find_mut(id);
            match pointer {
                PointerMut::None => continue,
                _ => return pointer,
            }
        }
        PointerMut::None
    }
}
