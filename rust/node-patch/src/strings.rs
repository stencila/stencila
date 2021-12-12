use super::prelude::*;
use itertools::Itertools;
use similar::{ChangeTag, TextDiff};
use std::{
    hash::{Hash, Hasher},
    time::Duration,
};
use unicode_segmentation::UnicodeSegmentation;

/// The number of seconds before a diff times out (falls back to a `Replace`)
const DIFF_TIMEOUT_SECS: u64 = 1;

/// Implements patching for strings
///
/// Diffing and patching of strings is done on the basis of Unicode graphemes.
/// These more closely match the human unit of writing than Unicode characters or bytes
/// (which are alternative sub-units that string diffing and patching could be based upon).
/// Note then that patch string indices i.e. slot indices, `length`, `items` represent
/// graphemes and that this has to be taken into account when applying `DomOperation`s
/// in JavaScript.
///
/// `Add`, `Remove` and `Replace` operations are implemented.
/// The `Move` operation, whilst possible for strings, adds complexity
/// and a performance hit to diffing so is not used.
impl Patchable for String {
    fn is_equal(&self, other: &Self) -> Result<()> {
        if self == other {
            Ok(())
        } else {
            bail!(Error::NotEqual)
        }
    }

    fn make_hash<H: Hasher>(&self, state: &mut H) {
        self.hash(state)
    }

    fn diff(&self, other: &Self, differ: &mut Differ) {
        if self == other {
            return;
        }

        let mut text_differ = TextDiff::configure();

        // Do not allow diffs to take too long (but not when testing, for determinism)
        if !cfg!(test) {
            text_differ.timeout(Duration::from_secs(DIFF_TIMEOUT_SECS));
        }

        let diff = text_differ.diff_graphemes(self, other);
        let mut ops: Vec<Operation> = Vec::new();
        let mut curr: char = 'e';
        let mut replace = false;
        let mut position: usize = 0;
        let mut start: usize = 0;
        let mut items: usize = 0;
        let mut value: String = String::new();

        let changes = diff.iter_all_changes().collect_vec();
        for (index, change) in changes.iter().enumerate() {
            let last = curr;
            match change.tag() {
                ChangeTag::Equal => {
                    position += 1;
                    curr = 'e';
                }
                ChangeTag::Delete => match last {
                    'd' => {
                        items += 1;
                        value.push_str(change.value());
                    }
                    _ => {
                        curr = 'd';
                        start = position;
                        items = 1;
                        value = change.value().into();
                    }
                },
                ChangeTag::Insert => {
                    match last {
                        'i' => {
                            value.push_str(change.value());
                        }
                        _ => {
                            curr = 'i';
                            if last == 'd' {
                                replace = true;
                            } else {
                                replace = false;
                                start = position;
                            }
                            value = change.value().into();
                        }
                    }
                    position += 1;
                }
            }

            let end = index == changes.len() - 1;
            if (index > 0 && curr != last) || end {
                let address = Address::from(start);
                if (curr == 'e' && last == 'd') || (end && curr == 'd') {
                    ops.push(Operation::Remove { address, items });
                } else if (curr == 'e' && last == 'i') || (end && curr == 'i') {
                    if replace {
                        ops.push(Operation::Replace {
                            address,
                            items,
                            value: Box::new(value.clone()),
                            length: value.graphemes(true).count(),
                            html: None,
                        });
                    } else {
                        ops.push(Operation::Add {
                            address,
                            value: Box::new(value.clone()),
                            length: value.graphemes(true).count(),
                            html: None,
                        });
                    }
                };
            }
        }

        differ.append(ops)
    }

    fn apply_add(&mut self, address: &mut Address, value: &Value) -> Result<()> {
        let value = Self::from_value(value)?;
        if let Some(Slot::Index(index)) = address.pop_front() {
            let graphemes = self.graphemes(true).collect_vec();
            let graphemes = [
                &graphemes[..index],
                &value.graphemes(true).collect_vec(),
                &graphemes[index..],
            ]
            .concat();
            *self = graphemes.into_iter().collect();
            Ok(())
        } else {
            bail!(invalid_address::<Self>("first slot should be an index"))
        }
    }

    fn apply_remove(&mut self, address: &mut Address, items: usize) -> Result<()> {
        if let Some(Slot::Index(index)) = address.pop_front() {
            let graphemes = self.graphemes(true).collect_vec();
            let graphemes = [&graphemes[..index], &graphemes[(index + items)..]].concat();
            *self = graphemes.into_iter().collect();
            Ok(())
        } else {
            bail!(invalid_address::<Self>("first slot should be an index"))
        }
    }

    fn apply_replace(&mut self, address: &mut Address, items: usize, value: &Value) -> Result<()> {
        let value = Self::from_value(value)?;
        if address.is_empty() {
            *self = value
        } else if let Some(Slot::Index(index)) = address.pop_front() {
            let graphemes = self.graphemes(true).collect_vec();
            let graphemes = [
                &graphemes[..index],
                &value.graphemes(true).collect_vec(),
                &graphemes[(index + items)..],
            ]
            .concat();
            *self = graphemes.into_iter().collect();
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::many_single_char_names)]
mod tests {
    use test_utils::assert_json_is;

    use super::*;
    use crate::{apply_new, diff, equal};

