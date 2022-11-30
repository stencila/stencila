//! Patching macros for `enum`s

/// Generate the `diff` method for an `enum` having variants of different types
macro_rules! patchable_variants_diff {
    ($( $variant:path )*) => {
        fn diff(&self, other: &Self, differ: &mut Differ) {
            match (self, other) {
                $(
                    ($variant(me), $variant(other)) => me.diff(other, differ),
                )*
                #[allow(unreachable_patterns)]
                _ => differ.replace(other)
            }
        }
    };
}

/// Generate the `apply_add` method for an `enum` having variants of different types
macro_rules! patchable_variants_apply_add {
    ($( $variant:path )*) => {
        fn apply_add(&mut self, address: &mut Address, value: Value) -> Result<()> {
            match self {
                $(
                    $variant(me) => me.apply_add(address, value),
                )*
                #[allow(unreachable_patterns)]
                _ => bail!(invalid_patch_operation::<Self>("add"))
            }
        }
    };
}

/// Generate the `apply_add_many` method for an `enum` having variants of different types
macro_rules! patchable_variants_apply_add_many {
    ($( $variant:path )*) => {
        fn apply_add_many(&mut self, address: &mut Address, values: Values) -> Result<()> {
            match self {
                $(
                    $variant(me) => me.apply_add_many(address, values),
                )*
                #[allow(unreachable_patterns)]
                _ => bail!(invalid_patch_operation::<Self>("add_many"))
            }
        }
    };
}

/// Generate the `apply_remove` method for an `enum` having variants of different types
macro_rules! patchable_variants_apply_remove {
    ($( $variant:path )*) => {
        fn apply_remove(&mut self, address: &mut Address) -> Result<()> {
            match self {
                $(
                    $variant(me) => me.apply_remove(address),
                )*
                #[allow(unreachable_patterns)]
                _ => bail!(invalid_patch_operation::<Self>("remove"))
            }
        }
    };
}

/// Generate the `apply_remove_many` method for an `enum` having variants of different types
macro_rules! patchable_variants_apply_remove_many {
    ($( $variant:path )*) => {
        fn apply_remove_many(&mut self, address: &mut Address, items: usize) -> Result<()> {
            match self {
                $(
                    $variant(me) => me.apply_remove_many(address, items),
                )*
                #[allow(unreachable_patterns)]
                _ => bail!(invalid_patch_operation::<Self>("remove_many"))
            }
        }
    };
}

/// Generate the `apply_replace` method for an `enum` having variants of different types
macro_rules! patchable_variants_apply_replace {
    ($( $variant:path )*) => {
        fn apply_replace(&mut self, address: &mut Address, value: Value) -> Result<()> {
            match self {
                $(
                    $variant(me) => me.apply_replace(address, value),
                )*
                #[allow(unreachable_patterns)]
                _ => bail!(invalid_patch_operation::<Self>("replace"))
            }
        }
    };
}

/// Generate the `apply_replace_many` method for an `enum` having variants of different types
macro_rules! patchable_variants_apply_replace_many {
    ($( $variant:path )*) => {
        fn apply_replace_many(&mut self, address: &mut Address, items: usize, values: Values) -> Result<()> {
            match self {
                $(
                    $variant(me) => me.apply_replace_many(address, items, values),
                )*
                #[allow(unreachable_patterns)]
                _ => bail!(invalid_patch_operation::<Self>("replace_many"))
            }
        }
    };
}

/// Generate the `apply_move` method for an `enum` having variants of different types
macro_rules! patchable_variants_apply_move {
    ($( $variant:path )*) => {
        fn apply_move(&mut self, from: &mut Address, to: &mut Address) -> Result<()> {
            match self {
                $(
                    $variant(me) => me.apply_move(from, to),
                )*
                #[allow(unreachable_patterns)]
                _ => bail!(invalid_patch_operation::<Self>("move"))
            }
        }
    };
}

/// Generate the `apply_transform` method for an `enum` having variants of different types
macro_rules! patchable_variants_apply_transform {
    ($( $variant:path )*) => {
        fn apply_transform(&mut self, address: &mut Address, from: &str, to: &str) -> Result<()> {
            match self {
                $(
                    $variant(me) => me.apply_transform(address, from, to),
                )*
                #[allow(unreachable_patterns)]
                _ => bail!(invalid_patch_operation::<Self>("transform"))
            }
        }
    };
}

/// Generate a `impl Patchable` for a simple `enum`.
///
/// Implements `to_value` and `from_value` based on the string representation
/// of the enum variant. This however, does not improve performance over using
/// falling back to JSON serialization.
macro_rules! patchable_enum {
    ($type:ty) => {
        impl Patchable for $type {
            fn diff(&self, other: &Self, differ: &mut Differ) {
                if self != other {
                    differ.replace(other)
                }
            }

            fn apply_replace(&mut self, _address: &mut Address, value: Value) -> Result<()> {
                *self = Self::from_value(value)?;
                Ok(())
            }

            fn to_value(&self) -> Value {
                Value::String(self.as_ref().to_string())
            }

            fn from_value(value: Value) -> Result<Self> {
                use std::str::FromStr;
                match value {
                    Value::String(string) => Ok(Self::from_str(&string)?),
                    Value::Json(json) => Ok(serde_json::from_value::<Self>(json)?),
                    _ => bail!(invalid_patch_value::<Self>(value)),
                }
            }
        }
    };
}

/// Generate a `impl Patchable` for an `enum` having variants of different types.
macro_rules! patchable_variants {
    ($type:ty $(, $variant:path )*) => {
        impl Patchable for $type {
            patchable_variants_diff!($( $variant )*);

            patchable_variants_apply_add!($( $variant )*);
            patchable_variants_apply_add_many!($( $variant )*);

            patchable_variants_apply_remove!($( $variant )*);
            patchable_variants_apply_remove_many!($( $variant )*);

            patchable_variants_apply_replace!($( $variant )*);
            patchable_variants_apply_replace_many!($( $variant )*);

            patchable_variants_apply_move!($( $variant )*);

            patchable_variants_apply_transform!($( $variant )*);
        }
    };
}
