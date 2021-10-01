/// Generate the `resolve` method for an `enum` having variants of different types
macro_rules! patchable_variants_resolve {
    ($( $variant:path )*) => {
        fn resolve(&mut self, address: &mut Address) -> Result<Pointer> {
            match self {
                $(
                    $variant(me) => me.resolve(address),
                )*
                #[allow(unreachable_patterns)]
                _ => bail!("Unhandled variant `{}` of `{}`", self.as_ref(), type_name::<Self>())
            }
        }
    };
}

/// Generate the `find` method for an `enum` having variants of different types
macro_rules! patchable_variants_find {
    ($( $variant:path )*) => {
        fn find(&mut self, id: &str) -> Pointer {
            match self {
                $(
                    $variant(me) => me.find(id),
                )*
                #[allow(unreachable_patterns)]
                _ => Pointer::None
            }
        }
    };
}

/// Generate the `is_equal` method for an `enum` having variants of different types
macro_rules! patchable_variants_is_equal {
    ($( $variant:path )*) => {
        fn is_equal(&self, other: &Self) -> Result<()> {
            match (self, other) {
                $(
                    ($variant(me), $variant(other)) => me.is_equal(other),
                )*
                _ => bail!(Error::NotEqual),
            }
        }
    };
}

/// Generate the `make_hash` method for an `enum` having variants of different types
macro_rules! patchable_variants_hash {
    ($( $variant:path )*) => {
        fn make_hash<H: std::hash::Hasher>(&self, state: &mut H) {
            match self {
                $(
                    $variant(me) => me.make_hash(state),
                )*
                #[allow(unreachable_patterns)]
                _ => tracing::error!("Unhandled variant `{}` of `{}`", self.as_ref(), type_name::<Self>())
            }
        }
    };
}

/// Generate the `diff_same` method for an `enum` having variants of different types
macro_rules! patchable_variants_diff_same {
    ($( $variant:path )*) => {
        fn diff_same(&self, differ: &mut Differ, other: &Self) {
            match (self, other) {
                $(
                    ($variant(me), $variant(other)) => me.diff_same(differ, other),
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
            patchable_is_same!();

            fn is_equal(&self, other: &Self) -> Result<()> {
                match std::mem::discriminant(self) == std::mem::discriminant(other) {
                    true => Ok(()),
                    false => bail!(Error::NotEqual),
                }
            }

            fn make_hash<H: std::hash::Hasher>(&self, state: &mut H) {
                use std::hash::Hash;
                std::mem::discriminant(self).hash(state)
            }

            patchable_diff!();

            fn diff_same(&self, differ: &mut Differ, other: &Self) {
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
            patchable_variants_resolve!($( $variant )*);
            patchable_variants_find!($( $variant )*);

            patchable_is_same!();
            patchable_variants_is_equal!($( $variant )*);
            patchable_variants_hash!($( $variant )*);

            patchable_diff!();
            patchable_variants_diff_same!($( $variant )*);

            patchable_variants_apply_add!($( $variant )*);
            patchable_variants_apply_remove!($( $variant )*);
            patchable_variants_apply_replace!($( $variant )*);
            patchable_variants_apply_move!($( $variant )*);
            patchable_variants_apply_transform!($( $variant )*);
        }
    };
}
