//! Conservative text matching for content moved between parents.
//!
//! Candidates have already been restricted to one concrete block type. A move must
//! be the best candidate in both directions, with a margin over either runner-up.
//! Equal or nearly equal alternatives stay unmatched rather than being resolved by
//! document order. This makes selection deterministic and symmetric without making
//! repeated boilerplate into evidence of a move.

use crate::{alignment::AlignmentCost, features::Features, text};

/// Minimum text similarity for a move, stricter than local sibling alignment.
const MIN_SIMILARITY: AlignmentCost = AlignmentCost::from_ratio(85, 100);

/// Required separation from the next-best alternative on each side.
const MIN_MARGIN: AlignmentCost = AlignmentCost::from_ratio(5, 100);

/// Short text must match exactly; a tiny edit to a short label is weak evidence.
const MIN_EDITED_GRAMS: usize = 24;

#[derive(Default)]
struct Best {
    index: Option<usize>,
    score: AlignmentCost,
    runner_up: AlignmentCost,
}

impl Best {
    fn observe(&mut self, index: usize, score: AlignmentCost) {
        if score > self.score {
            self.runner_up = self.score;
            self.score = score;
            self.index = Some(index);
        } else {
            self.runner_up = self.runner_up.max(score);
        }
    }

    fn unambiguous(&self) -> bool {
        self.score >= MIN_SIMILARITY && self.score >= self.runner_up.saturating_add(MIN_MARGIN)
    }
}

/// Find mutually unambiguous moves, using linear storage for the candidate matrix.
///
/// The caller charges `left.len() * right.len()` cells before calling this function.
pub fn find(left: &[&Features], right: &[&Features]) -> Vec<(usize, usize)> {
    let mut left_best: Vec<Best> = (0..left.len()).map(|_| Best::default()).collect();
    let mut right_best: Vec<Best> = (0..right.len()).map(|_| Best::default()).collect();
    for (left_index, left) in left.iter().enumerate() {
        for (right_index, right) in right.iter().enumerate() {
            let score = text::similarity(&left.grams, &right.grams);
            // Still record short alternatives as competitors: weak evidence must
            // not make a different, ambiguous candidate appear uniquely best.
            left_best[left_index].observe(right_index, score);
            right_best[right_index].observe(left_index, score);
        }
    }

    left_best
        .iter()
        .enumerate()
        .filter_map(|(left_index, best)| {
            let right_index = best.index?;
            let reverse = &right_best[right_index];
            // Compare the text itself: distinct strings, such as rotations of one
            // another, can share every n-gram and so score as identical
            let enough_text = left[left_index].text == right[right_index].text
                || left[left_index]
                    .grams
                    .len()
                    .min(right[right_index].grams.len())
                    >= MIN_EDITED_GRAMS;
            (best.unambiguous()
                && reverse.unambiguous()
                && reverse.index == Some(left_index)
                && enough_text)
                .then_some((left_index, right_index))
        })
        .collect()
}
