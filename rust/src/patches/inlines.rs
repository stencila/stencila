use super::prelude::*;
use crate::{dispatch_inline, methods::encode::txt::ToTxt};
use std::hash::Hasher;
use stencila_schema::{
    AudioObjectSimple, Cite, CiteGroup, CodeExpression, CodeFragment, Delete, Emphasis,
    ImageObjectSimple, InlineContent, Link, MathFragment, NontextualAnnotation, Note, Parameter,
    Quote, Strong, Subscript, Superscript, VideoObjectSimple,
};

/// Implements patching for `InlineContent`
///
/// Generates and applies `Replace` and `Transform` operations between variants of inline content.
/// All other operations are passed through to variants.
impl Patchable for InlineContent {
    /// Resolve an [`Address`] into a node [`Pointer`].
    ///
    /// `InlineContent` is one of the pointer variants so return a `Pointer::Inline` if
    /// the address is empty. Otherwise dispatch to variant.
    fn resolve(&mut self, address: &mut Address) -> Result<Pointer> {
        match address.is_empty() {
            true => Ok(Pointer::Inline(self)),
            false => dispatch_inline!(self, resolve, address),
        }
    }

    /// Find a node based on its `id` and return a [`Pointer`] to it.
    ///
    /// Dispatch to variant and if it returns `Pointer::Some` then rewrite to `Pointer::Inline`
    fn find(&mut self, id: &str) -> Pointer {
        let pointer = dispatch_inline!(self, find, id);
        match pointer {
            Pointer::Some => Pointer::Inline(self),
            _ => Pointer::None,
        }
    }

    patchable_is_same!();

    #[rustfmt::skip]
    fn is_equal(&self, other: &Self) -> Result<()> {
        match (self, other) {
            // Same variant so compare the two values
            (InlineContent::AudioObject(me), InlineContent::AudioObject(other)) => me.is_equal(other),
            (InlineContent::Boolean(me), InlineContent::Boolean(other)) => me.is_equal(other),
            (InlineContent::Cite(me), InlineContent::Cite(other)) => me.is_equal(other),
            (InlineContent::CiteGroup(me), InlineContent::CiteGroup(other)) => me.is_equal(other),
            (InlineContent::CodeExpression(me), InlineContent::CodeExpression(other)) => me.is_equal(other),
            (InlineContent::CodeFragment(me), InlineContent::CodeFragment(other)) => me.is_equal(other),
            (InlineContent::Delete(me), InlineContent::Delete(other)) => me.is_equal(other),
            (InlineContent::Emphasis(me), InlineContent::Emphasis(other)) => me.is_equal(other),
            (InlineContent::ImageObject(me), InlineContent::ImageObject(other)) => me.is_equal(other),
            (InlineContent::Integer(me), InlineContent::Integer(other)) => me.is_equal(other),
            (InlineContent::Link(me), InlineContent::Link(other)) => me.is_equal(other),
            (InlineContent::MathFragment(me), InlineContent::MathFragment(other)) => me.is_equal(other),
            (InlineContent::NontextualAnnotation(me), InlineContent::NontextualAnnotation(other)) => me.is_equal(other),
            (InlineContent::Note(me), InlineContent::Note(other)) => me.is_equal(other),
            (InlineContent::Null(me), InlineContent::Null(other)) => me.is_equal(other),
            (InlineContent::Number(me), InlineContent::Number(other)) => me.is_equal(other),
            (InlineContent::Parameter(me), InlineContent::Parameter(other)) => me.is_equal(other),
            (InlineContent::Quote(me), InlineContent::Quote(other)) => me.is_equal(other),
            (InlineContent::String(me), InlineContent::String(other)) => me.is_equal(other),
            (InlineContent::Strong(me), InlineContent::Strong(other)) => me.is_equal(other),
            (InlineContent::Subscript(me), InlineContent::Subscript(other)) => me.is_equal(other),
            (InlineContent::Superscript(me), InlineContent::Superscript(other)) => me.is_equal(other),
            (InlineContent::VideoObject(me), InlineContent::VideoObject(other)) => me.is_equal(other),

            // Different variants so by definition not equal
            _ => bail!(Error::NotEqual),
        }
    }

    fn make_hash<H: Hasher>(&self, state: &mut H) {
        dispatch_inline!(self, make_hash, state)
    }

    patchable_diff!();

