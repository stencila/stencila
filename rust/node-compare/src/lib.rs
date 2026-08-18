//! Schema-native comparison of Stencila Schema nodes
//!
//! This crate compares any two [`stencila_schema::Node`] trees and produces two
//! versioned artifacts:
//!
//! 1. an alignment: a complete, symmetric correspondence between the structured
//!    occurrences of the two trees;
//! 2. a comparison: that alignment plus sparse, atomic observations about differences
//!    between paired occurrences.
//!
//! It is deliberately independent of source formats, document regions, benchmark
//! metrics, and presentation. Neither side is presumed correct: they are the *left*
//! and *right* snapshots the caller selected. Evaluative terminology such as expected,
//! actual, false positive, and false negative belongs in an adapter over these
//! artifacts, not in this crate.

mod align;
mod alignment;
mod anchors;
mod comparison;
mod differences;
mod error;
mod features;
mod filter;
mod fingerprint;
mod increasing;
mod options;
mod policy;
mod reorder;
mod scalar;
mod sequence;
mod text;

mod projection;

pub use alignment::{
    AlgorithmInfo, Alignment, AlignmentCost, AlignmentFormatVersion, AlignmentSignal,
    Correspondence, EvidenceValue, MatchEvidence, MatchInfo, MatchRule, NodeRef, PairCost,
    UnmatchedReason,
};
pub use comparison::{
    Comparison, ComparisonFormatVersion, Difference, OneSidedTally, PropertyPresence,
    ValueLocation, ValueState,
};
pub use error::{CompareError, CompareResult, Side};
pub use filter::{DifferenceFilter, Selector, SelectorError};
pub use options::{CompareOptions, DEFAULT_ALIGNMENT_CELL_BUDGET};
pub use projection::MAX_DEPTH as MAX_COMPARISON_DEPTH;
pub use scalar::{CanonicalNumber, DuplicateObjectKeyError, ObjectEntries, ScalarValue};

use stencila_schema::Node;

use crate::{align::Aligner, features::FeatureSet, projection::Projection};

/// Canonical projections and alignment-independent features for both inputs
struct PreparedInputs {
    left: Projection,
    left_features: FeatureSet,
    right: Projection,
    right_features: FeatureSet,
}

impl PreparedInputs {
    fn new(left: &Node, right: &Node) -> CompareResult<Self> {
        let left = Projection::new(left, Side::Left)?;
        let left_features = FeatureSet::new(&left)?;
        let right = Projection::new(right, Side::Right)?;
        let right_features = FeatureSet::new(&right)?;

        Ok(Self {
            left,
            left_features,
            right,
            right_features,
        })
    }

    fn aligner<'prepared>(
        &'prepared self,
        options: &'prepared CompareOptions,
    ) -> Aligner<'prepared> {
        Aligner::new(
            &self.left,
            &self.left_features,
            &self.right,
            &self.right_features,
            options,
        )
    }
}

/// Align two nodes
///
/// Returns a complete, symmetric correspondence between the structured occurrences of
/// the two nodes. Neither node is presumed correct.
///
/// Does not accept a caller-supplied alignment, because without binding an alignment
/// to hashes of its inputs, stale paths could silently be applied to the wrong
/// snapshots.
pub fn align(left: &Node, right: &Node) -> CompareResult<Alignment> {
    align_with_options(left, right, &CompareOptions::default())
}

/// Align two nodes, with options
pub fn align_with_options(
    left: &Node,
    right: &Node,
    options: &CompareOptions,
) -> CompareResult<Alignment> {
    Ok(PreparedInputs::new(left, right)?
        .aligner(options)
        .align()?
        .alignment)
}

/// Compare two nodes
///
/// Returns the alignment of the two nodes, plus sparse, atomic observations about how
/// their paired occurrences differ. Neither node is presumed correct.
pub fn compare(left: &Node, right: &Node) -> CompareResult<Comparison> {
    compare_with_options(left, right, &CompareOptions::default())
}

/// Compare two nodes, with options
pub fn compare_with_options(
    left: &Node,
    right: &Node,
    options: &CompareOptions,
) -> CompareResult<Comparison> {
    let prepared = PreparedInputs::new(left, right)?;
    let aligned = prepared.aligner(options).align()?;

    // Differences are derived only after the final alignment is complete
    let mut differences = differences::derive(&prepared.left, &prepared.right, &aligned)?;
    differences.extend(reorder::derive(
        &prepared.left,
        &prepared.left_features,
        &prepared.right,
        &prepared.right_features,
        &aligned,
    )?);

    // Filtering happens after every difference has been derived, so that what a filter
    // selects can never depend on the order differences were produced in
    Comparison::new_filtered(aligned.alignment, differences, options.filter.clone())
}

/// Whether the canonical projections of two nodes are exactly equal
///
/// Equality is defined by the canonical projections, not by Rust's `PartialEq`: it
/// covers every declared schema property except the intrinsic implementation
/// machinery described in [`projection`].
#[cfg(test)]
fn projections_equal(left: &Node, right: &Node) -> CompareResult<bool> {
    let left = Projection::new(left, Side::Left)?;
    let right = Projection::new(right, Side::Right)?;

    left.eq_canonically(&right)
}
