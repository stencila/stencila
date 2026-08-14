//! Order-preserving alignment of two sequences, with explicit gaps
//!
//! Every repeated property is ordered under the schema-native policy. Items are
//! aligned by weighted dynamic programming over explicit gaps: the policy supplies a
//! pair cost for compatible candidates, a left-gap cost, a right-gap cost, and an
//! outright refusal for incompatible ones. The program minimizes total cost.
//!
//! Because the policy never offers a pair whose cost is below the sum of the two gap
//! costs it displaces unless the pair is genuinely better, an implausible candidate is
//! left as two gaps. Refusing to force implausible matches is therefore an invariant
//! of the algorithm, not a cleanup pass afterwards.
//!
//! # Tie-breaking
//!
//! Ties are resolved in a documented, swap-symmetric order:
//!
//! 1. **Strong identity evidence wins**, because a unique explicit `id` or a verified
//!    unique exact subtree is applied as a compulsory anchor before the dynamic
//!    program runs, so those pairs are never up for tie-breaking at all.
//! 2. **A pair must strictly beat leaving both items unmatched**: when pairing costs
//!    exactly as much as two gaps, the gaps win.
//! 3. **Same-type pairs win over cross-type pairs**, because the policy charges a
//!    cross-type pair more, so the two are never actually tied.
//! 4. **A stable, content-derived key breaks what remains**: when a left gap and a
//!    right gap cost the same, the item with the smaller `(node type, fingerprint,
//!    position)` key is consumed first. That key belongs to the item rather than to
//!    the side it is on, so swapping the inputs selects the same item and yields the
//!    same alignment.

use stencila_node_type::NodeType;

use crate::{
    alignment::{AlignmentCost, PairCost},
    error::CompareResult,
};

/// A stable, content-derived key used as the final tie-break
///
/// Deliberately a property of the item rather than of the side it is on, so that it is
/// unchanged by swapping the inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TieKey {
    /// The concrete node type, or `None` for a scalar item
    pub node_type: Option<NodeType>,

    /// The canonical structural fingerprint of the item
    pub fingerprint: u64,

    /// The position of the item within its sequence
    pub position: usize,
}

/// The costs of aligning two sequences
///
/// Supplied as callbacks so that the dynamic program stays free of policy. Indices are
/// positions within the two sequences being aligned.
pub struct Costs<'policy> {
    /// The cost of pairing two items, or a refusal to pair them
    #[allow(clippy::type_complexity)]
    pub pair: &'policy dyn Fn(usize, usize) -> CompareResult<PairCost>,

    /// The cost of leaving a left item unmatched
    pub left_gap: &'policy dyn Fn(usize) -> CompareResult<AlignmentCost>,

    /// The cost of leaving a right item unmatched
    pub right_gap: &'policy dyn Fn(usize) -> CompareResult<AlignmentCost>,

    /// The tie-break key of a left item
    pub left_key: &'policy dyn Fn(usize) -> CompareResult<TieKey>,

    /// The tie-break key of a right item
    pub right_key: &'policy dyn Fn(usize) -> CompareResult<TieKey>,
}

/// What an aligned sequence position resolved to
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// The two items at these positions are paired
    Pair { left: usize, right: usize },

    /// The left item at this position has no counterpart
    LeftGap { left: usize },

    /// The right item at this position has no counterpart
    RightGap { right: usize },
}

/// The choice recorded in one cell of the dynamic program
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Choice {
    Start,
    Pair,
    LeftGap,
    RightGap,
}

/// Align two ranges of items, order-preserving, with explicit gaps
///
/// `left` and `right` are the half-open ranges of the two sequences to align. Returns
/// the steps in sequence order.
pub fn align(
    left: std::ops::Range<usize>,
    right: std::ops::Range<usize>,
    costs: &Costs<'_>,
) -> CompareResult<Vec<Step>> {
    let (left_start, right_start) = (left.start, right.start);
    let (rows, columns) = (left.len(), right.len());

    if rows == 0 && columns == 0 {
        return Ok(Vec::new());
    }

    let width = columns + 1;
    let cell = |row: usize, column: usize| row * width + column;

    let mut total = vec![AlignmentCost::ZERO; (rows + 1) * width];
    let mut choice = vec![Choice::Start; (rows + 1) * width];

    for row in 1..=rows {
        total[cell(row, 0)] =
            total[cell(row - 1, 0)].saturating_add((costs.left_gap)(left_start + row - 1)?);
        choice[cell(row, 0)] = Choice::LeftGap;
    }
    for column in 1..=columns {
        total[cell(0, column)] =
            total[cell(0, column - 1)].saturating_add((costs.right_gap)(right_start + column - 1)?);
        choice[cell(0, column)] = Choice::RightGap;
    }

    for row in 1..=rows {
        for column in 1..=columns {
            let left_index = left_start + row - 1;
            let right_index = right_start + column - 1;

            let left_gap =
                total[cell(row - 1, column)].saturating_add((costs.left_gap)(left_index)?);
            let right_gap =
                total[cell(row, column - 1)].saturating_add((costs.right_gap)(right_index)?);

            // Between two gaps of equal cost, consume the item with the smaller
            // content-derived key, which is the same item whichever way round the
            // inputs are
            let (mut best_cost, mut best_choice) = match left_gap.cmp(&right_gap) {
                std::cmp::Ordering::Less => (left_gap, Choice::LeftGap),
                std::cmp::Ordering::Greater => (right_gap, Choice::RightGap),
                std::cmp::Ordering::Equal => {
                    if (costs.left_key)(left_index)? <= (costs.right_key)(right_index)? {
                        (left_gap, Choice::LeftGap)
                    } else {
                        (right_gap, Choice::RightGap)
                    }
                }
            };

            // A pair is taken only when it is strictly better than every alternative,
            // so pairing that merely ties with two gaps is refused
            if let PairCost::Cost(pair_cost) = (costs.pair)(left_index, right_index)? {
                let paired = total[cell(row - 1, column - 1)].saturating_add(pair_cost);
                if paired < best_cost {
                    best_cost = paired;
                    best_choice = Choice::Pair;
                }
            }

            total[cell(row, column)] = best_cost;
            choice[cell(row, column)] = best_choice;
        }
    }

    let mut steps = Vec::new();
    let (mut row, mut column) = (rows, columns);
    while row > 0 || column > 0 {
        match choice[cell(row, column)] {
            Choice::Pair => {
                steps.push(Step::Pair {
                    left: left_start + row - 1,
                    right: right_start + column - 1,
                });
                row -= 1;
                column -= 1;
            }
            Choice::LeftGap => {
                steps.push(Step::LeftGap {
                    left: left_start + row - 1,
                });
                row -= 1;
            }
            Choice::RightGap => {
                steps.push(Step::RightGap {
                    right: right_start + column - 1,
                });
                column -= 1;
            }
            Choice::Start => break,
        }
    }
    steps.reverse();

    Ok(steps)
}

/// The number of candidate cells that aligning two ranges requires
pub fn cells(left: usize, right: usize) -> usize {
    left.saturating_mul(right)
}
