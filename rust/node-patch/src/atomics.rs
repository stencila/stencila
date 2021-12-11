use super::prelude::*;
use std::hash::{Hash, Hasher};
use stencila_schema::{Boolean, Integer, Null, Number};

impl Patchable for Null {
    fn is_equal(&self, _other: &Self) -> Result<()> {
        // By definition, equal
        Ok(())
    }

    fn make_hash<H: Hasher>(&self, state: &mut H) {
        self.to_string().hash(state);
    }

    fn diff(&self, _differ: &mut Differ, _other: &Self) {
        // By definition, no difference
    }
}

/// Macro to generate `impl Patchable` for atomic types
macro_rules! patchable_atomic {
    ($type:ty, $hash:ident) => {
        impl Patchable for $type {
            fn is_equal(&self, other: &Self) -> Result<()> {
                #[allow(clippy::float_cmp)]
                match self == other {
                    true => Ok(()),
                    false => bail!(Error::NotEqual),
                }
            }

            fn make_hash<H: Hasher>(&self, state: &mut H) {
                $hash(self, state)
            }

            fn diff(&self, differ: &mut Differ, other: &Self) {
                #[allow(clippy::float_cmp)]
                if self != other {
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

/// Hash an atomic
fn hash<T: Hash, H: Hasher>(value: &T, state: &mut H) {
    value.hash(state)
}

/// Hash a float
///
/// See caveats to this approach: https://stackoverflow.com/a/39647997
fn hash_float<H: Hasher>(value: &f64, state: &mut H) {
    value.to_bits().hash(state)
}

// Implementations for types used in some struct fields
// instead of the Stencila primitives (usually as optimizations)

patchable_atomic!(u8, hash);
patchable_atomic!(i32, hash);
patchable_atomic!(u32, hash);

// Implementations for Stencila primitive types

patchable_atomic!(Boolean, hash);
patchable_atomic!(Integer, hash);
patchable_atomic!(Number, hash_float);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{apply_new, diff, equal};
    use test_utils::assert_json_is;

    #[test]
    fn booleans() -> Result<()> {
        assert!(equal(&true, &true));
        assert!(equal(&false, &false));
        assert!(!equal(&true, &false));

        assert_json_is!(diff(&true, &true).ops, []);
        assert_json_is!(diff(&false, &false).ops, []);
        assert_json_is!(diff(&true, &false).ops, [{"type": "Replace", "address": [], "items": 1, "value": false, "length": 1}]);

        assert_json_is!(apply_new(&true, &diff(&true, &false))?, false);
        assert_json_is!(apply_new(&false, &diff(&false, &true))?, true);

        Ok(())
    }

    #[test]
    fn integers() -> Result<()> {
        assert!(equal(&42, &42));
        assert!(!equal(&42, &1));

        assert_json_is!(diff(&42, &42).ops, []);
        assert_json_is!(diff(&42, &1).ops, [{"type": "Replace", "address": [], "items": 1, "value": 1, "length": 1}]);

        assert_json_is!(apply_new(&1, &diff(&1, &42))?, 42);

        Ok(())
    }

    #[test]
    fn numbers() -> Result<()> {
        assert!(equal(&1.23, &1.23));
        assert!(!equal(&1.23, &1e6));

        assert_json_is!(diff(&1.23, &1.23).ops, []);
        assert_json_is!(diff(&1.23, &1e6).ops, [{"type": "Replace", "address": [], "items": 1, "value": 1e6, "length": 1}]);

        assert_json_is!(apply_new(&1e6, &diff(&1e6, &1.23))?, 1.23);

        Ok(())
    }
}
