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
mod error;
mod features;
mod fingerprint;
mod options;
mod policy;
mod scalar;
mod sequence;
mod text;

pub mod projection;

pub use alignment::{
    AlgorithmInfo, Alignment, AlignmentCost, AlignmentFormatVersion, AlignmentSignal,
    Correspondence, EvidenceValue, MatchEvidence, MatchInfo, MatchRule, NodeRef, PairCost,
    UnmatchedReason,
};
pub use error::{CompareError, CompareResult, Side};
pub use options::{CompareOptions, DEFAULT_ALIGNMENT_CELL_BUDGET};
pub use scalar::{CanonicalNumber, ScalarValue};

use stencila_schema::Node;

use crate::{align::Aligner, features::FeatureSet, projection::Projection};

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
    let left = Projection::new(left, Side::Left)?;
    let left_features = FeatureSet::new(&left)?;
    let right = Projection::new(right, Side::Right)?;
    let right_features = FeatureSet::new(&right)?;

    Ok(
        Aligner::new(&left, &left_features, &right, &right_features, options)
            .align()?
            .alignment,
    )
}