    #[rustfmt::skip]
    fn diff_same(&self, differ: &mut Differ, other: &Self) {
        match (self, other) {
            // Same variant so diff the two values
            (InlineContent::AudioObject(me), InlineContent::AudioObject(other)) => me.diff_same(differ, other),
            (InlineContent::Boolean(me), InlineContent::Boolean(other)) => me.diff_same(differ, other),
            (InlineContent::Cite(me), InlineContent::Cite(other)) => me.diff_same(differ, other),
            (InlineContent::CiteGroup(me), InlineContent::CiteGroup(other)) => me.diff_same(differ, other),
            (InlineContent::CodeExpression(me), InlineContent::CodeExpression(other)) => me.diff_same(differ, other),
            (InlineContent::CodeFragment(me), InlineContent::CodeFragment(other)) => me.diff_same(differ, other),
            (InlineContent::Delete(me), InlineContent::Delete(other)) => me.diff_same(differ, other),
            (InlineContent::Emphasis(me), InlineContent::Emphasis(other)) => me.diff_same(differ, other),
            (InlineContent::ImageObject(me), InlineContent::ImageObject(other)) => me.diff_same(differ, other),
            (InlineContent::Integer(me), InlineContent::Integer(other)) => me.diff_same(differ, other),
            (InlineContent::Link(me), InlineContent::Link(other)) => me.diff_same(differ, other),
            (InlineContent::MathFragment(me), InlineContent::MathFragment(other)) => me.diff_same(differ, other),
            (InlineContent::NontextualAnnotation(me), InlineContent::NontextualAnnotation(other)) => me.diff_same(differ, other),
            (InlineContent::Note(me), InlineContent::Note(other)) => me.diff_same(differ, other),
            (InlineContent::Null(me), InlineContent::Null(other)) => me.diff_same(differ, other),
            (InlineContent::Number(me), InlineContent::Number(other)) => me.diff_same(differ, other),
            (InlineContent::Parameter(me), InlineContent::Parameter(other)) => me.diff_same(differ, other),
            (InlineContent::Quote(me), InlineContent::Quote(other)) => me.diff_same(differ, other),
            (InlineContent::String(me), InlineContent::String(other)) => me.diff_same(differ, other),
            (InlineContent::Strong(me), InlineContent::Strong(other)) => me.diff_same(differ, other),
            (InlineContent::Subscript(me), InlineContent::Subscript(other)) => me.diff_same(differ, other),
            (InlineContent::Superscript(me), InlineContent::Superscript(other)) => me.diff_same(differ, other),
            (InlineContent::VideoObject(me), InlineContent::VideoObject(other)) => me.diff_same(differ, other),

            // Different variants so attempt to transform from one to the other
            _ => diff_transform(differ, self, other)
        }
    }

    fn apply_add(&mut self, address: &mut Address, value: &Value) -> Result<()> {
        dispatch_inline!(self, apply_add, address, value)
    }

    fn apply_remove(&mut self, address: &mut Address, items: usize) -> Result<()> {
        dispatch_inline!(self, apply_remove, address, items)
    }

    fn apply_replace(&mut self, address: &mut Address, items: usize, value: &Value) -> Result<()> {
        if address.is_empty() {
            *self = Self::from_value(value)?;
            Ok(())
        } else {
            dispatch_inline!(self, apply_replace, address, items, value)
        }
    }

    fn apply_move(&mut self, from: &mut Address, items: usize, to: &mut Address) -> Result<()> {
        dispatch_inline!(self, apply_move, from, items, to)
    }

    fn apply_transform(&mut self, address: &mut Address, from: &str, to: &str) -> Result<()> {
        if address.is_empty() {
            assert_eq!(from, self.as_ref(), "Expected the same type");
            *self = apply_transform(self, to);
            Ok(())
        } else {
            dispatch_inline!(self, apply_transform, address, from, to)
        }
    }
}

fn diff_transform(differ: &mut Differ, from: &InlineContent, to: &InlineContent) {
    match from {
        InlineContent::String(string) => match to {
            InlineContent::Emphasis(Emphasis { content, .. })
            | InlineContent::Delete(Delete { content, .. })
            | InlineContent::Strong(Strong { content, .. })
            | InlineContent::Subscript(Subscript { content, .. })
            | InlineContent::Superscript(Superscript { content, .. }) => {
                if *string == content.to_txt() {
                    return differ.transform(from.as_ref(), to.as_ref());
                }
            }
            _ => (),
        },
        InlineContent::Emphasis(Emphasis { content, .. })
        | InlineContent::Delete(Delete { content, .. })
        | InlineContent::Strong(Strong { content, .. })
        | InlineContent::Subscript(Subscript { content, .. })
        | InlineContent::Superscript(Superscript { content, .. }) => match to {
            InlineContent::String(string) => {
                if content.to_txt() == *string {
                    return differ.transform(from.as_ref(), to.as_ref());
                }
            }
            InlineContent::Emphasis(Emphasis { content: to_c, .. })
            | InlineContent::Delete(Delete { content: to_c, .. })
            | InlineContent::Strong(Strong { content: to_c, .. })
            | InlineContent::Subscript(Subscript { content: to_c, .. })
            | InlineContent::Superscript(Superscript { content: to_c, .. }) => {
                if content.is_equal(to_c).is_ok() {
                    return differ.transform(from.as_ref(), to.as_ref());
                }
            }
            _ => (),
        },
        _ => (),
    }
    differ.replace(to)
}

