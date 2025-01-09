use comfy_table::{modifiers::UTF8_ROUND_CORNERS, presets::UTF8_BORDERS_ONLY, ContentArrangement};

// Re-exports
pub use comfy_table::{Attribute, Cell, CellAlignment, Color, Table};

use crate::ToStdout;

/// Create a new table for displaying data in the terminal
///
/// Returns a [`comfy_table::Table`] with Stencila's default style
/// which `set_header` and `add_row` can be called on.
pub fn new() -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_BORDERS_ONLY)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);
    table
}

impl ToStdout for Table {
    fn to_terminal(&self) -> impl std::fmt::Display {
        self
    }
}
