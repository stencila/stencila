//! Construction and answer extraction for the unified pre-run interview.
//!
//! Before executing a workflow pipeline, the runner may present a short
//! interview to the user to gather:
//! 1. A goal (when the workflow has `goal_hint` but no fixed `goal`)
//! 2. A gate-timeout mode (when the pipeline contains human gates)
//! 3. A duration (when the user chooses "Timed" mode)
//!
//! [`build_pre_run_interview`] constructs an [`InterviewSpec`] containing
//! only the questions relevant to the given workflow and CLI configuration.
//!
//! [`extract_pre_run_answers`] converts the interview answers back into
//! domain types: an optional goal string and a [`GateTimeoutConfig`].

use eyre::{Result, bail};
use stencila_interviews::conduct::{ConductedInterview, conduct_conditional};
use stencila_interviews::interviewer::{Interviewer, canonical_answer_string};
use stencila_interviews::spec::{InterviewSpec, OptionSpec, QuestionSpec, QuestionTypeSpec};

use crate::GateTimeoutConfig;
use crate::WorkflowInstance;

/// Store key for the user-provided goal answer.
const STORE_GOAL: &str = "pre_run.goal";
/// Store key for the gate-mode selection (Interactive / Auto-approve / Timed).
const STORE_GATE_MODE: &str = "pre_run.gate_mode";
/// Store key for the duration when gate mode is Timed.
const STORE_GATE_DURATION: &str = "pre_run.gate_duration";

/// Build an [`InterviewSpec`] for the pre-run interview, or `None` if all
/// questions would be skipped.
///
/// # Arguments
///
/// * `workflow` — the loaded workflow instance
/// * `has_cli_goal` — `true` when the user provided `--goal` on the CLI
/// * `has_cli_gate_config` — `true` when the user provided `--auto-approve`
///   or `--auto-approve-after` on the CLI
pub fn build_pre_run_interview(
    workflow: &WorkflowInstance,
    has_cli_goal: bool,
    has_cli_gate_config: bool,
) -> Option<InterviewSpec> {
    let mut questions = Vec::new();

    // Q1: Goal question — shown when workflow has goal_hint, no fixed goal,
    // and no --goal CLI flag.
    if let Some(hint) = &workflow.goal_hint
        && workflow.goal.is_none()
        && !has_cli_goal
    {
        questions.push(QuestionSpec {
            question: hint.clone(),
            r#type: QuestionTypeSpec::Freeform,
            store: Some(STORE_GOAL.into()),
            ..QuestionSpec::default()
        });
    }

    // Q2 & Q3: Gate questions — shown when pipeline has human gates and no
    // --auto-approve / --auto-approve-after CLI flag.
    let has_gates = workflow.human_gate_nodes().is_ok_and(|g| !g.is_empty());

    if has_gates && !has_cli_gate_config {
        questions.push(QuestionSpec {
            question: "How should human gates be handled?".into(),
            r#type: QuestionTypeSpec::SingleSelect,
            store: Some(STORE_GATE_MODE.into()),
            options: ["Interactive", "Auto-approve", "Timed"]
                .into_iter()
                .map(|label| OptionSpec {
                    label: label.into(),
                    description: None,
                })
                .collect(),
            ..QuestionSpec::default()
        });

        questions.push(QuestionSpec {
            question: "How long before auto-approving?".into(),
            r#type: QuestionTypeSpec::Freeform,
            store: Some(STORE_GATE_DURATION.into()),
            show_if: Some(format!("{STORE_GATE_MODE} == Timed")),
            ..QuestionSpec::default()
        });
    }

    if questions.is_empty() {
        None
    } else {
        Some(InterviewSpec {
            preamble: None,
            questions,
        })
    }
}

/// Result of extracting pre-run interview answers.
#[derive(Debug)]
pub struct PreRunAnswers {
    /// The user-provided goal, if any.
    pub goal: Option<String>,
    /// The gate timeout configuration derived from the user's choices.
    pub gate_timeout: GateTimeoutConfig,
}

