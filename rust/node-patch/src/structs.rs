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

/// Generate the `diff` method for a `struct`
macro_rules! patchable_struct_diff {
    ($( $field:ident )*) => {
        #[allow(unused_variables)]
        fn diff(&self, differ: &mut Differ, other: &Self) {
            $(
                differ.field(stringify!($field), &self.$field, &other.$field);
            )*
        }
    };
}

/// Generate the `apply_add` method for a `struct`
macro_rules! patchable_struct_apply_add {
    ($( $field:ident )*) => {
        #[allow(unused_variables)]
        fn apply_add(&mut self, address: &mut Address, value: &Value) -> Result<()> {
            if let Some(Slot::Name(name)) = address.pop_front() {
                match name.as_str() {
                    $(
                        stringify!($field) => self.$field.apply_add(address, value),
                    )*
                    _ => bail!(invalid_slot_name::<Self>(&name)),
                }
            } else {
                bail!(invalid_address::<Self>("first slot should be a name"))
            }
        }
    };
}

/// Generate the `apply_remove` method for a `struct`
macro_rules! patchable_struct_apply_remove {
    ($( $field:ident )*) => {
        #[allow(unused_variables)]
        fn apply_remove(&mut self, address: &mut Address, items: usize) -> Result<()> {
            if let Some(Slot::Name(name)) = address.pop_front() {
                match name.as_str() {
                    $(
                        stringify!($field) => self.$field.apply_remove(address, items),
                    )*
                    _ => bail!(invalid_slot_name::<Self>(&name)),
                }
            } else {
                bail!(invalid_address::<Self>("first slot should be a name"))
            }
        }
    };
}

/// Generate the `apply_replace` method for a `struct`
macro_rules! patchable_struct_apply_replace {
    ($( $field:ident )*) => {
        #[allow(unused_variables)]
        fn apply_replace(&mut self, address: &mut Address, items: usize, value: &Value) -> Result<()> {
            if let Some(Slot::Name(name)) = address.pop_front() {
                match name.as_str() {
                    $(
                        stringify!($field) => self.$field.apply_replace(address, items, value),
                    )*
                    _ => bail!(invalid_slot_name::<Self>(&name)),
                }
            } else {
                bail!(invalid_address::<Self>("first slot should be a name"))
            }
        }
    };
}

/// Generate the `apply_move` method for a `struct`
macro_rules! patchable_struct_apply_move {
    ($( $field:ident )*) => {
        #[allow(unused_variables)]
        fn apply_move(&mut self, from: &mut Address, items: usize, to: &mut Address) -> Result<()> {
            if let (Some(Slot::Name(name)), Some(Slot::Name(_name_again))) = (from.pop_front(), to.pop_front()) {
                match name.as_str() {
                    $(
                        stringify!($field) => self.$field.apply_move(from, items, to),
                    )*
                    _ => bail!(invalid_slot_name::<Self>(&name)),
                }
            } else {
                bail!(invalid_address::<Self>("first slot should be a name"))
            }
        }
    };
}

/// Generate the `apply_transform` method for a `struct`
macro_rules! patchable_struct_apply_transform {
    ($( $field:ident )*) => {
        #[allow(unused_variables)]
        fn apply_transform(&mut self, address: &mut Address, from: &str, to: &str) -> Result<()> {
            if let Some(Slot::Name(name)) = address.pop_front() {
                match name.as_str() {
                    $(
                        stringify!($field) => self.$field.apply_transform(address, from, to),
                    )*
                    _ => bail!(invalid_slot_name::<Self>(&name)),
                }
            } else {
                bail!(invalid_address::<Self>("first slot should be a name"))
            }
        }
    };
}

/// Generate a `impl Patchable` for a `struct`, passing
/// a list of fields for comparison, diffing, and applying ops.
macro_rules! patchable_struct {
    ($type:ty $(, $field:ident )*) => {
        impl Patchable for $type {
            patchable_struct_is_equal!($( $field )*);
            patchable_struct_hash!($( $field )*);


            patchable_struct_diff!($( $field )*);

            patchable_struct_apply_add!($( $field )*);
            patchable_struct_apply_remove!($( $field )*);
            patchable_struct_apply_replace!($( $field )*);
            patchable_struct_apply_move!($( $field )*);
            patchable_struct_apply_transform!($( $field )*);
        }
    };
}

/// Generate a `impl Patchable` for a `struct` that should
/// be replaced as a whole, rather than patching its fields separately.
///
/// Use this, for example, for `struct`s whose HTML encoding is more complicated
/// than a set of elements for each field.
///
/// The `diff` method returns a `Replace` operation and it only implements
/// `apply_replace`.
macro_rules! replaceable_struct {
    ($type:ty $(, $field:ident )*) => {
        impl Patchable for $type {
            patchable_struct_is_equal!($( $field )*);
            patchable_struct_hash!($( $field )*);



            fn diff(&self, differ: &mut Differ, other: &Self) {
                if !self.is_equal(other).is_ok() {
                    differ.replace(other)
                }
            }

            patchable_struct_apply_replace!($( $field )*);
        }
    };
}
