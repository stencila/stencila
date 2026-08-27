//! Finding the runs of a text that differ
//!
//! A `Cord` holds as much prose as its node does — a whole paragraph, a whole abstract —
//! so a difference in one is almost never a difference in all of it. Marking the node
//! that holds it would report a two-character correction as a rewrite of everything
//! around it, which is both wrong as a description and unreadable as a review.
//!
//! Words, not characters, are the unit. A character-level diff of prose produces marks
//! that begin and end inside words, which reads as noise; a word-level one produces the
//! runs a person would point at. Runs are merged across the whitespace between two
//! changed words for the same reason, so that a rewritten phrase is one mark rather than
//! a scattering of them.

use std::time::Duration;

use similar::{Algorithm, ChangeTag, TextDiff};

/// A stretch of text, and whether it differs between the two sides
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum TextRun {
    /// Text that is the same on both sides
    Unchanged(String),

    /// Text that differs
    ///
    /// Either side may be empty, which is an insertion or a deletion rather than a
    /// substitution.
    Changed { before: String, after: String },
}

/// The most text worth diffing within
///
/// Beyond this the marks stop being a useful description of an edit, and the diff stops
/// being cheap. The whole text is reported as one changed run instead, which is what
/// marking the node would have done anyway.
const MAX_DIFFED_CHARACTERS: usize = 10_000;

/// How long to spend looking for the runs
const DIFF_TIMEOUT: Duration = Duration::from_millis(250);

/// Split two texts into their shared and differing runs
pub(crate) fn text_runs(left: &str, right: &str) -> Vec<TextRun> {
    if left == right {
        return vec![TextRun::Unchanged(left.to_string())];
    }

    if left.len() > MAX_DIFFED_CHARACTERS || right.len() > MAX_DIFFED_CHARACTERS {
        return vec![TextRun::Changed {
            before: left.to_string(),
            after: right.to_string(),
        }];
    }

    let diff = TextDiff::configure()
        .algorithm(Algorithm::Patience)
        .timeout(DIFF_TIMEOUT)
        .diff_unicode_words(left, right);

    let mut runs: Vec<TextRun> = Vec::new();
    for change in diff.iter_all_changes() {
        let value = change.value();
        match change.tag() {
            ChangeTag::Equal => push_unchanged(&mut runs, value),
            ChangeTag::Delete => push_changed(&mut runs, value, ""),
            ChangeTag::Insert => push_changed(&mut runs, "", value),
        }
    }

    merge_across_whitespace(&mut runs);

    runs
}

/// Add unchanged text, extending the run in progress when there is one
fn push_unchanged(runs: &mut Vec<TextRun>, value: &str) {
    match runs.last_mut() {
        Some(TextRun::Unchanged(text)) => text.push_str(value),
        _ => runs.push(TextRun::Unchanged(value.to_string())),
    }
}

/// Add changed text, extending the run in progress when there is one
fn push_changed(runs: &mut Vec<TextRun>, before: &str, after: &str) {
    match runs.last_mut() {
        Some(TextRun::Changed {
            before: existing_before,
            after: existing_after,
        }) => {
            existing_before.push_str(before);
            existing_after.push_str(after);
        }
        _ => runs.push(TextRun::Changed {
            before: before.to_string(),
            after: after.to_string(),
        }),
    }
}

/// Absorb the whitespace between two changed runs into a single one
///
/// A rewritten phrase comes back as changed words separated by the unchanged spaces
/// between them. Marking each word on its own would say the spaces survived a rewrite,
/// which is true and useless: what the reader wants to see is the phrase.
fn merge_across_whitespace(runs: &mut Vec<TextRun>) {
    let mut index = 1;
    while index + 1 < runs.len() {
        let separates_changes = matches!(runs[index - 1], TextRun::Changed { .. })
            && matches!(runs[index + 1], TextRun::Changed { .. })
            && matches!(&runs[index], TextRun::Unchanged(text) if is_blank(text));

        if !separates_changes {
            index += 1;
            continue;
        }

        let TextRun::Unchanged(gap) = runs.remove(index) else {
            unreachable!("checked immediately above")
        };
        let TextRun::Changed { before, after } = runs.remove(index) else {
            unreachable!("checked immediately above")
        };

        if let Some(TextRun::Changed {
            before: into_before,
            after: into_after,
        }) = runs.get_mut(index - 1)
        {
            into_before.push_str(&gap);
            into_before.push_str(&before);
            into_after.push_str(&gap);
            into_after.push_str(&after);
        }
    }
}

/// Whether a run is only whitespace
fn is_blank(text: &str) -> bool {
    !text.is_empty() && text.chars().all(char::is_whitespace)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_text_is_one_unchanged_run() {
        assert_eq!(
            text_runs("the same", "the same"),
            vec![TextRun::Unchanged("the same".to_string())]
        );
    }

    #[test]
    fn only_the_differing_words_are_marked() {
        // The case that prompted this: a long text differing in one word
        let runs = text_runs(
            "a quick brown fox jumps over the lazy dog",
            "a quick brown fox leaps over the lazy dog",
        );

        assert_eq!(
            runs,
            vec![
                TextRun::Unchanged("a quick brown fox ".to_string()),
                TextRun::Changed {
                    before: "jumps".to_string(),
                    after: "leaps".to_string(),
                },
                TextRun::Unchanged(" over the lazy dog".to_string()),
            ]
        );
    }

    #[test]
    fn a_rewritten_phrase_is_one_run_not_several() {
        let runs = text_runs("keep the old wording here", "keep a new phrasing here");

        assert_eq!(
            runs.iter()
                .filter(|run| matches!(run, TextRun::Changed { .. }))
                .count(),
            1,
            "{runs:?}"
        );
    }

    #[test]
    fn an_addition_has_nothing_before_it() {
        let runs = text_runs("one three", "one two three");

        assert!(
            runs.iter().any(|run| matches!(
                run,
                TextRun::Changed { before, after } if before.is_empty() && after.contains("two")
            )),
            "{runs:?}"
        );
    }

    #[test]
    fn very_long_text_is_not_diffed_within() {
        let left = "word ".repeat(MAX_DIFFED_CHARACTERS);
        let right = format!("{left}and more");

        assert_eq!(
            text_runs(&left, &right),
            vec![TextRun::Changed {
                before: left,
                after: right,
            }]
        );
    }
}
