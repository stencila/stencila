//! Text normalization and similarity used to find candidate pairs
//!
//! Matching normalization and equality normalization are separate concerns. Text is
//! normalized here only in order to *locate* a pair; the value policy still compares
//! the original strings exactly, so normalization can never suppress a value
//! difference.
//!
//! Normalization is deliberately conservative: Unicode NFC, normalized line endings,
//! trimmed and collapsed Unicode whitespace. No case folding, punctuation stripping,
//! stemming, or label parsing — those are domain judgements that belong in a
//! node-specific recipe, not in the generic policy.

use unicode_normalization::UnicodeNormalization;

use crate::alignment::AlignmentCost;

/// The size of the character n-grams used for similarity
const GRAM_SIZE: usize = 3;

/// The greatest number of characters of an occurrence's text used for matching
///
/// Candidate scoring is linear in the compared text sizes, and the number of candidate
/// pairs is bounded by the cell budget, so bounding the text as well is what turns the
/// budget into a bound on work rather than merely on cells. A section's worth of text
/// is far more than similarity needs to tell two candidates apart.
///
/// This bounds only the *matching* signal. The value policy still compares the whole
/// of every string exactly, so truncation cannot hide a value difference.
pub(crate) const MAX_MATCH_CHARACTERS: usize = 1024;

/// Incrementally normalized text, bounded to the prefix used for matching
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct NormalizedText {
    text: String,
    characters: usize,
}

impl NormalizedText {
    /// Append another logical text part, separated from earlier content by one space
    pub fn append(&mut self, text: &str) {
        if self.characters >= MAX_MATCH_CHARACTERS {
            return;
        }

        let mut pending_space = self.characters > 0;
        for character in text.nfc() {
            if character.is_whitespace() {
                pending_space = self.characters > 0;
                continue;
            }
            if pending_space {
                self.text.push(' ');
                self.characters += 1;
                if self.characters >= MAX_MATCH_CHARACTERS {
                    break;
                }
                pending_space = false;
            }
            self.text.push(character);
            self.characters += 1;
            if self.characters >= MAX_MATCH_CHARACTERS {
                break;
            }
        }
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }
}

/// Normalize text for matching
pub fn normalize(text: &str) -> String {
    // Public normalization is not truncated; feature extraction uses `NormalizedText`
    // directly so it never retains text beyond the matching limit.
    let mut normalized = String::with_capacity(text.len());
    let mut pending_space = false;
    for character in text.nfc() {
        if character.is_whitespace() {
            pending_space = !normalized.is_empty();
        } else {
            if pending_space {
                normalized.push(' ');
                pending_space = false;
            }
            normalized.push(character);
        }
    }
    normalized
}

/// The character n-grams of a normalized string, as a sorted multiset
///
/// Sorted so that similarity is a linear merge of the two multisets rather than a
/// quadratic comparison, keeping candidate scoring bounded and linear in the compared
/// text sizes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Grams {
    /// The n-grams, sorted
    grams: Vec<String>,

    /// Whether the text was too short to form an n-gram, in which case the whole
    /// string is the single gram
    short: bool,
}

impl Grams {
    /// The n-grams of a normalized string
    pub fn new(normalized: &str) -> Self {
        let characters: Vec<char> = normalized.chars().take(MAX_MATCH_CHARACTERS).collect();

        // Explicit short-string handling: a string shorter than one n-gram has no
        // n-grams at all, so it is compared as a whole
        if characters.len() < GRAM_SIZE {
            return Self {
                grams: if normalized.is_empty() {
                    Vec::new()
                } else {
                    vec![normalized.to_string()]
                },
                short: true,
            };
        }

        let mut grams: Vec<String> = characters
            .windows(GRAM_SIZE)
            .map(|window| window.iter().collect())
            .collect();
        grams.sort();

        Self {
            grams,
            short: false,
        }
    }

    /// The number of n-grams
    pub fn len(&self) -> usize {
        self.grams.len()
    }

    /// Whether there are no n-grams, which is the case only for empty text
    pub fn is_empty(&self) -> bool {
        self.grams.is_empty()
    }
}

