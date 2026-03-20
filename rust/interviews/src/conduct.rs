//! Conditional interview conductor.
//!
//! When an [`InterviewSpec`] uses `show-if` or `finish-if`, the interview
//! cannot be presented as a flat batch — questions must be evaluated
//! progressively based on prior answers.
//!
//! [`conduct_conditional`] converts all spec questions into a single
//! [`Interview`] and delegates to [`Interviewer::conduct`], which handles
//! condition evaluation. Frontends that override `conduct()` (e.g. the
//! TUI) present the questions as one visual interview, skipping
//! conditional questions inline rather than spawning separate interviews.

use crate::interviewer::{AnswerValue, Interview, InterviewError, Interviewer};
use crate::spec::InterviewSpec;

/// Result of a conditional interview.
///
/// Contains the final [`Interview`] (with all questions, including those
/// that were skipped) and the indices of non-skipped (answered) questions
/// in the interview arrays.
pub struct ConductedInterview {
    /// The interview with all questions and their answers (skipped
    /// questions have [`AnswerValue::Skipped`]).
    pub interview: Interview,

    /// Indices of non-skipped (answered) questions in the interview
    /// arrays. This has fewer elements than `interview.questions` when
    /// questions were skipped. Use this to look up `store` keys in the
    /// spec.
    pub answered_indices: Vec<usize>,
}

