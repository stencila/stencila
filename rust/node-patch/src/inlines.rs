use super::prelude::*;
use codec_txt::ToTxt;
use node_dispatch::{dispatch_inline, dispatch_inline_pair};
use std::hash::Hasher;
use stencila_schema::*;

/// Implements patching for `InlineContent`
///
/// Generates and applies `Replace` and `Transform` operations between variants of inline content.
/// All other operations are passed through to variants.
impl Patchable for InlineContent {
    fn is_equal(&self, other: &Self) -> Result<()> {
        dispatch_inline_pair!(self, other, bail!(Error::NotEqual), is_equal)
    }

    fn make_hash<H: Hasher>(&self, state: &mut H) {
        dispatch_inline!(self, make_hash, state)
    }

    fn diff(&self, other: &Self, differ: &mut Differ) {
        dispatch_inline_pair!(
            self,
            other,
            diff_transform(differ, self, other),
            diff,
            differ
        )
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

// Implementations for `InlineContent` structs, including related "full" structs
// (e.g. `ImageObject` vs `ImageObjectSimple`) which are actually "works".

replaceable_struct!(
    Cite,
    target,
    citation_intent,
    citation_mode,
    citation_prefix,
    citation_suffix,
    content,
    page_end,
    page_start,
    pagination
);
patchable_enum!(CitationIntentEnumeration);
patchable_enum!(CiteCitationMode);
patchable_enum!(CitePageEnd);
patchable_enum!(CitePageStart);
patchable_struct!(CiteGroup, items);

patchable_struct!(
    CodeExpression,
    programming_language,
    text,
    output,
    errors,
    code_dependencies,
    code_dependents,
    compile_digest,
    execute_digest,
    execute_required,
    execute_status,
    execute_ended,
    execute_duration
);
patchable_struct!(CodeFragment, programming_language, text);
patchable_struct!(Delete, content);
patchable_struct!(Emphasis, content);
patchable_struct!(Link, content, target);
patchable_struct!(MathFragment, math_language, text);
patchable_struct!(NontextualAnnotation, content);
patchable_struct!(Note, content);
patchable_struct!(Quote, content);
patchable_struct!(Strong, content);
patchable_struct!(Subscript, content);
patchable_struct!(Superscript, content);

/// Generate a `impl Patchable` for a `MediaObject` `struct` which avoids creating
/// a very large number of operations when diffing a base64 encoded images (which
/// can swamp client as well as being slow to generate)
macro_rules! patchable_media_object {
    ($type:ty $(, $field:ident )*) => {
        impl Patchable for $type {
            patchable_struct_is_equal!($( $field )*);
            patchable_struct_hash!($( $field )*);

            fn diff(&self, other: &Self, differ: &mut Differ) {
                $(
                    let field = stringify!($field);
                    if field == "content_url" &&
                       (self.content_url.starts_with("data:") || other.content_url.starts_with("data:")) &&
                       self.content_url != other.content_url {
                        differ.push(
                            Operation::Replace {
                                address: Address::from("content_url"),
                                items: 1,
                                value: Box::new(other.content_url.clone()),
                                length: 1,
                                html: None,
                            }
                        )
                    } else {
                        differ.field(field, &self.$field, &other.$field);
                    }
                )*
            }

            patchable_struct_apply_add!($( $field )*);
            patchable_struct_apply_remove!($( $field )*);
            patchable_struct_apply_replace!($( $field )*);
            patchable_struct_apply_move!($( $field )*);
            patchable_struct_apply_transform!($( $field )*);
        }
    };
}

patchable_media_object!(AudioObject, content_url);
patchable_media_object!(AudioObjectSimple, content_url);
patchable_media_object!(ImageObject, content_url);
patchable_media_object!(ImageObjectSimple, content_url);
patchable_media_object!(VideoObject, content_url);
patchable_media_object!(VideoObjectSimple, content_url);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{apply_new, diff, equal};
    use serde_json::json;
    use stencila_schema::Node;
    use test_utils::{assert_json_eq, assert_json_is};

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
        assert_json_is!(patch.ops, [
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
        assert_json_is!(patch.ops, [
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
        assert_json_is!(patch.ops, [
            {"type": "Transform", "address": [], "from": "String", "to": "Emphasis"}
        ]);
        assert_json_eq!(apply_new(&a, &patch)?, b);

        let patch = diff(&b, &a);
        assert_json_is!(patch.ops, [
            {"type": "Transform", "address": [], "from": "Emphasis", "to": "String"}
        ]);
        assert_json_eq!(apply_new(&b, &patch)?, a);

        let patch = diff(&b, &c);
        assert_json_is!(patch.ops, [
            {"type": "Transform", "address": [], "from": "Emphasis", "to": "Strong"}
        ]);
        assert_json_eq!(apply_new(&b, &patch)?, c);

        let patch = diff(&c, &b);
        assert_json_is!(patch.ops, [
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
        assert_json_is!(patch.ops, [
            {
                "type": "Replace", "address": [], "items": 1,
                "value": {"type": "Emphasis", "content": ["b"]},
                "length": 1
            }
        ]);
        assert_json_eq!(apply_new(&a, &patch)?, b);

        let patch = diff(&b, &a);
        assert_json_is!(patch.ops, [
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
        assert_json_is!(patch.ops, [
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
        assert_json_is!(patch.ops, [
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
        assert_json_is!(patch.ops, [
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
