use async_lsp::lsp_types::{Position, Range};
use codecs::{Mapping, PoshMap, Position16, Range16};
use schema::{
    Block, CallBlock, CodeChunk, CodeExpression, DeleteBlock, ForBlock, IfBlock, IncludeBlock,
    Inline, InsertBlock, InstructionBlock, InstructionInline, NodeId, NodeType, Parameter,
    ReplaceBlock, Visitor, WalkControl,
};

/// A struct that implements the [`Visitor`] trait to collect
/// diagnostics, code lenses etc from the nodes in a document
pub(super) struct Inspector<'source, 'generated>
where
    'source: 'generated,
    'generated: 'source,
{
    /// The [`PoshMap`] used to correlate nodes with positions in the document
    poshmap: PoshMap<'source, 'generated>,

    /// The range, type and id of collected nodes
    pub nodes: Vec<(Range, NodeType, NodeId)>,
}

impl<'source, 'generated> Inspector<'source, 'generated> {
    pub fn new(source: &'source str, generated: &'generated str, mapping: Mapping) -> Self {
        Self {
            poshmap: PoshMap::new(source, generated, mapping),
            nodes: Vec::new(),
        }
    }

    // Push a node onto the `nodes` collection
    fn push_node(&mut self, node_type: NodeType, node_id: NodeId) {
        if let Some(range) = self.poshmap.node_id_to_range16(&node_id) {
            self.nodes
                .push((range16_to_range(range), node_type, node_id))
        }
    }
}

/// Convert a Stencila [`Range16`] to a LSP [`Range`]
fn range16_to_range(range: Range16) -> Range {
    Range {
        start: position16_to_position(range.start),
        end: position16_to_position(range.end),
    }
}

/// Convert a Stencila [`Position16`] to a LSP [`Position`]
fn position16_to_position(position: Position16) -> Position {
    Position {
        line: position.line as u32,
        character: position.column as u32,
    }
}

impl<'source, 'generated> Visitor for Inspector<'source, 'generated> {
    fn visit_block(&mut self, block: &Block) -> WalkControl {
        macro_rules! variants {
            ($( $variant:ident ),*) => {
                match block {
                    $(Block::$variant(node) => node.inspect(self),)*
                    _ => {}
                }
            };
        }
        variants!(
            CallBlock,
            CodeChunk,
            DeleteBlock,
            ForBlock,
            IfBlock,
            IncludeBlock,
            InsertBlock,
            InstructionBlock,
            ReplaceBlock
        );

        WalkControl::Continue
    }

    fn visit_inline(&mut self, inline: &Inline) -> WalkControl {
        macro_rules! variants {
            ($( $variant:ident ),*) => {
                match inline {
                    $(Inline::$variant(node) => node.inspect(self),)*
                    _ => {}
                }
            };
        }
        variants!(CodeExpression, InstructionInline, Parameter);

        WalkControl::Continue
    }
}

trait Inspect {
    fn inspect(&self, inspector: &mut Inspector);
}

macro_rules! executable {
    ($( $type:ident ),*) => {
        $(impl Inspect for $type {
            fn inspect(&self, inspector: &mut Inspector) {
                inspector.push_node(self.node_type(), self.node_id())
            }
        })*
    };
}

executable!(
    CallBlock,
    CodeChunk,
    CodeExpression,
    ForBlock,
    IfBlock,
    IncludeBlock,
    InstructionBlock,
    InstructionInline,
    Parameter
);

macro_rules! not_executable {
    ($( $type:ident ),*) => {
        $(impl Inspect for $type {
            fn inspect(&self, inspector: &mut Inspector) {
                inspector.push_node(self.node_type(), self.node_id())
            }
        })*
    };
}

not_executable!(InsertBlock, ReplaceBlock, DeleteBlock);
