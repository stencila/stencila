use common::serde::de::DeserializeOwned;

use super::prelude::*;

/// Implements patching for `Option`
///
/// Generates `Add` and `Remove` operations (with no address) given differences in
/// `Some` and `None` between the two options.
/// When applying `Add` and `Remove` operations, if there are no address
/// then apply here, otherwise pass operation through to any value.
/// All other operations passed through.
impl<Type: Patchable> Patchable for Option<Type>
where
    Type: Clone + DeserializeOwned + Send + 'static,
{
    fn diff(&self, other: &Self, differ: &mut Differ) {
        match (self, other) {
            (None, None) => (),
            (None, Some(value)) => differ.add(value),
            (Some(_), None) => differ.remove(),
            (Some(me), Some(other)) => me.diff(other, differ),
        }
    }

    fn apply_add(&mut self, address: &mut Address, value: &Value) -> Result<()> {
        if address.is_empty() {
            *self = Self::from_value(value)?;
            Ok(())
        } else if let Some(me) = self {
            me.apply_add(address, value)
        } else {
            bail!(invalid_address::<Self>(
                "option is empty but address is not"
            ))
        }
    }

    fn apply_remove(&mut self, address: &mut Address, items: usize) -> Result<()> {
        if address.is_empty() {
            *self = None;
            Ok(())
        } else if let Some(me) = self {
            me.apply_remove(address, items)
        } else {
            bail!(invalid_address::<Self>(
                "option is empty but address is not"
            ))
        }
    }

    fn apply_replace(&mut self, address: &mut Address, items: usize, value: &Value) -> Result<()> {
        if address.is_empty() {
            *self = Self::from_value(value)?;
            Ok(())
        } else if let Some(me) = self {
            me.apply_replace(address, items, value)
        } else {
            bail!(invalid_address::<Self>(
                "option is empty but address is not"
            ))
        }
    }

    fn apply_move(&mut self, from: &mut Address, items: usize, to: &mut Address) -> Result<()> {
        if let Some(me) = self {
            me.apply_move(from, items, to)
        } else {
            bail!(invalid_patch_operation::<Self>("move"))
        }
    }

    fn apply_transform(&mut self, address: &mut Address, from: &str, to: &str) -> Result<()> {
        if let Some(me) = self {
            me.apply_transform(address, from, to)
        } else {
            bail!(invalid_patch_operation::<Self>("transform"))
        }
    }

    fn from_value(value: &Value) -> Result<Self> {
        Ok(Some(Type::from_value(value)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{apply_new, diff};
    use stencila_schema::Integer;
    use test_utils::assert_json_is;

    #[test]
    fn basic() -> Result<()> {
        // No diff

        assert_json_is!(diff::<Option<Integer>>(&None, &None).ops, []);
        assert_json_is!(diff(&Some(1), &Some(1)).ops, []);

        // None to Some: Add with no key
        let a = None;
        let b = Some("abc".to_string());
        let patch = diff(&a, &b);
        assert_json_is!(
            patch.ops,
            [{"type": "Add", "address": [], "value": "abc".to_string(), "length": 1}]
        );
        assert_json_is!(apply_new(&a, &patch)?, b);

        // Some to Some: Add with a key
        let a = Some("a".to_string());
        let b = Some("abc".to_string());
        let patch = diff(&a, &b);
        assert_json_is!(
            patch.ops,
            [{"type": "Add", "address": [1], "value": "bc".to_string(), "length": 2}]
        );
        assert_json_is!(apply_new(&a, &patch)?, b);

        // Some to None: Remove with no key
        let a = Some("abc".to_string());
        let b = None;
        let patch = diff(&a, &b);
        assert_json_is!(
            patch.ops,
            [{"type": "Remove", "address": [], "items": 1}]
        );
        assert_json_is!(apply_new(&a, &patch)?, b);

        // Some to Some: Remove with key
        let a = Some("abc".to_string());
        let b = Some("ac".to_string());
        let patch = diff(&a, &b);
        assert_json_is!(
            patch.ops,
            [{"type": "Remove", "address": [1], "items": 1}]
        );
        assert_json_is!(apply_new(&a, &patch)?, b);

        // Replace
        let a = Some("abc".to_string());
        let b = Some("a@c".to_string());
        let patch = diff(&a, &b);
        assert_json_is!(
            patch.ops,
            [{"type": "Replace", "address": [1], "items": 1, "value": "@", "length": 1}]
        );
        assert_json_is!(apply_new(&a, &patch)?, b);

        Ok(())
    }
}
