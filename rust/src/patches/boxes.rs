use super::prelude::*;
use std::ops::{Deref, DerefMut};

/// Implements patching for `Box`
///
/// All methods simply pass throught to the boxed value.
impl<Type: Patchable> Patchable for Box<Type>
where
    Type: Clone + 'static,
{
    patchable_is_same!();

    fn is_equal(&self, other: &Self) -> Result<()> {
        self.deref().is_equal(other)
    }

    patchable_diff!();

    fn diff_same(&self, differ: &mut Differ, other: &Self) {
        self.deref().diff_same(differ, other)
    }

    fn apply_add(&mut self, keys: &mut Keys, value: &Box<dyn Any>) {
        self.deref_mut().apply_add(keys, value)
    }

    fn apply_remove(&mut self, keys: &mut Keys, items: usize) {
        self.deref_mut().apply_remove(keys, items)
    }

    fn apply_replace(&mut self, keys: &mut Keys, items: usize, value: &Box<dyn Any>) {
        self.deref_mut().apply_replace(keys, items, value)
    }

    fn apply_move(&mut self, from: &mut Keys, items: usize, to: &mut Keys) {
        self.deref_mut().apply_move(from, items, to)
    }

    fn apply_transform(&mut self, keys: &mut Keys, from: &str, to: &str) {
        self.deref_mut().apply_transform(keys, from, to)
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
        assert!(equal::<Box<Integer>>(&Box::new(1), &Box::new(1)));

        // Add, remove, replace
        let a = Box::new("abcd".to_string());
        let b = Box::new("eacp".to_string());
        let patch = diff(&a, &b);
        assert_json!(
            patch,
            [
                {"op": "add", "keys": [0], "value": "e"},
                {"op": "remove", "keys": [2], "items": 1},
                {"op": "replace", "keys": [3], "items": 1, "value": "p"}
            ]
        );
        assert_json!(apply_new(&a, &patch), b);
    }
}
