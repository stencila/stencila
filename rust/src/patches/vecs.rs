use super::{keys_from_index, prelude::*};
use crate::patches::Add;
use similar::DiffOp;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

/// Implements patching for vectors
impl<Type: Patchable> Patchable for Vec<Type>
where
    Type: Clone + 'static,
{
    patchable_is_same!();

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

    fn make_hash<H: Hasher>(&self, state: &mut H) {
        for item in self {
            item.make_hash(state)
        }
    }

    patchable_diff!();

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

        if self.is_empty() && !other.is_empty() {
            return differ.append(vec![Operation::Add(Add {
                keys: keys_from_index(0),
                value: Box::new(other.clone()),
            })]);
        }

        if !self.is_empty() && other.is_empty() {
            return differ.append(vec![Operation::Remove(Remove {
                keys: keys_from_index(0),
                items: self.len(),
            })]);
        }

        let mapper = Mapper::new(self, other);

        let diff = similar::capture_diff_slices(
            similar::Algorithm::Patience,
            &mapper.a_ids,
            &mapper.b_ids,
        );

        let mut index = 0;
        let mut ops = Vec::new();
        for change in diff {
            match change {
                DiffOp::Equal { len, .. } => index += len,
                DiffOp::Insert {
                    new_index, new_len, ..
                } => {
                    ops.push(Operation::Add(Add {
                        keys: keys_from_index(index),
                        value: Box::new(other[new_index..(new_index + new_len)].to_vec()),
                    }));
                    index += new_len
                }
                DiffOp::Delete {
                    old_index: _,
                    old_len,
                    new_index: _,
                } => {
                    // See if there are any previous `Add` operations with the same value
                    // and if so replace it with a `Move` from here
                    let matched = false;
                    #[cfg(ignore)]
                    for (prev, op) in ops.iter().enumerate() {
                        if let Operation::Add(Add { keys, value }) = op {
                            let add_value = value
                                .deref()
                                .downcast_ref::<Self>()
                                .expect("To be of same type");
                            let remove_value = self[old_index..(old_index + old_len)].to_vec();
                            if add_value.is_equal(&remove_value).is_ok() {
                                ops[prev] = Operation::Move(Move {
                                    from: keys_from_index(new_index),
                                    items: old_len,
                                    to: keys.clone(),
                                });
                                matched = true;
                                break;
                            }
                        }
                    }

                    if !matched {
                        ops.push(Operation::Remove(Remove {
                            keys: keys_from_index(index),
                            items: old_len,
                        }));
                    }
                }
                DiffOp::Replace {
                    old_len,
                    new_index,
                    new_len,
                    ..
                } => {
                    ops.push(Operation::Replace(Replace {
                        keys: keys_from_index(index),
                        items: old_len,
                        value: Box::new(other[new_index..(new_index + new_len)].to_vec()),
                    }));
                    index += new_len;
                }
            }
        }
        differ.append(ops);
    }

    fn apply_add(&mut self, keys: &mut Keys, value: &Box<dyn Any>) {
        if keys.len() == 1 {
            if let Some(Key::Index(index)) = keys.pop_front() {
                let value = if let Some(value) = value.deref().downcast_ref::<Self>() {
                    value
                } else {
                    return invalid_value!();
                };
                *self = [&self[..index], value, &self[index..]].concat().to_vec();
            } else {
                invalid_keys!(keys)
            }
        } else if let Some(Key::Index(index)) = keys.pop_front() {
            if let Some(item) = self.get_mut(index) {
                item.apply_add(keys, value);
            } else {
                invalid_index!(index)
            }
        } else {
            invalid_keys!(keys)
        }
    }

    fn apply_remove(&mut self, keys: &mut Keys, items: usize) {
        if keys.len() == 1 {
            if let Some(Key::Index(index)) = keys.pop_front() {
                *self = [&self[..index], &self[(index + items)..]].concat().to_vec();
            } else {
                invalid_keys!(keys)
            }
        } else if let Some(Key::Index(index)) = keys.pop_front() {
            if let Some(item) = self.get_mut(index) {
                item.apply_remove(keys, items);
            } else {
                invalid_index!(index)
            }
        } else {
            invalid_keys!(keys)
        }
    }

    fn apply_replace(&mut self, keys: &mut Keys, items: usize, value: &Box<dyn Any>) {
        if keys.len() == 1 {
            let value = if let Some(value) = value.deref().downcast_ref::<Self>() {
                value
            } else {
                return invalid_value!();
            };
            if let Some(Key::Index(index)) = keys.pop_front() {
                *self = [&self[..index], value, &self[(index + items)..]]
                    .concat()
                    .to_vec();
            } else {
                invalid_keys!(keys)
            }
        } else if let Some(Key::Index(index)) = keys.pop_front() {
            if let Some(item) = self.get_mut(index) {
                item.apply_replace(keys, items, value);
            } else {
                invalid_index!(index)
            }
        } else {
            invalid_keys!(keys)
        }
    }

    fn apply_move(&mut self, from: &mut Keys, items: usize, to: &mut Keys) {
        if from.len() == 1 {
            if let (Some(Key::Index(from)), Some(Key::Index(to))) =
                (from.pop_front(), to.pop_front())
            {
                *self = if from < to {
                    [
                        &self[..from],
                        &self[(from + items)..to],
                        &self[from..(from + items)],
                        &self[to..],
                    ]
                } else {
                    [
                        &self[..to],
                        &self[from..(from + items)],
                        &self[to..from],
                        &self[(from + items)..],
                    ]
                }
                .concat()
                .to_vec();
            } else {
                invalid_keys!(from)
            }
        } else if let Some(Key::Index(index)) = from.pop_front() {
            if let Some(item) = self.get_mut(index) {
                item.apply_move(from, items, to);
            } else {
                invalid_index!(index)
            }
        } else {
            invalid_keys!(from)
        }
    }

    fn apply_transform(&mut self, keys: &mut Keys, from: &str, to: &str) {
        if keys.len() == 1 {
            todo!()
        } else if let Some(Key::Index(index)) = keys.pop_front() {
            if let Some(item) = self.get_mut(index) {
                item.apply_transform(keys, from, to);
            } else {
                invalid_index!(index)
            }
        } else {
            invalid_keys!(keys)
        }
    }
}

