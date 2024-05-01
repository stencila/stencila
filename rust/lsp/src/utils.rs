use async_lsp::lsp_types::{Position, Range};

use codecs::{Position16, Range16};

/// Convert a Stencila [`Range16`] to a LSP [`Range`]
pub(super) fn range16_to_range(range: Range16) -> Range {
    Range {
        start: position16_to_position(range.start),
        end: position16_to_position(range.end),
    }
}

/// Convert a Stencila [`Position16`] to a LSP [`Position`]
pub(super) fn position16_to_position(position: Position16) -> Position {
    Position {
        line: position.line as u32,
        character: position.column as u32,
    }
}

/// Convert a LSP [`Position`] to a Stencila [`Position16`]
pub(super) fn position_to_position16(position: Position) -> Position16 {
    Position16 {
        line: position.line as usize,
        column: position.character as usize,
    }
}
