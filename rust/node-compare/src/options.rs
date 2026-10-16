//! Options for aligning and comparing nodes

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::{filter::DifferenceFilter, normalization::Normalization};

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
/// Alignment weights, thresholds, and the normalization used while matching belong
/// to the built-in schema-native policy, rather than caller options. The rules in
/// [`CompareOptions::normalize`] are separate: they only classify differences after
/// matching.
///
/// [`CompareOptions::filter`] and [`CompareOptions::normalize`] only affect reporting.
/// Neither is consulted while matching, so the alignment is bit-for-bit unchanged.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CompareOptions {
    /// The maximum number of candidate cells that local sequence alignment and
    /// cross-parent text reconciliation may use together
    ///
    /// Exceeding the budget returns [`crate::CompareError::BudgetExhausted`] rather
    /// than silently selecting an approximate result.
    pub alignment_cell_budget: usize,

    /// Which differences the comparison reports
    ///
    /// Empty by default, which reports every difference. See [`DifferenceFilter`] for
    /// the selector grammar and the precedence rule.
    pub filter: DifferenceFilter,

    /// Normalizations used to classify value differences as cosmetic
    ///
    /// Empty by default. Classification preserves raw values, every difference, and
    /// the equality verdict. The selected rules are recorded in the comparison.
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    pub normalize: BTreeSet<Normalization>,
}

impl Default for CompareOptions {
    fn default() -> Self {
        Self {
            alignment_cell_budget: DEFAULT_ALIGNMENT_CELL_BUDGET,
            filter: DifferenceFilter::none(),
            normalize: BTreeSet::new(),
        }
    }
}
