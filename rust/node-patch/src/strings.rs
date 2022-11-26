//! Patching for [`Strings`]s

use std::time::Duration;

use similar::{ChangeTag, TextDiff};
use unicode_segmentation::UnicodeSegmentation;

use common::itertools::Itertools;
use common::serde_json;

use crate::value::Values;

use super::prelude::*;

/// The number of seconds before a diff times out
const DIFF_TIMEOUT_SECS: u64 = 10;

/// Implements patching for [`String`]s
///
/// Diffing and patching of strings is done on the basis of Unicode graphemes.
/// These more closely match the human unit of writing than Unicode characters or bytes
/// (which are alternative sub-units that string diffing and patching could be based upon).
/// Note then that patch string indices i.e. slot indices represent
/// graphemes and that this has to be taken into account when applying `Operation`s
/// in JavaScript.
///
/// `Add`, `Remove` and `Replace` operations (and their `Many` equivalents) are supported.
/// The `Move` and `Copy` operations, whilst possible for strings, add complexity
/// and a performance hit to diffing, which are probably not warranted given the
/// small size of the values (c.f moving and copying larger content objects).
impl Patchable for String {
    /// Generate operations for the difference between two vectors
    ///
    /// If the strings are equal, will generate no operations.
    /// If self is empty, will generate an `Add` operation for each of other's graphemes.
    /// If other is empty, will generate a `Remove` operation for each of self's graphemes.
    /// Otherwise, performs a diff on the strings using `similar::TextDiff`.
    fn diff(&self, other: &Self, differ: &mut Differ) {
        // No difference
        if self == other {
            return;
        }

        // Self is empty: create an `Add` operation for each of other's graphemes
        if self.is_empty() && !other.is_empty() {
            for (index, grapheme) in other.graphemes(true).enumerate() {
                differ.push(Operation::add(
                    Address::from(index),
                    grapheme.to_string().to_value(),
                ));
            }
            return;
        }

        // Other is empty: create a `Remove` operation for each of self's graphemes
        // Note that the address is always 0.
        if self.is_empty() && !other.is_empty() {
            for _index in 0..self.graphemes(true).count() {
                differ.push(Operation::remove(Address::from(0)));
            }
            return;
        }

        let mut text_differ = TextDiff::configure();

        // Do not allow diffs to take too long (but not when testing, for determinism)
        if !cfg!(test) {
            text_differ.timeout(Duration::from_secs(DIFF_TIMEOUT_SECS));
        }

        // Generate `similar::Change`s
        let diff = text_differ.diff_graphemes(self, other);
        let changes = diff.iter_all_changes();

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
                    let value = change.value().to_string().to_value();
                    let op = if differ.ops_allowed.contains(OperationFlag::Replace)
                        && last_delete_position == position
                    {
                        differ.pop();
                        Operation::replace(address, value)
                    } else {
                        Operation::add(address, value)
                    };
                    differ.push(op);
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
        let value = Self::from_value(value)?;
        if let Some(Slot::Index(index)) = address.pop_front() {
            let mut graphemes = self.graphemes(true).collect_vec();

            let len = graphemes.len();
            if index > len {
                bail!(invalid_address::<Self>(&format!(
                    "attempting to add grapheme at `{index}` when length is `{len}`"
                )))
            }

            graphemes.insert(index, &value);
            *self = graphemes.into_iter().collect();

            Ok(())
        } else {
            bail!(invalid_address::<Self>("first slot should be an index"))
        }
    }

    fn apply_add_many(&mut self, address: &mut Address, values: Values) -> Result<()> {
        if let Some(Slot::Index(index)) = address.pop_front() {
            let mut graphemes = self.graphemes(true).collect_vec();

            let len = graphemes.len();
            if index > len {
                bail!(invalid_address::<Self>(&format!(
                    "attempting to add graphemes at `{index}` when length is `{len}`"
                )))
            }

            let values = Self::from_values(values)?;
            graphemes.splice(
                index..index,
                values.iter().map(|grapheme| grapheme.as_str()),
            );
            *self = graphemes.into_iter().collect();

            Ok(())
        } else {
            bail!(invalid_address::<Self>("first slot should be an index"))
        }
    }

    fn apply_remove(&mut self, address: &mut Address) -> Result<()> {
        if let Some(Slot::Index(index)) = address.pop_front() {
            let mut graphemes = self.graphemes(true).collect_vec();

            let len = graphemes.len();
            if index >= len {
                bail!(invalid_address::<Self>(&format!(
                    "attempting to remove grapheme at `{index}` when length is `{len}`"
                )))
            }

            graphemes.remove(index);
            *self = graphemes.into_iter().collect();

            Ok(())
        } else {
            bail!(invalid_address::<Self>("first slot should be an index"))
        }
    }