    #[test]
    fn basic() -> Result<()> {
        let empty = "".to_string();
        let a = "1".to_string();
        let b = "123".to_string();
        let c = "a2b3".to_string();
        let d = "abcdef".to_string();
        let e = "adbcfe".to_string();

        assert!(equal(&empty, &empty));
        assert!(equal(&a, &a));
        assert!(equal(&b, &b));
        assert!(equal(&c, &c));
        assert!(equal(&d, &d));

        // No diff

        assert_json_is!(diff(&empty, &empty).ops, []);
        assert_json_is!(diff(&a, &a).ops, []);
        assert_json_is!(diff(&d, &d).ops, []);

        // Add

        let patch = diff(&empty, &a);
        assert_json_is!(
            patch.ops,
            [{ "type": "Add", "address": [0], "value": "1", "length": 1 }]
        );
        assert_eq!(apply_new(&empty, &patch)?, a);

        let patch = diff(&empty, &d);
        assert_json_is!(
            patch.ops,
            [{ "type": "Add", "address": [0], "value": "abcdef", "length": 6 }]
        );
        assert_eq!(apply_new(&empty, &patch)?, d);

        let patch = diff(&a, &b);
        assert_json_is!(
            patch.ops,
            [{ "type": "Add", "address": [1], "value": "23", "length": 2 }]
        );
        assert_eq!(apply_new(&a, &patch)?, b);

        // Remove

        let patch = diff(&a, &empty);
        assert_json_is!(
            patch.ops,
            [{ "type": "Remove", "address": [0], "items": 1 }]
        );

        let patch = diff(&d, &empty);
        assert_json_is!(
            patch.ops,
            [{ "type": "Remove", "address": [0], "items": 6 }]
        );

        let patch = diff(&b, &a);
        assert_json_is!(
            patch.ops,
            [{ "type": "Remove", "address": [1], "items": 2 }]
        );

        // Replace

        let patch = diff(&a, &c);
        assert_json_is!(
            patch.ops,
            [{ "type": "Replace", "address": [0], "items": 1, "value": "a2b3", "length": 4 }]
        );
        assert_eq!(apply_new(&a, &patch)?, c);

        let patch = diff(&b, &d);
        assert_json_is!(
            patch.ops,
            [{ "type": "Replace", "address": [0], "items": 3, "value": "abcdef", "length": 6 }]
        );
        assert_eq!(apply_new(&b, &patch)?, d);

        // Mixed

        let patch = diff(&c, &d);
        assert_json_is!(
            patch.ops,
            [
                { "type": "Remove", "address": [1], "items": 1 },
                { "type": "Replace", "address": [2], "items": 1, "value": "cdef", "length": 4 }
            ]
        );
        assert_eq!(apply_new(&c, &patch)?, d);

        let patch = diff(&d, &c);
        assert_json_is!(
            patch.ops,
            [
                { "type": "Add", "address": [1], "value": "2", "length": 1 },
                { "type": "Replace", "address": [3], "items": 4, "value": "3", "length": 1 }
            ]
        );
        assert_eq!(apply_new(&d, &patch)?, c);

        let patch = diff(&d, &e);
        assert_json_is!(
            patch.ops,
            [
                { "type": "Add", "address": [1], "value": "d", "length": 1 },
                { "type": "Replace", "address": [4], "items": 1, "value": "f", "length": 1 },
                { "type": "Remove", "address": [6], "items": 1 }
            ]
        );
        assert_eq!(apply_new(&d, &patch)?, e);

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
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [1], "value": "1👍🏻2", "length": 3 },
        ]);
        assert_eq!(apply_new(&a, &patch)?, b);

        let patch = diff(&b, &c);
        assert_json_is!(patch.ops, [
            { "type": "Remove", "address": [0], "items": 1 },
            { "type": "Replace", "address": [1], "items": 1, "value": "👍🏿", "length": 1 },
        ]);
        assert_eq!(apply_new(&b, &patch)?, c);

        let patch = diff(&c, &b);
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [0], "value": "ä", "length": 1 },
            { "type": "Replace", "address": [2], "items": 1, "value": "👍🏻", "length": 1 },
        ]);
        assert_eq!(apply_new(&c, &patch)?, b);

        // 🌷 and 🎁 = 2 Unicode chars each
        // 🏳️‍🌈 = 6 Unicode chars
        let d = "🌷🏳️‍🌈🎁".to_string();
        let e = "🎁🏳️‍🌈🌷".to_string();

        let patch = diff(&d, &e);
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [0], "value": "🎁🏳️‍🌈", "length": 2 },
            { "type": "Remove", "address": [3], "items": 2 },
        ]);
        assert_eq!(apply_new(&d, &patch)?, e);

        let patch = diff(&e, &d);
        assert_json_is!(patch.ops, [
            { "type": "Add", "address": [0], "value": "🌷🏳️‍🌈", "length": 2 },
            { "type": "Remove", "address": [3], "items": 2 },
        ]);
        assert_eq!(apply_new(&e, &patch)?, d);

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
            { "type": "Remove", "address": [0], "items": 1 },
            { "type": "Add", "address": [1], "value": "c", "length": 1 },
        ]);
        assert_eq!(apply_new(&a, &patch)?, b);

        Ok(())
    }

    #[test]
    fn regression_2() -> Result<()> {
        let a = "ac".to_string();
        let b = "bcd".to_string();
        let patch = diff(&a, &b);
        assert_json_is!(
            patch.ops,
            [
                { "type": "Replace", "address": [0], "items": 1, "value": "b", "length": 1 },
                { "type": "Add", "address": [2], "value": "d", "length": 1 },
            ]
        );
        assert_eq!(apply_new(&a, &patch)?, b);

        Ok(())
    }
}