struct Item<'lt, Type>
where
    Type: Patchable,
{
    item: &'lt Type,
}

impl<'lt, Type> Hash for Item<'lt, Type>
where
    Type: Patchable,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.item.make_hash(state)
    }
}

impl<'lt, Type> PartialEq for Item<'lt, Type>
where
    Type: Patchable,
{
    fn eq(&self, other: &Self) -> bool {
        self.item.is_equal(other.item).is_ok()
    }
}

impl<'lt, Type> Eq for Item<'lt, Type> where Type: Patchable {}

struct Mapper<'lt, Type>
where
    Type: Patchable,
{
    #[allow(dead_code)]
    map: HashMap<Item<'lt, Type>, u32>,

    // The `a` vector represented as ids
    a_ids: Vec<u32>,

    // The `b` vector represented as ids
    b_ids: Vec<u32>,
}

impl<'lt, Type> Mapper<'lt, Type>
where
    Type: Patchable,
{
    fn new(a: &'lt [Type], b: &'lt [Type]) -> Self {
        let mut map = HashMap::new();
        let mut id = 0;
        let mut a_ids = Vec::new();
        let mut b_ids = Vec::new();

        for item in a {
            let id = match map.entry(Item { item }) {
                Entry::Occupied(occupied) => *occupied.get(),
                Entry::Vacant(vacant) => {
                    id += 1;
                    *vacant.insert(id)
                }
            };
            a_ids.push(id);
        }

        for item in b {
            let id = match map.entry(Item { item }) {
                Entry::Occupied(occupied) => *occupied.get(),
                Entry::Vacant(vacant) => {
                    id += 1;
                    *vacant.insert(id)
                }
            };
            b_ids.push(id);
        }

        Mapper { map, a_ids, b_ids }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        assert_json, assert_json_eq,
        patches::{apply_new, diff, equal},
    };
    use pretty_assertions::assert_eq;
    use stencila_schema::Integer;

    #[test]
    fn basic() {
        let empty: Vec<Integer> = vec![];
        let a: Vec<Integer> = vec![1];
        let b: Vec<Integer> = vec![1, 2];

        assert!(equal(&empty, &empty));
        assert!(equal(&a, &a));
        assert!(equal(&b, &b));

        assert!(!equal(&empty, &a));
        assert!(!equal(&empty, &b));
        assert!(!equal(&a, &b));

        // Add / replace all

        assert_json!(diff(&empty, &empty), []);

        let patch = diff(&empty, &b);
        assert_json!(
            patch,
            [{ "op": "add", "keys": [0], "value": [1, 2] }]
        );
        assert_json_eq!(apply_new(&empty, &patch), b);

        let patch = diff(&b, &empty);
        assert_json!(
            patch,
            [{ "op": "remove", "keys": [0], "items": 2 }]
        );
        assert_json_eq!(apply_new(&b, &patch), empty);

        // Add

        let a: Vec<Integer> = vec![1];
        let b: Vec<Integer> = vec![1, 2];
        let patch = diff(&a, &b);
        assert_json!(
            patch,
            [{ "op": "add", "keys": [1], "value": [2] }]
        );
        assert_json_eq!(apply_new(&a, &patch), b);

        // Remove

        let a: Vec<Integer> = vec![1, 2];
        let b: Vec<Integer> = vec![1];
        let patch = diff(&a, &b);
        assert_json!(
            patch,
            [{ "op": "remove", "keys": [1], "items": 1 }]
        );
        assert_json_eq!(apply_new(&a, &patch), b);

        // Replace

        let a: Vec<Integer> = vec![1, 2];
        let b: Vec<Integer> = vec![3, 4];
        let patch = diff(&a, &b);
        assert_json!(
            patch,
            [{ "op": "replace", "keys": [0], "items": 2, "value": [3, 4] }]
        );
        assert_json_eq!(apply_new(&a, &patch), b);

        // Move

        let a: Vec<Integer> = vec![1, 3];
        let b: Vec<Integer> = vec![3, 1];
        let patch = diff(&a, &b);
        assert_json!(
            patch, [
                // { "op": "move", "from": [1], "items": 1, "to": [0] }
                { "op": "add", "keys": [0], "value": [3] },
                { "op": "remove", "keys": [2], "items": 1 }
            ]
        );
        assert_json_eq!(apply_new(&a, &patch), b);
    }

    // Regression tests of minimal failing cases found using property testing
    // and elsewhere.

    #[test]
    fn regression_1() {
        let a = vec![7, 0, 4, 1];
        let b = vec![4, 7, 1, 0, 1];
        let patch = diff(&a, &b);
        assert_json!(patch, [
            //{ "op": "move", "from": [2], "items": 1, "to": [0] },
              { "op": "add", "keys": [0], "value": [4] },
            { "op": "add", "keys": [2], "value": [1] },
              { "op": "remove", "keys": [4], "items": 1 },

        ]);
        assert_eq!(apply_new(&a, &patch), b);
    }

    #[test]
    fn regression_2() {
        let a = vec![0, 6, 2, 4, 2];
        let b = vec![2, 2, 4];
        let patch = diff(&a, &b);
        assert_json!(patch, [
            { "op": "remove", "keys": [0], "items": 2 },
            //{ "op": "move", "from": [2], "items": 1, "to": [1] },
              { "op": "add", "keys": [1], "value": [2] },
              { "op": "remove", "keys": [3], "items": 1 },
        ]);
        assert_eq!(apply_new(&a, &patch), b);
    }
}
