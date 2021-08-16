use super::prelude::*;
use std::cmp::min;
use std::ops::Deref;

impl<Type: Diffable> Diffable for Vec<Type>
where
    Type: Clone + 'static,
{
    diffable_is_same!(Vec<Type>);
    diffable_diff!(Vec<Type>);

    /// Is this vector equal to another?
    ///
    /// Checks that the vectors of equal length first, and then
    /// compares each item with early return on the first difference.
    fn is_equal(&self, other: &Self) -> Result<()> {
        if self.len() != other.len() {
            bail!(Error::NotEqual)
        }
        for index in 0..self.len() {
            self[index].is_equal(&other[index])?
        }
        Ok(())
    }

    /// Generate the difference between two vectors.
    ///
    /// If both vectors are zero length, will generate no operations.
    /// Otherwise, if either of the vectors are of zero length, will generate
    /// a `Replace` operation. Otherwise, will perform a Patience diff on the
    /// vectors.
    fn diff_same(&self, differ: &mut Differ, other: &Self) {
        if self.is_empty() && other.is_empty() {
            return;
        }

        if (self.is_empty() && !other.is_empty()) || (!self.is_empty() && other.is_empty()) {
            differ.replace(other)
        }

        // TODO This is temporary; implement Patience diff
        for index in 0..min(self.len(), other.len()) {
            differ.item(index, &self[index], &other[index])
        }
    }

    fn apply_remove(&mut self, keys: &mut Keys, items: usize) {
        todo!()
    }

    fn apply_replace(&mut self, keys: &mut Keys, items: usize, value: &Box<dyn Any>) {
        // TODO publish change with id of object
        if let Some(key) = keys.pop_front() {
            if let Key::Index(index) = key {
                if let Some(item) = self.get_mut(index) {
                    item.apply_replace(keys, items, value);
                } else {
                    unreachable!(
                        "Invalid index for type 'Vec<{}>': {}",
                        type_name::<Type>(),
                        index
                    )
                }
            } else {
                unreachable!("Invalid key for vector: {:?}", key)
            }
        } else {
            let value = value.deref();
            if let Some(value) = value.downcast_ref::<Self>() {
                *self = value.deref().to_vec();
            } else {
                unreachable!("Invalid replacement value for Vec<{}>", type_name::<Type>())
            }
        }
    }

    fn apply_transform(&mut self, keys: &mut Keys, from: &str, to: &str) {
        if keys.is_empty() {
            todo!()
        } else {
            if let Some(Key::Index(index)) = keys.pop_front() {
                if let Some(item) = self.get_mut(index) {
                    return item.apply_transform(keys, from, to);
                }
                unreachable!(
                    "Invalid index for type 'Vec<{}>': {}",
                    type_name::<Type>(),
                    index
                )
            }
            unreachable!(
                "Invalid keys for type 'Vec<{}>': {:?}",
                type_name::<Type>(),
                keys
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_json,
        patches::{apply_new, diff, equal},
    };
    use pretty_assertions::assert_eq;
    use stencila_schema::{Emphasis, InlineContent, Integer, Paragraph};

    #[test]
    fn test_vector() {
        let empty: Vec<Integer> = vec![];
        let a: Vec<Integer> = vec![1, 2, 3];
        let b: Vec<Integer> = vec![3, 2, 1];

        assert!(equal(&empty, &empty));
        assert!(equal(&a, &a));
        assert!(equal(&b, &b));

        assert!(!equal(&empty, &a));
        assert!(!equal(&empty, &b));
        assert!(!equal(&a, &b));

        assert_json!(diff(&empty, &empty), []);
        assert_json!(
            diff(&empty, &a),
            [{
                "op": "replace",
                "keys": [],
                "items": 1,
                "value": "<unserialized type>"
            }]
        );
        assert_json!(
            diff(&a, &empty),
            [{
                "op": "replace",
                "keys": [],
                "items": 1,
                "value": "<unserialized type>"
            }]
        );
    }
}
