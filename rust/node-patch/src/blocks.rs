use super::prelude::*;
use node_dispatch::{dispatch_block, dispatch_block_pair};
use stencila_schema::*;

/// Implements patching for `BlockContent`
///
/// Generates and applies `Replace` and `Transform` operations between variants of block content.
/// All other operations are passed through to variants.
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

// Implementations for `BlockContent` structs, including related structs
// (e.g. `Figure` vs `FigureSimple`, which are actually "works").

patchable_struct!(Heading, content, depth);

patchable_struct!(Paragraph, content);

patchable_struct!(MathBlock, math_language, text);

patchable_struct!(QuoteBlock, content);

patchable_struct!(CodeBlock, programming_language, text);
patchable_struct!(
    CodeChunk,
    id,
    programming_language,
    text,
    outputs,
    errors,
    label,
    caption,
    code_dependencies,
    code_dependents,
    compile_digest,
    execute_auto,
    execute_pure,
    execute_digest,
    execute_required,
    execute_kernel,
    execute_status,
    execute_ended,
    execute_duration
);
patchable_variants!(
    CodeChunkCaption,
    CodeChunkCaption::VecBlockContent,
    CodeChunkCaption::String
);

// Make `CodeError`s replaceable to avoid large patches associated with
// changes in stack trace and to simplify Web Component development (no
// need to observe for change in error level or presence/absence of stacktrace)
replaceable_struct!(CodeError, error_message, error_type, stack_trace);

patchable_variants!(
    ExecutableCodeDependencies,
    ExecutableCodeDependencies::CodeChunk,
    ExecutableCodeDependencies::Parameter,
    ExecutableCodeDependencies::File
);
patchable_variants!(
    ExecutableCodeDependents,
    ExecutableCodeDependents::Call,
    ExecutableCodeDependents::CodeChunk,
    ExecutableCodeDependents::CodeExpression
);
patchable_enum!(ExecuteAuto);
patchable_enum!(ExecuteRequired);
patchable_enum!(ExecuteStatus);

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

patchable_struct!(
    Include,
    id,
    source,
    media_type,
    select,
    code_dependents,
    compile_digest,
    execute_digest,
    execute_required,
    execute_kernel,
    execute_status,
    execute_ended,
    execute_duration,
    errors,
    content,
);

patchable_struct!(
    Call,
    id,
    source,
    media_type,
    select,
    arguments,
    code_dependencies,
    compile_digest,
    execute_digest,
    execute_required,
    execute_kernel,
    execute_status,
    execute_ended,
    execute_duration,
    errors,
    content,
);
patchable_struct!(CallArgument, id, name, validator, value, symbol);

patchable_struct!(ThematicBreak);

patchable_struct!(Claim, content, claim_type);
patchable_struct!(ClaimSimple, content, claim_type);
patchable_enum!(ClaimClaimType);

patchable_struct!(CollectionSimple);
