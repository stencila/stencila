use super::prelude::*;
use itertools::Itertools;
use similar::{ChangeTag, TextDiff};
use std::any::{type_name, Any};
use std::collections::VecDeque;
use std::iter::FromIterator;
use std::ops::Deref;

/// Implements patching for strings
///
/// `Add`, `Remove` and `Replace` operations are implemented.
/// The `Move` operation, whilst possible for strings, adds complexity
/// and a performance hit to diffing, but is likely to be uncommon at the
/// word level (word level moves are dealt with in `Vec<InlineContent>`).
impl Diffable for String {
    diffable_is_same!();

    fn is_equal(&self, other: &Self) -> Result<()> {
        if self == other {
            Ok(())
        } else {
            bail!(Error::NotEqual)
        }
    }

    diffable_diff!();

    fn diff_same(&self, differ: &mut Differ, other: &Self) {
        if self == other {
            return;
        }

        let diff = TextDiff::from_chars(self, other);
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
                let keys = VecDeque::from_iter(vec![Key::Index(start)]);
                if (curr == 'e' && last == 'd') || (end && curr == 'd') {
                    ops.push(Operation::Remove(Remove { keys, items }));
                } else if (curr == 'e' && last == 'i') || (end && curr == 'i') {
                    if replace {
                        ops.push(Operation::Replace(Replace {
                            keys,
                            items,
                            value: Box::new(value.clone()),
                        }));
                    } else {
                        ops.push(Operation::Add(Add {
                            keys,
                            value: Box::new(value.clone()),
                        }));
                    }
                };
            }
        }

        differ.append(ops)
    }

    fn apply_add(&mut self, keys: &mut Keys, value: &Box<dyn Any>) {
        let value = if let Some(value) = value.deref().downcast_ref::<Self>() {
            value
        } else {
            unreachable!("Invalid replacement value for {}", type_name::<Self>())
        };

        if let Some(Key::Index(index)) = keys.pop_front() {
            let chars: Vec<char> = self.chars().collect();
            let chars = [
                &chars[..index],
                &value.chars().collect_vec(),
                &chars[index..],
            ]
            .concat();
            *self = chars.into_iter().collect();
        } else {
            invalid_keys!(keys)
        }
    }

    fn apply_remove(&mut self, keys: &mut Keys, items: usize) {
        if let Some(Key::Index(index)) = keys.pop_front() {
            let chars: Vec<char> = self.chars().collect();
            let chars = [&chars[..index], &chars[(index + items)..]].concat();
            *self = chars.into_iter().collect();
        } else {
            invalid_keys!(keys)
        }
    }

    fn apply_replace(&mut self, keys: &mut Keys, items: usize, value: &Box<dyn Any>) {
        let value = if let Some(value) = value.deref().downcast_ref::<Self>() {
            value
        } else {
            unreachable!("Invalid replacement value for {}", type_name::<Self>())
        };

        if keys.is_empty() {
            *self = value.clone();
        } else if let Some(Key::Index(index)) = keys.pop_front() {
            let chars: Vec<char> = self.chars().collect();
            let chars = [
                &chars[..index],
                &value.chars().collect_vec(),
                &chars[(index + items)..],
            ]
            .concat();
            *self = chars.into_iter().collect();
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

    #[test]
    fn basic() {
        let empty = "".to_string();
        let a = "1".to_string();
        let b = "123".to_string();
        let c = "a2b3".to_string();
        let d = "abcdef".to_string();
        let e = "adefbc".to_string();
        let f = "adbcfe".to_string();

        assert!(equal(&empty, &empty));
        assert!(equal(&a, &a));
        assert!(equal(&b, &b));
        assert!(equal(&c, &c));
        assert!(equal(&d, &d));

        // No diff

        assert_json!(diff(&empty, &empty), []);
        assert_json!(diff(&a, &a), []);
        assert_json!(diff(&d, &d), []);

        // Add

        let patch = diff(&empty, &a);
        assert_json!(
            patch,
            [{ "op": "add", "keys": [0], "value": "1" }]
        );
        assert_eq!(apply_new(&empty, &patch), a);

        let patch = diff(&empty, &d);
        assert_json!(
            patch,
            [{ "op": "add", "keys": [0], "value": "abcdef" }]
        );
        assert_eq!(apply_new(&empty, &patch), d);

        let patch = diff(&a, &b);
        assert_json!(
            patch,
            [{ "op": "add", "keys": [1], "value": "23" }]
        );
        assert_eq!(apply_new(&a, &patch), b);

        // Remove

        let patch = diff(&a, &empty);
        assert_json!(
            patch,
            [{ "op": "remove", "keys": [0], "items": 1 }]
        );

        let patch = diff(&d, &empty);
        assert_json!(
            patch,
            [{ "op": "remove", "keys": [0], "items": 6 }]
        );

        let patch = diff(&b, &a);
        assert_json!(
            patch,
            [{ "op": "remove", "keys": [1], "items": 2 }]
        );

        // Replace

        let patch = diff(&a, &c);
        assert_json!(
            patch,
            [{ "op": "replace", "keys": [0], "items": 1, "value": "a2b3" }]
        );
        assert_eq!(apply_new(&a, &patch), c);

        let patch = diff(&b, &d);
        assert_json!(
            patch,
            [{ "op": "replace", "keys": [0], "items": 3, "value": "abcdef" }]
        );
        assert_eq!(apply_new(&b, &patch), d);

        // Mixed

        let patch = diff(&c, &d);
        assert_json!(
            patch,
            [
                { "op": "remove", "keys": [1], "items": 1 },
                { "op": "replace", "keys": [2], "items": 1, "value": "cdef" }
            ]
        );
        assert_eq!(apply_new(&c, &patch), d);

        let patch = diff(&d, &c);
        assert_json!(
            patch,
            [
                { "op": "add", "keys": [1], "value": "2" },
                { "op": "replace", "keys": [3], "items": 4, "value": "3" }
            ]
        );
        assert_eq!(apply_new(&d, &patch), c);

        let patch = diff(&d, &f);
        assert_json!(
            patch,
            [
                { "op": "add", "keys": [1], "value": "d" },
                { "op": "replace", "keys": [4], "items": 1, "value": "f" },
                { "op": "remove", "keys": [6], "items": 1 }
            ]
        );
        assert_eq!(apply_new(&d, &patch), f);
    }

    /// Test that works with Unicode graphemes (which are made
    /// up of multiple `char`s).
    #[test]
    fn unicode() {
        let a = "ä".to_string();
        let b = "ä1👍🏻2".to_string();
        let c = "1👍🏿2".to_string();

        let patch = diff(&a, &b);
        assert_json!(patch, [
            { "op": "add", "keys": [1], "value": "1👍🏻2" },
        ]);
        assert_eq!(apply_new(&a, &patch), b);

        let patch = diff(&b, &c);
        assert_json!(patch, [
            { "op": "remove", "keys": [0], "items": 1 },
            { "op": "replace", "keys": [2], "items": 1, "value": "🏿" },
        ]);
        assert_eq!(apply_new(&b, &patch), c);

        let patch = diff(&c, &b);
        assert_json!(patch, [
            { "op": "add", "keys": [0], "value": "ä" },
            { "op": "replace", "keys": [3], "items": 1, "value": "🏻" },
        ]);
        assert_eq!(apply_new(&c, &patch), b);
    }

    // Regression tests of minimal failing cases found using property testing
    // and elsewhere.

    #[test]
    fn regression_1() {
        let a = "ab".to_string();
        let b = "bc".to_string();
        let patch = diff(&a, &b);
        assert_json!(patch, [
            { "op": "remove", "keys": [0], "items": 1 },
            { "op": "add", "keys": [1], "value": "c" },
        ]);
        assert_eq!(apply_new(&a, &patch), b);
    }

    #[test]
    fn regression_2() {
        let a = "ac".to_string();
        let b = "bcd".to_string();
        let patch = diff(&a, &b);
        assert_json!(
            patch,
            [
                { "op": "replace", "keys": [0], "items": 1, "value": "b" },
                { "op": "add", "keys": [2], "value": "d" },
            ]
        );
        assert_eq!(apply_new(&a, &patch), b);
    }
}
