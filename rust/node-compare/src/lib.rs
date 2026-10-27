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
//!
//! Paragraph merges and splits are recognized conservatively within a paired sibling
//! scope: two to eight consecutive paragraphs must have the same complete inline
//! content as one paragraph, joined with a single space. Formatting, inline IDs and
//! reference identity are preserved. Paragraphs with IDs or other metadata, ambiguous
//! groups, and changes to the content retain ordinary alignment. Grouped paragraphs
//! and their descendants are covered collectively; artifacts containing groups use
//! format version 2, while ordinary artifacts retain format version 1.
//!
//! In author lists, consecutive roles wrapping a person or organization that is
//! identical apart from IDs compare as one author. Role names compare as one
//! set-valued `roleName` property on the representative person or organization,
//! whose path remains its original source path. A wrapper's ID is ignored, as IDs are
//! throughout comparison, while wrappers with formats or timestamps retain ordinary
//! comparison.

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
mod moves;
mod normalization;
mod options;
mod paragraphs;
mod person;
mod policy;
mod references;
mod reorder;
mod scalar;
mod sequence;
mod text;

mod projection;

pub use alignment::{
    AlgorithmInfo, Alignment, AlignmentCost, AlignmentFormatVersion, AlignmentSignal,
    Correspondence, EvidenceValue, MatchEvidence, MatchInfo, MatchRule, NodeRef, PairCost,
    ParagraphGroup, UnmatchedReason,
};
pub use comparison::{
    Comparison, ComparisonFormatVersion, Difference, OneSidedTally, PropertyPresence,
    ValueLocation, ValueState,
};
pub use error::{BudgetScope, CompareError, CompareResult, Side};
pub use filter::{DifferenceFilter, Selector, SelectorError};
pub use normalization::Normalization;
pub use options::{CompareOptions, DEFAULT_ALIGNMENT_CELL_BUDGET};
pub use projection::MAX_DEPTH as MAX_COMPARISON_DEPTH;
pub use scalar::{CanonicalNumber, DuplicateObjectKeyError, ObjectEntries, ScalarValue};

use std::collections::HashMap;

use stencila_node_path::NodePath;
use stencila_schema::{Node, NodeId};

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
/// Local `id` changes are ignored; citation and internal link targets compare through
/// the alignment when they resolve uniquely. Targets that are unresolved on both
/// sides retain literal comparison.
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

    // Classify only completed differences, preserving their raw values and alignment
    normalization::classify(
        &mut differences,
        &prepared.left,
        &prepared.right,
        &options.normalize,
    )?;

    // Filtering happens after every difference has been derived, so that what a filter
    // selects can never depend on the order differences were produced in
    Comparison::new_filtered(
        aligned.alignment,
        differences,
        options.filter.clone(),
        options.normalize.clone(),
    )
}

/// The path of each structured node within a node, by its runtime identifier
///
/// Paths resolve in the original node, including author role wrappers and repeated
/// authors represented collectively in comparison. Comparison representatives keep
/// these source paths, so information such as page coordinates can be related to them.
/// When clones share a runtime identifier, its first source occurrence is used.
pub fn node_paths(node: &Node) -> CompareResult<HashMap<NodeId, NodePath>> {
    let projection = Projection::source(node, Side::Left)?;
    let mut paths = HashMap::new();
    for occurrence in projection.occurrences() {
        if let Some(id) = &occurrence.node_id {
            paths
                .entry(id.clone())
                .or_insert_with(|| occurrence.path.clone());
        }
    }
    Ok(paths)
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
