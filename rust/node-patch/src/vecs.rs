//! Patching for [`Vec`]s

use std::{
    hash::Hash,
    time::{Duration, Instant},
};

use common::itertools::Itertools;
use similar::{algorithms::IdentifyDistinct, capture_diff_deadline, Change, ChangeTag};

use crate::value::Values;

use super::prelude::*;

/// Implements patching for [`Vec`]s
impl<Type> Patchable for Vec<Type>
where
    Type: Patchable + Clone + Eq + Hash,
{
    /// Generate operations for the difference between two vectors
    ///
    /// If both vectors are empty, will generate no operations.
    /// If self is empty, will generate an `Add` operation for each of other's items.
    /// If other is empty, will generate a `Remove` operation for each of self's items.
    /// Otherwise, performs a Patience diff on the vectors.
    fn diff(&self, other: &Self, differ: &mut Differ) {
        // No difference
        if self.is_empty() && other.is_empty() {
            return;
        }

        // Self is empty: create an `Add` operation for each of other's items
        if self.is_empty() && !other.is_empty() {
            for (index, item) in other.iter().enumerate() {
                differ.push(Operation::add(Address::from(index), item.to_value()));
            }
            return;
        }

        // Other is empty: create a `Remove` operation for each of self's items
        // Note that the address is always 0.
        if !self.is_empty() && other.is_empty() {
            for _index in 0..self.len() {
                differ.push(Operation::remove(Address::from(0)));
            }
            return;
        }

        if Type::is_atomic() {
            diff_vecs_atomic(self, other, differ)
        } else {
            diff_vecs_non_atomic(self, other, differ)
        }
    }

    fn apply_add(&mut self, address: &mut Address, value: Value) -> Result<()> {
        if let Some(Slot::Index(index)) = address.pop_front() {
            let len = self.len();
            if address.is_empty() {
                if index > len {
                    bail!(invalid_address::<Self>(&format!(
                        "attempting to add item at `{index}` when length is `{len}`"
                    )))
                }

                let value = Type::from_value(value)?;
                self.insert(index, value);
            } else if let Some(item) = self.get_mut(index) {
                item.apply_add(address, value)?;
            } else {
                bail!(invalid_slot_index::<Self>(index))
            }
            Ok(())
        } else {
            bail!(invalid_address::<Self>(
                "address is empty or does not start with an index slot"
            ))
        }
    }

    fn apply_add_many(&mut self, address: &mut Address, values: Values) -> Result<()> {
        if let Some(Slot::Index(index)) = address.pop_front() {
            let len = self.len();
            if address.is_empty() {
                if index > len {
                    bail!(invalid_address::<Self>(&format!(
                        "attempting to add items at `{index}` when length is `{len}`"
                    )))
                }

                let values = Type::from_values(values)?;
                self.splice(index..index, values);
            } else if let Some(item) = self.get_mut(index) {
                item.apply_add_many(address, values)?;
            } else {
                bail!(invalid_slot_index::<Self>(index))
            }
            Ok(())
        } else {
            bail!(invalid_address::<Self>(
                "address is empty or does not start with an index slot"
            ))
        }
    }

    fn apply_remove(&mut self, address: &mut Address) -> Result<()> {
        if let Some(Slot::Index(index)) = address.pop_front() {
            let len = self.len();
            if address.is_empty() {
                if index > len {
                    bail!(invalid_address::<Self>(&format!(
                        "attempting to remove item at `{index}` when length is `{len}`"
                    )))
                }

                self.remove(index);
            } else if let Some(item) = self.get_mut(index) {
                item.apply_remove(address)?;
            } else {
                bail!(invalid_slot_index::<Self>(index))
            }
            Ok(())
        } else {
            bail!(invalid_address::<Self>(
                "address is empty or does not start with an index slot"
            ))
        }
    }

    fn apply_remove_many(&mut self, address: &mut Address, items: usize) -> Result<()> {
        if let Some(Slot::Index(index)) = address.pop_front() {
            let len = self.len();
            if address.is_empty() {
                if index > len {
                    bail!(invalid_address::<Self>(&format!(
                        "attempting to remove items at `{index}` when length is `{len}`"
                    )))
                }

                self.drain(index..(index + items));
            } else if let Some(item) = self.get_mut(index) {
                item.apply_remove_many(address, items)?;
            } else {
                bail!(invalid_slot_index::<Self>(index))
            }
            Ok(())
        } else {
            bail!(invalid_address::<Self>(
                "address is empty or does not start with an index slot"
            ))
        }
    }

    fn apply_replace(&mut self, address: &mut Address, value: Value) -> Result<()> {
        if let Some(Slot::Index(index)) = address.pop_front() {
            let len = self.len();
            if address.is_empty() {
                if index > len {
                    bail!(invalid_address::<Self>(&format!(
                        "attempting to replace item at `{index}` when length is `{len}`"
                    )))
                }

                let value = Type::from_value(value)?;
                self[index] = value;
            } else if let Some(item) = self.get_mut(index) {
                item.apply_replace(address, value)?;
            } else {
                bail!(invalid_slot_index::<Self>(index))
            }
            Ok(())
        } else {
            bail!(invalid_address::<Self>(
                "address is empty or does not start with an index slot"
            ))
        }
    }

    fn apply_replace_many(
        &mut self,
        address: &mut Address,
        items: usize,
        values: Values,
    ) -> Result<()> {
        if let Some(Slot::Index(index)) = address.pop_front() {
            let len = self.len();
            if address.is_empty() {
                if index > len {
                    bail!(invalid_address::<Self>(&format!(
                        "attempting to replace item at `{index}` when length is `{len}`"
                    )))
                }

                let values = Type::from_values(values)?;
                self.splice(index..index + items, values);
            } else if let Some(item) = self.get_mut(index) {
                item.apply_replace_many(address, items, values)?;
            } else {
                bail!(invalid_slot_index::<Self>(index))
            }
            Ok(())
        } else {
            bail!(invalid_address::<Self>(
                "address is empty or does not start with an index slot"
            ))
        }
    }

    fn apply_move(&mut self, from: &mut Address, to: &mut Address) -> Result<()> {
        if from.len() == 1 {
            if let (Some(Slot::Index(from)), Some(Slot::Index(to))) =
                (from.pop_front(), to.pop_front())
            {
                let len = self.len();
                if from >= len {
                    bail!(invalid_address::<Self>(&format!(
                        "attempting to move item from `{from}` when length is `{len}`"
                    )))
                }
                if to >= len {
                    bail!(invalid_address::<Self>(&format!(
                        "attempting to move item to `{to}` when length is `{len}`"
                    )))
                }

                let value = self.remove(from);
                self.insert(to, value);
            } else {
                bail!(invalid_address::<Self>("first slot should be an index"))
            }
        } else if let Some(Slot::Index(index)) = from.pop_front() {
            if let Some(item) = self.get_mut(index) {
                item.apply_move(from, to)?;
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
}

/// Generate `similar::Change`s for two sequences
fn diff_changes<Type: Patchable + Clone + Eq + Hash>(
    old: &[Type],
    new: &[Type],
    timeout_secs: u64,
) -> Vec<Change<Type>> {
    // Do not allow diffs to take too long (but not when testing, for determinism)
    let deadline = if cfg!(test) {
        None
    } else {
        Some(Instant::now() + Duration::from_secs(timeout_secs))
    };

    // Generate integer ids, that are unique across both self and other, for each item
    // The `similar` crate generally recommends only doing that for large sequences
    // (the text diffing does it for those greater than 100). But we do it here to avoid having
    // to make Type be `Ord` (which is what `capture_diff_deadline` requires).
    let identities = IdentifyDistinct::<u32>::new(old, 0..old.len(), new, 0..new.len());

    // Generate `similar::DiffOps`s and flatten them into `Similar::Change`s
    let diffops = capture_diff_deadline(
        similar::Algorithm::Patience,
        identities.old_lookup(),
        identities.old_range(),
        identities.new_lookup(),
        identities.new_range(),
        deadline,
    );
    diffops
        .iter()
        .flat_map(|diffop| diffop.iter_changes(old, new))
        .collect_vec()
}

/// Diff two vectors of atomics
///
/// Uses a Patience diff with a `Remove` followed by an `Add`
/// in the same position transformed into a `Replace`.
fn diff_vecs_atomic<Type: Patchable + Clone + Eq + Hash>(
    old: &[Type],
    new: &[Type],
    differ: &mut Differ,
) {
    // Get the changes using Patience diff
    let changes = diff_changes(old, new, differ.timeout);

    // The position of the operation in the new vector (because ops are applied sequentially
    // this is not the same as the change's new_index)
    let mut position = 0;

    // The position of the last delete
    let mut last_delete_position = usize::MAX;

    // Transform each change into an `Add`, `Remove`, or `Replace` (if a delete is
    // followed by an insert in the same position)
    for change in changes {
        match change.tag() {
            ChangeTag::Equal => {
                position += 1;
            }

            ChangeTag::Insert => {
                let address = Address::from(position);
                let value = change.value().to_value();
                if differ.ops_allowed.contains(OperationFlag::Replace)
                    && last_delete_position == position
                {
                    differ.pop();
                    differ.push(Operation::replace(address, value));
                } else {
                    differ.push(Operation::add(address, value));
                };
                position += 1;
            }

            ChangeTag::Delete => {
                differ.push(Operation::remove(Address::from(position)));
                last_delete_position = position;
            }
        }
    }
}

/// Diff two vectors of non-atomics
///
/// Does an initial Patience diff and then finds the least cost pairs
/// (in terms of number of operations) to transform pairs of `Remove`/`Add`
/// operations into a `Move`, possibly followed by operations within the item,
/// or a `Copy`.
fn diff_vecs_non_atomic<Type: Patchable + Clone + Eq + Hash>(
    old: &[Type],
    new: &[Type],
    differ: &mut Differ,
) {
    diff_vecs_atomic(old, new, differ)

    /*
    TODO: Complete this WIP improved diffing for non-atomic types and reinstate the related
          tests that are currently ignored

    // Get the changes using Patience diff
    let changes = diff_changes(old, new, differ.timeout);

    // Collect the indices of the `Add`s (inserts) and `Remove`s (deletes) for potential pairing
    let mut adds = Vec::new();
    let mut removes = Vec::new();
    for (index, change) in changes.iter().enumerate() {
        match change.tag() {
            ChangeTag::Equal => {}
            ChangeTag::Insert => {
                adds.push(index);
            }
            ChangeTag::Delete => {
                removes.push(index);
            }
        }
    }

    // Determine the number of post-`Move` operations required to transform each pair of `Remove`/`Add` ops.
    // Record delta of positions of adds and removes for tie breaking in the following sort step.
    let mut post_move_ops = Vec::with_capacity(adds.len() * removes.len());
    for add_index in adds {
        let add = &changes[add_index];
        let add_value = add.value();
        let add_position = add.new_index().expect("Add to have new index");

        for remove_index in &removes {
            let remove = &changes[*remove_index];
            let remove_value = remove.value();
            let remove_position = remove.old_index().expect("Remove to have old index");

            let delta = remove_position as i64 - add_position as i64;

            let ops = if add_value != remove_value {
                diff(&remove_value, &add_value).ops
            } else {
                vec![]
            };

            post_move_ops.push((add_index, *remove_index, add_position, remove_position, ops));
        }
    }

    // Sort by the number of ops with tie breaks based on delta
    post_move_ops.sort_by(
        |(.., a_add_position, a_remove_position, a_ops),
         (.., b_add_position, b_remove_position, b_ops)| {
            if a_ops.len() == b_ops.len() {
                let a_dist = (*a_add_position as i64 - *a_remove_position as i64).abs();
                let b_dist = (*b_add_position as i64 - *b_remove_position as i64).abs();
                a_dist.cmp(&b_dist)
            } else {
                a_ops.len().cmp(&b_ops.len())
            }
        },
    );

    // Iterate over the pairs in post_move_ops and pick the pairs that have
    // the lowest cost (in terms of number of operations / distance).
    let mut moves = vec![None; changes.len()];
    for (add_index, remove_index, .., remove_position, ops) in post_move_ops {
        if moves[add_index].is_none() && moves[remove_index].is_none() {
            moves[add_index] = Some((remove_index, remove_position, ops));
            moves[remove_index] = Some((add_index, 0, vec![]));
        }
    }

    // The position of the operation in the new vector (because ops are applied sequentially
    // this is not the same as the change's new_index)
    let mut position = 0usize;

    let mut shift = 0;

    // Transform each change into a `Move` (plus post move ops) `Add`, `Remove`,
    // or `Replace` (if a delete is followed by an insert in the same position)
    for (index, change) in changes.iter().enumerate() {
        println!("{index} {} {position} {:?}", change.tag(), moves[index]);

        match change.tag() {
            ChangeTag::Equal => {
                position += 1;
            }

            ChangeTag::Insert => {
                if let Some((remove_index, remove_position, post_move_ops)) = &moves[index] {
                    let mut from_position = (*remove_position as i64) + shift;
                    if from_position < 0 {
                        from_position = 0;
                    }
                    let from_position = from_position as usize;
                    let to_position = position;

                    if from_position != to_position {
                        differ.push(Operation::mov(
                            Address::from(from_position),
                            Address::from(to_position),
                        ));
                    }

                    differ.enter(Slot::Index(to_position));
                    differ.append(post_move_ops.clone());
                    differ.exit();

                    if *remove_position > position {
                        shift += 1;
                        position += 1;
                    } else {
                        shift -= 1;
                        position -= 1;
                    }
                } else {
                    differ.push(Operation::add(
                        Address::from(position),
                        change.value().to_value(),
                    ));

                    shift += 1;
                    position += 1;
                }
            }

            ChangeTag::Delete => {
                if let Some((add_index, ..)) = &moves[index] {
                    // Delete is part of a move, so don't do anything
                    if *add_index > index {
                        position += 1;
                    }
                } else {
                    // Not part of a move, so push a `Remove` at this position to the differ
                    differ.push(Operation::remove(Address::from(position)));
                    shift -= 1;
                }
            }
        }
    }

    */
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{apply_new, diff};
    use stencila_schema::{Emphasis, InlineContent, Integer, Strong};
    use test_utils::assert_json_is;
    use utils::vec_string;

    // Test patches that operate on atomic items (in this case, integers) with no
    // operations at the within item level
    #[test]
    fn atomic() -> Result<()> {
        // Add/remove all

        let empty: Vec<Integer> = vec![];
        let not_empty: Vec<Integer> = vec![1, 2];

        assert_json_is!(diff(&empty, &empty).ops, []);

        let patch = diff(&empty, &not_empty);
        let patch_compact = patch.compact(OperationFlag::all());
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [0], "value": 1 },
            { "type": "Add", "address": [1], "value": 2 }
        ]);
        assert_json_is!(patch_compact.ops, [
            { "type": "AddMany", "address": [0], "values": [1, 2] }
        ]);
        assert_eq!(apply_new(&empty, patch)?, not_empty);
        assert_eq!(apply_new(&empty, patch_compact)?, not_empty);

        let patch = diff(&not_empty, &empty);
        let patch_compact = patch.compact(OperationFlag::all());
        assert_json_is!(patch.ops, [
            { "type": "Remove", "address": [0] },
            { "type": "Remove", "address": [0] }
        ]);
        assert_json_is!(patch_compact.ops, [
            { "type": "RemoveMany", "address": [0], "items": 2 }
        ]);
        assert_eq!(apply_new(&not_empty, patch)?, empty);
        assert_eq!(apply_new(&not_empty, patch_compact)?, empty);

        // Add

        let a: Vec<Integer> = vec![1];
        let b: Vec<Integer> = vec![1, 2, 3];
        let patch = diff(&a, &b);
        let patch_compact = patch.compact(OperationFlag::all());
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [1], "value": 2 },
            { "type": "Add", "address": [2], "value": 3 }
        ]);
        assert_json_is!(patch_compact.ops, [
            { "type": "AddMany", "address": [1], "values": [2, 3] }
        ]);
        assert_eq!(apply_new(&a, patch)?, b);
        assert_eq!(apply_new(&a, patch_compact)?, b);

        // Remove

        let a: Vec<Integer> = vec![1, 2, 3];
        let b: Vec<Integer> = vec![1];
        let patch = diff(&a, &b);
        let patch_compact = patch.compact(OperationFlag::all());
        assert_json_is!(patch.ops, [
            { "type": "Remove", "address": [1] },
            { "type": "Remove", "address": [1] }
        ]);
        assert_json_is!(patch_compact.ops, [
            { "type": "RemoveMany", "address": [1], "items": 2 },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);
        assert_eq!(apply_new(&a, patch_compact)?, b);

        // Replace

        let a: Vec<Integer> = vec![1, 2];
        let b: Vec<Integer> = vec![3, 4];
        let patch = diff(&a, &b);
        let patch_compact = patch.compact(OperationFlag::all());
        assert_json_is!(patch.ops, [
            { "type": "Remove", "address": [0] },
            { "type": "Replace", "address": [0], "value": 3 },
            { "type": "Add", "address": [1], "value": 4 }
        ]);
        assert_json_is!(patch_compact.ops, [
            { "type": "ReplaceMany", "address": [0], "items": 2, "values": [3, 4] },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);
        assert_eq!(apply_new(&a, patch_compact)?, b);

        Ok(())
    }

    // Test patches that operate on compound items (in this case, strings) to check that
    // within-items operations are generated for each item and passed through on apply.
    #[ignore]
    #[test]
    fn compound_equal_lengths() -> Result<()> {
        // Add

        let a = vec!["a".to_string()];
        let b = vec!["ab".to_string()];
        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [0, 1], "value": "b" },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        // Move right, then add or replace

        let a = vec!["ab".to_string(), "cd".to_string()];
        let b = vec!["cde".to_string(), "abc".to_string()];

        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Move", "from": [1], "to": [0] },
            { "type": "Add", "address": [0, 2], "value": "e" },
            { "type": "Add", "address": [1, 2], "value": "c" },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        let patch = diff(&b, &a);
        assert_json_is!(patch.ops, [
            { "type": "Move", "from": [1], "to": [0] },
            { "type": "Remove", "address": [0, 2] },
            { "type": "Remove", "address": [1, 2] },
        ]);
        assert_eq!(apply_new(&b, patch)?, a);

        // Move left, then add or replace

        let a = vec!["ab".to_string(), "cd".to_string(), "ef".to_string()];
        let b = vec!["ab1".to_string(), "ef2".to_string(), "cd3".to_string()];

        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [0, 2], "value": "1" },
            { "type": "Move", "from": [2], "to": [1] },
            { "type": "Add", "address": [1, 2], "value": "2" },
            { "type": "Add", "address": [2, 2], "value": "3" },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        let patch = diff(&b, &a);
        assert_json_is!(patch.ops, [
            { "type": "Remove", "address": [0, 2] },
            { "type": "Move", "from": [2], "to": [1] },
            { "type": "Remove", "address": [1, 2] },
            { "type": "Remove", "address": [2, 2] },
        ]);
        assert_eq!(apply_new(&b, patch)?, a);

        // Remove

        let a = vec!["ab".to_string()];
        let b = vec!["a".to_string()];
        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Remove", "address": [0, 1] },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        // Replace

        let a = vec!["a".to_string()];
        let b = vec!["b".to_string()];
        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Replace", "address": [0, 0], "value": "b" },
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

    // Compound items in vectors of different lengths
    #[ignore]
    #[test]
    fn compound_unequal_lengths() -> Result<()> {
        let a = vec!["a".to_string()];
        let b = vec!["ab".to_string(), "c".to_string()];

        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [0, 1], "value": "b" },
            { "type": "Add", "address": [1], "value": "c" },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        let patch = diff(&b, &a);
        assert_json_is!(patch.ops, [
            { "type": "Remove", "address": [1] },
            { "type": "Remove", "address": [0, 1] },
        ]);
        assert_eq!(apply_new(&b, patch)?, a);

        let a = vec!["a".to_string(), "c".to_string()];
        let b = vec!["a".to_string(), "b".to_string(), "c".to_string()];

        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [1], "value": "b" },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        let patch = diff(&b, &a);
        assert_json_is!(patch.ops, [
            { "type": "Remove", "address": [1] },
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
        let patch = diff(&a, &b).compact_all();
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [0], "value": 4 },
            { "type": "Add", "address": [2], "value": 1 },
            { "type": "Remove", "address": [4] },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        Ok(())
    }

    #[test]
    fn regression_2() -> Result<()> {
        let a = vec![0, 6, 2, 4, 2];
        let b = vec![2, 2, 4];
        let patch = diff(&a, &b).compact_all();
        assert_json_is!(patch.ops, [
            { "type": "RemoveMany", "address": [0], "items": 2 },
            { "type": "Add", "address": [1], "value": 2 },
            { "type": "Remove", "address": [3] },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        Ok(())
    }

    #[test]
    fn regression_3() -> Result<()> {
        let a = vec![6, 1, 1, 1];
        let b = vec![2, 2, 0];
        let patch = diff(&a, &b).compact_all();
        assert_json_is!(patch.ops, [
            { "type": "ReplaceMany", "address": [0], "items": 4, "values": [2, 2, 0] },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        Ok(())
    }

    #[test]
    fn regression_4() -> Result<()> {
        let a = vec![1, 7, 3];
        let b = vec![7, 3, 1];
        let patch = diff(&a, &b).compact_all();
        assert_json_is!(patch.ops, [
            { "type": "Remove", "address": [0] },
            { "type": "Add", "address": [2], "value": 1 },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        Ok(())
    }

    #[test]
    fn regression_5() -> Result<()> {
        let a = vec![3, 0, 7];
        let b = vec![0, 1, 7, 3];
        let patch = diff(&a, &b).compact_all();
        assert_json_is!(patch.ops, [
            { "type": "Remove", "address": [0] },
            { "type": "Add", "address": [1], "value": 1 },
            { "type": "Add", "address": [3], "value": 3 },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        Ok(())
    }

    #[test]
    fn regression_6() -> Result<()> {
        let a = vec![1, 2, 3, 4];

        let patch = Patch {
            ops: vec![Operation::Move(Move {
                from: Address::from(0),
                to: Address::from(1),
            })],
            ..Default::default()
        };
        assert_eq!(apply_new(&a, patch)?, vec![2, 1, 3, 4]);

        let patch = Patch {
            ops: vec![Operation::Move(Move {
                from: Address::from(0),
                to: Address::from(2),
            })],
            ..Default::default()
        };
        assert_eq!(apply_new(&a, patch)?, vec![2, 3, 1, 4]);

        let patch = Patch {
            ops: vec![Operation::Move(Move {
                from: Address::from(2),
                to: Address::from(3),
            })],
            ..Default::default()
        };
        assert_eq!(apply_new(&a, patch)?, vec![1, 2, 4, 3]);

        Ok(())
    }

    #[ignore]
    #[test]
    fn regression_7() -> Result<()> {
        let a = vec_string!["aa", ""];
        let b = vec_string!["aaa", "aa", "a"];
        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [0], "value": "aaa" },
            { "type": "Add", "address": [2, 0], "value": "a" },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        Ok(())
    }

    #[ignore]
    #[test]
    fn regression_8() -> Result<()> {
        let a = vec_string!["", "ab", "c"];
        let b = vec_string![""];
        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Remove", "address": [1] },
            { "type": "Remove", "address": [1] },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        Ok(())
    }

    #[ignore]
    #[test]
    fn regression_9() -> Result<()> {
        let a = vec_string!["", "ab", "c"];
        let b = vec_string!["a", ""];
        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Move", "from": [1], "to": [0] },
            { "type": "Remove", "address": [0, 1] },
            { "type": "Remove", "address": [2] },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        Ok(())
    }

    #[ignore]
    #[test]
    fn regression_10() -> Result<()> {
        let a = vec_string!["c", "ab"];
        let b = vec_string!["ad", ""];
        let patch = diff(&a, &b);
        assert_json_is!(
            patch.ops,
            [
            //{ "type": "Move", "from": [1], "to": [0] },
            //{ "type": "Remove", "address": [0, 1] },
            //{ "type": "Remove", "address": [2] },
        ]
        );
        assert_eq!(apply_new(&a, patch)?, b);

        Ok(())
    }
}
