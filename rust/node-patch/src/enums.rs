/// Generate the `make_hash` method for an `enum` having variants of different types
macro_rules! patchable_variants_hash {
    ($( $variant:path )*) => {
        fn make_hash<H: std::hash::Hasher>(&self, state: &mut H) {
            match self {
                $(
                    $variant(me) => me.make_hash(state),
                )*
                #[allow(unreachable_patterns)]
                _ => common::tracing::error!("Unhandled variant `{}` of `{}` in `Patchable::hash`", self.as_ref(), type_name::<Self>())
            }
        }
    };
}

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
        fn apply_add(&mut self, address: &mut Address, value: &Value) -> Result<()> {
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

/// Generate the `apply_remove` method for an `enum` having variants of different types
macro_rules! patchable_variants_apply_remove {
    ($( $variant:path )*) => {
        fn apply_remove(&mut self, address: &mut Address, items: usize) -> Result<()> {
            match self {
                $(
                    $variant(me) => me.apply_remove(address, items),
                )*
                #[allow(unreachable_patterns)]
                _ => bail!(invalid_patch_operation::<Self>("remove"))
            }
        }
    };
}

/// Generate the `apply_replace` method for an `enum` having variants of different types
macro_rules! patchable_variants_apply_replace {
    ($( $variant:path )*) => {
        fn apply_replace(&mut self, address: &mut Address, items: usize, value: &Value) -> Result<()> {
            match self {
                $(
                    $variant(me) => me.apply_replace(address, items, value),
                )*
                #[allow(unreachable_patterns)]
                _ => bail!(invalid_patch_operation::<Self>("replace"))
            }
        }
    };
}

/// Generate the `apply_move` method for an `enum` having variants of different types
macro_rules! patchable_variants_apply_move {
    ($( $variant:path )*) => {
        fn apply_move(&mut self, from: &mut Address, items: usize, to: &mut Address) -> Result<()> {
            match self {
                $(
                    $variant(me) => me.apply_move(from, items, to),
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
macro_rules! patchable_enum {
    ($type:ty) => {
        impl Patchable for $type {
            fn make_hash<H: std::hash::Hasher>(&self, state: &mut H) {
                use std::hash::Hash;
                std::mem::discriminant(self).hash(state)
            }

            fn diff(&self, other: &Self, differ: &mut Differ) {
                if std::mem::discriminant(self) != std::mem::discriminant(other) {
                    differ.replace(other)
                }
            }

            fn apply_replace(
                &mut self,
                _address: &mut Address,
                _items: usize,
                value: &Value,
            ) -> Result<()> {
                *self = Self::from_value(value)?;
                Ok(())
            }
        }
    };
}

/// Generate a `impl Patchable` for an `enum` having variants of different types.
macro_rules! patchable_variants {
    ($type:ty $(, $variant:path )*) => {
        impl Patchable for $type {
            patchable_variants_hash!($( $variant )*);
            patchable_variants_diff!($( $variant )*);
            patchable_variants_apply_add!($( $variant )*);
            patchable_variants_apply_remove!($( $variant )*);
            patchable_variants_apply_replace!($( $variant )*);
            patchable_variants_apply_move!($( $variant )*);
            patchable_variants_apply_transform!($( $variant )*);
        }
    };
}