/// Extract domain values from a conducted pre-run interview.
///
/// Maps the interview answers back to the original spec's `store` keys
/// and converts them into a goal string and [`GateTimeoutConfig`].
///
/// The `spec` must be the same [`InterviewSpec`] that was used to conduct
/// the interview (needed to look up `store` keys).
pub fn extract_pre_run_answers(
    spec: &InterviewSpec,
    conducted: &ConductedInterview,
) -> PreRunAnswers {
    // Build a map from store key → canonical answer string.
    let store_values: std::collections::HashMap<&str, String> = conducted
        .spec_indices
        .iter()
        .enumerate()
        .filter_map(|(i, &spec_idx)| {
            let store = spec.questions[spec_idx].store.as_deref()?;
            let canonical = canonical_answer_string(
                &conducted.interview.answers[i].value,
                &conducted.interview.questions[i],
            );
            Some((store, canonical))
        })
        .collect();

    // Extract goal — canonical_answer_string already returns the raw text
    // for freeform answers, so a simple map lookup suffices.
    let goal = store_values.get(STORE_GOAL).cloned();

    // Extract gate timeout config.
    let gate_timeout = match store_values.get(STORE_GATE_MODE).map(String::as_str) {
        Some("Auto-approve") => GateTimeoutConfig::AutoApprove,
        Some("Timed") => {
            let seconds = store_values
                .get(STORE_GATE_DURATION)
                .map(|d| parse_duration(d))
                .unwrap_or(0.0);
            GateTimeoutConfig::Timed { seconds }
        }
        _ => GateTimeoutConfig::Interactive,
    };

    PreRunAnswers { goal, gate_timeout }
}

/// Parse a human-friendly duration string into seconds.
///
/// Supports formats like `"30s"`, `"5m"`, `"2h"`, or bare numbers
/// (interpreted as seconds).
fn parse_duration(s: &str) -> f64 {
    let s = s.trim();

    let (num, multiplier) = if let Some(n) = s.strip_suffix('s') {
        (n, 1.0)
    } else if let Some(n) = s.strip_suffix('m') {
        (n, 60.0)
    } else if let Some(n) = s.strip_suffix('h') {
        (n, 3600.0)
    } else {
        (s, 1.0)
    };

    num.trim().parse::<f64>().unwrap_or(0.0) * multiplier
}

/// Conduct the pre-run interview end-to-end and apply the results.
///
/// This is the top-level orchestrator called by `Run::run()` between
/// validation and `run_workflow_with_options`. It:
///
/// 1. Calls [`build_pre_run_interview`] to construct the spec
/// 2. If the spec is `None` (all questions suppressed), returns `None`
/// 3. Conducts the interview via [`conduct_conditional`]
/// 4. Rejects empty goal input (returns an error)
/// 5. Extracts answers via [`extract_pre_run_answers`]
/// 6. Returns `Some(PreRunAnswers)` with the goal and gate timeout
///
/// The caller is responsible for applying the answers to the workflow
/// and run options.
///
/// # Arguments
///
/// * `workflow` — the loaded workflow instance
/// * `has_cli_goal` — `true` when the user provided `--goal` on the CLI
/// * `has_cli_gate_config` — `true` when `--auto-approve` or
///   `--auto-approve-after` was provided
/// * `interviewer` — the interviewer to use for conducting the interview
pub async fn conduct_pre_run_interview(
    workflow: &WorkflowInstance,
    has_cli_goal: bool,
    has_cli_gate_config: bool,
    interviewer: &dyn Interviewer,
) -> Result<Option<PreRunAnswers>> {
    let spec = match build_pre_run_interview(workflow, has_cli_goal, has_cli_gate_config) {
        Some(spec) => spec,
        None => return Ok(None),
    };

    let conducted = conduct_conditional(&spec, interviewer, "pre-run")
        .await
        .map_err(|e| eyre::eyre!("{e}"))?;

    // Reject empty/whitespace-only goal answers.
    let answers = extract_pre_run_answers(&spec, &conducted);
    if let Some(ref goal) = answers.goal
        && goal.trim().is_empty()
    {
        bail!("goal cannot be empty");
    }

    Ok(Some(answers))
}

#[cfg(test)]
mod tests {
    use super::*;
    use stencila_interviews::spec::QuestionTypeSpec;