    fn apply_remove_many(&mut self, address: &mut Address, items: usize) -> Result<()> {
        if let Some(Slot::Index(index)) = address.pop_front() {
            let mut graphemes = self.graphemes(true).collect_vec();

            let len = graphemes.len();
            if index + items > len {
                bail!(invalid_address::<Self>(&format!(
                    "attempting to remove `{items}` graphemes at `{index}` when length is `{len}`"
                )))
            }

            graphemes.drain(index..(index + items));
            *self = graphemes.into_iter().collect();

            Ok(())
        } else {
            bail!(invalid_address::<Self>("first slot should be an index"))
        }
    }

    fn apply_replace(&mut self, address: &mut Address, value: Value) -> Result<()> {
        let value = Self::from_value(value)?;
        if address.is_empty() {
            *self = value
        } else if let Some(Slot::Index(index)) = address.pop_front() {
            let mut graphemes = self.graphemes(true).collect_vec();

            let len = graphemes.len();
            if index >= len {
                bail!(invalid_address::<Self>(&format!(
                    "attempting to replace grapheme at `{index}` when length is `{len}`"
                )))
            }

            graphemes[index] = &value;
            *self = graphemes.into_iter().collect();
        }
        Ok(())
    }

    fn apply_replace_many(
        &mut self,
        address: &mut Address,
        items: usize,
        values: Values,
    ) -> Result<()> {
        if let Some(Slot::Index(index)) = address.pop_front() {
            let mut graphemes = self.graphemes(true).collect_vec();

            let len = graphemes.len();
            if index + items > len {
                bail!(invalid_address::<Self>(&format!(
                    "attempting to replace `{items}` graphemes at `{index}` when length is `{len}`"
                )))
            }

            let values = Self::from_values(values)?;
            graphemes.splice(
                index..index + items,
                values.iter().map(|grapheme| grapheme.as_str()),
            );
            *self = graphemes.into_iter().collect();

            Ok(())
        } else {
            bail!(invalid_address::<Self>("first slot should be an index"))
        }
    }

    fn to_value(&self) -> Value {
        Value::String(self.clone())
    }

    fn from_value(value: Value) -> Result<Self> {
        match value {
            Value::String(string) => Ok(string),
            Value::Json(json) => Ok(serde_json::from_value::<Self>(json)?),
            _ => bail!(invalid_patch_value::<Self>(value)),
        }
    }
}

#[cfg(test)]
#[allow(clippy::many_single_char_names)]
mod tests {
    use test_utils::assert_json_is;

    use super::*;
    use crate::{apply_new, diff};

