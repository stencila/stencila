use super::prelude::*;
use std::{hash::Hasher, ops::Deref};

/// Implements patching for `Option`
///
/// Generates `Add` and `Remove` operations (with no address) given differences in
/// `Some` and `None` between the two options.
/// When applying `Add` and `Remove` operations, if there are no address
/// then apply here, otherwise pass operation through to any value.
/// All other operations passed through.
impl<Type: Patchable> Patchable for Option<Type>
where
    Type: Clone + 'static,
{
    patchable_is_same!();

    fn is_equal(&self, other: &Self) -> Result<()> {
        match (self, other) {
            (None, None) => Ok(()),
            (None, Some(_)) | (Some(_), None) => bail!(Error::NotEqual),
            (Some(me), Some(other)) => me.is_equal(other),
        }
    }

    fn make_hash<H: Hasher>(&self, state: &mut H) {
        if let Some(value) = self {
            value.make_hash(state)
        }
    }

    patchable_diff!();

    fn diff_same(&self, differ: &mut Differ, other: &Self) {
        match (self, other) {
            (None, None) => (),
            (None, Some(value)) => differ.add(value),
            (Some(_), None) => differ.remove(),
            (Some(me), Some(other)) => me.diff_same(differ, other),
        }
    }

    fn apply_add(&mut self, address: &mut Address, value: &Box<dyn Any>) {
        if address.is_empty() {
            if let Some(value) = value.deref().downcast_ref::<Type>() {
                *self = Some(value.clone())
            } else {
                invalid_value!()
            }
        } else if let Some(me) = self {
            me.apply_add(address, value)
        } else {
            invalid_address!(address)
        }
    }

    fn apply_remove(&mut self, address: &mut Address, items: usize) {
        if address.is_empty() {
            *self = None
        } else if let Some(me) = self {
            me.apply_remove(address, items)
        } else {
            invalid_address!(address)
        }
    }

    fn apply_replace(&mut self, address: &mut Address, items: usize, value: &Box<dyn Any>) {
        if let Some(me) = self {
            me.apply_replace(address, items, value)
        } else {
            invalid_op!("replace")
        }
    }

    fn apply_move(&mut self, from: &mut Address, items: usize, to: &mut Address) {
        if let Some(me) = self {
            me.apply_move(from, items, to)
        } else {
            invalid_op!("move")
        }
    }

    fn apply_transform(&mut self, address: &mut Address, from: &str, to: &str) {
        if let Some(me) = self {
            me.apply_transform(address, from, to)
        } else {
            invalid_op!("transform")
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_json,
        patches::{apply_new, diff, equal},
    };
    use stencila_schema::Integer;

    #[test]
    fn basic() {
        assert!(equal::<Option<Integer>>(&None, &None));
        assert!(equal(&Some(1), &Some(1)));

        assert!(!equal(&None, &Some(1)));
        assert!(!equal(&Some(1), &Some(2)));

        // No diff

        assert_json!(diff::<Option<Integer>>(&None, &None), []);
        assert_json!(diff(&Some(1), &Some(1)), []);

        // None to Some: Add with no key
        let a = None;
        let b = Some("abc".to_string());
        let patch = diff(&a, &b);
        assert_json!(
            patch,
            [{"op": "add", "address": [], "value": "abc".to_string(), "length": 1}]
        );
        assert_json!(apply_new(&a, &patch), b);

        // Some to Some: Add with a key
        let a = Some("a".to_string());
        let b = Some("abc".to_string());
        let patch = diff(&a, &b);
        assert_json!(
            patch,
            [{"op": "add", "address": [1], "value": "bc".to_string(), "length": 2}]
        );
        assert_json!(apply_new(&a, &patch), b);

        // Some to None: Remove with no key
        let a = Some("abc".to_string());
        let b = None;
        let patch = diff(&a, &b);
        assert_json!(
            patch,
            [{"op": "remove", "address": [], "items": 1}]
        );
        assert_json!(apply_new(&a, &patch), b);

        // Some to Some: Remove with key
        let a = Some("abc".to_string());
        let b = Some("ac".to_string());
        let patch = diff(&a, &b);
        assert_json!(
            patch,
            [{"op": "remove", "address": [1], "items": 1}]
        );
        assert_json!(apply_new(&a, &patch), b);

        // Replace
        let a = Some("abc".to_string());
        let b = Some("a@c".to_string());
        let patch = diff(&a, &b);
        assert_json!(
            patch,
            [{"op": "replace", "address": [1], "items": 1, "value": "@", "length": 1}]
        );
        assert_json!(apply_new(&a, &patch), b);
    }
}
