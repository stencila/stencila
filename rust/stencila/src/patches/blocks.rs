use super::prelude::*;
use crate::dispatch_block;
use std::hash::Hasher;
use stencila_schema::{
    BlockContent, ClaimClaimType, ClaimSimple, CodeBlock, CodeChunk, CollectionSimple,
    FigureSimple, Heading, Include, List, ListItem, ListItemContent, ListOrder, MathBlock,
    Paragraph, QuoteBlock, TableCell, TableCellCellType, TableCellContent, TableRow,
    TableRowRowType, TableSimple, ThematicBreak,
};

/// Implements patching for `BlockContent`
///
/// Generates and applies `Replace` and `Transform` operations between variants of block content.
/// All other operations are passed through to variants.
impl Patchable for BlockContent {
    /// Resolve an [`Address`] into a node [`Pointer`].
    ///
    /// `BlockContent` is one of the pointer variants so return a `Pointer::Block` if
    /// the address is empty. Otherwise dispatch to variant.
    fn resolve(&mut self, address: &mut Address) -> Result<Pointer> {
        match address.is_empty() {
            true => Ok(Pointer::Block(self)),
            false => dispatch_block!(self, resolve, address),
        }
    }

    /// Find a node based on its `id` and return a [`Pointer`] to it.
    ///
    /// Dispatch to variant and if it returns `Pointer::Some` then rewrite to `Pointer::Block`
    fn find(&mut self, id: &str) -> Pointer {
        let pointer = dispatch_block!(self, find, id);
        match pointer {
            Pointer::Some => Pointer::Block(self),
            _ => Pointer::None,
        }
    }

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

    fn apply_add(&mut self, address: &mut Address, value: &Value) -> Result<()> {
        dispatch_block!(self, apply_add, address, value)
    }

    fn apply_remove(&mut self, address: &mut Address, items: usize) -> Result<()> {
        dispatch_block!(self, apply_remove, address, items)
    }

    fn apply_replace(&mut self, address: &mut Address, items: usize, value: &Value) -> Result<()> {
        if address.is_empty() {
            *self = Self::from_value(value)?;
            Ok(())
        } else {
            dispatch_block!(self, apply_replace, address, items, value)
        }
    }

    fn apply_move(&mut self, from: &mut Address, items: usize, to: &mut Address) -> Result<()> {
        dispatch_block!(self, apply_move, from, items, to)
    }

    fn apply_transform(&mut self, address: &mut Address, from: &str, to: &str) -> Result<()> {
        if address.is_empty() {
            assert_eq!(from, self.as_ref(), "Expected the same type");
            *self = apply_transform(self, to);
            Ok(())
        } else {
            dispatch_block!(self, apply_transform, address, from, to)
        }
    }
}

fn diff_transform(differ: &mut Differ, _from: &BlockContent, to: &BlockContent) {
    // TODO implement generation of `Transform` operations
    // Default is to generate a replace operation
    differ.replace(to)
}

fn apply_transform(_from: &BlockContent, _to: &str) -> BlockContent {
    // TODO implement application of `Transform` operations
    todo!()
}

// Implementations for `BlockContent` structs
// TODO: add all relevant fields to each struct

patchable_struct!(ClaimSimple, content, claim_type);
patchable_struct!(CodeBlock, programming_language, text);
patchable_struct!(CodeChunk, programming_language, text, outputs);
patchable_struct!(CollectionSimple);
patchable_struct!(FigureSimple);
patchable_struct!(Heading, content, depth);
patchable_struct!(Include, source);
patchable_struct!(List, items, order);
patchable_struct!(ListItem, content);
patchable_struct!(MathBlock, math_language, text);
patchable_struct!(Paragraph, content);
patchable_struct!(QuoteBlock, content);
patchable_struct!(TableSimple, rows);
patchable_struct!(TableRow, cells, row_type);
patchable_struct!(TableCell, content, cell_type, colspan, rowspan);
patchable_struct!(ThematicBreak);

// Implementations for enum fields of `BlockContent` structs

patchable_enum!(ClaimClaimType);

patchable_enum!(ListOrder);
patchable_variants!(
    ListItemContent,
    ListItemContent::VecBlockContent,
    ListItemContent::VecInlineContent
);

patchable_enum!(TableRowRowType);
patchable_enum!(TableCellCellType);
patchable_variants!(
    TableCellContent,
    TableCellContent::VecBlockContent,
    TableCellContent::VecInlineContent
);
