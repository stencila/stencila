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
                if self == other {
                    Ok(())
                } else {
                    bail!(Error::NotEqual)
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

            fn apply_replace(&mut self, _keys: &mut Keys, _items: usize, value: &Box<dyn Any>) {
                if let Some(value) = value.deref().downcast_ref::<Self>() {
                    *self = *value
                } else {
                    invalid_value!()
                }
            }
        }
    };
}

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

    fn apply_replace(&mut self, _keys: &mut Keys, _items: usize, value: &Box<dyn Any>) {
        if let Some(value) = value.deref().downcast_ref::<Self>() {
            *self = *value
        } else {
            invalid_value!()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_json,
        patches::{apply_new, diff, equal},
    };

    #[test]
    fn booleans() {
        assert!(equal(&true, &true));
        assert!(equal(&false, &false));
        assert!(!equal(&true, &false));

        assert_json!(diff(&true, &true), []);
        assert_json!(diff(&false, &false), []);
        assert_json!(diff(&true, &false), [{"op": "replace", "keys": [], "items": 1, "value": false}]);

        assert_json!(apply_new(&true, &diff(&true, &false)), false);
        assert_json!(apply_new(&false, &diff(&false, &true)), true);
    }

    #[test]
    fn integers() {
        assert!(equal(&42, &42));
        assert!(!equal(&42, &1));

        assert_json!(diff(&42, &42), []);
        assert_json!(diff(&42, &1), [{"op": "replace", "keys": [], "items": 1, "value": 1}]);

        assert_json!(apply_new(&1, &diff(&1, &42)), 42);
    }

    #[test]
    fn numbers() {
        assert!(equal(&3.14, &3.14));
        assert!(!equal(&3.14, &1e6));

        assert_json!(diff(&3.14, &3.14), []);
        assert_json!(diff(&3.14, &1e6), [{"op": "replace", "keys": [], "items": 1, "value": 1e6}]);

        assert_json!(apply_new(&1e6, &diff(&1e6, &3.14)), 3.14);
    }
}
