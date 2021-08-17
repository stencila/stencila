use crate::patches::Move;

use super::prelude::*;
use itertools::Itertools;
use similar::{ChangeTag, TextDiff};
use std::any::{type_name, Any};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use std::iter::FromIterator;
use std::ops::Deref;

impl Diffable for String {
    diffable_is_same!(String);

    fn is_equal(&self, other: &Self) -> Result<()> {
        if self == other {
            Ok(())
        } else {
            bail!(Error::NotEqual)
        }
    }

    diffable_diff!(String);

    fn diff_same(&self, differ: &mut Differ, other: &Self) {
        if self == other {
            return;
        }

        let diff = TextDiff::from_chars(self, other);
        let mut ops: Vec<Operation> = Vec::new();
        let mut last: char = 'e';
        let mut curr: char = 'e';
        let mut next: char = 'e';
        let mut position = 0;
        let mut key: usize = 0;
        let mut items: usize = 0;
        let mut value: String = String::new();
        let mut adds: HashMap<String, usize> = HashMap::new();
        let mut removes: HashMap<String, usize> = HashMap::new();

        let changes = diff.iter_all_changes().collect_vec();
        for (index, change) in changes.iter().enumerate() {
            match change.tag() {
                ChangeTag::Equal => {
                    position += 1;
                    next = 'e';
                }
                ChangeTag::Delete => match curr {
                    'd' => {
                        items += 1;
                        value.push_str(change.value());
                    }
                    _ => {
                        next = 'd';
                        key = position;
                        items = 1;
                        value = change.value().into();
                    }
                },
                ChangeTag::Insert => {
                    match curr {
                        'i' => {
                            value.push_str(change.value());
                        }
                        _ => {
                            next = 'i';
                            if last != 'd' {
                                key = position;
                            }
                            value = change.value().into();
                        }
                    }
                    position += 1;
                }
            }

            let end = index == changes.len() - 1;
            if next != curr || end {
                // Generate a keys for a string position index
                fn keys(index: usize) -> VecDeque<Key> {
                    VecDeque::from_iter(vec![Key::Index(index)])
                }

                match if end { next } else { curr } {
                    'd' => {
                        if next != 'i' {
                            if let Entry::Occupied(entry) = adds.entry(value.clone()) {
                                let index = *entry.get();
                                let move_ = if let Some(Operation::Add(add)) = ops.get(index) {
                                    Operation::Move(Move {
                                        from: keys(position - value.len()),
                                        items,
                                        to: add.keys.clone(),
                                    })
                                } else {
                                    unreachable!()
                                };
                                ops.remove(index);
                                ops.push(move_);
                                entry.remove_entry();
                            } else {
                                ops.push(Operation::Remove(Remove {
                                    keys: keys(key),
                                    items,
                                }));
                                removes.insert(value.clone(), ops.len() - 1);
                            }
                        }
                    }
                    'i' => {
                        if last == 'd' || end && curr == 'd' {
                            ops.push(Operation::Replace(Replace {
                                keys: keys(key),
                                items,
                                value: Box::new(value.clone()),
                            }));
                        } else {
                            // Clippy seems to think this is collapsible with the above. It's not, tests break.
                            #[allow(clippy::collapsible_else_if)]
                            if let Entry::Occupied(entry) = removes.entry(value.clone()) {
                                let index = *entry.get();
                                let move_ = if let Some(Operation::Remove(remove)) = ops.get(index)
                                {
                                    Operation::Move(Move {
                                        from: remove.keys.clone(),
                                        items: remove.items,
                                        to: keys(position - value.len()),
                                    })
                                } else {
                                    unreachable!()
                                };
                                ops.remove(index);
                                ops.push(move_);
                                entry.remove_entry();
                            } else {
                                ops.push(Operation::Add(Add {
                                    keys: keys(key),
                                    value: Box::new(value.clone()),
                                }));
                                adds.insert(value.clone(), ops.len() - 1);
                            }
                        }
                    }
                    _ => {}
                };

                last = curr;
            }

            curr = next;
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
            self.insert_str(index, value)
        } else {
            invalid_keys!(keys)
        }
    }

    fn apply_remove(&mut self, keys: &mut Keys, items: usize) {
        if let Some(Key::Index(index)) = keys.pop_front() {
            self.replace_range(index..(index + items), "")
        } else {
            invalid_keys!(keys)
        }
    }

    fn apply_move(&mut self, from: &mut Keys, items: usize, to: &mut Keys) {
        if let (Some(Key::Index(from)), Some(Key::Index(to))) = (from.pop_front(), to.pop_front()) {
            let chars: Vec<char> = self.chars().collect();
            let chars = if from < to {
                [
                    &chars[..from],
                    &chars[(from + items)..(to + items)],
                    &chars[from..(from + items)],
                    &chars[(to + items)..],
                ]
                .concat()
            } else {
                [
                    &chars[..to],
                    &chars[from..(from + items)],
                    &chars[to..from],
                    &chars[(from + items)..],
                ]
                .concat()
            };
            *self = chars.into_iter().collect();
        } else {
            invalid_keys!(from)
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
            self.replace_range(index..(index + items), value);
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
    fn test_string() {
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

        // Move

        let patch = diff(&d, &e);
        assert_json!(
            patch,
            [
                { "op": "move", "from": [1], "items": 2, "to": [4] },
            ]
        );
        assert_eq!(apply_new(&d, &patch), e);

        let patch = diff(&e, &d);
        assert_json!(
            patch,
            [
                { "op": "move", "from": [4], "items": 2, "to": [1] },
            ]
        );
        assert_eq!(apply_new(&e, &patch), d);

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
}
