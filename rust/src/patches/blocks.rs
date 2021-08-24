use super::prelude::*;
use crate::dispatch_block;
use std::hash::Hasher;
use std::ops::Deref;
use stencila_schema::{
    BlockContent, ClaimSimple, CodeBlock, CodeChunk, CollectionSimple, FigureSimple, Heading,
    Include, List, MathBlock, Paragraph, QuoteBlock, TableSimple, ThematicBreak,
};

/// Implements patching for `BlockContent`
///
/// Generates and applies `Replace` and `Transform` operations between variants of block content.
/// All other operations are passed through to variants.
impl Patchable for BlockContent {
    patchable_is_same!();

    #[rustfmt::skip]
    fn is_equal(&self, other: &Self) -> Result<()> {
        match (self, other) {
            // Same variant so compare the two values
            (BlockContent::Claim(me), BlockContent::Claim(other)) => me.is_equal(other),
            (BlockContent::CodeBlock(me), BlockContent::CodeBlock(other)) => me.is_equal(other),
            (BlockContent::CodeChunk(me), BlockContent::CodeChunk(other)) => me.is_equal(other),
            (BlockContent::Collection(me), BlockContent::Collection(other)) => me.is_equal(other),
            (BlockContent::Figure(me), BlockContent::Figure(other)) => me.is_equal(other),
            (BlockContent::Heading(me), BlockContent::Heading(other)) => me.is_equal(other),
            (BlockContent::Include(me), BlockContent::Include(other)) => me.is_equal(other),
            (BlockContent::List(me), BlockContent::List(other)) => me.is_equal(other),
            (BlockContent::MathBlock(me), BlockContent::MathBlock(other)) => me.is_equal(other),
            (BlockContent::Paragraph(me), BlockContent::Paragraph(other)) => me.is_equal(other),
            (BlockContent::QuoteBlock(me), BlockContent::QuoteBlock(other)) => me.is_equal(other),
            (BlockContent::Table(me), BlockContent::Table(other)) => me.is_equal(other),
            (BlockContent::ThematicBreak(me), BlockContent::ThematicBreak(other)) => me.is_equal(other),

            // Different variants so by definition not equal
            _ => bail!(Error::NotEqual),
        }
    }

    fn make_hash<H: Hasher>(&self, state: &mut H) {
        dispatch_block!(self, make_hash, state)
    }

    patchable_diff!();

    #[rustfmt::skip]
    fn diff_same(&self, differ: &mut Differ, other: &Self) {
        match (self, other) {
            // Same variant so diff the two values
            (BlockContent::Claim(me), BlockContent::Claim(other)) => me.diff_same(differ, other),
            (BlockContent::CodeBlock(me), BlockContent::CodeBlock(other)) => me.diff_same(differ, other),
            (BlockContent::CodeChunk(me), BlockContent::CodeChunk(other)) => me.diff_same(differ, other),
            (BlockContent::Collection(me), BlockContent::Collection(other)) => me.diff_same(differ, other),
            (BlockContent::Figure(me), BlockContent::Figure(other)) => me.diff_same(differ, other),
            (BlockContent::Heading(me), BlockContent::Heading(other)) => me.diff_same(differ, other),
            (BlockContent::Include(me), BlockContent::Include(other)) => me.diff_same(differ, other),
            (BlockContent::List(me), BlockContent::List(other)) => me.diff_same(differ, other),
            (BlockContent::MathBlock(me), BlockContent::MathBlock(other)) => me.diff_same(differ, other),
            (BlockContent::Paragraph(me), BlockContent::Paragraph(other)) => me.diff_same(differ, other),
            (BlockContent::QuoteBlock(me), BlockContent::QuoteBlock(other)) => me.diff_same(differ, other),
            (BlockContent::Table(me), BlockContent::Table(other)) => me.diff_same(differ, other),
            (BlockContent::ThematicBreak(me), BlockContent::ThematicBreak(other)) => me.diff_same(differ, other),

            // Different variants so attempt to transform from one to the other
            _ => diff_transform(differ, self, other)
        }
    }

    fn apply_add(&mut self, keys: &mut Keys, value: &Box<dyn Any>) {
        dispatch_block!(self, apply_add, keys, value);
    }

    fn apply_remove(&mut self, keys: &mut Keys, items: usize) {
        dispatch_block!(self, apply_remove, keys, items);
    }

    fn apply_replace(&mut self, keys: &mut Keys, items: usize, value: &Box<dyn Any>) {
        if keys.is_empty() {
            if let Some(value) = value.deref().downcast_ref::<Self>() {
                *self = value.clone()
            } else {
                return invalid_value!();
            };
        } else {
            dispatch_block!(self, apply_replace, keys, items, value)
        }
    }

    fn apply_move(&mut self, from: &mut Keys, items: usize, to: &mut Keys) {
        dispatch_block!(self, apply_move, from, items, to);
    }

    fn apply_transform(&mut self, keys: &mut Keys, from: &str, to: &str) {
        if keys.is_empty() {
            assert_eq!(from, self.as_ref(), "Expected the same type");
            *self = apply_transform(self, to)
        } else {
            dispatch_block!(self, apply_transform, keys, from, to)
        }
    }
}

fn diff_transform(differ: &mut Differ, from: &BlockContent, to: &BlockContent) {
    match from {
        _ => (),
    }
    differ.replace(to)
}

fn apply_transform(from: &BlockContent, _to: &str) -> BlockContent {
    match from {
        _ => unreachable!(),
    }
}

// Implementations for `BlockContent` structs
// TODO: add all relevant fields to each struct

patchable_struct!(ClaimSimple, content);
patchable_struct!(CodeBlock, programming_language, text);
patchable_struct!(CodeChunk, programming_language, text);
patchable_struct!(CollectionSimple);
patchable_struct!(FigureSimple);
patchable_struct!(Heading);
patchable_struct!(Include, source);
patchable_struct!(List);
patchable_struct!(MathBlock, math_language, text);
patchable_struct!(Paragraph, content);
patchable_struct!(QuoteBlock, content);
patchable_struct!(TableSimple);
patchable_struct!(ThematicBreak, id);
