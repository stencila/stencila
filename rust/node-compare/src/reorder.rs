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
//! Paired scalar items of mixed collections retain their sequence alignment. They
//! partition the structured pairs into gaps: a structured item crossing a scalar
//! pair is reordered, and the maximum preserved subset is chosen within the gaps.
//! Scalars remain value observations rather than gaining occurrence references.
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

use std::collections::HashMap;

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
    let pairing: HashMap<_, _> = aligned.pairs.iter().copied().collect();

    for scope in &aligned.properties {
        let left_parent = left.occurrence(scope.left_parent)?;
        let right_parent = right.occurrence(scope.right_parent)?;
        let left_items = items(left_parent, scope.property);
        let right_items = items(right_parent, scope.property);

        // Reconciliation can pair siblings after their local sequence alignment.
        // Use the final pairs, restricted to these two corresponding properties, so
        // those siblings participate but moves between parents or properties do not.
        let right_positions: HashMap<_, _> = right_items
            .iter()
            .enumerate()
            .filter_map(|(index, item)| match item {
                Item::Structured(id) => Some((*id, index)),
                Item::Scalar(..) => None,
            })
            .collect();
        let pairs: Vec<_> = left_items
            .iter()
            .enumerate()
            .filter_map(|(left_index, item)| {
                let Item::Structured(left_id) = item else {
                    return None;
                };
                let right_id = pairing.get(left_id)?;
                let right_index = right_positions.get(right_id)?;
                Some(PositionedPair {
                    positions: (left_index, *right_index),
                    occurrences: (*left_id, *right_id),
                })
            })
            .collect();

        if pairs.is_empty() {
            continue;
        }

        // Scalar pairs come only from the order-preserving sequence alignment.
        // Keep them fixed: they have no occurrence to carry a reorder observation.
        let mut scalars: Vec<_> = scope
            .steps
            .iter()
            .filter_map(|step| match *step {
                Step::Pair { left, right }
                    if matches!(left_items.get(left), Some(Item::Scalar(..)))
                        && matches!(right_items.get(right), Some(Item::Scalar(..))) =>
                {
                    Some((left, right))
                }
                _ => None,
            })
            .collect();
        scalars.sort_unstable();

        // A structured pair can be preserved only if it lies in the same gap
        // between scalar pairs on both sides. The remaining pairs are already in
        // left position order, and pairs in different gaps cannot cross each other.
        let order: Vec<usize> = pairs
            .iter()
            .enumerate()
            .filter(|(_, pair)| {
                scalars.partition_point(|scalar| scalar.0 < pair.positions.0)
                    == scalars.partition_point(|scalar| scalar.1 < pair.positions.1)
            })
            .map(|(index, _)| index)
            .collect();
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
        let mut in_order = vec![false; pairs.len()];
        for index in preserved {
            in_order[order[index]] = true;
        }

        let left_scope = Some(NodeRef::new(
            left_parent.path.clone(),
            left_parent.node_type,
        ));
        let right_scope = Some(NodeRef::new(
            right_parent.path.clone(),
            right_parent.node_type,
        ));

        for (position, pair) in pairs.iter().enumerate() {
            if in_order[position] {
                continue;
            }
            let (left_id, right_id) = pair.occurrences;
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

/// The items of a property of an occurrence
fn items(
    occurrence: &crate::projection::Occurrence,
    property: stencila_node_type::NodeProperty,
) -> &[Item] {
    occurrence
        .properties
        .iter()
        .find(|projected| projected.decl.property == property)
        .map_or(&[], |projected| projected.items.as_slice())
}
