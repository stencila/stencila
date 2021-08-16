use super::prelude::*;
use stencila_schema::{Emphasis, InlineContent};

impl Diffable for InlineContent {
    diffable_is_same!(InlineContent);

    fn is_equal(&self, other: &Self) -> Result<()> {
        match (self, other) {
            // TODO replace with a macro
            (InlineContent::String(me), InlineContent::String(other)) => me.is_equal(other),
            _ => bail!(Error::NotEqual),
        }
    }

    diffable_diff!(InlineContent);

    fn diff_same(&self, differ: &mut Differ, other: &Self) {
        match (self, other) {
            // Same variant so diff the two values
            (InlineContent::String(me), InlineContent::String(other)) => {
                me.diff_same(differ, other)
            }
            // Different variants so attempt to transform from one to the other
            _ => {
                let self_variant = self.as_ref();
                let other_variant = other.as_ref();
                match self_variant {
                    // Strings are transformable to all other variants
                    "String" => differ.transform(self_variant, other_variant),
                    _ => differ.replace(other),
                }
            }
        }
    }

    fn apply_replace(&mut self, keys: &mut Keys, items: usize, value: &Box<dyn Any>) {
        match self {
            InlineContent::String(me) => me.apply_replace(keys, items, value),
            _ => todo!(),
        }
    }

    fn apply_transform(&mut self, keys: &mut Keys, from: &str, to: &str) {
        if keys.is_empty() {
            assert_eq!(from, self.as_ref(), "Expected the same type");
            *self = match self {
                InlineContent::String(_) => match to {
                    "Emphasis" => InlineContent::Emphasis(Emphasis {
                        content: vec![self.clone()],
                        ..Default::default()
                    }),
                    _ => todo!(),
                },
                _ => todo!(),
            }
        } else {
            todo!()
        }
    }
}
