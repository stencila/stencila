use super::prelude::*;
use std::ops::{Deref, DerefMut};

impl<Type: Diffable> Diffable for Box<Type>
where
    Type: Clone + 'static,
{
    // All methods simply pass throught o the boxed value
    
    diffable_is_same!();

    fn is_equal(&self, other: &Self) -> Result<()> {
        self.deref().is_equal(other)
    }

    diffable_diff!();

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

        // Add, remove, replace, move
        let a = Box::new("abcde".to_string());
        let b = Box::new("dace".to_string());
        let patch = diff(&a, &b);
        assert_json!(
            patch,
            [
                {"op": "remove", "keys": [2], "items": 1},
                {"op": "move", "from": [2], "items": 1, "to": [0]},
                //{"op": "replace", "keys": [5], "items": 4, "value": "add"},
               // {"op": "replace", "keys": [14], "items": 6, "value": "n"},
                //{"op": "add", "keys": [16], "value": "w"},
            ]
        );
        assert_json!(apply_new(&a, &patch), b);
    }
}
