//! Progressive interview conductor for conditional specs.
//!
//! When an [`InterviewSpec`] uses `show_if` or `finish_if`, the interview
//! cannot be presented as a flat batch — questions must be evaluated and
//! presented one at a time based on prior answers. This module provides
//! [`conduct_conditional`], which drives that loop.

use indexmap::IndexMap;

use crate::condition::Condition;
use crate::interviewer::{Interview, InterviewError, Interviewer, canonical_answer_string};
use crate::spec::InterviewSpec;

/// Result of a progressive conditional interview.
///
/// Contains the final [`Interview`] (with only the questions that were
/// actually asked) and a mapping from each asked question back to its
/// index in the original [`InterviewSpec::questions`] list.
pub struct ConductedInterview {
    /// The interview containing only asked questions and their answers.
    pub interview: Interview,

    /// For each element in `interview.questions`, the index of the
    /// corresponding [`QuestionSpec`] in the original spec. Parallel
    /// to `interview.questions` / `interview.answers`.
    pub spec_indices: Vec<usize>,
}

/// Conduct a conditional interview progressively, one question at a time.
///
/// Iterates through the spec's questions in order, evaluating `show_if`
/// conditions against previously collected answers. Questions whose
/// conditions are false are skipped. After each answer, `finish_if` is
/// checked — if the answer matches, remaining questions are not presented.
///
/// The returned [`ConductedInterview`] contains only the questions that
/// were actually asked, plus a mapping back to spec indices so callers
/// can look up `store` keys and other spec-level metadata.
///
/// # Errors
///
/// Returns [`InterviewError`] if the underlying interviewer fails.
pub async fn conduct_conditional(
    spec: &InterviewSpec,
    interviewer: &dyn Interviewer,
    stage: &str,
) -> Result<ConductedInterview, InterviewError> {
    let mut interview = Interview::batch(vec![], stage);
    interview.preamble = spec.preamble.clone();

    let mut spec_indices = Vec::new();
    // Accumulated canonical answer strings, keyed by store name.
    let mut stored_answers: IndexMap<String, String> = IndexMap::new();

    for (spec_idx, question_spec) in spec.questions.iter().enumerate() {
        // Evaluate show_if: skip this question if the condition is false.
        if let Some(ref show_if_str) = question_spec.show_if {
            match Condition::parse(show_if_str) {
                Ok(cond) => {
                    if !cond.evaluate(&stored_answers) {
                        continue;
                    }
                }
                Err(e) => {
                    tracing::warn!(
                        question = spec_idx,
                        show_if = show_if_str.as_str(),
                        error = %e,
                        "show_if condition failed to parse; presenting question unconditionally"
                    );
                }
            }
        }

        // Convert spec question to runtime question.
        let question = question_spec.to_question().map_err(|e| {
            InterviewError::BackendFailure(format!("failed to build question {spec_idx}: {e}"))
        })?;

        // Ask the single question.
        let answer = interviewer.ask(&question).await?;

        // Compute canonical answer string for condition evaluation.
        let canonical = canonical_answer_string(&answer.value, &question);

        // Store under the question's store key if present.
        if let Some(ref store) = question_spec.store {
            stored_answers.insert(store.clone(), canonical.clone());
        }

        // Record this question and answer.
        interview.questions.push(question);
        interview.answers.push(answer);
        spec_indices.push(spec_idx);

        // Check finish_if: if the canonical answer matches, end the interview.
        if let Some(ref finish_if_str) = question_spec.finish_if
            && finish_if_str.trim().eq_ignore_ascii_case(canonical.trim())
        {
            break;
        }
    }

    Ok(ConductedInterview {
        interview,
        spec_indices,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interviewer::{Answer, AnswerValue, InterviewError, Interviewer, Question};
    use crate::spec::{InterviewSpec, OptionSpec, QuestionSpec, QuestionTypeSpec};
    use async_trait::async_trait;
    use std::sync::Mutex;

    /// A test interviewer that returns answers from a pre-loaded queue.
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
                    question_type: QuestionTypeSpec::Freeform,
                    options: vec![],
                    default: None,
                    store: Some("name".into()),
                    finish_if: None,
                    show_if: None,
                },
                QuestionSpec {
                    question: "Age?".into(),
                    header: None,
                    question_type: QuestionTypeSpec::Freeform,
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
        assert_eq!(result.interview.questions.len(), 2);
        assert_eq!(result.interview.answers.len(), 2);
        assert_eq!(result.spec_indices, vec![0, 1]);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // finish_if: interview ends early
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn finish_if_ends_interview_early() -> Result<(), InterviewError> {
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![
                QuestionSpec {
                    question: "Approve?".into(),
                    header: None,
                    question_type: QuestionTypeSpec::YesNo,
                    options: vec![],
                    default: None,
                    store: Some("approved".into()),
                    finish_if: Some("no".into()),
                    show_if: None,
                },
                QuestionSpec {
                    question: "Deploy where?".into(),
                    header: None,
                    question_type: QuestionTypeSpec::Freeform,
                    options: vec![],
                    default: None,
                    store: None,
                    finish_if: None,
                    show_if: None,
                },
            ],
        };

        // Answer "No" → should trigger finish_if, Q2 never asked
        let interviewer = ScriptedInterviewer::new(vec![Answer::new(AnswerValue::No)]);

        let result = conduct_conditional(&spec, &interviewer, "test").await?;
        assert_eq!(result.interview.questions.len(), 1);
        assert_eq!(result.interview.answers.len(), 1);
        assert_eq!(result.spec_indices, vec![0]);
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
                    question_type: QuestionTypeSpec::YesNo,
                    options: vec![],
                    default: None,
                    store: Some("approved".into()),
                    finish_if: Some("no".into()),
                    show_if: None,
                },
                QuestionSpec {
                    question: "Notes?".into(),
                    header: None,
                    question_type: QuestionTypeSpec::Freeform,
                    options: vec![],
                    default: None,
                    store: None,
                    finish_if: None,
                    show_if: None,
                },
            ],
        };

        // Answer "Yes" → finish_if("no") doesn't match, Q2 is asked
        let interviewer = ScriptedInterviewer::new(vec![
            Answer::new(AnswerValue::Yes),
            Answer::new(AnswerValue::Text("looks good".into())),
        ]);

        let result = conduct_conditional(&spec, &interviewer, "test").await?;
        assert_eq!(result.interview.questions.len(), 2);
        assert_eq!(result.spec_indices, vec![0, 1]);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // show_if: conditional visibility
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn show_if_skips_question_when_false() -> Result<(), InterviewError> {
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![
                QuestionSpec {
                    question: "Approve?".into(),
                    header: None,
                    question_type: QuestionTypeSpec::YesNo,
                    options: vec![],
                    default: None,
                    store: Some("approved".into()),
                    finish_if: None,
                    show_if: None,
                },
                QuestionSpec {
                    question: "Why not?".into(),
                    header: None,
                    question_type: QuestionTypeSpec::Freeform,
                    options: vec![],
                    default: None,
                    store: Some("reason".into()),
                    finish_if: None,
                    show_if: Some("approved == no".into()),
                },
                QuestionSpec {
                    question: "Deploy target?".into(),
                    header: None,
                    question_type: QuestionTypeSpec::Freeform,
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
        assert_eq!(result.interview.questions.len(), 2);
        assert_eq!(result.spec_indices, vec![0, 2]); // spec Q0 and Q2 asked
        assert_eq!(
            result.interview.answers[1].value,
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
                    question_type: QuestionTypeSpec::YesNo,
                    options: vec![],
                    default: None,
                    store: Some("approved".into()),
                    finish_if: None,
                    show_if: None,
                },
                QuestionSpec {
                    question: "Why not?".into(),
                    header: None,
                    question_type: QuestionTypeSpec::Freeform,
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
        assert_eq!(result.spec_indices, vec![0, 1]);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Combined: show_if + finish_if
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn show_if_and_finish_if_combined() -> Result<(), InterviewError> {
        let spec = InterviewSpec {
            preamble: Some("Review".into()),
            questions: vec![
                QuestionSpec {
                    question: "Approve?".into(),
                    header: None,
                    question_type: QuestionTypeSpec::YesNo,
                    options: vec![],
                    default: None,
                    store: Some("approved".into()),
                    finish_if: None,
                    show_if: None,
                },
                QuestionSpec {
                    question: "Why not?".into(),
                    header: None,
                    question_type: QuestionTypeSpec::Freeform,
                    options: vec![],
                    default: None,
                    store: Some("reason".into()),
                    finish_if: None,
                    show_if: Some("approved == no".into()),
                },
                QuestionSpec {
                    question: "Deploy?".into(),
                    header: None,
                    question_type: QuestionTypeSpec::YesNo,
                    options: vec![],
                    default: None,
                    store: Some("deploy".into()),
                    finish_if: Some("no".into()),
                    show_if: Some("approved == yes".into()),
                },
                QuestionSpec {
                    question: "Target?".into(),
                    header: None,
                    question_type: QuestionTypeSpec::Freeform,
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
            Answer::new(AnswerValue::No),  // Deploy → triggers finish_if
        ]);

        let result = conduct_conditional(&spec, &interviewer, "test").await?;
        assert_eq!(result.interview.questions.len(), 2);
        assert_eq!(result.spec_indices, vec![0, 2]); // Q0 (Approve) and Q2 (Deploy)
        assert_eq!(result.interview.preamble.as_deref(), Some("Review"));
        Ok(())
    }

    // -----------------------------------------------------------------------
    // finish_if with multiple_choice
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn finish_if_multiple_choice_by_label() -> Result<(), InterviewError> {
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![
                QuestionSpec {
                    question: "Action?".into(),
                    header: None,
                    question_type: QuestionTypeSpec::MultipleChoice,
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
                    question_type: QuestionTypeSpec::Freeform,
                    options: vec![],
                    default: None,
                    store: None,
                    finish_if: None,
                    show_if: None,
                },
            ],
        };

        // Select "Reject" (key "B") → finish_if triggers
        let interviewer =
            ScriptedInterviewer::new(vec![Answer::new(AnswerValue::Selected("B".into()))]);

        let result = conduct_conditional(&spec, &interviewer, "test").await?;
        assert_eq!(result.interview.questions.len(), 1);
        assert_eq!(result.spec_indices, vec![0]);
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
                    question_type: QuestionTypeSpec::Freeform,
                    options: vec![],
                    default: None,
                    store: Some("role".into()),
                    finish_if: None,
                    show_if: None,
                },
                QuestionSpec {
                    question: "Explain admin request".into(),
                    header: None,
                    question_type: QuestionTypeSpec::Freeform,
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
        assert_eq!(result.interview.questions.len(), 1);
        assert_eq!(result.spec_indices, vec![0]);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // canonical_answer_string used for finish_if comparison
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
        use crate::interviewer::QuestionOption;
        let q = Question::multiple_choice(
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