/// The similarity of two n-gram multisets, as a cost between zero and one
///
/// Multiset Dice: twice the size of the multiset intersection, divided by the sum of
/// the two sizes. Two empty texts are identical, and a short string is compared
/// against another short string as a whole.
pub fn similarity(left: &Grams, right: &Grams) -> AlignmentCost {
    if left.is_empty() && right.is_empty() {
        return AlignmentCost::ONE;
    }
    if left.is_empty() || right.is_empty() {
        return AlignmentCost::ZERO;
    }

    // A short string and a long one share no comparable representation, so they are
    // similar only if the short string is identical to the other
    if left.short != right.short {
        return AlignmentCost::ZERO;
    }

    let mut shared = 0usize;
    let (mut left_index, mut right_index) = (0usize, 0usize);
    while left_index < left.grams.len() && right_index < right.grams.len() {
        match left.grams[left_index].cmp(&right.grams[right_index]) {
            std::cmp::Ordering::Less => left_index += 1,
            std::cmp::Ordering::Greater => right_index += 1,
            std::cmp::Ordering::Equal => {
                shared += 1;
                left_index += 1;
                right_index += 1;
            }
        }
    }

    AlignmentCost::from_ratio(2 * shared as i64, (left.len() + right.len()) as i64)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Normalization collapses whitespace and line endings, and trims, but does not
    /// case fold, strip punctuation, or stem
    #[test]
    fn normalization_is_conservative() {
        assert_eq!(normalize("  Hello,\r\n  World!  "), "Hello, World!");
        assert_eq!(normalize("Hello\t\u{2003}World"), "Hello World");
        assert_eq!(normalize(""), "");
        assert_eq!(normalize("   "), "");

        // Case, punctuation and word forms are all preserved
        assert_eq!(normalize("The Cats' Paws."), "The Cats' Paws.");

        // Composed and decomposed forms normalize to the same string
        assert_eq!(normalize("e\u{0301}"), normalize("\u{00e9}"));
    }

    /// Incremental feature text retains exactly the bounded normalized prefix
    #[test]
    fn incremental_normalization_is_bounded() {
        let left = "a".repeat(700);
        let right = "b".repeat(700);
        let mut text = NormalizedText::default();
        text.append(&left);
        text.append(&right);

        let expected: String = normalize(&format!("{left} {right}"))
            .chars()
            .take(MAX_MATCH_CHARACTERS)
            .collect();
        assert_eq!(text.as_str(), expected);
        assert_eq!(text.as_str().chars().count(), MAX_MATCH_CHARACTERS);
    }

    /// Similarity is a multiset Dice coefficient over character trigrams
    #[test]
    fn similarity_is_dice_over_trigrams() {
        let grams = |text: &str| Grams::new(&normalize(text));

        // Identical text is completely similar
        assert_eq!(
            similarity(&grams("Hello world"), &grams("Hello world")),
            AlignmentCost::ONE
        );

        // Two empty texts are identical, and an empty text resembles nothing else
        assert_eq!(similarity(&grams(""), &grams("")), AlignmentCost::ONE);
        assert_eq!(similarity(&grams(""), &grams("Hello")), AlignmentCost::ZERO);

        // Unrelated text shares nothing
        assert_eq!(
            similarity(&grams("aaaaaaa"), &grams("zzzzzzz")),
            AlignmentCost::ZERO
        );

        // A small edit is still mostly similar
        let close = similarity(
            &grams("The quick brown fox"),
            &grams("The quick brown foxes"),
        );
        assert!(close > AlignmentCost::from_ratio(1, 2));

        // A string shorter than one trigram is compared whole
        assert_eq!(similarity(&grams("ab"), &grams("ab")), AlignmentCost::ONE);
        assert_eq!(similarity(&grams("ab"), &grams("cd")), AlignmentCost::ZERO);
        assert_eq!(
            similarity(&grams("ab"), &grams("abcdef")),
            AlignmentCost::ZERO
        );
    }

    /// Similarity is symmetric
    #[test]
    fn similarity_is_symmetric() {
        let left = Grams::new(&normalize("The quick brown fox jumps"));
        let right = Grams::new(&normalize("The quick brown foxes jump"));

        assert_eq!(similarity(&left, &right), similarity(&right, &left));
    }
}
