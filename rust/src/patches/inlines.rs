use crate::dispatch_inline;

use super::prelude::*;
use stencila_schema::{
    AudioObjectSimple, Cite, CiteGroup, CodeExpression, CodeFragment, Delete, Emphasis,
    ImageObjectSimple, InlineContent, Link, MathFragment, NontextualAnnotation, Note, Parameter,
    Quote, Strong, Subscript, Superscript, VideoObjectSimple,
};

impl Diffable for InlineContent {
    diffable_is_same!();

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
            (InlineContent::Null, InlineContent::Null) => Ok(()),
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

    diffable_diff!();

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
            (InlineContent::Null, InlineContent::Null) => {},
            (InlineContent::Number(me), InlineContent::Number(other)) => me.diff_same(differ, other),
            (InlineContent::Parameter(me), InlineContent::Parameter(other)) => me.diff_same(differ, other),
            (InlineContent::Quote(me), InlineContent::Quote(other)) => me.diff_same(differ, other),
            (InlineContent::String(me), InlineContent::String(other)) => me.diff_same(differ, other),
            (InlineContent::Strong(me), InlineContent::Strong(other)) => me.diff_same(differ, other),
            (InlineContent::Subscript(me), InlineContent::Subscript(other)) => me.diff_same(differ, other),
            (InlineContent::Superscript(me), InlineContent::Superscript(other)) => me.diff_same(differ, other),
            (InlineContent::VideoObject(me), InlineContent::VideoObject(other)) => me.diff_same(differ, other),

            // Different variants so attempt to transform from one to the other
            _ => {
                let self_variant = self.as_ref();
                let other_variant = other.as_ref();
                match self_variant {
                    // Strings are transformable to many other variants
                    "String" => differ.transform(self_variant, other_variant),
                    _ => differ.replace(other),
                }
            }
        }
    }

    // All operations, except `Transform`, are passed through to the variant

    fn apply_add(&mut self, keys: &mut Keys, value: &Box<dyn Any>) {
        dispatch_inline!(self, apply_add, keys, value);
    }

    fn apply_remove(&mut self, keys: &mut Keys, items: usize) {
        dispatch_inline!(self, apply_remove, keys, items);
    }

    fn apply_replace(&mut self, keys: &mut Keys, items: usize, value: &Box<dyn Any>) {
        dispatch_inline!(self, apply_replace, keys, items, value);
    }

    fn apply_move(&mut self, from: &mut Keys, items: usize, to: &mut Keys) {
        dispatch_inline!(self, apply_move, from, items, to);
    }

    /// Apply a transform between variants of `InlineContent`
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

diffable_todo!(AudioObjectSimple);
diffable_todo!(Cite);
diffable_todo!(CiteGroup);
diffable_struct!(CodeExpression, programming_language, text);
diffable_struct!(CodeFragment, programming_language, text);
diffable_struct!(Delete, content);
diffable_struct!(Emphasis, content);
diffable_todo!(ImageObjectSimple);
diffable_todo!(Link);
diffable_struct!(MathFragment, math_language, text);
diffable_struct!(NontextualAnnotation, content);
diffable_todo!(Note);
diffable_todo!(Parameter);
diffable_todo!(Quote);
diffable_struct!(Strong, content);
diffable_struct!(Subscript, content);
diffable_struct!(Superscript, content);
diffable_todo!(VideoObjectSimple);
