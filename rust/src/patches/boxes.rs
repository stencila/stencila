use super::prelude::*;
use std::{
    hash::Hasher,
    ops::{Deref, DerefMut},
};

/// Implements patching for `Box`
///
/// All methods simply pass throught to the boxed value.
impl<Type: Patchable> Patchable for Box<Type>
where
    Type: Clone + Send + 'static,
{
    patchable_is_same!();

    fn is_equal(&self, other: &Self) -> Result<()> {
        self.deref().is_equal(other)
    }

    fn make_hash<H: Hasher>(&self, state: &mut H) {
        self.deref().make_hash(state)
    }

    patchable_diff!();

    fn diff_same(&self, differ: &mut Differ, other: &Self) {
        self.deref().diff_same(differ, other)
    }

    fn apply_maybe(&mut self, id: &str, patch: &Patch) -> Result<bool> {
        self.deref_mut().apply_maybe(id, patch)
    }

    fn apply_add(&mut self, address: &mut Address, value: &Value) -> Result<()> {
        self.deref_mut().apply_add(address, value)
    }

    fn apply_remove(&mut self, address: &mut Address, items: usize) -> Result<()> {
        self.deref_mut().apply_remove(address, items)
    }

    fn apply_replace(&mut self, address: &mut Address, items: usize, value: &Value) -> Result<()> {
        self.deref_mut().apply_replace(address, items, value)
    }

    fn apply_move(&mut self, from: &mut Address, items: usize, to: &mut Address) -> Result<()> {
        self.deref_mut().apply_move(from, items, to)
    }

    fn apply_transform(&mut self, address: &mut Address, from: &str, to: &str) -> Result<()> {
        self.deref_mut().apply_transform(address, from, to)
    }

    fn from_value(value: &Value) -> Result<Self> {
        Ok(Box::new(Type::from_value(value)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        assert_json,
        patches::{apply_new, diff, equal},
    };
    use stencila_schema::Integer;

    #[test]
    fn basic() -> Result<()> {
        assert!(equal::<Box<Integer>>(&Box::new(1), &Box::new(1)));

        // Add, remove, replace
        let a = Box::new("abcd".to_string());
        let b = Box::new("eacp".to_string());
        let patch = diff(&a, &b);
        assert_json!(
            patch,
            [
                {"type": "Add", "address": [0], "value": "e", "length": 1},
                {"type": "Remove", "address": [2], "items": 1},
                {"type": "Replace", "address": [3], "items": 1, "value": "p", "length": 1}
            ]
        );
        assert_json!(apply_new(&a, &patch)?, b);

        Ok(())
    }
}
