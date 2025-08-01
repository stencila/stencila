use std::{
    env::{var, var_os},
    fmt::Display,
};

use comfy_table::{
    ContentArrangement, Table as ComfyTable, modifiers::UTF8_ROUND_CORNERS,
    presets::UTF8_BORDERS_ONLY,
};

// Re-exports
pub use comfy_table::{Attribute, Cell, CellAlignment, Color};
use derive_more::{Deref, DerefMut};

use crate::ToStdout;

/// Tabulated data to display in the terminal
///
/// Wraps a [`comfy_table::Table`] with Stencila's default style.
/// Named [`Tabulated`] to avoid confusion with Stencila's `Table` node type.
#[derive(Deref, DerefMut)]
pub struct Tabulated {
    inner: ComfyTable,
}

impl Default for Tabulated {
    fn default() -> Self {
        Self::new()
    }
}

impl Tabulated {
    pub fn new() -> Self {
        let mut inner = ComfyTable::new();
        inner
            .load_preset(UTF8_BORDERS_ONLY)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic);

        if let Some(cols) = var("COLUMNS").ok().and_then(|s| s.parse().ok()) {
            inner.set_width(cols);
        }

        if var_os("NO_COLOR").is_some() {
            inner.force_no_tty(); // disable styling
        } else if var_os("FORCE_COLOR").is_some() {
            inner.enforce_styling(); // force colours even when piped
        }

        Self { inner }
    }
}

impl Display for Tabulated {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.inner)
    }
}

impl ToStdout for Tabulated {
    fn to_terminal(&self) -> impl std::fmt::Display {
        &self.inner
    }
}