    /// Helper: create a minimal `WorkflowInstance` by writing a WORKFLOW.md
    /// to a tempdir and loading it. The `goal`, `goal_hint`, and pipeline
    /// DOT are configurable.
    async fn make_workflow(
        goal: Option<&str>,
        goal_hint: Option<&str>,
        has_human_gates: bool,
    ) -> (tempfile::TempDir, WorkflowInstance) {
        let tmp = tempfile::tempdir().expect("tempdir");
        let wf_dir = tmp.path().join(".stencila/workflows/test-pre-run");
        std::fs::create_dir_all(&wf_dir).expect("create workflow dir");

        let goal_line = goal.map(|g| format!("goal: {g}\n")).unwrap_or_default();
        let hint_line = goal_hint
            .map(|h| format!("goal_hint: {h}\n"))
            .unwrap_or_default();

        let pipeline = if has_human_gates {
            r#"```dot
digraph test {
    Start [shape=Mdiamond]
    Exit  [shape=Msquare]
    Work  [agent="code-engineer", prompt="Do something"]
    Review [shape=hexagon, label="Approve?"]
    Start -> Work -> Review
    Review -> Exit [label="yes"]
    Review -> Work [label="no"]
}
```"#
        } else {
            r#"```dot
digraph test {
    Start [shape=Mdiamond]
    Exit  [shape=Msquare]
    Work  [agent="code-engineer", prompt="Do something"]
    Start -> Work -> Exit
}
```"#
        };

        let content = format!(
            "---\nname: test-pre-run\ndescription: test\n{goal_line}{hint_line}---\n\n{pipeline}\n"
        );

        std::fs::write(wf_dir.join("WORKFLOW.md"), content).expect("write");
        let instance = crate::definition::load_workflow(&wf_dir.join("WORKFLOW.md"))
            .await
            .expect("load workflow");
        (tmp, instance)
    }

    // -----------------------------------------------------------------------
    // AC-1: Workflow with goal_hint + human gates → 3 questions
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn goal_hint_and_gates_returns_three_questions() {
        let (_tmp, wf) = make_workflow(None, Some("What do you want to build?"), true).await;

        let spec = build_pre_run_interview(&wf, false, false);

        assert!(
            spec.is_some(),
            "should return Some when both goal_hint and gates exist"
        );
        let spec = spec.expect("spec");
        assert_eq!(
            spec.questions.len(),
            3,
            "expected 3 questions (goal, gate mode, duration), got {}",
            spec.questions.len()
        );

        // Q1: goal question
        let q1 = &spec.questions[0];
        assert_eq!(q1.r#type, QuestionTypeSpec::Freeform);
        assert_eq!(q1.store.as_deref(), Some("pre_run.goal"));

        // Q2: gate mode question
        let q2 = &spec.questions[1];
        assert_eq!(q2.r#type, QuestionTypeSpec::SingleSelect);
        assert_eq!(q2.store.as_deref(), Some("pre_run.gate_mode"));
        let labels: Vec<&str> = q2.options.iter().map(|o| o.label.as_str()).collect();
        assert!(labels.contains(&"Interactive"), "options: {labels:?}");
        assert!(labels.contains(&"Auto-approve"), "options: {labels:?}");
        assert!(labels.contains(&"Timed"), "options: {labels:?}");

        // Q3: duration question
        let q3 = &spec.questions[2];
        assert_eq!(q3.r#type, QuestionTypeSpec::Freeform);
        assert_eq!(q3.store.as_deref(), Some("pre_run.gate_duration"));
    }

    // -----------------------------------------------------------------------
    // AC-2: Workflow with goal_hint only (no gates) → 1 question
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn goal_hint_only_returns_one_question() {
        let (_tmp, wf) = make_workflow(None, Some("Describe your goal"), false).await;

        let spec = build_pre_run_interview(&wf, false, false);

        assert!(spec.is_some(), "should return Some when goal_hint exists");
        let spec = spec.expect("spec");
        assert_eq!(
            spec.questions.len(),
            1,
            "expected 1 question (goal only), got {}",
            spec.questions.len()
        );
        assert_eq!(spec.questions[0].store.as_deref(), Some("pre_run.goal"));
    }

    // -----------------------------------------------------------------------
    // AC-3: Workflow with gates only (no goal_hint) → 2 questions
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn gates_only_returns_two_questions() {
        let (_tmp, wf) = make_workflow(None, None, true).await;

        let spec = build_pre_run_interview(&wf, false, false);

        assert!(spec.is_some(), "should return Some when gates exist");
        let spec = spec.expect("spec");
        assert_eq!(
            spec.questions.len(),
            2,
            "expected 2 questions (gate mode + duration), got {}",
            spec.questions.len()
        );
        assert_eq!(
            spec.questions[0].store.as_deref(),
            Some("pre_run.gate_mode")
        );
        assert_eq!(
            spec.questions[1].store.as_deref(),
            Some("pre_run.gate_duration")
        );
    }

    // -----------------------------------------------------------------------
    // AC-4: Neither goal_hint nor gates → returns None
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn no_hint_no_gates_returns_none() {
        let (_tmp, wf) = make_workflow(None, None, false).await;

        let spec = build_pre_run_interview(&wf, false, false);

        assert!(spec.is_none(), "should return None when nothing to ask");
    }

    // -----------------------------------------------------------------------
    // AC-5: CLI --goal provided → goal question omitted
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn cli_goal_omits_goal_question() {
        let (_tmp, wf) = make_workflow(None, Some("Describe your goal"), true).await;

        let spec = build_pre_run_interview(&wf, true, false);

        assert!(
            spec.is_some(),
            "should still return Some for gate questions"
        );
        let spec = spec.expect("spec");
        // Only gate mode + duration questions should remain
        assert_eq!(
            spec.questions.len(),
            2,
            "expected 2 questions (gate mode + duration), goal should be omitted, got {}",
            spec.questions.len()
        );
        let stores: Vec<Option<&str>> = spec.questions.iter().map(|q| q.store.as_deref()).collect();
        assert!(
            !stores.contains(&Some("pre_run.goal")),
            "goal question should be omitted when --goal is provided, stores: {stores:?}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-6: CLI --auto-approve provided → gate questions omitted
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn cli_auto_approve_omits_gate_questions() {
        let (_tmp, wf) = make_workflow(None, Some("Describe your goal"), true).await;

        let spec = build_pre_run_interview(&wf, false, true);

        assert!(spec.is_some(), "should still return Some for goal question");
        let spec = spec.expect("spec");
        assert_eq!(
            spec.questions.len(),
            1,
            "expected 1 question (goal only), gate questions should be omitted, got {}",
            spec.questions.len()
        );
        assert_eq!(spec.questions[0].store.as_deref(), Some("pre_run.goal"));
    }

    // -----------------------------------------------------------------------
    // AC-7: Both CLI flags provided → returns None
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn both_cli_flags_returns_none() {
        let (_tmp, wf) = make_workflow(None, Some("Describe your goal"), true).await;

        let spec = build_pre_run_interview(&wf, true, true);

        assert!(
            spec.is_none(),
            "should return None when both CLI flags are provided"
        );
    }

    // -----------------------------------------------------------------------
    // AC-8: Workflow with goal set directly (not just goal_hint) → goal
    //       question skipped
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn fixed_goal_skips_goal_question() {
        let (_tmp, wf) =
            make_workflow(Some("Build a widget"), Some("Describe your goal"), true).await;

        let spec = build_pre_run_interview(&wf, false, false);

        assert!(spec.is_some(), "should return Some for gate questions");
        let spec = spec.expect("spec");
        // Only gate mode + duration questions should remain
        assert_eq!(
            spec.questions.len(),
            2,
            "expected 2 questions (gate mode + duration), goal should be skipped when workflow has a fixed goal, got {}",
            spec.questions.len()
        );
        let stores: Vec<Option<&str>> = spec.questions.iter().map(|q| q.store.as_deref()).collect();
        assert!(
            !stores.contains(&Some("pre_run.goal")),
            "goal question should be skipped when workflow has a fixed goal, stores: {stores:?}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-9: show_if on duration question references pre_run.gate_mode == Timed
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn duration_question_has_correct_show_if() {
        let (_tmp, wf) = make_workflow(None, None, true).await;

        let spec = build_pre_run_interview(&wf, false, false);

        let spec = spec.expect("spec should be Some for workflow with gates");
        // Find the duration question
        let duration_q = spec
            .questions
            .iter()
            .find(|q| q.store.as_deref() == Some("pre_run.gate_duration"))
            .expect("duration question should exist");

        assert_eq!(
            duration_q.show_if.as_deref(),
            Some("pre_run.gate_mode == Timed"),
            "duration question should have show_if referencing pre_run.gate_mode == Timed"
        );
    }

    // -----------------------------------------------------------------------
    // AC-5 (edge case): CLI --goal provided but no gates → None
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn cli_goal_no_gates_returns_none() {
        let (_tmp, wf) = make_workflow(None, Some("Describe your goal"), false).await;

        let spec = build_pre_run_interview(&wf, true, false);

        assert!(
            spec.is_none(),
            "should return None when --goal suppresses the only question"
        );
    }

    // -----------------------------------------------------------------------
    // AC-6 (edge case): CLI --auto-approve but no goal_hint → None
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn cli_auto_approve_no_hint_returns_none() {
        let (_tmp, wf) = make_workflow(None, None, true).await;

        let spec = build_pre_run_interview(&wf, false, true);

        assert!(
            spec.is_none(),
            "should return None when --auto-approve suppresses the only questions"
        );
    }

    // -----------------------------------------------------------------------
    // Spec validation: the returned spec should pass validate()
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn returned_spec_passes_validation() {
        let (_tmp, wf) = make_workflow(None, Some("Describe your goal"), true).await;

        let spec = build_pre_run_interview(&wf, false, false);
        let spec = spec.expect("spec");

        let validation = spec.validate();
        assert!(
            validation.is_ok(),
            "returned InterviewSpec should pass validation: {:?}",
            validation.err()
        );
    }

    // ===================================================================
    // extract_pre_run_answers tests (Phase 5 / Slice 2)
    // ===================================================================

    use stencila_interviews::conduct::conduct_conditional;
    use stencila_interviews::interviewer::{Answer, AnswerValue};
    use stencila_interviews::interviewers::QueueInterviewer;

    /// Helper: build a 3-question pre-run spec (goal + gate mode + duration)
    /// and conduct it with the provided queue answers.
    async fn conduct_full_pre_run(answers: Vec<Answer>) -> (InterviewSpec, ConductedInterview) {
        // Build a spec with all 3 questions (goal + gate mode + duration).
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![
                QuestionSpec {
                    question: "What do you want to build?".into(),
                    r#type: QuestionTypeSpec::Freeform,
                    store: Some("pre_run.goal".into()),
                    ..QuestionSpec::default()
                },
                QuestionSpec {
                    question: "How should human gates be handled?".into(),
                    r#type: QuestionTypeSpec::SingleSelect,
                    store: Some("pre_run.gate_mode".into()),
                    options: ["Interactive", "Auto-approve", "Timed"]
                        .into_iter()
                        .map(|label| OptionSpec {
                            label: label.into(),
                            description: None,
                        })
                        .collect(),
                    ..QuestionSpec::default()
                },
                QuestionSpec {
                    question: "How long before auto-approving?".into(),
                    r#type: QuestionTypeSpec::Freeform,
                    store: Some("pre_run.gate_duration".into()),
                    show_if: Some("pre_run.gate_mode == Timed".into()),
                    ..QuestionSpec::default()
                },
            ],
        };

        let interviewer = QueueInterviewer::new(answers);
        let conducted = conduct_conditional(&spec, &interviewer, "pre-run")
            .await
            .expect("interview should not fail");
        (spec, conducted)
    }

    // -----------------------------------------------------------------------
    // AC-1: goal "Build X" + "Auto-approve" → goal="Build X",
    //       GateTimeoutConfig::AutoApprove
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn extract_goal_and_auto_approve() {
        // Q1: freeform goal → "Build X"
        // Q2: single-select gate mode → select "B" (Auto-approve, 0-indexed: A=Interactive, B=Auto-approve, C=Timed)
        // Q3: duration — skipped because gate_mode != Timed (show_if condition)
        let (spec, conducted) = conduct_full_pre_run(vec![
            Answer::new(AnswerValue::Text("Build X".into())),
            Answer::new(AnswerValue::Selected("B".into())), // "Auto-approve"
        ])
        .await;

        let result = extract_pre_run_answers(&spec, &conducted);

        assert_eq!(
            result.goal.as_deref(),
            Some("Build X"),
            "goal should be extracted from pre_run.goal answer"
        );
        assert!(
            matches!(result.gate_timeout, GateTimeoutConfig::AutoApprove),
            "gate_timeout should be AutoApprove when user selects 'Auto-approve', got {:?}",
            result.gate_timeout
        );
    }

    // -----------------------------------------------------------------------
    // AC-2: goal + "Timed" with "30s" → GateTimeoutConfig::Timed { seconds: 30.0 }
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn extract_goal_and_timed_30s() {
        // Q1: freeform goal → "Build Y"
        // Q2: single-select gate mode → "C" (Timed)
        // Q3: freeform duration → "30s" (shown because gate_mode == Timed)
        let (spec, conducted) = conduct_full_pre_run(vec![
            Answer::new(AnswerValue::Text("Build Y".into())),
            Answer::new(AnswerValue::Selected("C".into())), // "Timed"
            Answer::new(AnswerValue::Text("30s".into())),
        ])
        .await;

        let result = extract_pre_run_answers(&spec, &conducted);

        assert_eq!(result.goal.as_deref(), Some("Build Y"));
        match result.gate_timeout {
            GateTimeoutConfig::Timed { seconds } => {
                assert!(
                    (seconds - 30.0).abs() < f64::EPSILON,
                    "expected 30.0 seconds, got {seconds}"
                );
            }
            other => panic!("gate_timeout should be Timed {{ seconds: 30.0 }}, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // AC-3: goal + "Interactive" → GateTimeoutConfig::Interactive
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn extract_goal_and_interactive() {
        // Q1: freeform goal → "Build Z"
        // Q2: single-select gate mode → "A" (Interactive)
        // Q3: duration — skipped because gate_mode != Timed
        let (spec, conducted) = conduct_full_pre_run(vec![
            Answer::new(AnswerValue::Text("Build Z".into())),
            Answer::new(AnswerValue::Selected("A".into())), // "Interactive"
        ])
        .await;

        let result = extract_pre_run_answers(&spec, &conducted);

        assert_eq!(result.goal.as_deref(), Some("Build Z"));
        assert!(
            matches!(result.gate_timeout, GateTimeoutConfig::Interactive),
            "gate_timeout should be Interactive when user selects 'Interactive', got {:?}",
            result.gate_timeout
        );
    }

    // -----------------------------------------------------------------------
    // AC-5: Duration parsing — "5m" → 300.0 seconds
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn extract_timed_5m() {
        let (spec, conducted) = conduct_full_pre_run(vec![
            Answer::new(AnswerValue::Text("Build it".into())),
            Answer::new(AnswerValue::Selected("C".into())), // "Timed"
            Answer::new(AnswerValue::Text("5m".into())),
        ])
        .await;

        let result = extract_pre_run_answers(&spec, &conducted);

        match result.gate_timeout {
            GateTimeoutConfig::Timed { seconds } => {
                assert!(
                    (seconds - 300.0).abs() < f64::EPSILON,
                    "expected 300.0 seconds for '5m', got {seconds}"
                );
            }
            other => panic!("gate_timeout should be Timed {{ seconds: 300.0 }}, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // AC-5: Duration parsing — "2h" → 7200.0 seconds
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn extract_timed_2h() {
        let (spec, conducted) = conduct_full_pre_run(vec![
            Answer::new(AnswerValue::Text("Build it".into())),
            Answer::new(AnswerValue::Selected("C".into())), // "Timed"
            Answer::new(AnswerValue::Text("2h".into())),
        ])
        .await;

        let result = extract_pre_run_answers(&spec, &conducted);

        match result.gate_timeout {
            GateTimeoutConfig::Timed { seconds } => {
                assert!(
                    (seconds - 7200.0).abs() < f64::EPSILON,
                    "expected 7200.0 seconds for '2h', got {seconds}"
                );
            }
            other => panic!("gate_timeout should be Timed {{ seconds: 7200.0 }}, got {other:?}"),
        }
    }

    // -----------------------------------------------------------------------
    // Edge case: only gate questions (no goal in spec)
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn extract_without_goal_question() {
        // Spec with only gate mode + duration (no goal question).
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![
                QuestionSpec {
                    question: "How should human gates be handled?".into(),
                    r#type: QuestionTypeSpec::SingleSelect,
                    store: Some("pre_run.gate_mode".into()),
                    options: ["Interactive", "Auto-approve", "Timed"]
                        .into_iter()
                        .map(|label| OptionSpec {
                            label: label.into(),
                            description: None,
                        })
                        .collect(),
                    ..QuestionSpec::default()
                },
                QuestionSpec {
                    question: "How long before auto-approving?".into(),
                    r#type: QuestionTypeSpec::Freeform,
                    store: Some("pre_run.gate_duration".into()),
                    show_if: Some("pre_run.gate_mode == Timed".into()),
                    ..QuestionSpec::default()
                },
            ],
        };

        let interviewer = QueueInterviewer::new(vec![
            Answer::new(AnswerValue::Selected("B".into())), // "Auto-approve"
        ]);
        let conducted = conduct_conditional(&spec, &interviewer, "pre-run")
            .await
            .expect("interview should not fail");

        let result = extract_pre_run_answers(&spec, &conducted);

        assert!(
            result.goal.is_none(),
            "goal should be None when no goal question was in the spec"
        );
        assert!(
            matches!(result.gate_timeout, GateTimeoutConfig::AutoApprove),
            "gate_timeout should be AutoApprove, got {:?}",
            result.gate_timeout
        );
    }

    // -----------------------------------------------------------------------
    // Edge case: only goal question (no gate questions)
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn extract_with_only_goal_question() {
        let spec = InterviewSpec {
            preamble: None,
            questions: vec![QuestionSpec {
                question: "What do you want to build?".into(),
                r#type: QuestionTypeSpec::Freeform,
                store: Some("pre_run.goal".into()),
                ..QuestionSpec::default()
            }],
        };

        let interviewer =
            QueueInterviewer::new(vec![Answer::new(AnswerValue::Text("A widget".into()))]);
        let conducted = conduct_conditional(&spec, &interviewer, "pre-run")
            .await
            .expect("interview should not fail");

        let result = extract_pre_run_answers(&spec, &conducted);

        assert_eq!(
            result.goal.as_deref(),
            Some("A widget"),
            "goal should be extracted from the sole question"
        );
        assert!(
            matches!(result.gate_timeout, GateTimeoutConfig::Interactive),
            "gate_timeout should default to Interactive when no gate questions exist, got {:?}",
            result.gate_timeout
        );
    }

    // ===================================================================
    // conduct_pre_run_interview end-to-end tests (Phase 5 / Slice 3)
    // ===================================================================

    // -----------------------------------------------------------------------
    // AC-1 / AC-6: Full flow — goal_hint workflow, QueueInterviewer provides
    //              goal and gate mode, answers correctly extracted
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn conduct_goal_hint_workflow_extracts_goal_and_gate_config() {
        let (_tmp, wf) = make_workflow(None, Some("What do you want to build?"), true).await;

        // Queue answers: goal text, then gate mode "Auto-approve" (key "B")
        let interviewer = QueueInterviewer::new(vec![
            Answer::new(AnswerValue::Text("Build a REST API".into())),
            Answer::new(AnswerValue::Selected("B".into())), // "Auto-approve"
        ]);

        let result = conduct_pre_run_interview(&wf, false, false, &interviewer)
            .await
            .expect("should not error");

        assert!(
            result.is_some(),
            "should return Some(PreRunAnswers) when interview was conducted"
        );
        let answers = result.expect("answers");

        assert_eq!(
            answers.goal.as_deref(),
            Some("Build a REST API"),
            "goal should be extracted from interview answers"
        );
        assert!(
            matches!(answers.gate_timeout, GateTimeoutConfig::AutoApprove),
            "gate_timeout should be AutoApprove, got {:?}",
            answers.gate_timeout
        );
    }

    // -----------------------------------------------------------------------
    // AC-3: --goal flag bypass — goal question not shown, only gate questions
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn conduct_with_cli_goal_skips_goal_question() {
        let (_tmp, wf) = make_workflow(None, Some("What do you want to build?"), true).await;

        // Only gate mode answer needed — goal question should be skipped
        let interviewer = QueueInterviewer::new(vec![
            Answer::new(AnswerValue::Selected("A".into())), // "Interactive"
        ]);

        let result = conduct_pre_run_interview(&wf, true, false, &interviewer)
            .await
            .expect("should not error");

        assert!(
            result.is_some(),
            "should return Some when gate questions remain"
        );
        let answers = result.expect("answers");

        assert!(
            answers.goal.is_none(),
            "goal should be None when CLI provides --goal (question skipped)"
        );
        assert!(
            matches!(answers.gate_timeout, GateTimeoutConfig::Interactive),
            "gate_timeout should be Interactive, got {:?}",
            answers.gate_timeout
        );
    }

    // -----------------------------------------------------------------------
    // AC-4: Fixed goal in workflow → goal question skipped, gate questions shown
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn conduct_fixed_goal_skips_goal_question() {
        let (_tmp, wf) =
            make_workflow(Some("Build a widget"), Some("What do you want?"), true).await;

        // Only gate mode answer — goal question skipped because workflow has fixed goal
        let interviewer = QueueInterviewer::new(vec![
            Answer::new(AnswerValue::Selected("B".into())), // "Auto-approve"
        ]);

        let result = conduct_pre_run_interview(&wf, false, false, &interviewer)
            .await
            .expect("should not error");

        assert!(result.is_some(), "should return Some for gate questions");
        let answers = result.expect("answers");

        assert!(
            answers.goal.is_none(),
            "goal should be None when workflow has a fixed goal (question skipped)"
        );
        assert!(
            matches!(answers.gate_timeout, GateTimeoutConfig::AutoApprove),
            "gate_timeout should be AutoApprove, got {:?}",
            answers.gate_timeout
        );
    }

    // -----------------------------------------------------------------------
    // AC-7: All questions suppressed by CLI flags → returns None
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn conduct_all_flags_returns_none() {
        let (_tmp, wf) = make_workflow(None, Some("What do you want?"), true).await;

        // No answers needed — everything suppressed
        let interviewer = QueueInterviewer::new(vec![]);

        let result = conduct_pre_run_interview(&wf, true, true, &interviewer)
            .await
            .expect("should not error");

        assert!(
            result.is_none(),
            "should return None when both CLI flags suppress all questions"
        );
    }

    // -----------------------------------------------------------------------
    // AC-7: No goal_hint, no gates → returns None
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn conduct_no_questions_returns_none() {
        let (_tmp, wf) = make_workflow(None, None, false).await;

        let interviewer = QueueInterviewer::new(vec![]);

        let result = conduct_pre_run_interview(&wf, false, false, &interviewer)
            .await
            .expect("should not error");

        assert!(
            result.is_none(),
            "should return None when workflow has no goal_hint and no gates"
        );
    }

    // -----------------------------------------------------------------------
    // AC-2: Empty goal input is rejected
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn conduct_rejects_empty_goal() {
        let (_tmp, wf) = make_workflow(None, Some("What do you want to build?"), false).await;

        // Provide empty text as the goal answer
        let interviewer =
            QueueInterviewer::new(vec![Answer::new(AnswerValue::Text(String::new()))]);

        let result = conduct_pre_run_interview(&wf, false, false, &interviewer).await;

        assert!(
            result.is_err(),
            "should return an error when goal answer is empty"
        );
        let err_msg = result.expect_err("error").to_string();
        assert!(
            err_msg.to_lowercase().contains("goal") || err_msg.to_lowercase().contains("empty"),
            "error message should mention goal or empty, got: {err_msg}"
        );
    }

    // -----------------------------------------------------------------------
    // AC-2: Whitespace-only goal is also rejected
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn conduct_rejects_whitespace_goal() {
        let (_tmp, wf) = make_workflow(None, Some("What do you want to build?"), false).await;

        let interviewer =
            QueueInterviewer::new(vec![Answer::new(AnswerValue::Text("   \n  ".into()))]);

        let result = conduct_pre_run_interview(&wf, false, false, &interviewer).await;

        assert!(
            result.is_err(),
            "should return an error when goal answer is only whitespace"
        );
    }

    // -----------------------------------------------------------------------
    // AC-6: Timed gate timeout correctly applied through conduct
    // -----------------------------------------------------------------------

    #[tokio::test]
    async fn conduct_timed_gate_timeout_applied() {
        let (_tmp, wf) = make_workflow(None, None, true).await;

        // Gate mode "Timed" (key "C") + duration "45s"
        let interviewer = QueueInterviewer::new(vec![
            Answer::new(AnswerValue::Selected("C".into())), // "Timed"
            Answer::new(AnswerValue::Text("45s".into())),
        ]);

        let result = conduct_pre_run_interview(&wf, false, false, &interviewer)
            .await
            .expect("should not error");

        let answers = result.expect("answers");
        match answers.gate_timeout {
            GateTimeoutConfig::Timed { seconds } => {
                assert!(
                    (seconds - 45.0).abs() < f64::EPSILON,
                    "expected 45.0 seconds, got {seconds}"
                );
            }
            other => panic!("gate_timeout should be Timed {{ seconds: 45.0 }}, got {other:?}"),
        }
    }
}
