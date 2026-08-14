//! Options for aligning and comparing nodes

use serde::{Deserialize, Serialize};

/// The default candidate-cell budget
///
/// Chosen so that ordinary documents align without any tuning, while a pathological
/// input fails with an explicit error rather than running for an unbounded time. Exact
/// anchors partition sequences into smaller gaps before the dynamic program runs, so
/// the budget is reached only by genuinely ambiguous collections: a thousand mutually
/// unrecognisable siblings in one property is already at the limit.
///
/// Together with the bound on how much text a single candidate comparison examines,
/// this bounds the work of an alignment and not merely its cell count.
pub const DEFAULT_ALIGNMENT_CELL_BUDGET: usize = 1_000_000;

/// Operational options for aligning and comparing nodes
///
/// Deliberately operational rather than semantic: there are no weights, thresholds, or
/// normalization knobs, because those belong to the built-in schema-native policy, and
/// changing them would change what an artifact means rather than what it costs to
/// produce.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CompareOptions {
    /// The maximum number of candidate cells that the ordered sequence alignment of
    /// repeated properties may use
    ///
    /// Exceeding the budget returns [`crate::CompareError::BudgetExhausted`] rather
    /// than silently selecting an approximate result.
    pub alignment_cell_budget: usize,
}

impl Default for CompareOptions {
    fn default() -> Self {
        Self {
            alignment_cell_budget: DEFAULT_ALIGNMENT_CELL_BUDGET,
        }
    }
}
