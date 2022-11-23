use std::{
    cmp::min,
    collections::{hash_map::Entry, HashMap},
    hash::{Hash, Hasher},
    time::{Duration, Instant},
};

use similar::DiffOp;

use super::prelude::*;

/// The number of seconds before a diff times out (falls back to a `Replace`)
const DIFF_TIMEOUT_SECS: u64 = 1;

/// Implements patching for vectors
impl<Type> Patchable for Vec<Type>
where
    Type: Patchable + Clone + PartialEq + Hash,
{
    /// Generate the difference between two vectors.
    ///
    /// If both vectors are zero length, will generate no operations.
    /// Otherwise, if either of the vectors are of zero length, will generate
    /// a `Replace` operation. Otherwise, will perform a Patience diff on the
    /// vectors.
    fn diff(&self, other: &Self, differ: &mut Differ) {
        // Shortcuts
        if self.is_empty() && other.is_empty() {
            return;
        } else if self.is_empty() && !other.is_empty() {
            return differ.append(vec![Operation::add(
                Address::from(0),
                other.to_value(),
                other.len(),
            )]);
        } else if !self.is_empty() && other.is_empty() {
            return differ.append(vec![Operation::remove(Address::from(0), self.len())]);
        }

        let (me_ids, other_ids) = unique_items(self, other);

        // Do not allow diffs to take too long (but not when testing, for determinism)
        let deadline = if cfg!(test) {
            None
        } else {
            Some(Instant::now() + Duration::from_secs(DIFF_TIMEOUT_SECS))
        };

        let diff_ops = similar::capture_diff_slices_deadline(
            similar::Algorithm::Patience,
            &me_ids,
            &other_ids,
            deadline,
        );

        let mut index = 0;
        let mut ops = Vec::new();
        let mut removes: HashMap<usize, (usize, usize)> = HashMap::new();
        for diff_op in diff_ops {
            match diff_op {
                DiffOp::Equal { len, .. } => index += len,
                DiffOp::Insert {
                    new_index, new_len, ..
                } => {
                    // Attempt to find a previous `Remove` operation, at the top level, with the same value,
                    // remove it, and add a `Move` here. Otherwise add a `Add`.
                    let mut matched = false;
                    let mut shift = 0i32;
                    let added_value = other[new_index..(new_index + new_len)].to_vec();
                    for prev in (0..ops.len()).rev() {
                        let op = &ops[prev];
                        match op {
                            Operation::Add(Add {
                                address, length, ..
                            }) => {
                                if address.len() == 1 {
                                    shift -= *length as i32;
                                }
                            }
                            Operation::Remove(Remove { address, items, .. }) => {
                                if address.len() == 1 {
                                    shift += *items as i32;
                                    let remove_index = if let Slot::Index(remove_index) = address[0]
                                    {
                                        remove_index
                                    } else {
                                        panic!("Should be a index")
                                    };
                                    let removed = removes
                                        .get(&remove_index)
                                        .expect("To have an entry for all removes");
                                    let removed_value =
                                        self[removed.0..(removed.0 + removed.1)].to_vec();
                                    if added_value == removed_value {
                                        ops[prev] = Operation::Move(Move {
                                            from: address.clone(),
                                            items: *items,
                                            to: Address::from(
                                                (index as i32 + shift - *items as i32) as usize,
                                            ),
                                        });
                                        matched = true;
                                        break;
                                    }
                                }
                            }
                            Operation::Replace(Replace {
                                address,
                                items,
                                length,
                                ..
                            }) => {
                                if address.len() == 1 {
                                    shift -= *length as i32 - *items as i32
                                }
                            }
                            _ => {}
                        }
                    }
                    if !matched {
                        ops.push(Operation::add(
                            Address::from(index),
                            added_value.to_value(),
                            new_len,
                        ))
                    }

                    index += new_len
                }
                DiffOp::Delete {
                    old_index, old_len, ..
                } => {
                    // Attempt to find a previous `Add` operations, at the top level, with the same value
                    // and replace it with a `Move` from here.
                    let mut matched = false;
                    let mut shift = 0i32;
                    let removed_value = self[old_index..(old_index + old_len)].to_vec();
                    for prev in (0..ops.len()).rev() {
                        let op = &ops[prev];
                        match op {
                            Operation::Add(Add {
                                address,
                                value,
                                length,
                                ..
                            }) => {
                                if address.len() == 1 {
                                    shift -= *length as i32;
                                    let added_value = Vec::<Type>::from_value(value.clone())
                                        .expect("To be a Vec<Type>");
                                    if added_value == removed_value {
                                        ops[prev] = Operation::Move(Move {
                                            from: Address::from((index as i32 + shift) as usize),
                                            items: old_len,
                                            to: address.clone(),
                                        });
                                        matched = true;
                                        break;
                                    }
                                }
                            }
                            Operation::Remove(Remove { address, items, .. }) => {
                                if address.len() == 1 {
                                    shift += *items as i32
                                }
                            }
                            Operation::Replace(Replace {
                                address,
                                items,
                                length,
                                ..
                            }) => {
                                if address.len() == 1 {
                                    shift -= *length as i32 - *items as i32
                                }
                            }
                            _ => {}
                        }
                    }
                    if !matched {
                        ops.push(Operation::Remove(Remove {
                            address: Address::from(index),
                            items: old_len,
                        }));
                        removes.insert(index, (old_index, old_len));
                    }
                }
                DiffOp::Replace {
                    old_index,
                    old_len,
                    new_index,
                    new_len,
                } => {
                    // Attempt to generate more fine-grained operations for each item instead of
                    // just replacing them all
                    let mut replace_ops = Vec::new();

                    // Diff each item for which there is an old and new item.
                    // Merge `Replace` operations together at this level, rather than have several
                    // replaces at the lower level
                    for item_index in 0usize..min(old_len, new_len) {
                        let mut differ = Differ::default();
                        differ.item(
                            index,
                            &self[old_index + item_index],
                            &other[new_index + item_index],
                        );
                        index += 1;

                        let mut item_ops = differ.ops;
                        // If there is only one operation...
                        if item_ops.len() == 1 {
                            // and its a `Replace`...
                            if let Some(Operation::Replace(Replace { address, .. })) =
                                item_ops.get(0)
                            {
                                // at the root of the item.
                                if address.len() == 1 {
                                    // Otherwise, add it
                                    replace_ops.push(Operation::replace_one(
                                        address.clone(),
                                        vec![other[new_index + item_index].clone()].to_value(),
                                    ));
                                    continue;
                                }
                            }
                        }
                        // Otherwise append to replacement ops
                        replace_ops.append(&mut item_ops);
                    }

                    #[allow(clippy::comparison_chain)]
                    if new_len > old_len {
                        // Add remaining items
                        let length = new_len - old_len;
                        replace_ops.push(Operation::add(
                            Address::from(index),
                            other[(new_index + old_len)..(new_index + new_len)]
                                .to_vec()
                                .to_value(),
                            length,
                        ));
                        index += length;
                    } else if new_len < old_len {
                        // If the last op was a `Replace` at level of the vector, them just add to
                        // the number of items. Otherwise, remove remaining items.
                        let mut remove = true;
                        if let Some(Operation::Replace(Replace { address, items, .. })) =
                            replace_ops.last_mut()
                        {
                            if address.len() == 1 {
                                *items = *items + old_len - new_len;
                                remove = false;
                            }
                        }
                        if remove {
                            replace_ops
                                .push(Operation::remove(Address::from(index), old_len - new_len));
                            removes.insert(index, (old_index, old_len));
                        }
                    }

                    ops.append(&mut replace_ops);
                }
            }
        }
        differ.append(ops);
    }

    fn apply_add(&mut self, address: &mut Address, value: Value) -> Result<()> {
        if address.len() == 1 {
            if let Some(Slot::Index(index)) = address.pop_front() {
                if index > self.len() {
                    bail!(invalid_address::<Self>(&format!(
                        "vector: attempting to add items at index {} but only {} items present",
                        index,
                        self.len(),
                    )))
                }

                let value = Self::from_value(value)?;
                *self = [&self[..index], &value, &self[index..]].concat().to_vec();
            } else {
                bail!(invalid_address::<Self>("first slot should be an index"))
            }
        } else if let Some(Slot::Index(index)) = address.pop_front() {
            if let Some(item) = self.get_mut(index) {
                item.apply_add(address, value)?;
            } else {
                bail!(invalid_slot_index::<Self>(index))
            }
        } else {
            bail!(invalid_address::<Self>(
                "address is empty or does not start with an index slot"
            ))
        }
        Ok(())
    }

    fn apply_remove(&mut self, address: &mut Address, items: usize) -> Result<()> {
        if address.len() == 1 {
            if let Some(Slot::Index(index)) = address.pop_front() {
                if index + items > self.len() {
                    bail!(invalid_address::<Self>(&format!(
                        "vector: attempting to remove {} items at index {} but only {} items present",
                        items, index,
                        self.len(),
                    )))
                }

                *self = [&self[..index], &self[(index + items)..]].concat().to_vec();
            } else {
                bail!(invalid_address::<Self>("first slot should be an index"))
            }
        } else if let Some(Slot::Index(index)) = address.pop_front() {
            if let Some(item) = self.get_mut(index) {
                item.apply_remove(address, items)?;
            } else {
                bail!(invalid_slot_index::<Self>(index))
            }
        } else {
            bail!(invalid_address::<Self>(
                "address is empty or does not start with an index slot"
            ))
        }
        Ok(())
    }

    fn apply_replace(&mut self, address: &mut Address, items: usize, value: Value) -> Result<()> {
        if address.len() == 0 {
            // Replace the entire vector
            let value = Self::from_value(value)?;
            *self = value;
        } else if address.len() == 1 {
            // Replace part of the vector stating from slot
            let value = Self::from_value(value)?;
            if let Some(Slot::Index(index)) = address.pop_front() {
                if index + items > self.len() {
                    bail!(invalid_address::<Self>(&format!(
                        "vector: attempting to replace {} items at index {} but only {} items present",
                        items, index,
                        self.len(),
                    )))
                }

                *self = [&self[..index], &value, &self[(index + items)..]]
                    .concat()
                    .to_vec();
            } else {
                bail!(invalid_address::<Self>("first slot should be an index"))
            }
        } else if let Some(Slot::Index(index)) = address.pop_front() {
            // Apply replace operation to an item in the vector
            if let Some(item) = self.get_mut(index) {
                item.apply_replace(address, items, value)?;
            } else {
                bail!(invalid_slot_index::<Self>(index))
            }
        } else {
            bail!(invalid_address::<Self>(
                "address does not start with an index slot"
            ))
        }
        Ok(())
    }

    fn apply_move(&mut self, from: &mut Address, items: usize, to: &mut Address) -> Result<()> {
        if from.len() == 1 {
            if let (Some(Slot::Index(from)), Some(Slot::Index(to))) =
                (from.pop_front(), to.pop_front())
            {
                if from + items > self.len() {
                    bail!(invalid_address::<Self>(&format!(
                        "vector: attempting to move {} items from index {} but only {} items present",
                        items, from,
                        self.len(),
                    )))
                }
                if to + items > self.len() {
                    bail!(invalid_address::<Self>(&format!(
                        "vector: attempting to move {} items to index {} but only {} items present",
                        items,
                        to,
                        self.len(),
                    )))
                }

                *self = if from < (to + items) {
                    [
                        &self[..from],
                        &self[(from + items)..(to + items)],
                        &self[from..(from + items)],
                        &self[(to + items)..],
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
                bail!(invalid_address::<Self>("first slot should be an index"))
            }
        } else if let Some(Slot::Index(index)) = from.pop_front() {
            if let Some(item) = self.get_mut(index) {
                item.apply_move(from, items, to)?;
            } else {
                bail!(invalid_slot_index::<Self>(index))
            }
        } else {
            bail!(invalid_address::<Self>(
                "address is empty or does not start with an index slot"
            ))
        }
        Ok(())
    }

    fn apply_transform(&mut self, address: &mut Address, from: &str, to: &str) -> Result<()> {
        if address.len() == 1 {
            if let Some(Slot::Index(index)) = address.pop_front() {
                if let Some(item) = self.get_mut(index) {
                    item.apply_transform(address, from, to)?;
                } else {
                    bail!(invalid_slot_index::<Self>(index))
                }
            }
        } else {
            bail!(invalid_address::<Self>(
                "address should have a single index slot"
            ))
        }
        Ok(())
    }

    /*
    /// Cast a [`Value`] to a `Vec<Type>`
    ///
    /// Why? To be able to handle single items of `Type` in addition to a `Vec<Type>`
    fn from_value(value: Value) -> Result<Self>
    where
        Self: Clone + DeserializeOwned + Sized + 'static,
    {
        let instance = if let Some(vec) = value.downcast_ref::<Vec<Type>>() {
            vec.clone()
        } else if let Some(item) = value.downcast_ref::<Type>() {
            vec![item.clone()]
        } else if let Some(json) = value.downcast_ref::<serde_json::Value>() {
            if let Ok(vec) = serde_json::from_value::<Vec<Type>>(json.clone()) {
                vec
            } else if let Ok(item) = serde_json::from_value::<Type>(json.clone()) {
                vec![item]
            } else {
                bail!(
                    "Invalid JSON patch value for type `{}`: {}",
                    type_name::<Self>(),
                    json.to_string()
                )
            }
        } else {
            bail!(invalid_patch_value::<Self>())
        };
        Ok(instance)
    }
    */
}

/// An item used in the hash map for the `unique_items` function below
struct Item<'lt, Type>
where
    Type: Patchable,
{
    item: &'lt Type,
}

impl<'lt, Type> Hash for Item<'lt, Type>
where
    Type: Patchable + Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.item.hash(state)
    }
}

