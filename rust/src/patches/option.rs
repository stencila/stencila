use super::prelude::*;
use std::ops::Deref;

impl<Type: Diffable> Diffable for Option<Type>
where
    Type: Clone + 'static,
{
    diffable_is_same!(Option<Type>);

    fn is_equal(&self, other: &Self) -> Result<()> {
        match (self, other) {
            (None, None) => Ok(()),
            (None, Some(_)) | (Some(_), None) => bail!(Error::NotEqual),
            (Some(me), Some(other)) => me.is_equal(other),
        }
    }

    diffable_diff!(Option<Type>);

    fn diff_same(&self, differ: &mut Differ, other: &Self) {
        match (self, other) {
            (None, None) => (),
            (None, Some(value)) => differ.add(value),
            (Some(_), None) => differ.remove(),
            (Some(me), Some(other)) => me.diff_same(differ, other),
        }
    }

    // For `Add` and `Remove`, if there are no keys then apply here, otherwise
    // pass operation through to any value

    fn apply_add(&mut self, keys: &mut Keys, value: &Box<dyn Any>) {
        if keys.is_empty() {
            if let Some(value) = value.deref().downcast_ref::<Type>() {
                *self = Some(value.clone())
            } else {
                invalid_value!()
            }
        } else if let Some(me) = self {
            me.apply_add(keys, value)
        } else {
            invalid_keys!(keys)
        }
    }

    fn apply_remove(&mut self, keys: &mut Keys, items: usize) {
        if keys.is_empty() {
            *self = None
        } else if let Some(me) = self {
            me.apply_remove(keys, items)
        } else {
            invalid_keys!(keys)
        }
    }

    fn apply_replace(&mut self, keys: &mut Keys, items: usize, value: &Box<dyn Any>) {
        if let Some(me) = self {
            me.apply_replace(keys, items, value)
        } else {
            invalid_op!("replace")
        }
    }

    fn apply_move(&mut self, from: &mut Keys, items: usize, to: &mut Keys) {
        if let Some(me) = self {
            me.apply_move(from, items, to)
        } else {
            invalid_op!("move")
        }
    }

    fn apply_transform(&mut self, keys: &mut Keys, from: &str, to: &str) {
        if let Some(me) = self {
            me.apply_transform(keys, from, to)
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
            [{"op": "add", "keys": [], "value": "abc".to_string()}]
        );
        assert_json!(apply_new(&a, &patch), b);

        // Some to Some: Add with a key
        let a = Some("a".to_string());
        let b = Some("abc".to_string());
        let patch = diff(&a, &b);
        assert_json!(
            patch,
            [{"op": "add", "keys": [1], "value": "bc".to_string()}]
        );
        assert_json!(apply_new(&a, &patch), b);

        // Some to None: Remove with no key
        let a = Some("abc".to_string());
        let b = None;
        let patch = diff(&a, &b);
        assert_json!(
            patch,
            [{"op": "remove", "keys": [], "items": 1}]
        );
        assert_json!(apply_new(&a, &patch), b);

        // Some to Some: Remove with key
        let a = Some("abc".to_string());
        let b = Some("ac".to_string());
        let patch = diff(&a, &b);
        assert_json!(
            patch,
            [{"op": "remove", "keys": [1], "items": 1}]
        );
        assert_json!(apply_new(&a, &patch), b);

        // Replace
        let a = Some("abc".to_string());
        let b = Some("a@c".to_string());
        let patch = diff(&a, &b);
        assert_json!(
            patch,
            [{"op": "replace", "keys": [1], "items": 1, "value": "@"}]
        );
        assert_json!(apply_new(&a, &patch), b);

        // Move
        let a = Some("abc".to_string());
        let b = Some("bca".to_string());
        let patch = diff(&a, &b);
        assert_json!(
            patch,
            [{"op": "move", "from": [0], "items": 1, "to": [2]}]
        );
        assert_json!(apply_new(&a, &patch), b);
    }
}
