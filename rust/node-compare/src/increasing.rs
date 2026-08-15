//! A deterministic maximum increasing subset of paired positions
//!
//! Two problems in this crate reduce to the same one: choosing which of a set of
//! candidate correspondences can be kept in an order-preserving alignment, and
//! choosing which of a set of established pairs count as having kept their order. In
//! both cases the answer is a maximum subset whose left and right positions increase
//! together — a longest strictly increasing subsequence — and in both cases the choice
//! must be deterministic and unchanged by swapping the two inputs.
//!
//! Where several subsets are of maximum size, the tie is settled by a caller-supplied
//! key. A key that is a property of the pair rather than of the side it is on makes
//! the whole selection swap invariant.
//!
//! One case admits no swap-invariant answer from position alone: when the candidate
//! pairs are exact mirrors of one another — two pairs `(a, b)` and `(b, a)` that cannot
//! both be kept — swapping the inputs maps the candidate set onto itself while
//! exchanging the two pairs, so every maximum subset is the mirror image of another and
//! none is fixed by the swap. Callers avoid this by extending the key with something
//! that distinguishes the pairs by content, as [`crate::reorder`] does with an
//! unordered pair of subtree fingerprints. Where content is genuinely identical too,
//! the two answers are mirror images of one another and the earlier position is chosen.

/// Choose a maximum subset of pairs whose right positions strictly increase
///
/// `pairs` must be sorted by left position, and both its left and its right positions
/// must be distinct. Returns the indices of the chosen pairs, in order.
pub fn maximum_increasing<Key, ToKey>(pairs: &[(usize, usize)], key: ToKey) -> Vec<usize>
where
    Key: Ord,
    ToKey: Fn(usize) -> Key,
{
    let count = pairs.len();
    if count < 2 {
        return (0..count).collect();
    }

    // The length of the longest increasing chain starting at each pair
    let mut chain = vec![1usize; count];
    for index in (0..count).rev() {
        for next in (index + 1)..count {
            if pairs[next].1 > pairs[index].1 {
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
        // Among the pairs that can still start a chain of the remaining length,
        // without crossing what has already been chosen, take the one with the
        // smallest key
        let best = (start..count)
            .filter(|candidate| {
                chain[*candidate] == remaining
                    && previous_right.is_none_or(|right| pairs[*candidate].1 > right)
            })
            .min_by_key(|candidate| key(*candidate));
        let Some(best) = best else { break };

        chosen.push(best);
        previous_right = Some(pairs[best].1);
        start = best + 1;
        remaining -= 1;
    }

    chosen
}

/// A tie-break key that is a property of the pair rather than of either side
///
/// Unchanged by swapping the two inputs, because swapping a pair `(left, right)` into
/// `(right, left)` leaves both the lesser and the greater position the same.
pub fn symmetric_key(pair: (usize, usize)) -> (usize, usize) {
    (pair.0.min(pair.1), pair.0.max(pair.1))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The chosen subset is of maximum size
    #[test]
    fn chooses_a_maximum_subset() {
        // Already increasing: everything is kept
        let pairs = [(0, 0), (1, 1), (2, 2)];
        assert_eq!(
            maximum_increasing(&pairs, |index| symmetric_key(pairs[index])),
            vec![0, 1, 2]
        );

        // Fully reversed: only one can be kept
        let pairs = [(0, 2), (1, 1), (2, 0)];
        assert_eq!(
            maximum_increasing(&pairs, |index| symmetric_key(pairs[index])).len(),
            1
        );

        // One item moved to the end: the rest keep their order
        let pairs = [(0, 3), (1, 0), (2, 1), (3, 2)];
        assert_eq!(
            maximum_increasing(&pairs, |index| symmetric_key(pairs[index])),
            vec![1, 2, 3]
        );
    }

    /// The selection is unchanged by swapping the two sides
    #[test]
    fn is_swap_invariant() {
        for pairs in [
            vec![(0, 2), (1, 0), (2, 1)],
            vec![(0, 3), (1, 1), (2, 0), (3, 2)],
            vec![(0, 1), (1, 4), (2, 2), (3, 0), (4, 3)],
        ] {
            let forward: Vec<(usize, usize)> =
                maximum_increasing(&pairs, |index| symmetric_key(pairs[index]))
                    .into_iter()
                    .map(|index| pairs[index])
                    .collect();

            let mut swapped: Vec<(usize, usize)> =
                pairs.iter().map(|(left, right)| (*right, *left)).collect();
            swapped.sort();
            let mut inverted: Vec<(usize, usize)> =
                maximum_increasing(&swapped, |index| symmetric_key(swapped[index]))
                    .into_iter()
                    .map(|index| (swapped[index].1, swapped[index].0))
                    .collect();

            inverted.sort();
            let mut forward = forward;
            forward.sort();
            assert_eq!(forward, inverted, "for {pairs:?}");
        }
    }

    /// A set that is its own mirror image has no swap-invariant maximum subset, and
    /// the two answers are mirror images of each other
    #[test]
    fn mirrored_input_gives_mirrored_answers() {
        // Swapping the sides maps this set of pairs onto itself, exchanging `(1, 2)`
        // with `(2, 1)`, which cannot both be kept
        let pairs = [(0, 0), (1, 2), (2, 1), (3, 3)];

        let forward: Vec<(usize, usize)> =
            maximum_increasing(&pairs, |index| symmetric_key(pairs[index]))
                .into_iter()
                .map(|index| pairs[index])
                .collect();
        let mirrored: Vec<(usize, usize)> = forward
            .iter()
            .map(|(left, right)| (*right, *left))
            .collect();

        assert_eq!(forward, vec![(0, 0), (1, 2), (3, 3)]);
        assert_eq!(mirrored, vec![(0, 0), (2, 1), (3, 3)]);
        assert_eq!(forward.len(), mirrored.len(), "both are of maximum size");
    }

    /// Empty and single inputs are handled
    #[test]
    fn handles_small_inputs() {
        assert!(maximum_increasing(&[], |_| 0usize).is_empty());
        assert_eq!(maximum_increasing(&[(3, 7)], |_| 0usize), vec![0]);
    }
}
