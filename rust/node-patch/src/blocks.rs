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

patchable_struct!(
    MathBlock,
    id,
    math_language,
    code,
    compile_digest,
    errors,
    mathml
);

patchable_struct!(QuoteBlock, content);

patchable_struct!(CodeBlock, programming_language, code);
patchable_struct!(
    CodeChunk,
    id,
    programming_language,
    guess_language,
    code,
    outputs,
    errors,
    label,
    caption,
    execution_dependencies,
    execution_dependents,
    compile_digest,
    execution_auto,
    execution_pure,
    execute_digest,
    execution_required,
    execution_kernel,
    execution_status,
    execution_ended,
    execution_duration,
    execution_count
);
patchable_variants!(
    CodeChunkCaption,
    CodeChunkCaption::VecBlockContent,
    CodeChunkCaption::String
);

patchable_struct!(
    ExecutionDigest,
    state_digest,
    semantic_digest,
    dependencies_digest,
    dependencies_failed,
    dependencies_stale
);

// Make `CodeError`s replaceable to avoid large patches associated with
// changes in stack trace and to simplify Web Component development (no
// need to observe for change in error level or presence/absence of stacktrace)
replaceable_struct!(CodeError, error_message, error_type, stack_trace);

patchable_struct!(ExecutionDependency, dependency_relation, dependency_node);
patchable_enum!(ExecutionDependencyRelation);
patchable_variants!(
    ExecutionDependencyNode,
    ExecutionDependencyNode::Button,
    ExecutionDependencyNode::CodeChunk,
    ExecutionDependencyNode::File,
    ExecutionDependencyNode::Parameter,
    ExecutionDependencyNode::SoftwareSourceCode,
    ExecutionDependencyNode::Variable
);

patchable_struct!(ExecutionDependent, dependent_relation, dependent_node);
patchable_enum!(ExecutionDependentRelation);
patchable_variants!(
    ExecutionDependentNode,
    ExecutionDependentNode::Button,
    ExecutionDependentNode::Call,
    ExecutionDependentNode::CodeChunk,
    ExecutionDependentNode::CodeExpression,
    ExecutionDependentNode::Division,
    ExecutionDependentNode::File,
    ExecutionDependentNode::Parameter,
    ExecutionDependentNode::Span,
    ExecutionDependentNode::Variable
);

patchable_enum!(ExecutionAuto);
patchable_enum!(ExecutionRequired);
patchable_enum!(ExecutionStatus);

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
    Division,
    id,
    programming_language,
    guess_language,
    code,
    css,
    classes,
    errors,
    content,
    execution_dependencies,
    execution_dependents,
    compile_digest,
    execution_auto,
    execute_digest,
    execution_required,
    execution_kernel,
    execution_status,
    execution_ended,
    execution_duration,
    execution_count
);

patchable_struct!(
    Include,
    id,
    source,
    media_type,
    select,
    execution_dependents,
    compile_digest,
    execute_digest,
    execution_required,
    execution_kernel,
    execution_status,
    execution_ended,
    execution_duration,
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
    execution_dependencies,
    compile_digest,
    execute_digest,
    execution_required,
    execution_kernel,
    execution_status,
    execution_ended,
    execution_duration,
    errors,
    content,
);
patchable_struct!(
    CallArgument,
    id,
    name,
    validator,
    value,
    code,
    programming_language,
    guess_language
);

patchable_struct!(
    For,
    id,
    symbol,
    code,
    programming_language,
    guess_language,
    content,
    otherwise,
    iterations,
    execution_dependents,
    compile_digest,
    execute_digest,
    execution_required,
    execution_kernel,
    execution_status,
    execution_ended,
    execution_duration,
    errors,
);

patchable_struct!(
    Form,
    id,
    derive_from,
    derive_action,
    derive_item,
    content,
    execution_dependents,
    compile_digest,
    execute_digest,
    execution_required,
    execution_kernel,
    execution_status,
    execution_ended,
    execution_duration,
    errors,
);

patchable_enum!(FormDeriveAction);
patchable_variants!(
    FormDeriveItem,
    FormDeriveItem::Integer,
    FormDeriveItem::String
);

patchable_struct!(
    If,
    id,
    clauses,
    execution_dependents,
    compile_digest,
    execute_digest,
    execution_required,
    execution_kernel,
    execution_status,
    execution_ended,
    execution_duration,
);
patchable_struct!(
    IfClause,
    code,
    programming_language,
    guess_language,
    is_active,
    content,
    errors,
);

patchable_struct!(ThematicBreak);

patchable_struct!(Claim, content, claim_type);
patchable_struct!(ClaimSimple, content, claim_type);
patchable_enum!(ClaimClaimType);
