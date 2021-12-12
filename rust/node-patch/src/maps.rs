use super::prelude::*;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

/// Implements `Patchable` for `BTreeMap<String, Type>`
///
/// This is mainly provided for `impl Patchable for Object`.
impl<Type: Patchable> Patchable for BTreeMap<String, Type>
where
    Type: Clone + DeserializeOwned + Send + 'static,
{
    fn is_equal(&self, other: &Self) -> Result<()> {
        if self.len() != other.len() {
            bail!(Error::NotEqual)
        }

        for (key, value) in self {
            if let Some(other_value) = other.get(key) {
                if value.is_equal(other_value).is_err() {
                    bail!(Error::NotEqual)
                }
            } else {
                bail!(Error::NotEqual)
            }
        }

        Ok(())
    }

    fn make_hash<H: Hasher>(&self, state: &mut H) {
        for (key, value) in self {
            key.hash(state);
            value.make_hash(state);
        }
    }

    fn diff(&self, other: &Self, differ: &mut Differ) {
        // Shortcuts
        if self.is_empty() && other.is_empty() {
            return;
        } else if self.is_empty() && !other.is_empty() {
            return differ.replace(other);
        } else if !self.is_empty() && other.is_empty() {
            return differ.remove();
        }

        let mut adds = Vec::with_capacity(other.len());
        let mut removes = Vec::with_capacity(self.len());

        // Iterate over `self` items and generate operations for each
        for (key, value) in self {
            if let Some(other_value) = other.get(key) {
                differ.field(key, value, other_value)
            } else {
                removes.push((false, key, value));
            }
        }

        // Add any items in `other` and not in `self`
        for (key, value) in other {
            if !self.contains_key(key) {
                adds.push((false, key, value))
            }
        }

        // See if it is possible to transform and Remove/Add pairs into a Move
        // by matching the value added against a value removed.
        for (add_matched, add_key, add_value) in adds.iter_mut() {
            for (remove_matched, remove_key, remove_value) in removes.iter_mut() {
                if *remove_matched {
                    continue;
                }

                if remove_value.is_equal(add_value).is_ok() {
                    differ.push(Operation::Move {
                        from: Address::from(remove_key.as_str()),
                        to: Address::from(add_key.as_str()),
                        items: 1,
                    });
                    *remove_matched = true;
                    *add_matched = true;
                    continue;
                }
            }
        }

        // Append unmatched adds
        let adds = adds
            .into_iter()
            .filter_map(|(matched, key, value)| {
                if !matched {
                    Some(Operation::Add {
                        address: Address::from(key.as_str()),
                        value: Box::new(value.clone()),
                        length: 1,
                        html: None,
                    })
                } else {
                    None
                }
            })
            .collect();
        differ.append(adds);

        // Append unmatched removes
        let removes = removes
            .into_iter()
            .filter_map(|(matched, key, ..)| {
                if !matched {
                    Some(Operation::Remove {
                        address: Address::from(key.as_str()),
                        items: 1,
                    })
                } else {
                    None
                }
            })
            .collect();
        differ.append(removes);
    }

    fn apply_add(&mut self, address: &mut Address, value: &Value) -> Result<()> {
        if address.is_empty() {
            self.clear();
            self.append(&mut Self::from_value(value)?);
        } else {
            let slot = address.pop_front().expect("Should have at least one slot");
            if let Slot::Name(key) = slot {
                if address.len() == 0 {
                    self.insert(key, Type::from_value(value)?);
                } else if let Some(item) = self.get_mut(&key) {
                    item.apply_add(address, value)?;
                } else {
                    bail!(invalid_slot_name::<Self>(&key))
                }
            } else {
                bail!(invalid_slot_variant::<Self>(slot))
            }
        }

        Ok(())
    }

    fn apply_remove(&mut self, address: &mut Address, items: usize) -> Result<()> {
        if address.is_empty() {
            if items != 1 {
                bail!("When applying `Remove` operation to map, `items` should be 1")
            }
            self.clear();
        } else {
            let slot = address.pop_front().expect("Should have at least one slot");
            if let Slot::Name(key) = slot {
                if address.len() == 0 {
                    if items != 1 {
                        bail!("When applying `Remove` operation to map, `items` should be 1")
                    }
                    self.remove(&key);
                } else if let Some(item) = self.get_mut(&key) {
                    item.apply_remove(address, items)?;
                } else {
                    bail!(invalid_slot_name::<Self>(&key))
                }
            } else {
                bail!(invalid_slot_variant::<Self>(slot))
            }
        }

        Ok(())
    }

    fn apply_replace(&mut self, address: &mut Address, items: usize, value: &Value) -> Result<()> {
        if address.is_empty() {
            if items != 1 {
                bail!("When applying `Remove` operation to map, `items` should be 1")
            }
            self.clear();
            self.append(&mut Self::from_value(value)?);
        } else {
            let slot = address.pop_front().expect("Should have at least one slot");
            if let Slot::Name(key) = slot {
                if address.len() == 0 {
                    if items != 1 {
                        bail!("When applying `Remove` operation to map, `items` should be 1")
                    }
                    self.insert(key, Type::from_value(value)?);
                } else if let Some(item) = self.get_mut(&key) {
                    item.apply_replace(address, items, value)?;
                } else {
                    bail!(invalid_slot_name::<Self>(&key))
                }
            } else {
                bail!(invalid_slot_variant::<Self>(slot))
            }
        }

        Ok(())
    }

    fn apply_move(&mut self, from: &mut Address, items: usize, to: &mut Address) -> Result<()> {
        if from.len() == 1 {
            if let (Some(Slot::Name(from)), Some(Slot::Name(to))) =
                (from.pop_front(), to.pop_front())
            {
                let value = match self.remove(&from) {
                    Some(value) => value,
                    None => bail!(invalid_address::<Self>("`from` slot does not exist")),
                };
                self.insert(to, value);
            } else {
                bail!(invalid_address::<Self>(
                    "`from` and `to` slots should both be names"
                ))
            }
        } else if let Some(Slot::Name(key)) = from.pop_front() {
            if let Some(item) = self.get_mut(&key) {
                item.apply_move(from, items, to)?;
            } else {
                bail!(invalid_slot_name::<Self>(&key))
            }
        } else {
            bail!(invalid_address::<Self>(
                "`from` address is empty or does not start with name slot"
            ))
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{apply_new, diff};
    use test_utils::assert_json_is;

    macro_rules! mapint {
        ($json:tt) => {
            serde_json::from_value::<BTreeMap<String, u32>>(serde_json::json!($json))
        };
    }

    #[test]
    #[allow(clippy::many_single_char_names)]
    fn basics() -> Result<()> {
        let a = mapint!({})?;
        let b = mapint!({"a": 1 })?;
        let c = mapint!({"a": 1, "b": 2})?;
        let d = mapint!({"a": 1, "b": 3})?;
        let e = mapint!({"a": 1, "c": 3})?;

        let patch = diff(&a, &a);
        assert_json_is!(patch.ops, []);
        assert_eq!(apply_new(&a, &patch)?, a);

        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [{
            "type": "Replace",
            "address": [],
            "value": "<unserialized type>",
            "items": 1,
            "length": 1
        }]);
        assert_eq!(apply_new(&a, &patch)?, b);

        let patch = diff(&b, &a);
        assert_json_is!(patch.ops, [{
            "type": "Remove",
            "address": [],
            "items": 1
        }]);
        assert_eq!(apply_new(&b, &patch)?, a);

        let patch = diff(&b, &c);
        assert_json_is!(patch.ops, [{
            "type": "Add",
            "address": ["b"],
            "value": 2,
            "length": 1
        }]);
        assert_eq!(apply_new(&b, &patch)?, c);

        let patch = diff(&c, &d);
        assert_json_is!(patch.ops, [{
            "type": "Replace",
            "address": ["b"],
            "items": 1,
            "value": 3,
            "length": 1
        }]);
        assert_eq!(apply_new(&c, &patch)?, d);

        let patch = diff(&d, &e);
        assert_json_is!(patch.ops, [{
            "type": "Move",
            "from": ["b"],
            "to": ["c"],
            "items": 1
        }]);
        assert_eq!(apply_new(&d, &patch)?, e);

        let patch = diff(&e, &d);
        assert_json_is!(patch.ops, [{
            "type": "Move",
            "from": ["c"],
            "to": ["b"],
            "items": 1
        }]);
        assert_eq!(apply_new(&e, &patch)?, d);

        Ok(())
    }
}
