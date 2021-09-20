use super::prelude::*;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use stencila_schema::{Boolean, Integer, Number};

/// Macro to generate `impl Patchable` for atomic types
macro_rules! patchable_atomic {
    ($type:ty) => {
        impl Patchable for $type {
            patchable_is_same!();

            fn is_equal(&self, other: &Self) -> Result<()> {
                match self == other {
                    true => Ok(()),
                    false => bail!(Error::NotEqual),
                }
            }

            fn make_hash<H: Hasher>(&self, state: &mut H) {
                self.hash(state)
            }

            patchable_diff!();

            fn diff_same(&self, differ: &mut Differ, other: &Self) {
                if self != other {
                    differ.replace(other)
                }
            }

            fn apply_maybe(&mut self, _id: &str, _patch: &Patch) -> Result<bool> {
                Ok(false)
            }

            fn apply_replace(
                &mut self,
                _address: &mut Address,
                _items: usize,
                value: &Box<dyn Any + Send>,
            ) -> Result<()> {
                if let Some(value) = value.deref().downcast_ref::<Self>() {
                    *self = *value;
                    Ok(())
                } else {
                    bail!(invalid_patch_value(self))
                }
            }
        }
    };
}

// Implementations for types used in some struct fields
// instead of the Stencila primitives (usually as optimizations)

patchable_atomic!(u8);
patchable_atomic!(i32);
patchable_atomic!(u32);

// Implementations for Stencila primitive types

patchable_atomic!(Boolean);
patchable_atomic!(Integer);

impl Patchable for Number {
    patchable_is_same!();

    fn is_equal(&self, other: &Self) -> Result<()> {
        #[allow(clippy::float_cmp)]
        if self == other {
            Ok(())
        } else {
            bail!(Error::NotEqual)
        }
    }

    fn make_hash<H: Hasher>(&self, state: &mut H) {
        // See caveats to this approach
        // https://stackoverflow.com/a/39647997
        self.to_bits().hash(state)
    }

    patchable_diff!();

    fn diff_same(&self, differ: &mut Differ, other: &Self) {
        #[allow(clippy::float_cmp)]
        if self != other {
            differ.replace(other)
        }
    }

    fn apply_maybe(&mut self, _id: &str, _patch: &Patch) -> Result<bool> {
        Ok(false)
    }

    fn apply_replace(
        &mut self,
        _address: &mut Address,
        _items: usize,
        value: &Box<dyn Any + Send>,
    ) -> Result<()> {
        if let Some(value) = value.deref().downcast_ref::<Self>() {
            *self = *value;
            Ok(())
        } else {
            bail!(invalid_patch_value(self))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        assert_json,
        patches::{apply_new, diff, equal},
    };

    #[test]
    fn booleans() -> Result<()> {
        assert!(equal(&true, &true));
        assert!(equal(&false, &false));
        assert!(!equal(&true, &false));

        assert_json!(diff(&true, &true), []);
        assert_json!(diff(&false, &false), []);
        assert_json!(diff(&true, &false), [{"type": "Replace", "address": [], "items": 1, "value": false, "length": 1}]);

        assert_json!(apply_new(&true, &diff(&true, &false))?, false);
        assert_json!(apply_new(&false, &diff(&false, &true))?, true);

        Ok(())
    }

    #[test]
    fn integers() -> Result<()> {
        assert!(equal(&42, &42));
        assert!(!equal(&42, &1));

        assert_json!(diff(&42, &42), []);
        assert_json!(diff(&42, &1), [{"type": "Replace", "address": [], "items": 1, "value": 1, "length": 1}]);

        assert_json!(apply_new(&1, &diff(&1, &42))?, 42);

        Ok(())
    }

    #[test]
    fn numbers() -> Result<()> {
        assert!(equal(&3.14, &3.14));
        assert!(!equal(&3.14, &1e6));

        assert_json!(diff(&3.14, &3.14), []);
        assert_json!(diff(&3.14, &1e6), [{"type": "Replace", "address": [], "items": 1, "value": 1e6, "length": 1}]);

        assert_json!(apply_new(&1e6, &diff(&1e6, &3.14))?, 3.14);

        Ok(())
    }
}