fn apply_transform(from: &InlineContent, to: &str) -> InlineContent {
    match from {
        InlineContent::String(_) => {
            let content = vec![from.clone()];
            match to {
                "Emphasis" => InlineContent::Emphasis(Emphasis {
                    content,
                    ..Default::default()
                }),
                "Delete" => InlineContent::Delete(Delete {
                    content,
                    ..Default::default()
                }),
                "Strong" => InlineContent::Strong(Strong {
                    content,
                    ..Default::default()
                }),
                "Subscript" => InlineContent::Subscript(Subscript {
                    content,
                    ..Default::default()
                }),
                "Superscript" => InlineContent::Superscript(Superscript {
                    content,
                    ..Default::default()
                }),
                _ => unreachable!(),
            }
        }
        InlineContent::Emphasis(Emphasis { content, .. })
        | InlineContent::Delete(Delete { content, .. })
        | InlineContent::Strong(Strong { content, .. })
        | InlineContent::Subscript(Subscript { content, .. })
        | InlineContent::Superscript(Superscript { content, .. }) => {
            let content = content.clone();
            match to {
                "String" => InlineContent::String(content.to_txt()),
                "Emphasis" => InlineContent::Emphasis(Emphasis {
                    content,
                    ..Default::default()
                }),
                "Delete" => InlineContent::Delete(Delete {
                    content,
                    ..Default::default()
                }),
                "Strong" => InlineContent::Strong(Strong {
                    content,
                    ..Default::default()
                }),
                "Subscript" => InlineContent::Subscript(Subscript {
                    content,
                    ..Default::default()
                }),
                "Superscript" => InlineContent::Superscript(Superscript {
                    content,
                    ..Default::default()
                }),
                _ => unreachable!(),
            }
        }
        _ => unreachable!(),
    }
}

// Implementations for `InlineContent` structs
// TODO: add all relevant fields to each struct

