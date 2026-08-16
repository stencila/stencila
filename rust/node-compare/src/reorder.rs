//! Reorder observations within aligned sibling scopes
//!
//! Changed relative order is reported; absolute index shifts are not. Inserting one
//! early sibling must not mark every later sibling as moved, so movement is never
//! inferred from raw index inequality.
//!
//! Within each aligned sibling scope — one property of one pair of aligned parents —
//! the paired positions are mapped and a canonical maximum order-preserving subset is
//! derived. A reorder is emitted for exactly those pairs that lie outside that subset.
//! Every pairwise inversion is deliberately *not* emitted, because that can be
//! quadratic in the number of pairs; this is linear.
//!
//! A reorder observation is a statement about two snapshots, not a claim about
//! historical editing causation: it means the pair lies outside the canonical
//! preserved-order subset for its scope.
//!
//! Where two candidate subsets are equally large, the tie is settled by position and
//! then by the content of the competing pairs. Position alone is not enough: two pairs
//! that mirror one another, such as `(1, 2)` and `(2, 1)`, have the same positional key
//! from either side, so swapping the inputs would otherwise select the mirror image and
//! report the other pair as reordered.

use crate::{
    align::Aligned,
    alignment::NodeRef,
    comparison::Difference,
    error::CompareResult,
    features::FeatureSet,
    increasing::{maximum_increasing, symmetric_key},
    projection::{Item, OccurrenceId, Projection},
    sequence::Step,
};

/// A paired item and its positions within an aligned sibling scope
struct PositionedPair {
    positions: (usize, usize),
    occurrences: (OccurrenceId, OccurrenceId),
}

/// Derive the reorder observations of an alignment
pub(crate) fn derive(
    left: &Projection,
    left_features: &FeatureSet,
    right: &Projection,
    right_features: &FeatureSet,
    aligned: &Aligned,
) -> CompareResult<Vec<Difference>> {
    let mut differences = Vec::new();

    for scope in &aligned.properties {
        let left_parent = left.occurrence(scope.left_parent)?;
        let right_parent = right.occurrence(scope.right_parent)?;

        // Only structured items become correspondences, so only they can be reordered;
        // a scalar item of a mixed collection is an indexed value observation instead
        let mut pairs = Vec::new();
        for step in &scope.steps {
            let Step::Pair {
                left: left_index,
                right: right_index,
            } = *step
            else {
                continue;
            };
            let (Some(Item::Structured(left_id)), Some(Item::Structured(right_id))) = (
                item(left_parent, scope.property, left_index),
                item(right_parent, scope.property, right_index),
            ) else {
                continue;
            };
            pairs.push(PositionedPair {
                positions: (left_index, right_index),
                occurrences: (*left_id, *right_id),
            });
        }

        if pairs.len() < 2 {
            continue;
        }

        // The steps are in sequence order, but reconciliation may have appended pairs
        // that cross them, so sort before selecting
        let mut order: Vec<usize> = (0..pairs.len()).collect();
        order.sort_by_key(|index| pairs[*index].positions);
        let sorted: Vec<(usize, usize)> =
            order.iter().map(|index| pairs[*index].positions).collect();

        // The content of each pair, as an unordered pair of subtree fingerprints, so
        // that it too is unchanged by swapping the two inputs
        let mut content = Vec::with_capacity(sorted.len());
        for index in &order {
            let (left_id, right_id) = pairs[*index].occurrences;
            let left_fingerprint = left_features.get(left_id)?.fingerprint;
            let right_fingerprint = right_features.get(right_id)?.fingerprint;
            content.push((
                left_fingerprint.min(right_fingerprint),
                left_fingerprint.max(right_fingerprint),
            ));
        }

        let preserved = maximum_increasing(&sorted, |index| {
            (symmetric_key(sorted[index]), content[index])
        });
        let mut in_order = vec![false; sorted.len()];
        for index in preserved {
            in_order[index] = true;
        }

        let left_scope = Some(NodeRef::new(
            left_parent.path.clone(),
            left_parent.node_type,
        ));
        let right_scope = Some(NodeRef::new(
            right_parent.path.clone(),
            right_parent.node_type,
        ));

        for (position, original) in order.iter().enumerate() {
            if in_order[position] {
                continue;
            }
            let (left_id, right_id) = pairs[*original].occurrences;
            let left_occurrence = left.occurrence(left_id)?;
            let right_occurrence = right.occurrence(right_id)?;

            differences.push(Difference::Reordered {
                left: left_occurrence.node_ref(),
                right: right_occurrence.node_ref(),
                left_scope: left_scope.clone(),
                right_scope: right_scope.clone(),
                property: scope.property,
            });
        }
    }

    Ok(differences)
}

/// The item at a position of a property of an occurrence
fn item(
    occurrence: &crate::projection::Occurrence,
    property: stencila_node_type::NodeProperty,
    index: usize,
) -> Option<&Item> {
    occurrence
        .properties
        .iter()
        .find(|projected| projected.decl.property == property)
        .and_then(|projected| projected.items.get(index))
}
