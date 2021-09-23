/// Generate the `is_equal` method for an `enum`
macro_rules! patchable_enum_is_equal {
    () => {
        fn is_equal(&self, other: &Self) -> Result<()> {
            match std::mem::discriminant(self) == std::mem::discriminant(other) {
                true => Ok(()),
                false => bail!(Error::NotEqual),
            }
        }
    };
}

/// Generate the `make_hash` method for an `enum`
macro_rules! patchable_enum_hash {
    () => {
        fn make_hash<H: std::hash::Hasher>(&self, state: &mut H) {
            use std::hash::Hash;
            std::mem::discriminant(self).hash(state)
        }
    };
}

/// Generate the `diff_same` method for an `enum`
macro_rules! patchable_enum_diff_same {
    () => {
        fn diff_same(&self, differ: &mut Differ, other: &Self) {
            if std::mem::discriminant(self) != std::mem::discriminant(other) {
                differ.replace(other)
            }
        }
    };
}

/// Generate the `apply_replace` method for a `enum`
macro_rules! patchable_enum_apply_replace {
    () => {
        fn apply_replace(
            &mut self,
            _address: &mut Address,
            _items: usize,
            value: &Value,
        ) -> Result<()> {
            *self = Self::from_value(value)?;
            Ok(())
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
                _ => unimplemented!()
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

/// Generate the `apply_maybe` method for an `enum` having variants of different types
macro_rules! patchable_variants_apply_maybe {
    ($( $variant:path )*) => {
        fn apply_maybe(&mut self, id: &str, patch: &Patch) -> Result<bool> {
            match self {
                $(
                    $variant(me) => me.apply_maybe(id, patch),
                )*
                #[allow(unreachable_patterns)]
                _ => bail!("Unhandled variant in apply_maybe")
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
                _ => bail!(invalid_patch_operation("add", self))
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
                _ => bail!(invalid_patch_operation("remove", self))
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
                _ => bail!(invalid_patch_operation("replace", self))
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
                _ => bail!(invalid_patch_operation("move", self))
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
                _ => bail!(invalid_patch_operation("transform", self))
            }
        }
    };
}

/// Generate a `impl Patchable` for a simple `enum`.
macro_rules! patchable_enum {
    ($type:ty) => {
        impl Patchable for $type {
            patchable_is_same!();
            patchable_enum_is_equal!();
            patchable_enum_hash!();

            patchable_diff!();
            patchable_enum_diff_same!();

            fn apply_maybe(&mut self, _id: &str, _patch: &Patch) -> Result<bool> {
                Ok(false)
            }

            patchable_enum_apply_replace!();
        }
    };
}

/// Generate a `impl Patchable` for an `enum` having variants of different types.
macro_rules! patchable_variants {
    ($type:ty $(, $variant:path )*) => {
        impl Patchable for $type {
            patchable_is_same!();
            patchable_variants_is_equal!($( $variant )*);
            patchable_variants_hash!($( $variant )*);

            patchable_diff!();
            patchable_variants_diff_same!($( $variant )*);

            patchable_variants_apply_maybe!($( $variant )*);
            patchable_variants_apply_add!($( $variant )*);
            patchable_variants_apply_remove!($( $variant )*);
            patchable_variants_apply_replace!($( $variant )*);
            patchable_variants_apply_move!($( $variant )*);
            patchable_variants_apply_transform!($( $variant )*);
        }
    };
}
