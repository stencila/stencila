//! Patching for [`Vec`]s

use std::{
    hash::Hash,
    time::{Duration, Instant},
};

use common::itertools::Itertools;
use similar::{algorithms::IdentifyDistinct, capture_diff_deadline, ChangeTag};

use crate::value::Values;

use super::prelude::*;

/// The number of seconds before a diff times out
const DIFF_TIMEOUT_SECS: u64 = 10;

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

        // Do not allow diffs to take too long (but not when testing, for determinism)
        let deadline = if cfg!(test) {
            None
        } else {
            Some(Instant::now() + Duration::from_secs(DIFF_TIMEOUT_SECS))
        };

        // Generate integer ids, that are unique across both self and other, for each item
        // The `similar` crate generally recommends only doing that for large sequences
        // (the text diffing does it for those greater than 100). But we do it here to avoid having
        // to make Type be `Ord` (which is what `capture_diff_deadline` requires).
        let identities =
            IdentifyDistinct::<u32>::new(&self[..], 0..self.len(), &other[..], 0..other.len());

        // Generate `similar::DiffOps`s and flatten them into `Similar::Change`s
        let diffops = capture_diff_deadline(
            similar::Algorithm::Patience,
            identities.old_lookup(),
            identities.old_range(),
            identities.new_lookup(),
            identities.new_range(),
            deadline,
        );
        let changes = diffops
            .iter()
            .flat_map(|diffop| diffop.iter_changes(self, other))
            .collect_vec();

        // Convert `DiffOp`s into `Add` and `Remove` operation
        let mut position = 0;
        let mut last_delete_position = usize::MAX;
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

    // Test patches that operate on compound items (strings) to check that
    // fine grained operations are generated for each item and passed through on apply.
    #[ignore]
    #[test]
    fn item_ops() -> Result<()> {
        // Add

        let a = vec!["a".to_string()];
        let b = vec!["ab".to_string()];
        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [0, 1], "value": "b" },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

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

    // As above, but with an extra `Add` or `Remove` as needed.
    #[ignore]
    #[test]
    fn item_ops_plus() -> Result<()> {
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
            { "type": "Remove", "address": [0, 1] },
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
}
