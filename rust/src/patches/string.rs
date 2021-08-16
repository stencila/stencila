use super::prelude::*;
use itertools::Itertools;
use similar::{ChangeTag, TextDiff};
use std::any::{type_name, Any};
use std::collections::VecDeque;
use std::iter::FromIterator;
use std::ops::Deref;

impl Diffable for String {
    diffable_is_same!(String);
    diffable_diff!(String);

    fn is_equal(&self, other: &Self) -> Result<()> {
        if self == other {
            Ok(())
        } else {
            bail!(Error::NotEqual)
        }
    }

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
                    }
                    _ => {
                        next = 'd';
                        key = position;
                        items = 1;
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
                let keys = VecDeque::from_iter(vec![Key::Index(key)]);

                match if end { next } else { curr } {
                    'd' => {
                        if next != 'i' {
                            ops.push(Operation::Remove(Remove { keys, items }));
                        }
                    }
                    'i' => {
                        if last == 'd' || end && curr == 'd' {
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
    }
}
