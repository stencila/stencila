/// Generate the `is_equal` method for a `struct`
macro_rules! patchable_struct_is_equal {
    ($( $field:ident )*) => {
        #[allow(unused_variables)]
        fn is_equal(&self, other: &Self) -> Result<()> {
            $(
                self.$field.is_equal(&other.$field)?;
            )*
            Ok(())
        }
    };
}

/// Generate the `make_hash` method for a `struct`
macro_rules! patchable_struct_hash {
    ($( $field:ident )*) => {
        fn make_hash<H: std::hash::Hasher>(&self, state: &mut H) {
            // Include the type name in the hash (to avoid clash when structs
            // of different types have the same values for different fields)
            use std::hash::Hash;
            type_name::<Self>().hash(state);
            // Include the hash of supplied fields. Because we include the type
            // name in the hash, we do no need to include the field names.
            $(
                self.$field.make_hash(state);
            )*
        }
    };
}

/// Generate the `diff_same` method for a `struct`
macro_rules! patchable_struct_diff_same {
    ($( $field:ident )*) => {
        #[allow(unused_variables)]
        fn diff_same(&self, differ: &mut Differ, other: &Self) {
            $(
                differ.field(stringify!($field), &self.$field, &other.$field);
            )*
        }
    };
}

/// Generate the `apply_maybe` method for a `struct`
macro_rules! patchable_struct_apply_maybe {
    ($( $field:ident )*) => {
        #[allow(unused_variables)]
        fn apply_maybe(&mut self, id: &str, patch: &Patch) -> Result<bool> {
            if id == self.id.as_deref().map_or_else(|| "", |id| id) {
                self.apply_patch(patch)?;
                return Ok(true)
            }

            $(
                if self.$field.apply_maybe(id, patch)? {
                    return Ok(true);
                }
            )*

            return Ok(false);
        }
    };
}

/// Generate the `apply_add` method for a `struct`
macro_rules! patchable_struct_apply_add {
    ($( $field:ident )*) => {
        #[allow(unused_variables)]
        fn apply_add(&mut self, address: &mut Address, value: &Box<dyn Any + Send>) {
            if let Some(Slot::Name(name)) = address.pop_front() {
                match name.as_str() {
                    $(
                        stringify!($field) => self.$field.apply_add(address, value),
                    )*
                    _ => invalid_name!(name),
                }
            } else {
                invalid_address!(address)
            }
        }
    };
}

/// Generate the `apply_remove` method for a `struct`
macro_rules! patchable_struct_apply_remove {
    ($( $field:ident )*) => {
        #[allow(unused_variables)]
        fn apply_remove(&mut self, address: &mut Address, items: usize) {
            if let Some(Slot::Name(name)) = address.pop_front() {
                match name.as_str() {
                    $(
                        stringify!($field) => self.$field.apply_remove(address, items),
                    )*
                    _ => invalid_name!(name),
                }
            } else {
                invalid_address!(address)
            }
        }
    };
}

/// Generate the `apply_replace` method for a `struct`
macro_rules! patchable_struct_apply_replace {
    ($( $field:ident )*) => {
        #[allow(unused_variables)]
        fn apply_replace(&mut self, address: &mut Address, items: usize, value: &Box<dyn Any + Send>) {
            if let Some(Slot::Name(name)) = address.pop_front() {
                match name.as_str() {
                    $(
                        stringify!($field) => self.$field.apply_replace(address, items, value),
                    )*
                    _ => invalid_name!(name),
                }
            } else {
                invalid_address!(address)
            }
        }
    };
}

/// Generate the `apply_move` method for a `struct`
macro_rules! patchable_struct_apply_move {
    ($( $field:ident )*) => {
        #[allow(unused_variables)]
        fn apply_move(&mut self, from: &mut Address, items: usize, to: &mut Address) {
            if let (Some(Slot::Name(name)), Some(Slot::Name(_name_again))) = (from.pop_front(), to.pop_front()) {
                match name.as_str() {
                    $(
                        stringify!($field) => self.$field.apply_move(from, items, to),
                    )*
                    _ => invalid_name!(name),
                }
            } else {
                invalid_address!(from)
            }
        }
    };
}

/// Generate the `apply_transform` method for a `struct`
macro_rules! patchable_struct_apply_transform {
    ($( $field:ident )*) => {
        #[allow(unused_variables)]
        fn apply_transform(&mut self, address: &mut Address, from: &str, to: &str) {
            if let Some(Slot::Name(name)) = address.pop_front() {
                match name.as_str() {
                    $(
                        stringify!($field) => self.$field.apply_transform(address, from, to),
                    )*
                    _ => invalid_name!(name),
                }
            } else {
                invalid_address!(from)
            }
        }
    };
}

/// Generate a `impl Patchable` for a `struct`, passing
/// a list of fields for comparison, diffing, and applying ops.
macro_rules! patchable_struct {
    ($type:ty $(, $field:ident )*) => {
        impl Patchable for $type {

            patchable_is_same!();
            patchable_struct_is_equal!($( $field )*);
            patchable_struct_hash!($( $field )*);

            patchable_diff!();
            patchable_struct_diff_same!($( $field )*);

            patchable_struct_apply_maybe!($( $field )*);
            patchable_struct_apply_add!($( $field )*);
            patchable_struct_apply_remove!($( $field )*);
            patchable_struct_apply_replace!($( $field )*);
            patchable_struct_apply_move!($( $field )*);
            patchable_struct_apply_transform!($( $field )*);
        }
    };
}
