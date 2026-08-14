//! Strong anchors within a sibling scope
//!
//! Verified unique anchors reduce ambiguity and cost by fixing part of the alignment
//! before the dynamic program runs, and by partitioning the sequences into smaller
//! gaps.
//!
//! An anchor is compulsory only when it is unambiguous:
//!
//! - a non-empty explicit schema `id` anchors only when it occurs exactly once on each
//!   side of the scope and the two items are compatible candidates;
//! - a fingerprint anchors only after full canonical equality has been verified, and
//!   only when the equal subtree occurs once on each side.
//!
//! Duplicate ids and duplicate equal subtrees remain ordinary candidates, so that
//! repeated boilerplate cannot force an arbitrary pairing.
//!
//! When otherwise valid anchors cross, because the collection was reordered, a
//! deterministic maximum non-crossing subset is used for the local ordered alignment.
//! The identities that were dropped can be reconciled afterwards as reorders.

use std::collections::HashMap;

use stencila_node_type::NodeType;

use crate::{alignment::MatchRule, error::CompareResult};

/// A compulsory correspondence between two positions in a sibling scope
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Anchor {
    /// The position in the left sequence
    pub left: usize,

    /// The position in the right sequence
    pub right: usize,

    /// The rule that established the anchor
    pub rule: MatchRule,

    /// A swap-invariant key, used to break ties deterministically
    ///
    /// Derived from what the two items share rather than from which side they are on,
    /// so that swapping the inputs selects the same anchors.
    pub key: AnchorKey,
}

/// A swap-invariant key for an anchor
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AnchorKey {
    /// A hash of the shared explicit `id`, or zero when the anchor is not by `id`
    pub identity: u64,

    /// The shared node type, or `None` for a scalar item
    pub node_type: Option<NodeType>,

    /// The lesser of the two fingerprints
    pub lesser_fingerprint: u64,

    /// The greater of the two fingerprints
    pub greater_fingerprint: u64,
}

/// What an anchor candidate needs to know about one item
#[derive(Debug, Clone)]
pub struct Candidate {
    /// The explicit schema `id`, when present and not empty
    pub explicit_id: Option<String>,

    /// The canonical structural fingerprint
    pub fingerprint: u64,

    /// The concrete node type, or `None` for a scalar item
    pub node_type: Option<NodeType>,
}

/// Find the compulsory anchors between two sequences
///
/// `compatible` decides whether two positions may be paired at all, and `verified_eq`
/// confirms that two positions with equal fingerprints really do hold equal subtrees.
pub fn find(
    left: &[Option<Candidate>],
    right: &[Option<Candidate>],
    compatible: &dyn Fn(usize, usize) -> CompareResult<bool>,
    verified_eq: &dyn Fn(usize, usize) -> CompareResult<bool>,
) -> CompareResult<Vec<Anchor>> {
    let mut anchors = Vec::new();
    let mut left_taken = vec![false; left.len()];
    let mut right_taken = vec![false; right.len()];

    // Explicit ids first: an id is a deliberate identity, and a stronger signal than
    // an incidentally equal subtree
    let left_ids = unique_positions(left, |candidate| {
        candidate.explicit_id.as_ref().map(|id| id.to_string())
    });
    let right_ids = unique_positions(right, |candidate| {
        candidate.explicit_id.as_ref().map(|id| id.to_string())
    });
    for (id, left_index) in &left_ids {
        let Some(right_index) = right_ids.get(id) else {
            continue;
        };
        if !compatible(*left_index, *right_index)? {
            continue;
        }
        take(
            &mut anchors,
            &mut left_taken,
            &mut right_taken,
            left,
            right,
            *left_index,
            *right_index,
            MatchRule::UniqueId,
            hash_id(id),
        );
    }

    // Then verified exact subtrees
    let left_fingerprints = unique_positions(left, |candidate| Some(candidate.fingerprint));
    let right_fingerprints = unique_positions(right, |candidate| Some(candidate.fingerprint));
    for (fingerprint, left_index) in &left_fingerprints {
        let Some(right_index) = right_fingerprints.get(fingerprint) else {
            continue;
        };
        if left_taken[*left_index] || right_taken[*right_index] {
            continue;
        }
        if !compatible(*left_index, *right_index)? {
            continue;
        }
        // A fingerprint is an accelerator, never a proof
        if !verified_eq(*left_index, *right_index)? {
            continue;
        }
        take(
            &mut anchors,
            &mut left_taken,
            &mut right_taken,
            left,
            right,
            *left_index,
            *right_index,
            MatchRule::VerifiedExactFingerprint,
            0,
        );
    }

    anchors.sort_by_key(|anchor| (anchor.left, anchor.right));

    Ok(non_crossing(anchors))
}

