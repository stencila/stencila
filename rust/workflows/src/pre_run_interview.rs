//! Construction of the unified pre-run interview.
//!
//! Before executing a workflow pipeline, the runner may present a short
//! interview to the user to gather:
//! 1. A goal (when the workflow has `goal_hint` but no fixed `goal`)
//! 2. A gate-timeout mode (when the pipeline contains human gates)
//! 3. A duration (when the user chooses "Timed" mode)
//!
//! [`build_pre_run_interview`] constructs an [`InterviewSpec`] containing
//! only the questions relevant to the given workflow and CLI configuration.

use stencila_interviews::spec::{InterviewSpec, OptionSpec, QuestionSpec, QuestionTypeSpec};

use crate::WorkflowInstance;

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
            store: Some("pre_run.goal".into()),
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
            store: Some("pre_run.gate_mode".into()),
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
            store: Some("pre_run.gate_duration".into()),
            show_if: Some("pre_run.gate_mode == Timed".into()),
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
}