impl<'lt, Type> PartialEq for Item<'lt, Type>
where
    Type: Patchable + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.item == other.item
    }
}

impl<'lt, Type> Eq for Item<'lt, Type> where Type: Patchable + PartialEq {}

/// Generate unique integer ids for items across two vectors using the
/// the `make_hash` trait property.
fn unique_items<Type>(a: &[Type], b: &[Type]) -> (Vec<u32>, Vec<u32>)
where
    Type: Patchable + PartialEq + Hash,
{
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

    (a_ids, b_ids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{apply_new, diff};
    use stencila_schema::{Emphasis, InlineContent, Integer, Strong};
    use test_utils::assert_json_is;

    // Test patches that operate on atomic items (integers) with no
    // pass though.
    #[test]
    fn basic() -> Result<()> {
        let empty: Vec<Integer> = vec![];
        let _a: Vec<Integer> = vec![1];
        let b: Vec<Integer> = vec![1, 2];

        // Add / replace all

        assert_json_is!(diff(&empty, &empty).ops, []);

        let patch = diff(&empty, &b);
        assert_json_is!(
            patch.ops,
            [{ "type": "Add", "address": [0], "value": [1, 2], "length": 2 }]
        );
        assert_eq!(apply_new(&empty, patch)?, b);

        let patch = diff(&b, &empty);
        assert_json_is!(
            patch.ops,
            [{ "type": "Remove", "address": [0], "items": 2 }]
        );
        assert_eq!(apply_new(&b, patch)?, empty);

        let patch = Patch {
            ops: vec![Operation::replace(
                Address::default(),
                2,
                vec![5, 6, 7].to_value(),
                3,
            )],
            ..Default::default()
        };
        assert_eq!(apply_new(&vec![1, 2], patch)?, vec![5, 6, 7]);

        // Add

        let a: Vec<Integer> = vec![1];
        let b: Vec<Integer> = vec![1, 2];
        let patch = diff(&a, &b);
        assert_json_is!(
            patch.ops,
            [{ "type": "Add", "address": [1], "value": [2], "length": 1 }]
        );
        assert_eq!(apply_new(&a, patch)?, b);

        // Remove

        let a: Vec<Integer> = vec![1, 2];
        let b: Vec<Integer> = vec![1];
        let patch = diff(&a, &b);
        assert_json_is!(
            patch.ops,
            [{ "type": "Remove", "address": [1], "items": 1 }]
        );
        assert_eq!(apply_new(&a, patch)?, b);

        // Replace

        let a: Vec<Integer> = vec![1, 2];
        let b: Vec<Integer> = vec![3, 4];
        let patch = diff(&a, &b);
        assert_json_is!(
            patch.ops,
            [
                { "type": "Replace", "address": [0], "items": 1, "value": [3], "length": 1 },
                { "type": "Replace", "address": [1], "items": 1, "value": [4], "length": 1 }
            ]
        );
        assert_eq!(apply_new(&a, patch)?, b);

        // Move

        let a: Vec<Integer> = vec![1, 3];
        let b: Vec<Integer> = vec![3, 1];
        let patch = diff(&a, &b);
        assert_json_is!(
            patch.ops, [
                { "type": "Move", "from": [1], "items": 1, "to": [0] }
            ]
        );
        assert_eq!(apply_new(&a, patch)?, b);

        let a: Vec<Integer> = vec![1, 2, 3, 4];
        let b: Vec<Integer> = vec![2, 3, 1, 4];
        let patch = diff(&a, &b);
        assert_json_is!(
            patch.ops, [
                { "type": "Move", "from": [0], "items": 1, "to": [2] }
            ]
        );
        assert_eq!(apply_new(&a, patch)?, b);

        let a: Vec<Integer> = vec![1, 2, 3, 4];
        let b: Vec<Integer> = vec![3, 4, 1, 2];
        let patch = diff(&a, &b);
        assert_json_is!(
            patch.ops, [
                { "type": "Move", "from": [2], "items": 2, "to": [0] }
            ]
        );
        assert_eq!(apply_new(&a, patch)?, b);

        Ok(())
    }

    // Test patches that operate on compound items (strings) to check that
    // fine grained operations are generated for each item and passed through on apply.
    #[test]
    fn item_ops() -> Result<()> {
        // Add

        let a = vec!["a".to_string()];
        let b = vec!["ab".to_string()];
        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [0, 1], "value": "b", "length": 1 },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        // Remove

        let a = vec!["ab".to_string()];
        let b = vec!["a".to_string()];
        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Remove", "address": [0, 1], "items": 1 },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        // Replace

        let a = vec!["a".to_string()];
        let b = vec!["b".to_string()];
        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Replace", "address": [0, 0], "items": 1, "value": "b", "length": 1 },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        // Transform

        let a = vec![InlineContent::Emphasis(Emphasis {
            content: vec![InlineContent::String("word".to_string())],
            ..Default::default()
        })];
        let b = vec![InlineContent::Strong(Strong {
            content: vec![InlineContent::String("word".to_string())],
            ..Default::default()
        })];
        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Transform", "address": [0], "from": "Emphasis", "to": "Strong" },
        ]);
        assert_json_is!(apply_new(&a, patch)?, b);

        Ok(())
    }

    // As above, but with an extra `Add` or `Remove` as needed.
    #[test]
    fn item_ops_plus() -> Result<()> {
        let a = vec!["a".to_string()];
        let b = vec!["ab".to_string(), "c".to_string()];

        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [0, 1], "value": "b", "length": 1 },
            { "type": "Add", "address": [1], "value": ["c"], "length": 1 },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        let patch = diff(&b, &a);
        assert_json_is!(patch.ops, [
            { "type": "Remove", "address": [0, 1], "items": 1 },
            { "type": "Remove", "address": [1], "items": 1 },
        ]);
        assert_eq!(apply_new(&b, patch)?, a);

        Ok(())
    }

    // Regression tests of minimal failing cases found using property testing
    // and elsewhere.

    #[test]
    fn regression_1() -> Result<()> {
        let a = vec![7, 0, 4, 1];
        let b = vec![4, 7, 1, 0, 1];
        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Move", "from": [2], "items": 1, "to": [0] },
            { "type": "Add", "address": [2], "value": [1], "length": 1 },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        Ok(())
    }

    #[test]
    fn regression_2() -> Result<()> {
        let a = vec![0, 6, 2, 4, 2];
        let b = vec![2, 2, 4];
        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Remove", "address": [0], "items": 2 },
            { "type": "Move", "from": [2], "items": 1, "to": [1] },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        Ok(())
    }

    #[test]
    fn regression_3() -> Result<()> {
        let a = vec!["".to_string(), "".to_string()];
        let b = vec![
            "a".to_string(),
            "a".to_string(),
            "a".to_string(),
            "".to_string(),
            "a".to_string(),
            "a".to_string(),
        ];
        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [0], "value": ["a", "a", "a"], "length": 3 },
            { "type": "Add", "address": [4, 0], "value": "a", "length": 1 },
            { "type": "Add", "address": [5], "value": ["a"], "length": 1 },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        Ok(())
    }

    #[test]
    fn regression_4() -> Result<()> {
        let a = vec![6, 1, 1, 1];
        let b = vec![2, 2, 0];
        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Replace", "address": [0], "items": 1, "value": [2], "length": 1 },
            { "type": "Replace", "address": [1], "items": 1, "value": [2], "length": 1 },
            { "type": "Replace", "address": [2], "items": 2, "value": [0], "length": 1 },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        Ok(())
    }

    #[test]
    fn regression_5() -> Result<()> {
        let a = vec!["c".to_string(), "".to_string(), "d".to_string()];
        let b = vec!["cd".to_string(), "a".to_string(), "".to_string()];
        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [0, 1], "value": "d", "length": 1 },
            { "type": "Add", "address": [1], "value": ["a"], "length": 1 },
            { "type": "Remove", "address": [3], "items": 1 },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        Ok(())
    }

    #[test]
    fn regression_6() -> Result<()> {
        let a = vec!["".to_string(), "a".to_string(), "".to_string()];
        let b = vec![
            "b".to_string(),
            "".to_string(),
            "".to_string(),
            "b".to_string(),
        ];
        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [0], "value": ["b"], "length": 1 },
            { "type": "Remove", "address": [2], "items": 1 },
            { "type": "Add", "address": [3], "value": ["b"], "length": 1 },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        Ok(())
    }

    #[test]
    fn regression_7() -> Result<()> {
        let a = vec![1, 7, 3];
        let b = vec![7, 3, 1];
        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Move", "from": [0], "items": 1, "to": [2] },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        Ok(())
    }

    #[test]
    fn regression_8() -> Result<()> {
        let a = vec![3, 0, 7];
        let b = vec![0, 1, 7, 3];
        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Move", "from": [0], "items": 1, "to": [2] },
            { "type": "Add", "address": [1], "value": [1], "length": 1 },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        Ok(())
    }

    #[test]
    fn regression_9() -> Result<()> {
        let a = vec![
            "".to_string(),
            "".to_string(),
            "".to_string(),
            "d".to_string(),
        ];
        let b = vec!["a".to_string(), "d".to_string(), "".to_string()];
        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [0, 0], "value": "a", "length": 1 },
            { "type": "Remove", "address": [1], "items": 2 },
            { "type": "Add", "address": [2], "value": [""], "length": 1 },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        Ok(())
    }

    // Move ops generated by diff always seem to have `to` less than `from`.
    // However, manually written patches with the reverse were failing (but not
    // picked up by prop tests because never generated)
    #[test]
    fn regression_10() -> Result<()> {
        let a = vec![1, 2, 3, 4];

        let patch = Patch {
            ops: vec![Operation::Move(Move {
                from: Address::from(0),
                to: Address::from(1),
                items: 1,
            })],
            ..Default::default()
        };
        assert_eq!(apply_new(&a, patch)?, vec![2, 1, 3, 4]);

        let patch = Patch {
            ops: vec![Operation::Move(Move {
                from: Address::from(0),
                to: Address::from(2),
                items: 2,
            })],
            ..Default::default()
        };
        assert_eq!(apply_new(&a, patch)?, vec![3, 4, 1, 2]);

        let patch = Patch {
            ops: vec![Operation::Move(Move {
                from: Address::from(2),
                to: Address::from(3),
                items: 1,
            })],
            ..Default::default()
        };
        assert_eq!(apply_new(&a, patch)?, vec![1, 2, 4, 3]);

        Ok(())
    }
}
