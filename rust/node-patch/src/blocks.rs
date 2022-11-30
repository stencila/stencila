//! Patching for [`BlockContent`] nodes

use common::serde_json;
use node_dispatch::{dispatch_block, dispatch_block_pair};
use stencila_schema::*;

use crate::value::Values;

use super::prelude::*;

/// Implements [`Patchable`] for [`BlockContent`] nodes
///
/// Generates and applies `Replace` and `Transform` operations between variants of block content.
/// All other operations are passed through to the individual variants of [`BlockContent`].
impl Patchable for BlockContent {
    fn diff(&self, other: &Self, differ: &mut Differ) {
        dispatch_block_pair!(
            self,
            other,
            diff_transform(differ, self, other),
            diff,
            differ
        )
    }

    fn apply_add(&mut self, address: &mut Address, value: Value) -> Result<()> {
        dispatch_block!(self, apply_add, address, value)
    }

    fn apply_add_many(&mut self, address: &mut Address, values: Values) -> Result<()> {
        dispatch_block!(self, apply_add_many, address, values)
    }

    fn apply_remove(&mut self, address: &mut Address) -> Result<()> {
        dispatch_block!(self, apply_remove, address)
    }

    fn apply_remove_many(&mut self, address: &mut Address, items: usize) -> Result<()> {
        dispatch_block!(self, apply_remove_many, address, items)
    }

    fn apply_replace(&mut self, address: &mut Address, value: Value) -> Result<()> {
        if address.is_empty() {
            *self = Self::from_value(value)?;
            Ok(())
        } else {
            dispatch_block!(self, apply_replace, address, value)
        }
    }

    fn apply_replace_many(
        &mut self,
        address: &mut Address,
        items: usize,
        values: Values,
    ) -> Result<()> {
        dispatch_block!(self, apply_replace_many, address, items, values)
    }

    fn apply_move(&mut self, from: &mut Address, to: &mut Address) -> Result<()> {
        dispatch_block!(self, apply_move, from, to)
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

    fn to_value(&self) -> Value {
        Value::Block(self.clone())
    }

    fn from_value(value: Value) -> Result<Self> {
        match value {
            Value::Block(node) => Ok(node),
            Value::Json(json) => Ok(serde_json::from_value::<Self>(json)?),
            _ => bail!(invalid_patch_value::<Self>(value)),
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

// Implementations for `BlockContent` structs, including related structs
// (e.g. `Figure` vs `FigureSimple`, which are actually "works").

patchable_struct!(Heading, content, depth);
patchable_struct!(Paragraph, content);
patchable_struct!(QuoteBlock, content);
patchable_struct!(CodeBlock, programming_language, code);

patchable_struct!(List, items, order);
patchable_enum!(ListOrder);
patchable_struct!(ListItem, content);
patchable_variants!(
    ListItemContent,
    ListItemContent::VecBlockContent,
    ListItemContent::VecInlineContent
);

patchable_struct!(Table, label, caption, rows);
patchable_struct!(TableSimple, label, caption, rows);
patchable_variants!(
    TableCaption,
    TableCaption::VecBlockContent,
    TableCaption::String
);
patchable_struct!(TableRow, cells, row_type);
patchable_enum!(TableRowRowType);
patchable_struct!(TableCell, content, cell_type, colspan, rowspan);
patchable_enum!(TableCellCellType);
patchable_variants!(
    TableCellContent,
    TableCellContent::VecBlockContent,
    TableCellContent::VecInlineContent
);

patchable_struct!(Figure, label, caption, content);
patchable_struct!(FigureSimple, label, caption, content);
patchable_variants!(
    FigureCaption,
    FigureCaption::VecBlockContent,
    FigureCaption::String
);

patchable_struct!(ThematicBreak);

patchable_struct!(Claim, content, claim_type);
patchable_struct!(ClaimSimple, content, claim_type);
patchable_enum!(ClaimClaimType);