patchable_struct!(AudioObjectSimple, content_url);
patchable_struct!(Cite);
patchable_struct!(CiteGroup, items);
patchable_struct!(CodeExpression, programming_language, text, output);
patchable_struct!(CodeFragment, programming_language, text);
patchable_struct!(Delete, content);
patchable_struct!(Emphasis, content);
patchable_struct!(ImageObjectSimple, content_url);
patchable_struct!(Link, content, target);
patchable_struct!(MathFragment, math_language, text);
patchable_struct!(NontextualAnnotation, content);
patchable_struct!(Note, content);
patchable_struct!(Parameter, name, value);
patchable_struct!(Quote, content);
patchable_struct!(Strong, content);
patchable_struct!(Subscript, content);
patchable_struct!(Superscript, content);
patchable_struct!(VideoObjectSimple, content_url);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        assert_json, assert_json_eq,
        patches::{apply_new, diff, equal},
    };
    use serde_json::json;
    use stencila_schema::Node;

    // Test that operations with address are passed through
    #[test]
    fn passthrough() -> Result<()> {
        // Simple
        let a = InlineContent::String("abcd".to_string());
        let b = InlineContent::String("eacp".to_string());

        assert!(equal(&a, &a));
        assert!(!equal(&a, &b));
        assert!(!equal(&a, &InlineContent::Boolean(true)));

        let patch = diff(&a, &b);
        assert_json!(patch.ops, [
            {"type": "Add", "address": [0], "value": "e", "length": 1},
            {"type": "Remove", "address": [2], "items": 1},
            {"type": "Replace", "address": [3], "items": 1, "value": "p", "length": 1}
        ]);
        assert_json_eq!(apply_new(&a, &patch)?, b);

        // Nested
        let a = InlineContent::Delete(Delete {
            content: vec![InlineContent::String("abcd".to_string())],
            ..Default::default()
        });
        let b = InlineContent::Delete(Delete {
            content: vec![InlineContent::String("ab".to_string())],
            ..Default::default()
        });

        assert!(equal(&a, &a));
        assert!(!equal(&a, &b));

        let patch = diff(&a, &b);
        assert_json!(patch.ops, [
            {"type": "Remove", "address": ["content", 0, 2], "items": 2},
        ]);
        assert_json_eq!(apply_new(&a, &patch)?, b);

        Ok(())
    }

    // Test that strings and other simple inline content are transformed
    #[test]
    fn transform() -> Result<()> {
        let a = InlineContent::String("abcd".to_string());
        let b = InlineContent::Emphasis(Emphasis {
            content: vec![a.clone()],
            ..Default::default()
        });
        let c = InlineContent::Strong(Strong {
            content: vec![a.clone()],
            ..Default::default()
        });

        let patch = diff(&a, &b);
        assert_json!(patch.ops, [
            {"type": "Transform", "address": [], "from": "String", "to": "Emphasis"}
        ]);
        assert_json_eq!(apply_new(&a, &patch)?, b);

        let patch = diff(&b, &a);
        assert_json!(patch.ops, [
            {"type": "Transform", "address": [], "from": "Emphasis", "to": "String"}
        ]);
        assert_json_eq!(apply_new(&b, &patch)?, a);

        let patch = diff(&b, &c);
        assert_json!(patch.ops, [
            {"type": "Transform", "address": [], "from": "Emphasis", "to": "Strong"}
        ]);
        assert_json_eq!(apply_new(&b, &patch)?, c);

        let patch = diff(&c, &b);
        assert_json!(patch.ops, [
            {"type": "Transform", "address": [], "from": "Strong", "to": "Emphasis"}
        ]);
        assert_json_eq!(apply_new(&c, &patch)?, b);

        Ok(())
    }

    // Test that if content differs a replacement is done instead of a transform
    #[test]
    fn replace_different_content() -> Result<()> {
        let a = InlineContent::String("a".to_string());
        let b = InlineContent::Emphasis(Emphasis {
            content: vec![InlineContent::String("b".to_string())],
            ..Default::default()
        });

        let patch = diff(&a, &b);
        assert_json!(patch.ops, [
            {
                "type": "Replace", "address": [], "items": 1,
                "value": {"type": "Emphasis", "content": ["b"]},
                "length": 1
            }
        ]);
        assert_json_eq!(apply_new(&a, &patch)?, b);

        let patch = diff(&b, &a);
        assert_json!(patch.ops, [
            {
                "type": "Replace", "address": [], "items": 1,
                "value": "a",
                "length": 1
            }
        ]);
        assert_json_eq!(apply_new(&b, &patch)?, a);

        Ok(())
    }

    // Test that if content is same but types differ that replacement
    // is done.
    #[test]
    fn replace_different_types() -> Result<()> {
        let a = InlineContent::AudioObject(AudioObjectSimple {
            content_url: "a".to_string(),
            ..Default::default()
        });
        let b = InlineContent::ImageObject(ImageObjectSimple {
            content_url: "a".to_string(),
            ..Default::default()
        });

        let patch = diff(&a, &b);
        assert_json!(patch.ops, [
            {
                "type": "Replace", "address": [], "items": 1,
                "value": {"type": "ImageObject", "contentUrl": "a"},
                "length": 1
            }
        ]);
        assert_json_eq!(apply_new(&a, &patch)?, b);

        Ok(())
    }

    // Regression tests of minimal failing cases found using property testing
    // and elsewhere.

    #[test]
    fn regression_1() -> Result<()> {
        let a = vec![
            InlineContent::Superscript(Superscript {
                content: vec![InlineContent::String("a".to_string())],
                ..Default::default()
            }),
            InlineContent::ImageObject(ImageObjectSimple {
                content_url: "a.gif".to_string(),
                ..Default::default()
            }),
        ];
        let b = vec![
            InlineContent::Superscript(Superscript {
                content: vec![InlineContent::String("b".to_string())],
                ..Default::default()
            }),
            InlineContent::AudioObject(AudioObjectSimple {
                content_url: "a.flac".to_string(),
                ..Default::default()
            }),
        ];
        let patch = diff(&a, &b);
        assert_json!(patch.ops, [
            {
                "type": "Replace", "address": [0, "content", 0, 0], "items": 1,
                "value": "b", "length": 1
            },
            {
                "type": "Replace", "address": [1], "items": 1,
                "value": [{ "type": "AudioObject", "contentUrl": "a.flac"}], "length": 1
            }
        ]);
        assert_json_eq!(apply_new(&a, &patch)?, b);

        Ok(())
    }

    #[test]
    fn regression_2() -> Result<()> {
        let a = InlineContent::Parameter(Parameter {
            ..Default::default()
        });
        let b = InlineContent::Parameter(Parameter {
            value: Some(Box::new(Node::Number(1.23))),
            ..Default::default()
        });

        let patch = diff(&a, &b);
        assert_json!(patch.ops, [
            {
                "type": "Add",
                "address": ["value"],
                "value": 1.23,
                "length": 1
            },
        ]);
        assert_json_eq!(apply_new(&a, &patch)?, b);

        let patch: Patch = serde_json::from_value(json!({
            "ops": [
                {
                    "type": "Replace",
                    "address": ["value"],
                    "items": 1,
                    "value": 1.23,
                    "length": 1
                },
            ]
        }))?;
        assert_json_eq!(apply_new(&a, &patch)?, b);

        Ok(())
    }
}