    #[test]
    fn basic() -> Result<()> {
        let empty = "".to_string();

        // No diff

        let a = "1".to_string();
        assert_json_is!(diff(&empty, &empty).ops, []);
        assert_json_is!(diff(&a, &a).ops, []);

        // Add

        let a = "1".to_string();
        let patch = diff(&empty, &a);
        let patch_compact = patch.compact(OperationFlag::all());
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [0], "value": "1" }
        ]);
        assert_json_is!(patch_compact.ops, [
            { "type": "Add", "address": [0], "value": "1" }
        ]);
        assert_eq!(apply_new(&empty, patch)?, a);
        assert_eq!(apply_new(&empty, patch_compact)?, a);

        let a = "abcdef".to_string();
        let patch = diff(&empty, &a);
        let patch_compact = patch.compact(OperationFlag::all());
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [0], "value": "a" },
            { "type": "Add", "address": [1], "value": "b" },
            { "type": "Add", "address": [2], "value": "c" },
            { "type": "Add", "address": [3], "value": "d" },
            { "type": "Add", "address": [4], "value": "e" },
            { "type": "Add", "address": [5], "value": "f" }
        ]);
        assert_json_is!(patch_compact.ops, [
            { "type": "AddMany", "address": [0], "values": "abcdef" },
        ]);
        assert_eq!(apply_new(&empty, patch)?, a);
        assert_eq!(apply_new(&empty, patch_compact)?, a);

        let a = "1".to_string();
        let b = "123".to_string();
        let patch = diff(&a, &b);
        let patch_compact = patch.compact(OperationFlag::all());
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [1], "value": "2" },
            { "type": "Add", "address": [2], "value": "3" }
        ]);
        assert_json_is!(patch_compact.ops, [
            { "type": "AddMany", "address": [1], "values": "23" },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);
        assert_eq!(apply_new(&a, patch_compact)?, b);

        // Remove

        let a = "1".to_string();
        let patch = diff(&a, &empty);
        let patch_compact = patch.compact(OperationFlag::all());
        assert_json_is!(patch.ops, [
            { "type": "Remove", "address": [0] }
        ]);
        assert_json_is!(patch_compact.ops, [
            { "type": "Remove", "address": [0] }
        ]);
        assert_eq!(apply_new(&a, patch)?, empty);
        assert_eq!(apply_new(&a, patch_compact)?, empty);

        let a = "abcdef".to_string();
        let patch = diff(&a, &empty);
        let patch_compact = patch.compact(OperationFlag::all());
        assert_json_is!(patch.ops, [
            { "type": "Remove", "address": [0] },
            { "type": "Remove", "address": [0] },
            { "type": "Remove", "address": [0] },
            { "type": "Remove", "address": [0] },
            { "type": "Remove", "address": [0] },
            { "type": "Remove", "address": [0] }
        ]);
        assert_json_is!(patch_compact.ops, [

            { "type": "RemoveMany", "address": [0], "items": 6 }
        ]);
        assert_eq!(apply_new(&a, patch)?, empty);
        assert_eq!(apply_new(&a, patch_compact)?, empty);

        let a = "123".to_string();
        let b = "1".to_string();
        let patch = diff(&a, &b);
        let patch_compact = patch.compact(OperationFlag::all());
        assert_json_is!(patch.ops, [
            { "type": "Remove", "address": [1] },
            { "type": "Remove", "address": [1] }
        ]);
        assert_json_is!(patch_compact.ops, [
            { "type": "RemoveMany", "address": [1], "items": 2 }
        ]);
        assert_eq!(apply_new(&a, patch)?, b);
        assert_eq!(apply_new(&a, patch_compact)?, b);

        // Replace

        let a = "1".to_string();
        let b = "a2b3".to_string();
        let patch = diff(&a, &b);
        let patch_compact = patch.compact(OperationFlag::all());
        assert_json_is!(patch.ops, [
            { "type": "Replace", "address": [0], "value": "a"},
            { "type": "Add", "address": [1], "value": "2"},
            { "type": "Add", "address": [2], "value": "b"},
            { "type": "Add", "address": [3], "value": "3"}
        ]);
        assert_json_is!(patch_compact.ops, [
            { "type": "ReplaceMany", "address": [0], "items": 1, "values": "a2b3"},
        ]);
        assert_eq!(apply_new(&a, patch)?, b);
        assert_eq!(apply_new(&a, patch_compact)?, b);

        let a = "123".to_string();
        let b = "abcdef".to_string();
        let patch = diff(&a, &b);
        let patch_compact = patch.compact(OperationFlag::all());
        assert_json_is!(patch.ops, [
            { "type": "Remove", "address": [0] },
            { "type": "Remove", "address": [0] },
            { "type": "Replace", "address": [0], "value": "a" },
            { "type": "Add", "address": [1], "value": "b" },
            { "type": "Add", "address": [2], "value": "c" },
            { "type": "Add", "address": [3], "value": "d" },
            { "type": "Add", "address": [4], "value": "e" },
            { "type": "Add", "address": [5], "value": "f" },
        ]);
        assert_json_is!(patch_compact.ops, [
            { "type": "ReplaceMany", "address": [0], "items": 3, "values": "abcdef"},
        ]);
        assert_eq!(apply_new(&a, patch)?, b);
        assert_eq!(apply_new(&a, patch_compact)?, b);

        // Mixed

        let a = "a2b3".to_string();
        let b = "abcdef".to_string();
        let patch = diff(&a, &b);
        let patch_compact = patch.compact(OperationFlag::all());
        assert_json_is!(patch.ops, [
            { "type": "Remove", "address": [1] },
            { "type": "Replace", "address": [2], "value": "c" },
            { "type": "Add", "address": [3], "value": "d" },
            { "type": "Add", "address": [4], "value": "e" },
            { "type": "Add", "address": [5], "value": "f" }
        ]);
        assert_json_is!(patch_compact.ops, [
            { "type": "Remove", "address": [1] },
            { "type": "ReplaceMany", "address": [2], "items": 1, "values": "cdef" }
        ]);
        assert_eq!(apply_new(&a, patch)?, b);
        assert_eq!(apply_new(&a, patch_compact)?, b);

        let a = "abcdef".to_string();
        let b = "a2b3".to_string();
        let patch = diff(&a, &b);
        let patch_compact = patch.compact(OperationFlag::all());
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [1], "value": "2" },
            { "type": "Remove", "address": [3] },
            { "type": "Remove", "address": [3] },
            { "type": "Remove", "address": [3] },
            { "type": "Replace", "address": [3], "value": "3" }
        ]);
        assert_json_is!(patch_compact.ops, [
            { "type": "Add", "address": [1], "value": "2" },
            { "type": "ReplaceMany", "address": [3], "items": 4, "values": "3" }
        ]);
        assert_eq!(apply_new(&a, patch)?, b);
        assert_eq!(apply_new(&a, patch_compact)?, b);

        let a = "abcdef".to_string();
        let b = "adbcfe".to_string();
        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [1], "value": "d" },
            { "type": "Replace", "address": [4], "value": "f" },
            { "type": "Remove", "address": [6] }
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        Ok(())
    }

    /// Test that works with Unicode graphemes (which are made up of multiple Unicode characters
    /// which themselves can be made of several bytes).
    #[test]
    fn unicode() -> Result<()> {
        // ä = 1 Unicode char
        // 👍🏻 and 👍🏿 = 2 Unicode chars each
        let a = "ä".to_string();
        let b = "ä1👍🏻2".to_string();
        let c = "1👍🏿2".to_string();

        let patch = diff(&a, &b);
        let patch_compact = patch.compact(OperationFlag::all());
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [1], "value": "1" },
            { "type": "Add", "address": [2], "value": "👍🏻" },
            { "type": "Add", "address": [3], "value": "2" },
        ]);
        assert_json_is!(patch_compact.ops, [
            { "type": "AddMany", "address": [1], "values": "1👍🏻2" },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);
        assert_eq!(apply_new(&a, patch_compact)?, b);

        let patch = diff(&b, &c);
        let patch_compact = patch.compact(OperationFlag::all());
        assert_json_is!(patch.ops, [
            { "type": "Remove", "address": [0] },
            { "type": "Replace", "address": [1], "value": "👍🏿" },
        ]);
        assert_json_is!(patch_compact.ops, [
            { "type": "Remove", "address": [0] },
            { "type": "Replace", "address": [1], "value": "👍🏿" },
        ]);
        assert_eq!(apply_new(&b, patch)?, c);
        assert_eq!(apply_new(&b, patch_compact)?, c);

        let patch = diff(&c, &b);
        let patch_compact = patch.compact(OperationFlag::all());
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [0], "value": "ä" },
            { "type": "Replace", "address": [2], "value": "👍🏻" },
        ]);
        assert_json_is!(patch_compact.ops, [
            { "type": "Add", "address": [0], "value": "ä" },
            { "type": "Replace", "address": [2], "value": "👍🏻" },
        ]);
        assert_eq!(apply_new(&c, patch)?, b);
        assert_eq!(apply_new(&c, patch_compact)?, b);

        // 🌷 and 🎁 = 2 Unicode chars each
        // 🏳️‍🌈 = 6 Unicode chars
        let d = "🌷🏳️‍🌈🎁".to_string();
        let e = "🎁🏳️‍🌈🌷".to_string();

        let patch = diff(&d, &e);
        let patch_compact = patch.compact(OperationFlag::all());
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [0], "value": "🎁" },
            { "type": "Add", "address": [1], "value": "🏳️‍🌈" },
            { "type": "Remove", "address": [3] },
            { "type": "Remove", "address": [3] },
        ]);
        assert_json_is!(patch_compact.ops, [
            { "type": "AddMany", "address": [0], "values": "🎁🏳️‍🌈" },
            { "type": "RemoveMany", "address": [3], "items": 2 },
        ]);
        assert_eq!(apply_new(&d, patch)?, e);
        assert_eq!(apply_new(&d, patch_compact)?, e);

        let patch = diff(&e, &d);
        let patch_compact = patch.compact(OperationFlag::all());
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [0], "value": "🌷" },
            { "type": "Add", "address": [1], "value": "🏳️‍🌈" },
            { "type": "Remove", "address": [3] },
            { "type": "Remove", "address": [3] },
        ]);
        assert_json_is!(patch_compact.ops, [
            { "type": "AddMany", "address": [0], "values": "🌷🏳️‍🌈" },
            { "type": "RemoveMany", "address": [3], "items": 2 },
        ]);
        assert_eq!(apply_new(&e, patch)?, d);
        assert_eq!(apply_new(&e, patch_compact)?, d);

        Ok(())
    }

    // Regression tests of minimal failing cases found using property testing
    // and elsewhere.

    #[test]
    fn regression_1() -> Result<()> {
        let a = "ab".to_string();
        let b = "bc".to_string();
        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Remove", "address": [0] },
            { "type": "Add", "address": [1], "value": "c" },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        Ok(())
    }

    #[test]
    fn regression_2() -> Result<()> {
        let a = "ac".to_string();
        let b = "bcd".to_string();
        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            { "type": "Replace", "address": [0], "value": "b" },
            { "type": "Add", "address": [2], "value": "d" },
        ]);
        assert_eq!(apply_new(&a, patch)?, b);

        Ok(())
    }
}
