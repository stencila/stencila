//! Options for aligning and comparing nodes

use serde::{Deserialize, Serialize};

use crate::filter::DifferenceFilter;

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

/// Options for aligning and comparing nodes
///
/// Deliberately free of weights, thresholds, and normalization knobs: those belong to
/// the built-in schema-native policy, and changing them would change what an alignment
/// means rather than what it costs to produce.
///
/// [`CompareOptions::filter`] is the one option that is about meaning rather than cost,
/// and it is confined to reporting. A filter selects which differences a comparison
/// reports; it is never consulted while matching, so no filter can change which
/// occurrences pair with which. The alignment of a filtered comparison is bit-for-bit
/// the alignment of an unfiltered one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CompareOptions {
    /// The maximum number of candidate cells that the ordered sequence alignment of
    /// repeated properties may use
    ///
    /// Exceeding the budget returns [`crate::CompareError::BudgetExhausted`] rather
    /// than silently selecting an approximate result.
    pub alignment_cell_budget: usize,

    /// Which differences the comparison reports
    ///
    /// Empty by default, which reports every difference. See [`DifferenceFilter`] for
    /// the selector grammar and the precedence rule.
    pub filter: DifferenceFilter,
}

impl Default for CompareOptions {
    fn default() -> Self {
        Self {
            alignment_cell_budget: DEFAULT_ALIGNMENT_CELL_BUDGET,
            filter: DifferenceFilter::none(),
        }
    }
}
