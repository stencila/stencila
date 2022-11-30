//! Patching for [`Box`] properties of nodes

use std::ops::{Deref, DerefMut};

use super::prelude::*;

/// Implements patching for `Box`
///
/// All methods simply pass through to the boxed value.
impl<Type: Patchable> Patchable for Box<Type> {
    fn diff(&self, other: &Self, differ: &mut Differ) {
        self.deref().diff(other, differ)
    }

    fn apply_add(&mut self, address: &mut Address, value: Value) -> Result<()> {
        self.deref_mut().apply_add(address, value)
    }

    fn apply_add_many(&mut self, address: &mut Address, values: Values) -> Result<()> {
        self.deref_mut().apply_add_many(address, values)
    }

    fn apply_remove(&mut self, address: &mut Address) -> Result<()> {
        self.deref_mut().apply_remove(address)
    }

    fn apply_remove_many(&mut self, address: &mut Address, items: usize) -> Result<()> {
        self.deref_mut().apply_remove_many(address, items)
    }

    fn apply_replace(&mut self, address: &mut Address, value: Value) -> Result<()> {
        self.deref_mut().apply_replace(address, value)
    }

    fn apply_replace_many(
        &mut self,
        address: &mut Address,
        items: usize,
        values: Values,
    ) -> Result<()> {
        self.deref_mut().apply_replace_many(address, items, values)
    }

    fn apply_move(&mut self, from: &mut Address, to: &mut Address) -> Result<()> {
        self.deref_mut().apply_move(from, to)
    }

    fn apply_transform(&mut self, address: &mut Address, from: &str, to: &str) -> Result<()> {
        self.deref_mut().apply_transform(address, from, to)
    }

    fn to_value(&self) -> Value {
        self.deref().to_value()
    }

    fn from_value(value: Value) -> Result<Self> {
        Ok(Box::new(Type::from_value(value)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{apply_new, diff};
    use stencila_schema::CodeBlock;
    use test_utils::{assert_json_eq, assert_json_is};

    #[test]
    fn basic() -> Result<()> {
        // Add, remove, replace
        let a = Box::new("abcd".to_string());
        let b = Box::new("eacp".to_string());
        let patch = diff(&a, &b);
        assert_json_is!(
            patch.ops,
            [
                {"type": "Add", "address": [0], "value": "e"},
                {"type": "Remove", "address": [2]},
                {"type": "Replace", "address": [3], "value": "p"}
            ]
        );
        assert_json_is!(apply_new(&a, patch)?, b);

        Ok(())
    }

    // Regression, found by proptest, related to bug in `from_value`
    #[test]
    fn regression_1() -> Result<()> {
        let a = CodeBlock::default();
        let b = CodeBlock {
            programming_language: Some(Box::new("a".to_string())),
            ..Default::default()
        };
        let patch = diff(&a, &b);
        assert_json_is!(patch.ops, [
            {"type": "Add", "address": ["programmingLanguage"], "value": "a"},
        ]);
        assert_json_eq!(apply_new(&a, patch)?, b);

        Ok(())
    }
}