/// Conduct a conditional interview as a single batch.
///
/// Converts all spec questions to runtime [`Question`]s (preserving
/// `store`, `show_if`, and `finish_if` fields), builds one [`Interview`],
/// and calls [`Interviewer::conduct`]. The default `conduct()`
/// implementation evaluates conditions progressively; frontends that
/// override `conduct()` (like the TUI) present all questions as a
/// single visual group and skip conditional questions during navigation.
///
/// The returned [`ConductedInterview`] contains the indices of
/// non-skipped (answered) questions so callers can look up `store` keys.
///
/// # Errors
///
/// Returns [`InterviewError`] if the underlying interviewer fails.
pub async fn conduct_conditional(
    spec: &InterviewSpec,
    interviewer: &dyn Interviewer,
    stage: &str,
) -> Result<ConductedInterview, InterviewError> {
    // Convert all spec questions to runtime questions.
    let questions: Vec<_> = spec
        .questions
        .iter()
        .enumerate()
        .map(|(i, qs)| {
            qs.to_question().map_err(|e| {
                InterviewError::BackendFailure(format!("failed to build question {i}: {e}"))
            })
        })
        .collect::<Result<_, _>>()?;

    let mut interview = Interview::batch(questions, stage);
    interview.preamble = spec.preamble.clone();

    // Delegate to the interviewer, which handles show_if/finish_if.
    interviewer.conduct(&mut interview).await?;

    // Build answered_indices for non-skipped questions.
    let answered_indices: Vec<usize> = interview
        .answers
        .iter()
        .enumerate()
        .filter(|(_, a)| !matches!(a.value, AnswerValue::Skipped))
        .map(|(i, _)| i)
        .collect();

    Ok(ConductedInterview {
        interview,
        answered_indices,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interviewer::{
        Answer, AnswerValue, InterviewError, Interviewer, Question, QuestionOption,
        canonical_answer_string,
    };
    use crate::spec::{InterviewSpec, OptionSpec, QuestionSpec, QuestionTypeSpec};
    use async_trait::async_trait;
    use std::sync::Mutex;

    /// A test interviewer that returns answers from a pre-loaded queue.
    ///
    /// Uses the default `conduct()` which handles show_if/finish_if.
    struct ScriptedInterviewer {
        answers: Mutex<Vec<Answer>>,
    }

    impl ScriptedInterviewer {
        fn new(answers: Vec<Answer>) -> Self {
            Self {
                answers: Mutex::new(answers),
            }
        }
    }

    #[async_trait]
    impl Interviewer for ScriptedInterviewer {
        async fn ask(&self, _q: &Question) -> Result<Answer, InterviewError> {
            let mut answers = self.answers.lock().expect("lock answers");
            if answers.is_empty() {
                Err(InterviewError::BackendFailure(
                    "scripted interviewer exhausted".into(),
                ))
            } else {
                Ok(answers.remove(0))
            }
        }
    }

    // -----------------------------------------------------------------------
    // Basic: no conditions, all questions asked
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn all_questions_asked_without_conditions() -> Result<(), InterviewError> {
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![
                QuestionSpec {
                    question: "Name?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::Freeform,
                    options: vec![],
                    default: None,
                    store: Some("name".into()),
                    finish_if: None,
                    show_if: None,
                },
                QuestionSpec {
                    question: "Age?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::Freeform,
                    options: vec![],
                    default: None,
                    store: Some("age".into()),
                    finish_if: None,
                    show_if: None,
                },
            ],
        };

        let interviewer = ScriptedInterviewer::new(vec![
            Answer::new(AnswerValue::Text("Alice".into())),
            Answer::new(AnswerValue::Text("30".into())),
        ]);

        let result = conduct_conditional(&spec, &interviewer, "test").await?;
        // All questions present in the interview (including skipped ones)
        assert_eq!(result.interview.questions.len(), 2);
        assert_eq!(result.interview.answers.len(), 2);
        // answered_indices lists non-skipped question positions
        assert_eq!(result.answered_indices, vec![0, 1]);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // finish-if: interview ends early
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn finish_if_ends_interview_early() -> Result<(), InterviewError> {
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![
                QuestionSpec {
                    question: "Approve?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::YesNo,
                    options: vec![],
                    default: None,
                    store: Some("approved".into()),
                    finish_if: Some("no".into()),
                    show_if: None,
                },
                QuestionSpec {
                    question: "Deploy where?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::Freeform,
                    options: vec![],
                    default: None,
                    store: None,
                    finish_if: None,
                    show_if: None,
                },
            ],
        };

        // Answer "No" → should trigger finish-if, Q2 (index 1) answered as Skipped
        let interviewer = ScriptedInterviewer::new(vec![Answer::new(AnswerValue::No)]);

        let result = conduct_conditional(&spec, &interviewer, "test").await?;
        // All questions in the interview, but Q2 (index 1) is Skipped
        assert_eq!(result.interview.questions.len(), 2);
        assert_eq!(result.interview.answers.len(), 2);
        assert_eq!(result.interview.answers[1].value, AnswerValue::Skipped);
        // Only Q1 (index 0) was actually answered
        assert_eq!(result.answered_indices, vec![0]);
        Ok(())
    }

    #[tokio::test]
    async fn finish_if_does_not_trigger_on_non_match() -> Result<(), InterviewError> {
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![
                QuestionSpec {
                    question: "Approve?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::YesNo,
                    options: vec![],
                    default: None,
                    store: Some("approved".into()),
                    finish_if: Some("no".into()),
                    show_if: None,
                },
                QuestionSpec {
                    question: "Notes?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::Freeform,
                    options: vec![],
                    default: None,
                    store: None,
                    finish_if: None,
                    show_if: None,
                },
            ],
        };

        // Answer "Yes" → finish-if("no") doesn't match, Q2 is asked
        let interviewer = ScriptedInterviewer::new(vec![
            Answer::new(AnswerValue::Yes),
            Answer::new(AnswerValue::Text("looks good".into())),
        ]);

        let result = conduct_conditional(&spec, &interviewer, "test").await?;
        assert_eq!(result.interview.questions.len(), 2);
        assert_eq!(result.answered_indices, vec![0, 1]);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // finish-if without store key
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn finish_if_works_without_store_key() -> Result<(), InterviewError> {
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![
                QuestionSpec {
                    question: "Continue?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::YesNo,
                    options: vec![],
                    default: None,
                    store: None,
                    finish_if: Some("no".into()),
                    show_if: None,
                },
                QuestionSpec {
                    question: "Details?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::Freeform,
                    options: vec![],
                    default: None,
                    store: None,
                    finish_if: None,
                    show_if: None,
                },
            ],
        };

        // Answer "No" → finish-if triggers even without a store key
        let interviewer = ScriptedInterviewer::new(vec![Answer::new(AnswerValue::No)]);

        let result = conduct_conditional(&spec, &interviewer, "test").await?;
        assert_eq!(result.interview.questions.len(), 2);
        assert_eq!(result.interview.answers.len(), 2);
        assert_eq!(result.interview.answers[1].value, AnswerValue::Skipped);
        assert_eq!(result.answered_indices, vec![0]);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // show-if: conditional visibility
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn show_if_skips_question_when_false() -> Result<(), InterviewError> {
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![
                QuestionSpec {
                    question: "Approve?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::YesNo,
                    options: vec![],
                    default: None,
                    store: Some("approved".into()),
                    finish_if: None,
                    show_if: None,
                },
                QuestionSpec {
                    question: "Why not?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::Freeform,
                    options: vec![],
                    default: None,
                    store: Some("reason".into()),
                    finish_if: None,
                    show_if: Some("approved == no".into()),
                },
                QuestionSpec {
                    question: "Deploy target?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::Freeform,
                    options: vec![],
                    default: None,
                    store: Some("target".into()),
                    finish_if: None,
                    show_if: Some("approved == yes".into()),
                },
            ],
        };

        // Answer "Yes" → Q1 (why not?) skipped, Q2 (deploy target?) shown
        let interviewer = ScriptedInterviewer::new(vec![
            Answer::new(AnswerValue::Yes),
            Answer::new(AnswerValue::Text("production".into())),
        ]);

        let result = conduct_conditional(&spec, &interviewer, "test").await?;
        // All 3 questions present; Q1 is Skipped
        assert_eq!(result.interview.questions.len(), 3);
        assert_eq!(result.interview.answers[1].value, AnswerValue::Skipped);
        // Non-skipped indices: Q0 and Q2
        assert_eq!(result.answered_indices, vec![0, 2]);
        assert_eq!(
            result.interview.answers[2].value,
            AnswerValue::Text("production".into())
        );
        Ok(())
    }

    #[tokio::test]
    async fn show_if_shows_question_when_true() -> Result<(), InterviewError> {
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![
                QuestionSpec {
                    question: "Approve?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::YesNo,
                    options: vec![],
                    default: None,
                    store: Some("approved".into()),
                    finish_if: None,
                    show_if: None,
                },
                QuestionSpec {
                    question: "Why not?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::Freeform,
                    options: vec![],
                    default: None,
                    store: Some("reason".into()),
                    finish_if: None,
                    show_if: Some("approved == no".into()),
                },
            ],
        };

        // Answer "No" → Q1 (why not?) is shown
        let interviewer = ScriptedInterviewer::new(vec![
            Answer::new(AnswerValue::No),
            Answer::new(AnswerValue::Text("needs more tests".into())),
        ]);

        let result = conduct_conditional(&spec, &interviewer, "test").await?;
        assert_eq!(result.interview.questions.len(), 2);
        assert_eq!(result.answered_indices, vec![0, 1]);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Combined: show-if + finish-if
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn show_if_and_finish_if_combined() -> Result<(), InterviewError> {
        let spec = InterviewSpec {
            preamble: Some("Review".into()),
            questions: vec![
                QuestionSpec {
                    question: "Approve?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::YesNo,
                    options: vec![],
                    default: None,
                    store: Some("approved".into()),
                    finish_if: None,
                    show_if: None,
                },
                QuestionSpec {
                    question: "Why not?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::Freeform,
                    options: vec![],
                    default: None,
                    store: Some("reason".into()),
                    finish_if: None,
                    show_if: Some("approved == no".into()),
                },
                QuestionSpec {
                    question: "Deploy?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::YesNo,
                    options: vec![],
                    default: None,
                    store: Some("deploy".into()),
                    finish_if: Some("no".into()),
                    show_if: Some("approved == yes".into()),
                },
                QuestionSpec {
                    question: "Target?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::Freeform,
                    options: vec![],
                    default: None,
                    store: Some("target".into()),
                    finish_if: None,
                    show_if: Some("deploy == yes".into()),
                },
            ],
        };

        // Approve=Yes → skip "Why not?", Deploy=No → finish early, skip "Target?"
        let interviewer = ScriptedInterviewer::new(vec![
            Answer::new(AnswerValue::Yes), // Approve
            Answer::new(AnswerValue::No),  // Deploy → triggers finish-if
        ]);

        let result = conduct_conditional(&spec, &interviewer, "test").await?;
        // All 4 questions present; Q1, Q3 are Skipped
        assert_eq!(result.interview.questions.len(), 4);
        // Non-skipped: Q0 (Approve) and Q2 (Deploy)
        assert_eq!(result.answered_indices, vec![0, 2]);
        assert_eq!(result.interview.preamble.as_deref(), Some("Review"));
        Ok(())
    }

    // -----------------------------------------------------------------------
    // finish-if with multi_choice
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn finish_if_multi_choice_by_label() -> Result<(), InterviewError> {
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![
                QuestionSpec {
                    question: "Action?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::SingleSelect,
                    options: vec![
                        OptionSpec {
                            label: "Approve".into(),
                            description: None,
                        },
                        OptionSpec {
                            label: "Reject".into(),
                            description: None,
                        },
                    ],
                    default: None,
                    store: Some("action".into()),
                    finish_if: Some("Reject".into()),
                    show_if: None,
                },
                QuestionSpec {
                    question: "Notes?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::Freeform,
                    options: vec![],
                    default: None,
                    store: None,
                    finish_if: None,
                    show_if: None,
                },
            ],
        };

        // Select "Reject" (key "B") → finish-if triggers, Q1 Skipped
        let interviewer =
            ScriptedInterviewer::new(vec![Answer::new(AnswerValue::Selected("B".into()))]);

        let result = conduct_conditional(&spec, &interviewer, "test").await?;
        assert_eq!(result.interview.questions.len(), 2);
        assert_eq!(result.interview.answers[1].value, AnswerValue::Skipped);
        assert_eq!(result.answered_indices, vec![0]);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // show_if with not-equals
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn show_if_not_equals() -> Result<(), InterviewError> {
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![
                QuestionSpec {
                    question: "Role?".into(),
                    header: None,
                    r#type: QuestionTypeSpec::Freeform,
                    options: vec![],
                    default: None,
                    store: Some("role".into()),
                    finish_if: None,
                    show_if: None,
                },
                QuestionSpec {
                    question: "Explain admin request".into(),
                    header: None,
                    r#type: QuestionTypeSpec::Freeform,
                    options: vec![],
                    default: None,
                    store: None,
                    finish_if: None,
                    show_if: Some("role != admin".into()),
                },
            ],
        };

        // Answer "admin" → Q1 is NOT shown (role != admin is false)
        let interviewer =
            ScriptedInterviewer::new(vec![Answer::new(AnswerValue::Text("admin".into()))]);

        let result = conduct_conditional(&spec, &interviewer, "test").await?;
        // Both questions present; Q1 is Skipped
        assert_eq!(result.interview.questions.len(), 2);
        assert_eq!(result.interview.answers[1].value, AnswerValue::Skipped);
        assert_eq!(result.answered_indices, vec![0]);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // canonical_answer_string used for finish-if comparison
    // -----------------------------------------------------------------------

    #[test]
    fn canonical_yes_no_matches_finish_if() {
        let q = Question::yes_no("Q");
        let yes_str = canonical_answer_string(&AnswerValue::Yes, &q);
        let no_str = canonical_answer_string(&AnswerValue::No, &q);
        assert!("yes".eq_ignore_ascii_case(&yes_str));
        assert!("YES".eq_ignore_ascii_case(&yes_str));
        assert!("no".eq_ignore_ascii_case(&no_str));
        assert!(!"no".eq_ignore_ascii_case(&yes_str));
        assert!(!"yes".eq_ignore_ascii_case(&no_str));
    }

    #[test]
    fn canonical_selected_resolves_to_label() {
        let q = Question::single_select(
            "Q",
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
        );
        let canonical = canonical_answer_string(&AnswerValue::Selected("B".into()), &q);
        assert_eq!(canonical, "Reject");
        assert!("Reject".eq_ignore_ascii_case(&canonical));
    }

    #[test]
    fn canonical_text_is_raw() {
        let q = Question::freeform("Q");
        let canonical = canonical_answer_string(&AnswerValue::Text("anything".into()), &q);
        assert_eq!(canonical, "anything");
    }
}