/// The positions of the values that occur exactly once
fn unique_positions<Key, Extract>(
    candidates: &[Option<Candidate>],
    extract: Extract,
) -> HashMap<Key, usize>
where
    Key: std::hash::Hash + Eq,
    Extract: Fn(&Candidate) -> Option<Key>,
{
    let mut counts: HashMap<Key, (usize, usize)> = HashMap::new();
    for (index, candidate) in candidates.iter().enumerate() {
        let Some(candidate) = candidate else { continue };
        let Some(key) = extract(candidate) else {
            continue;
        };
        counts
            .entry(key)
            .and_modify(|(count, ..)| *count += 1)
            .or_insert((1, index));
    }

    counts
        .into_iter()
        .filter_map(|(key, (count, index))| (count == 1).then_some((key, index)))
        .collect()
}

#[allow(clippy::too_many_arguments)]
fn take(
    anchors: &mut Vec<Anchor>,
    left_taken: &mut [bool],
    right_taken: &mut [bool],
    left: &[Option<Candidate>],
    right: &[Option<Candidate>],
    left_index: usize,
    right_index: usize,
    rule: MatchRule,
    identity: u64,
) {
    if left_taken[left_index] || right_taken[right_index] {
        return;
    }
    let (Some(Some(left_candidate)), Some(Some(right_candidate))) =
        (left.get(left_index), right.get(right_index))
    else {
        return;
    };

    left_taken[left_index] = true;
    right_taken[right_index] = true;
    anchors.push(Anchor {
        left: left_index,
        right: right_index,
        rule,
        key: AnchorKey {
            identity,
            node_type: left_candidate.node_type.min(right_candidate.node_type),
            lesser_fingerprint: left_candidate.fingerprint.min(right_candidate.fingerprint),
            greater_fingerprint: left_candidate.fingerprint.max(right_candidate.fingerprint),
        },
    });
}

/// A stable hash of an explicit `id`
fn hash_id(id: &str) -> u64 {
    let mut fingerprinter = crate::fingerprint::Fingerprinter::new();
    fingerprinter.write_str(id);
    fingerprinter.finish()
}

/// A deterministic maximum non-crossing subset of anchors
///
/// Two anchors cross when one's left position precedes the other's while its right
/// position follows, so a maximum non-crossing subset is a longest strictly increasing
/// subsequence of right positions, taken in left order.
///
/// Where several subsets are of maximum size, the one whose anchors have the smallest
/// keys is chosen. Because those keys are swap invariant, so is the choice.
fn non_crossing(anchors: Vec<Anchor>) -> Vec<Anchor> {
    let count = anchors.len();
    if count < 2 {
        return anchors;
    }

    // The length of the longest chain starting at each anchor
    let mut chain = vec![1usize; count];
    for index in (0..count).rev() {
        for next in (index + 1)..count {
            if anchors[next].right > anchors[index].right {
                chain[index] = chain[index].max(chain[next] + 1);
            }
        }
    }

    let Some(longest) = chain.iter().copied().max() else {
        return Vec::new();
    };

    let mut chosen = Vec::with_capacity(longest);
    let mut remaining = longest;
    let mut start = 0usize;
    let mut previous_right: Option<usize> = None;
    while remaining > 0 {
        // Among the anchors that can still start a chain of the remaining length,
        // without crossing what has already been chosen, take the one with the
        // smallest key
        let best = (start..count)
            .filter(|candidate| {
                chain[*candidate] == remaining
                    && previous_right.is_none_or(|right| anchors[*candidate].right > right)
            })
            .min_by_key(|candidate| anchors[*candidate].key);
        let Some(best) = best else { break };

        chosen.push(anchors[best]);
        previous_right = Some(anchors[best].right);
        start = best + 1;
        remaining -= 1;
    }

    chosen.sort_by_key(|anchor| (anchor.left, anchor.right));
    chosen
}
