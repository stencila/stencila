//! Implementations of [`Patchable`] for [`Primitive`] node types
//!
//! Note that `Patchable` is implemented for some primitives elsewhere:
//! - `String`: in strings.rs`
//! - `Array`: is covered by `impl Patchable for Vec<Primitive>` in `vecs.rs`
//! - `Object`: is covered by `impl Patchable for BTreeMap<String, Primitive>` in `maps.rs`

use common::serde_json;
use node_dispatch::{dispatch_primitive, dispatch_primitive_pair};
use stencila_schema::*;

use super::prelude::*;

impl Patchable for Primitive {
    fn diff(&self, other: &Self, differ: &mut Differ) {
        dispatch_primitive_pair!(self, other, differ.replace(other), diff, differ)
    }

    fn apply_add(&mut self, address: &mut Address, value: Value) -> Result<()> {
        // Only expected for compound primitives ie. `String`, `Array`, `Object`
        dispatch_primitive!(self, apply_add, address, value)
    }

    fn apply_remove(&mut self, address: &mut Address) -> Result<()> {
        // Only expected for compound primitives ie. `String`, `Array`, `Object`
        dispatch_primitive!(self, apply_remove, address)
    }

    fn apply_replace(&mut self, address: &mut Address, value: Value) -> Result<()> {
        if address.is_empty() {
            *self = Self::from_value(value)?;
            Ok(())
        } else {
            // Only expected for compound primitives ie. `String`, `Array`, `Object`
            dispatch_primitive!(self, apply_replace, address, value)
        }
    }

    fn apply_move(&mut self, from: &mut Address, to: &mut Address) -> Result<()> {
        // Only expected for compound primitives ie. `String`, `Array`, `Object`
        dispatch_primitive!(self, apply_move, from, to)
    }

    fn to_value(&self) -> Value {
        Value::Primitive(self.clone())
    }

    fn from_value(value: Value) -> Result<Self> {
        match value {
            Value::Primitive(node) => Ok(node),
            Value::Json(json) => Ok(serde_json::from_value::<Self>(json)?),
            _ => bail!(invalid_patch_value::<Self>(value)),
        }
    }
}

impl Patchable for Null {
    fn diff(&self, _other: &Self, _differ: &mut Differ) {
        // By definition, no difference
    }

    fn to_value(&self) -> Value {
        Value::Null
    }

    fn from_value(value: Value) -> Result<Self> {
        match value {
            Value::Null => Ok(Self {}),
            Value::Json(json) => serde_json::from_value::<serde_json::Value>(json)?
                .is_null()
                .then_some(Null {})
                .ok_or_else(|| eyre!("Expected a JSON null")),
            _ => bail!(invalid_patch_value::<Self>(value)),
        }
    }
}

/// Macro to generate `impl Patchable` for atomic types
macro_rules! patchable_atomic {
    ($type:ty) => {
        impl Patchable for $type {
            fn diff(&self, other: &Self, differ: &mut Differ) {
                #[allow(clippy::float_cmp)]
                if self != other {
                    differ.replace(other)
                }
            }

            fn apply_replace(&mut self, _address: &mut Address, value: Value) -> Result<()> {
                *self = Self::from_value(value)?;
                Ok(())
            }
        }
    };
}

// Implementations for types used in some struct fields
// instead of the Stencila primitives (usually as optimizations)

patchable_atomic!(u8);
patchable_atomic!(i32);
patchable_atomic!(u32);
patchable_atomic!(u64);

// Implementations for Stencila primitive types

patchable_atomic!(Boolean);
patchable_atomic!(Integer);
patchable_atomic!(Number);

patchable_atomic!(suids::Suid);

// Implementations for time related types

