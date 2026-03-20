//! Shared helpers for interview condition evaluation, visibility, and
//! canonical answer storage.
//!
//! These functions are the single source of truth for:
//! - building a `store_key → canonical answer` map from questions and answers
//! - deciding whether a question should be shown (`show_if` evaluation)
//! - deciding whether a `finish_if` condition was triggered
//!
//! All interview frontends (default [`Interviewer::conduct`], TUI, message
//! rendering) should call through these helpers so that conditional
//! semantics stay in sync.

use indexmap::IndexMap;

use crate::interviewer::{Answer, AnswerValue, Question, canonical_answer_string};

/// Build a map of `store_key → canonical answer string` from parallel
/// question/answer slices, considering only questions up to (but not
/// including) `up_to`.
///
/// Skipped answers are excluded from the map so that `show_if`
/// conditions referencing a skipped question's store key evaluate as
/// "key missing" (which `Condition::NotEquals` treats as `true` and
/// `Condition::Equals` treats as `false`).
#[must_use]
pub fn build_stored_answers(
    questions: &[Question],
    answers: &[Answer],
    up_to: usize,
) -> IndexMap<String, String> {
    let mut stored = IndexMap::new();
    for (q, a) in questions.iter().zip(answers.iter()).take(up_to) {
        if let Some(ref store_key) = q.store
            && !matches!(a.value, AnswerValue::Skipped)
        {
            stored.insert(store_key.clone(), canonical_answer_string(&a.value, q));
        }
    }
    stored
}

/// Evaluate whether a question should be shown given answers collected
/// so far.
///
/// Returns `true` when:
/// - the question has no `show_if` condition, or
/// - the condition parses and evaluates to true, or
/// - the condition fails to parse (show unconditionally, matching the
///   "fail-open" policy used by the default `Interviewer::conduct`)
///
/// When `warn_on_parse_error` is true, a `tracing::warn` is emitted
/// for parse failures. Callers in the default `conduct()` set this to
/// `true`; UI callers may pass `false` to avoid duplicate warnings.
#[must_use]
pub fn should_show_question(
    question: &Question,
    stored_answers: &IndexMap<String, String>,
    warn_on_parse_error: bool,
) -> bool {
    let Some(ref show_if_str) = question.show_if else {
        return true;
    };
    match crate::condition::Condition::parse(show_if_str) {
        Ok(cond) => cond.evaluate(stored_answers),
        Err(e) => {
            if warn_on_parse_error {
                tracing::warn!(
                    show_if = show_if_str.as_str(),
                    error = %e,
                    "show_if condition failed to parse; presenting question unconditionally"
                );
            }
            true
        }
    }
}

