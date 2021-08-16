use super::prelude::*;
use stencila_schema::{Boolean, Integer, Number};

/// Macro to generate `impl Diffable` for atomic types
macro_rules! diffable_atomic {
    ($type:ty) => {
        impl Diffable for $type {
            diffable_is_same!($type);
            diffable_diff!($type);

            fn is_equal(&self, other: &Self) -> Result<()> {
                #[allow(clippy::float_cmp)]
                if self == other {
                    Ok(())
                } else {
                    bail!(Error::NotEqual)
                }
            }

            fn diff_same(&self, differ: &mut Differ, other: &Self) {
                #[allow(clippy::float_cmp)]
                if self != other {
                    differ.replace(other)
                }
            }
        }
    };
}

diffable_atomic!(Boolean);
diffable_atomic!(Integer);
diffable_atomic!(Number);