patchable_struct!(Date, value);
patchable_struct!(Time, value);
patchable_struct!(DateTime, value);
patchable_struct!(Timestamp, value, time_unit);
patchable_struct!(Duration, value, time_unit);
patchable_enum!(TimeUnit);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{apply_new, diff};
    use test_utils::{assert_json_eq, assert_json_is};

    macro_rules! obj {
        ($json:tt) => {
            common::serde_json::from_value::<Object>(common::serde_json::json!($json)).unwrap()
        };
    }

    #[test]
    fn primitives() -> Result<()> {
        let null = Primitive::Null(Null {});
        let bool = Primitive::Boolean(true);
        let int1 = Primitive::Integer(1);
        let int2 = Primitive::Integer(2);
        let str1 = Primitive::String("abcd".to_string());
        let str2 = Primitive::String("cbd".to_string());
        let obj1 = Primitive::Object(Object::new());
        let obj2 = Primitive::Object(obj!({
            "a": Primitive::String("abc".to_string())
        }));
        let obj3 = Primitive::Object(obj!({
            "a": Primitive::String("a".to_string()),
            "b": Primitive::Number(Number(1.23))
        }));

        let patch = diff(&null, &bool);
        assert_json_is!(patch.ops, [{"type": "Replace", "address": [], "value": true}]);
        assert_json_eq!(apply_new(&null, patch)?, &bool);

        let patch = diff(&bool, &int1);
        assert_json_is!(patch.ops, [{"type": "Replace", "address": [], "value": 1}]);
        assert_json_eq!(apply_new(&bool, patch)?, &int1);

        let patch = diff(&int1, &int2);
        assert_json_is!(patch.ops, [{"type": "Replace", "address": [], "value": 2}]);
        assert_json_eq!(apply_new(&int1, patch)?, &int2);

        let patch = diff(&int2, &str1);
        assert_json_is!(patch.ops, [{"type": "Replace", "address": [], "value": "abcd"}]);
        assert_json_eq!(apply_new(&int2, patch)?, &str1);

        let patch = diff(&str1, &str2);
        assert_json_is!(patch.ops, [
            {"type": "Remove", "address": [0]},
            {"type": "Remove", "address": [0]},
            {"type": "Add", "address": [1], "value": "b"}
        ]);
        assert_json_eq!(apply_new(&str1, patch)?, &str2);

        let patch = diff(&str2, &obj1);
        assert_json_is!(patch.ops, [{"type": "Replace", "address": [], "value": {}}]);
        assert_json_eq!(apply_new(&str2, patch)?, &obj1);

        let patch = diff(&obj1, &obj2);
        assert_json_is!(patch.ops, [
            {"type": "Replace", "address": [], "value": {"a": "abc"}}
        ]);
        assert_json_eq!(apply_new(&obj1, patch)?, &obj2);

        let patch = diff(&obj2, &obj3);
        assert_json_is!(patch.ops, [
            {"type": "Remove", "address": ["a", 1]},
            {"type": "Remove", "address": ["a", 1]},
            {"type": "Add", "address": ["b"], "value": 1.23}
        ]);
        assert_json_eq!(apply_new(&obj2, patch)?, &obj3);

        Ok(())
    }

    #[test]
    fn nulls() -> Result<()> {
        let null = Null {};
        assert_json_is!(diff(&null, &null).ops, []);

        Ok(())
    }

    #[test]
    fn booleans() -> Result<()> {
        assert_json_is!(diff(&true, &true).ops, []);
        assert_json_is!(diff(&false, &false).ops, []);
        assert_json_is!(diff(&true, &false).ops, [{"type": "Replace", "address": [], "value": false}]);

        assert_json_is!(apply_new(&true, diff(&true, &false))?, false);
        assert_json_is!(apply_new(&false, diff(&false, &true))?, true);

        Ok(())
    }

    #[test]
    fn integers() -> Result<()> {
        assert_json_is!(diff(&42, &42).ops, []);
        assert_json_is!(diff(&42, &1).ops, [{"type": "Replace", "address": [], "value": 1}]);

        assert_json_is!(apply_new(&1, diff(&1, &42))?, 42);

        Ok(())
    }

    #[test]
    fn numbers() -> Result<()> {
        assert_json_is!(diff(&Number(1.23), &Number(1.23)).ops, []);
        assert_json_is!(diff(&Number(1.23), &Number(1e6)).ops, [{"type": "Replace", "address": [], "value": 1e6}]);

        assert_json_is!(
            apply_new(&Number(1e6), diff(&Number(1e6), &Number(1.23)))?,
            1.23
        );

        Ok(())
    }
}
