use super::prelude::*;
use crate::dispatch_block;
use stencila_schema::{
    BlockContent, ClaimSimple, CodeBlock, CodeChunk, CollectionSimple, FigureSimple, Heading,
    Include, List, MathBlock, Paragraph, QuoteBlock, TableSimple, ThematicBreak,
};

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
            _ => todo!("transform types"),
        }
    }

    // All operations, except `Transform`, are passed through to the variant

    fn apply_add(&mut self, keys: &mut Keys, value: &Box<dyn Any>) {
        dispatch_block!(self, apply_add, keys, value);
    }

    fn apply_remove(&mut self, keys: &mut Keys, items: usize) {
        dispatch_block!(self, apply_remove, keys, items);
    }

    fn apply_replace(&mut self, keys: &mut Keys, items: usize, value: &Box<dyn Any>) {
        dispatch_block!(self, apply_replace, keys, items, value);
    }

    fn apply_move(&mut self, from: &mut Keys, items: usize, to: &mut Keys) {
        dispatch_block!(self, apply_move, from, items, to);
    }

    /// Apply a transform between variants of `BlockContent`
    fn apply_transform(&mut self, keys: &mut Keys, from: &str, _to: &str) {
        if keys.is_empty() {
            assert_eq!(from, self.as_ref(), "Expected the same type");
            *self = match self {
                _ => todo!(),
            }
        } else {
            todo!()
        }
    }
}

patchable_struct!(ClaimSimple, content);
patchable_struct!(CodeBlock, programming_language, text);
patchable_struct!(CodeChunk, programming_language, text);
patchable_todo!(CollectionSimple);
patchable_todo!(FigureSimple);
patchable_todo!(Heading);
patchable_struct!(Include, source);
patchable_todo!(List);
patchable_struct!(MathBlock, math_language, text);
patchable_struct!(Paragraph, content);
patchable_struct!(QuoteBlock, content);
patchable_todo!(TableSimple);
patchable_struct!(ThematicBreak, id);