/// Check whether a `finish_if` condition is triggered by the given answer.
///
/// Computes the canonical answer string and delegates to
/// [`Question::is_finish_triggered`]. This works regardless of whether
/// the question has a `store` key.
#[must_use]
pub fn is_finish_triggered(question: &Question, answer: &Answer) -> bool {
    if question.finish_if.is_none() {
        return false;
    }
    let canonical = canonical_answer_string(&answer.value, question);
    question.is_finish_triggered(&canonical)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interviewer::QuestionOption;

    fn yes_no_q(store: Option<&str>, finish_if: Option<&str>) -> Question {
        Question {
            store: store.map(Into::into),
            finish_if: finish_if.map(Into::into),
            ..Question::yes_no("test?")
        }
    }

    fn single_select_q(store: Option<&str>, finish_if: Option<&str>) -> Question {
        Question {
            store: store.map(Into::into),
            finish_if: finish_if.map(Into::into),
            ..Question::single_select(
                "pick?",
                vec![
                    QuestionOption {
                        key: "A".into(),
                        label: "Approve".into(),
                        description: None,
                    },
                    QuestionOption {
                        key: "B".into(),
                        label: "Reject".into(),
                        description: None,
                    },
                ],
            )
        }
    }

    // -----------------------------------------------------------------------
    // build_stored_answers
    // -----------------------------------------------------------------------

    #[test]
    fn build_stored_answers_basic() {
        let questions = vec![
            yes_no_q(Some("approved"), None),
            Question {
                store: Some("reason".into()),
                ..Question::freeform("why?")
            },
        ];
        let answers = vec![
            Answer::new(AnswerValue::Yes),
            Answer::new(AnswerValue::Text("because".into())),
        ];
        let stored = build_stored_answers(&questions, &answers, 2);
        assert_eq!(stored.get("approved").map(String::as_str), Some("yes"));
        assert_eq!(stored.get("reason").map(String::as_str), Some("because"));
    }

    #[test]
    fn build_stored_answers_skips_skipped() {
        let questions = vec![
            yes_no_q(Some("approved"), None),
            Question {
                store: Some("reason".into()),
                ..Question::freeform("why?")
            },
        ];
        let answers = vec![
            Answer::new(AnswerValue::Yes),
            Answer::new(AnswerValue::Skipped),
        ];
        let stored = build_stored_answers(&questions, &answers, 2);
        assert_eq!(stored.get("approved").map(String::as_str), Some("yes"));
        assert!(stored.get("reason").is_none());
    }

    #[test]
    fn build_stored_answers_up_to_limits() {
        let questions = vec![yes_no_q(Some("q1"), None), yes_no_q(Some("q2"), None)];
        let answers = vec![Answer::new(AnswerValue::Yes), Answer::new(AnswerValue::No)];
        let stored = build_stored_answers(&questions, &answers, 1);
        assert!(stored.get("q1").is_some());
        assert!(stored.get("q2").is_none());
    }

    #[test]
    fn build_stored_answers_no_store_key() {
        let questions = vec![Question::freeform("name?")];
        let answers = vec![Answer::new(AnswerValue::Text("Alice".into()))];
        let stored = build_stored_answers(&questions, &answers, 1);
        assert!(stored.is_empty());
    }

    // -----------------------------------------------------------------------
    // should_show_question
    // -----------------------------------------------------------------------

    #[test]
    fn show_question_no_condition() {
        let q = Question::freeform("test?");
        assert!(should_show_question(&q, &IndexMap::new(), false));
    }

    #[test]
    fn show_question_condition_true() {
        let q = Question {
            show_if: Some("role == admin".into()),
            ..Question::freeform("test?")
        };
        let mut stored = IndexMap::new();
        stored.insert("role".to_string(), "admin".to_string());
        assert!(should_show_question(&q, &stored, false));
    }

    #[test]
    fn show_question_condition_false() {
        let q = Question {
            show_if: Some("role == admin".into()),
            ..Question::freeform("test?")
        };
        let mut stored = IndexMap::new();
        stored.insert("role".to_string(), "user".to_string());
        assert!(!should_show_question(&q, &stored, false));
    }

    #[test]
    fn show_question_parse_error_returns_true() {
        let q = Question {
            show_if: Some("bad condition".into()),
            ..Question::freeform("test?")
        };
        assert!(should_show_question(&q, &IndexMap::new(), false));
    }

    // -----------------------------------------------------------------------
    // is_finish_triggered
    // -----------------------------------------------------------------------

    #[test]
    fn finish_triggered_with_store() {
        let q = yes_no_q(Some("approved"), Some("no"));
        assert!(is_finish_triggered(&q, &Answer::new(AnswerValue::No)));
        assert!(!is_finish_triggered(&q, &Answer::new(AnswerValue::Yes)));
    }

    #[test]
    fn finish_triggered_without_store() {
        let q = yes_no_q(None, Some("no"));
        assert!(is_finish_triggered(&q, &Answer::new(AnswerValue::No)));
        assert!(!is_finish_triggered(&q, &Answer::new(AnswerValue::Yes)));
    }

    #[test]
    fn finish_not_triggered_when_no_finish_if() {
        let q = yes_no_q(Some("approved"), None);
        assert!(!is_finish_triggered(&q, &Answer::new(AnswerValue::No)));
    }

    #[test]
    fn finish_triggered_single_select_by_label() {
        let q = single_select_q(Some("action"), Some("Reject"));
        assert!(is_finish_triggered(
            &q,
            &Answer::new(AnswerValue::Selected("B".into()))
        ));
        assert!(!is_finish_triggered(
            &q,
            &Answer::new(AnswerValue::Selected("A".into()))
        ));
    }

    #[test]
    fn finish_triggered_single_select_without_store() {
        let q = single_select_q(None, Some("Reject"));
        assert!(is_finish_triggered(
            &q,
            &Answer::new(AnswerValue::Selected("B".into()))
        ));
    }
}
