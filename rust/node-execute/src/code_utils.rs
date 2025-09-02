//! Utilities for executing code

use stencila_schema::{Block, CodeChunk, ExecutionBounds, VisitorMut, WalkControl, WalkNode};

/// Apply execution bounds to all executable nodes within a root node
pub(super) fn apply_execution_bounds<W: WalkNode>(node: &mut W, execution_bounds: ExecutionBounds) {
    ApplyBounds { execution_bounds }.walk(node);
}

struct ApplyBounds {
    execution_bounds: ExecutionBounds,
}

impl VisitorMut for ApplyBounds {
    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        if let Block::CodeChunk(CodeChunk {
            execution_bounds, ..
        }) = block
        {
            *execution_bounds = Some(self.execution_bounds);
        }

        WalkControl::Continue
    }
}
