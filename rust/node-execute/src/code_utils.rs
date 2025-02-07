//! Utilities for executing code

use schema::{Block, CodeChunk, ExecutionBounds, VisitorMut, WalkControl, WalkNode};

/// Apply execution bounds to all executable nodes within a root node
pub(super) fn apply_execution_bounds<W: WalkNode>(node: &mut W, execution_bounds: ExecutionBounds) {
    ApplyBounds { execution_bounds }.visit(node);
}

struct ApplyBounds {
    execution_bounds: ExecutionBounds,
}

impl VisitorMut for ApplyBounds {
    fn visit_block(&mut self, block: &mut Block) -> WalkControl {
        match block {
            Block::CodeChunk(CodeChunk {
                execution_bounds, ..
            }) => {
                *execution_bounds = Some(self.execution_bounds.clone());
            }
            _ => {}
        }

        WalkControl::Continue
    }
}
