use comfy_table::{
    modifiers::{UTF8_ROUND_CORNERS, UTF8_SOLID_INNER_BORDERS},
    presets::UTF8_FULL,
    Table,
};

// Re-exports
pub use comfy_table::{Attribute, Cell, Color};

/// Create a new table for displaying data in the terminal
///
/// Returns a [`comfy_table::Table`] with Stencila's default style
/// which `set_header` and `add_row` can be called on.
pub fn new() -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .apply_modifier(UTF8_SOLID_INNER_BORDERS);
    table
}
