//! Tests for human-in-the-loop (§6, §4.6).
//!
//! Covers interviewer types, all four interviewer implementations,
//! the WaitForHuman handler, accelerator key parsing, and end-to-end
//! pipeline execution with a human gate node.

mod common;

use std::sync::Arc;

use stencila_attractor::context::Context;
use stencila_attractor::error::AttractorResult;
use stencila_attractor::graph::{AttrValue, Edge, Graph, Node};
use stencila_attractor::handler::Handler;
use stencila_attractor::handlers::{WaitForHumanHandler, parse_accelerator_key};
use stencila_attractor::interviewer::{
    Answer, AnswerValue, Interviewer, Question, QuestionOption, QuestionType,
};
use stencila_attractor::interviewers::{
    AutoApproveInterviewer, CallbackInterviewer, QueueInterviewer, RecordingInterviewer,
};
use stencila_attractor::types::StageStatus;

use common::make_tempdir;

// ===========================================================================
// §6.2 — Question types
// ===========================================================================

#[test]
fn question_type_display() {
    assert_eq!(QuestionType::YesNo.to_string(), "YES_NO");
    assert_eq!(QuestionType::MultipleChoice.to_string(), "MULTIPLE_CHOICE");
    assert_eq!(QuestionType::Freeform.to_string(), "FREEFORM");
    assert_eq!(QuestionType::Confirmation.to_string(), "CONFIRMATION");
}

#[test]
fn question_yes_no_builder() {
    let q = Question::yes_no("Deploy?", "deploy_stage");
    assert_eq!(q.question_type, QuestionType::YesNo);
    assert_eq!(q.text, "Deploy?");
    assert_eq!(q.stage, "deploy_stage");
    assert!(q.options.is_empty());
    assert!(q.default.is_none());
    assert!(q.timeout_seconds.is_none());
}

#[test]
fn question_confirmation_builder() {
    let q = Question::confirmation("Are you sure?", "confirm_stage");
    assert_eq!(q.question_type, QuestionType::Confirmation);
    assert_eq!(q.text, "Are you sure?");
}

#[test]
fn question_multiple_choice_builder() {
    let opts = vec![
        QuestionOption {
            key: "A".into(),
            label: "Option A".into(),
        },
        QuestionOption {
            key: "B".into(),
            label: "Option B".into(),
        },
    ];
    let q = Question::multiple_choice("Choose:", opts.clone(), "choice_stage");
    assert_eq!(q.question_type, QuestionType::MultipleChoice);
    assert_eq!(q.options.len(), 2);
    assert_eq!(q.options[0].key, "A");
    assert_eq!(q.options[1].label, "Option B");
}

#[test]
fn question_freeform_builder() {
    let q = Question::freeform("Enter a name:", "name_stage");
    assert_eq!(q.question_type, QuestionType::Freeform);
    assert_eq!(q.text, "Enter a name:");
}

// ===========================================================================
// §6.3 — Answer types
// ===========================================================================

#[test]
fn answer_value_display() {
    assert_eq!(AnswerValue::Yes.to_string(), "YES");
    assert_eq!(AnswerValue::No.to_string(), "NO");
    assert_eq!(AnswerValue::Skipped.to_string(), "SKIPPED");
    assert_eq!(AnswerValue::Timeout.to_string(), "TIMEOUT");
    assert_eq!(AnswerValue::Selected("A".into()).to_string(), "SELECTED(A)");
    assert_eq!(AnswerValue::Text("hello".into()).to_string(), "TEXT(hello)");
}

#[test]
fn answer_new_and_predicates() {
    let timeout = Answer::new(AnswerValue::Timeout);
    assert!(timeout.is_timeout());
    assert!(!timeout.is_skipped());

    let skipped = Answer::new(AnswerValue::Skipped);
    assert!(skipped.is_skipped());
    assert!(!skipped.is_timeout());

    let yes = Answer::new(AnswerValue::Yes);
    assert!(!yes.is_timeout());
    assert!(!yes.is_skipped());
}

#[test]
fn answer_with_option() {
    let opt = QuestionOption {
        key: "X".into(),
        label: "Option X".into(),
    };
    let answer = Answer::with_option(AnswerValue::Selected("X".into()), opt.clone());
    assert_eq!(answer.selected_option, Some(opt));
}

// ===========================================================================
// §6.4 — AutoApproveInterviewer
// ===========================================================================

#[tokio::test]
async fn auto_approve_yes_no() {
    let interviewer = AutoApproveInterviewer;
    let q = Question::yes_no("Proceed?", "stage");
    let answer = interviewer.ask(&q).await;
    assert_eq!(answer.value, AnswerValue::Yes);
}

#[tokio::test]
async fn auto_approve_confirmation() {
    let interviewer = AutoApproveInterviewer;
    let q = Question::confirmation("Confirm?", "stage");
    let answer = interviewer.ask(&q).await;
    assert_eq!(answer.value, AnswerValue::Yes);
}

#[tokio::test]
async fn auto_approve_multiple_choice_selects_first() {
    let interviewer = AutoApproveInterviewer;
    let opts = vec![
        QuestionOption {
            key: "A".into(),
            label: "Alpha".into(),
        },
        QuestionOption {
            key: "B".into(),
            label: "Beta".into(),
        },
    ];
    let q = Question::multiple_choice("Pick:", opts, "stage");
    let answer = interviewer.ask(&q).await;
    assert_eq!(answer.value, AnswerValue::Selected("A".into()));
    assert!(answer.selected_option.is_some());
    assert_eq!(
        answer.selected_option.as_ref().map(|o| &o.key),
        Some(&"A".to_string())
    );
}

#[tokio::test]
async fn auto_approve_multiple_choice_no_options() {
    let interviewer = AutoApproveInterviewer;
    let q = Question::multiple_choice("Pick:", vec![], "stage");
    let answer = interviewer.ask(&q).await;
    assert_eq!(answer.value, AnswerValue::Text("auto-approved".into()));
}

#[tokio::test]
async fn auto_approve_freeform() {
    let interviewer = AutoApproveInterviewer;
    let q = Question::freeform("Name?", "stage");
    let answer = interviewer.ask(&q).await;
    assert_eq!(answer.value, AnswerValue::Text("auto-approved".into()));
}

// ===========================================================================
// §6.4 — QueueInterviewer
// ===========================================================================

#[tokio::test]
async fn queue_fifo_order() {
    let q1 = Answer::new(AnswerValue::Yes);
    let q2 = Answer::new(AnswerValue::No);
    let q3 = Answer::new(AnswerValue::Text("hello".into()));

    let interviewer = QueueInterviewer::new(vec![q1, q2, q3]);
    assert_eq!(interviewer.remaining(), 3);

    let q = Question::yes_no("Q1?", "s");
    let a1 = interviewer.ask(&q).await;
    assert_eq!(a1.value, AnswerValue::Yes);

    let a2 = interviewer.ask(&q).await;
    assert_eq!(a2.value, AnswerValue::No);

    let a3 = interviewer.ask(&q).await;
    assert_eq!(a3.value, AnswerValue::Text("hello".into()));

    assert_eq!(interviewer.remaining(), 0);
}

#[tokio::test]
async fn queue_returns_skipped_when_empty() {
    let interviewer = QueueInterviewer::new(vec![]);
    let q = Question::yes_no("Q?", "s");
    let answer = interviewer.ask(&q).await;
    assert_eq!(answer.value, AnswerValue::Skipped);
}

#[tokio::test]
async fn queue_exhaustion_returns_skipped() {
    let interviewer = QueueInterviewer::new(vec![Answer::new(AnswerValue::Yes)]);
    let q = Question::yes_no("Q?", "s");

    let _ = interviewer.ask(&q).await; // consume the one answer
    let answer = interviewer.ask(&q).await; // now empty
    assert_eq!(answer.value, AnswerValue::Skipped);
}

// ===========================================================================
// §6.4 — CallbackInterviewer
// ===========================================================================

#[tokio::test]
async fn callback_delegates_to_function() {
    let interviewer = CallbackInterviewer::new(|q: &Question| {
        if q.question_type == QuestionType::YesNo {
            Answer::new(AnswerValue::No)
        } else {
            Answer::new(AnswerValue::Text("callback".into()))
        }
    });

    let yes_no = Question::yes_no("Q?", "s");
    assert_eq!(interviewer.ask(&yes_no).await.value, AnswerValue::No);

    let freeform = Question::freeform("Q?", "s");
    assert_eq!(
        interviewer.ask(&freeform).await.value,
        AnswerValue::Text("callback".into())
    );
}

// ===========================================================================
// §6.4 — RecordingInterviewer
// ===========================================================================

#[tokio::test]
async fn recording_records_interactions() {
    let inner = AutoApproveInterviewer;
    let recording = RecordingInterviewer::new(inner);

    let q1 = Question::yes_no("Deploy?", "stage");
    let a1 = recording.ask(&q1).await;
    assert_eq!(a1.value, AnswerValue::Yes);

    let q2 = Question::freeform("Name?", "stage");
    let a2 = recording.ask(&q2).await;
    assert_eq!(a2.value, AnswerValue::Text("auto-approved".into()));

    let recs = recording.recordings();
    assert_eq!(recs.len(), 2);
    assert_eq!(recs[0].question_text, "Deploy?");
    assert_eq!(recs[0].answer.value, AnswerValue::Yes);
    assert_eq!(recs[1].question_text, "Name?");
}

#[tokio::test]
async fn recording_delegates_to_inner() {
    let inner = QueueInterviewer::new(vec![
        Answer::new(AnswerValue::Text("first".into())),
        Answer::new(AnswerValue::Text("second".into())),
    ]);
    let recording = RecordingInterviewer::new(inner);

    let q = Question::freeform("Q?", "s");
    assert_eq!(
        recording.ask(&q).await.value,
        AnswerValue::Text("first".into())
    );
    assert_eq!(
        recording.ask(&q).await.value,
        AnswerValue::Text("second".into())
    );

    let recs = recording.recordings();
    assert_eq!(recs.len(), 2);
}

// ===========================================================================
// Interviewer default methods
// ===========================================================================

#[tokio::test]
async fn ask_multiple_default() {
    let interviewer = AutoApproveInterviewer;
    let questions = vec![Question::yes_no("Q1?", "s"), Question::freeform("Q2?", "s")];
    let answers = interviewer.ask_multiple(&questions).await;
    assert_eq!(answers.len(), 2);
    assert_eq!(answers[0].value, AnswerValue::Yes);
    assert_eq!(answers[1].value, AnswerValue::Text("auto-approved".into()));
}

#[test]
fn inform_default_is_noop() {
    // Just verify it doesn't panic
    let interviewer = AutoApproveInterviewer;
    interviewer.inform("hello", "stage");
}

// ===========================================================================
// §4.6 — Accelerator key parsing
// ===========================================================================

#[test]
fn accelerator_bracket_format() {
    assert_eq!(parse_accelerator_key("[Y] Yes"), "Y");
    assert_eq!(parse_accelerator_key("[n] No"), "N");
    assert_eq!(parse_accelerator_key("[AB] Option AB"), "AB");
}

#[test]
fn accelerator_paren_format() {
    assert_eq!(parse_accelerator_key("A) Option A"), "A");
    assert_eq!(parse_accelerator_key("b) option b"), "B");
}

#[test]
fn accelerator_dash_format() {
    assert_eq!(parse_accelerator_key("X - Choice X"), "X");
    assert_eq!(parse_accelerator_key("q - quit"), "Q");
}

#[test]
fn accelerator_fallback_first_char() {
    assert_eq!(parse_accelerator_key("Deploy"), "D");
    assert_eq!(parse_accelerator_key("review"), "R");
}

#[test]
fn accelerator_empty_string() {
    assert_eq!(parse_accelerator_key(""), "");
}

#[test]
fn accelerator_whitespace_trimmed() {
    assert_eq!(parse_accelerator_key("  [Y] Yes  "), "Y");
    assert_eq!(parse_accelerator_key("  Deploy  "), "D");
}

// ===========================================================================
// §4.6 — WaitForHumanHandler
// ===========================================================================

/// Build a graph with a human gate node and outgoing edges.
fn human_gate_graph(choices: &[(&str, &str)]) -> Graph {
    let mut g = Graph::new("test");

    let mut gate = Node::new("gate");
    gate.attrs
        .insert("shape".into(), AttrValue::from("hexagon"));
    gate.attrs
        .insert("label".into(), AttrValue::from("Choose an option:"));
    g.add_node(gate);

    for (target, label) in choices {
        let node = Node::new(*target);
        g.add_node(node);

        let mut edge = Edge::new("gate", *target);
        edge.attrs.insert("label".into(), AttrValue::from(*label));
        g.add_edge(edge);
    }

    g
}

#[tokio::test]
async fn wait_human_selects_first_option() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let interviewer = Arc::new(AutoApproveInterviewer);
    let handler = WaitForHumanHandler::new(interviewer);

    let g = human_gate_graph(&[("deploy", "[D] Deploy"), ("rollback", "[R] Rollback")]);
    let node = g.get_node("gate").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "gate".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g, tmp.path()).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    assert!(outcome.suggested_next_ids.contains(&"deploy".to_string()));

    let selected = outcome
        .context_updates
        .get("human.gate.selected")
        .and_then(|v| v.as_str());
    assert_eq!(selected, Some("D"));

    Ok(())
}

#[tokio::test]
async fn wait_human_queue_selects_specific() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    // Queue with a "Selected R" answer
    let interviewer = Arc::new(QueueInterviewer::new(vec![Answer::new(
        AnswerValue::Selected("R".into()),
    )]));
    let handler = WaitForHumanHandler::new(interviewer);

    let g = human_gate_graph(&[("deploy", "[D] Deploy"), ("rollback", "[R] Rollback")]);
    let node = g.get_node("gate").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "gate".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g, tmp.path()).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    assert!(outcome.suggested_next_ids.contains(&"rollback".to_string()));
    Ok(())
}

#[tokio::test]
async fn wait_human_timeout_with_default() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let interviewer = Arc::new(QueueInterviewer::new(vec![Answer::new(
        AnswerValue::Timeout,
    )]));
    let handler = WaitForHumanHandler::new(interviewer);

    let mut g = human_gate_graph(&[("deploy", "[D] Deploy"), ("rollback", "[R] Rollback")]);
    // Set default choice
    g.get_node_mut("gate")
        .ok_or_else(|| stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "gate".into(),
        })?
        .attrs
        .insert("human.default_choice".into(), AttrValue::from("rollback"));

    let node = g.get_node("gate").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "gate".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g, tmp.path()).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    assert!(outcome.suggested_next_ids.contains(&"rollback".to_string()));
    Ok(())
}

#[tokio::test]
async fn wait_human_timeout_no_default_retries() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let interviewer = Arc::new(QueueInterviewer::new(vec![Answer::new(
        AnswerValue::Timeout,
    )]));
    let handler = WaitForHumanHandler::new(interviewer);

    let g = human_gate_graph(&[("deploy", "[D] Deploy")]);
    let node = g.get_node("gate").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "gate".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g, tmp.path()).await?;

    assert_eq!(outcome.status, StageStatus::Retry);
    Ok(())
}

#[tokio::test]
async fn wait_human_timeout_attr_propagated_to_question() -> AttractorResult<()> {
    use std::sync::Mutex;
    use stencila_attractor::types::Duration;

    let tmp = make_tempdir()?;
    let captured: Arc<Mutex<Option<f64>>> = Arc::new(Mutex::new(None));
    let captured_clone = Arc::clone(&captured);
    let interviewer = Arc::new(CallbackInterviewer::new(move |q: &Question| {
        *captured_clone
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = q.timeout_seconds;
        Answer::new(AnswerValue::Yes)
    }));
    let handler = WaitForHumanHandler::new(interviewer);

    // Duration variant
    let mut g = human_gate_graph(&[("deploy", "[D] Deploy")]);
    g.get_node_mut("gate")
        .ok_or_else(|| stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "gate".into(),
        })?
        .attrs
        .insert(
            "timeout".into(),
            AttrValue::Duration(Duration::from_spec_str("30s")?),
        );
    let node = g.get_node("gate").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "gate".into(),
        }
    })?;
    handler
        .execute(node, &Context::new(), &g, tmp.path())
        .await?;
    let secs = captured
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .ok_or_else(|| stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "timeout not set".into(),
        })?;
    assert!((secs - 30.0).abs() < f64::EPSILON);

    // String variant (quoted in DOT)
    let mut g = human_gate_graph(&[("deploy", "[D] Deploy")]);
    g.get_node_mut("gate")
        .ok_or_else(|| stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "gate".into(),
        })?
        .attrs
        .insert("timeout".into(), AttrValue::from("5m"));
    let node = g.get_node("gate").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "gate".into(),
        }
    })?;
    handler
        .execute(node, &Context::new(), &g, tmp.path())
        .await?;
    let secs = captured
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .ok_or_else(|| stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "timeout not set".into(),
        })?;
    assert!((secs - 300.0).abs() < f64::EPSILON);

    Ok(())
}

#[tokio::test]
async fn wait_human_skipped_fails() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let interviewer = Arc::new(QueueInterviewer::new(vec![Answer::new(
        AnswerValue::Skipped,
    )]));
    let handler = WaitForHumanHandler::new(interviewer);

    let g = human_gate_graph(&[("deploy", "[D] Deploy")]);
    let node = g.get_node("gate").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "gate".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g, tmp.path()).await?;

    assert_eq!(outcome.status, StageStatus::Fail);
    Ok(())
}

#[tokio::test]
async fn wait_human_no_edges_fails() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let interviewer = Arc::new(AutoApproveInterviewer);
    let handler = WaitForHumanHandler::new(interviewer);

    let mut g = Graph::new("test");
    let mut gate = Node::new("gate");
    gate.attrs
        .insert("shape".into(), AttrValue::from("hexagon"));
    g.add_node(gate);
    // No outgoing edges

    let node = g.get_node("gate").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "gate".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g, tmp.path()).await?;

    assert_eq!(outcome.status, StageStatus::Fail);
    Ok(())
}

#[tokio::test]
async fn wait_human_text_match_by_key() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    // Answer with text "R" which should match the key of the rollback choice
    let interviewer = Arc::new(QueueInterviewer::new(vec![Answer::new(AnswerValue::Text(
        "R".into(),
    ))]));
    let handler = WaitForHumanHandler::new(interviewer);

    let g = human_gate_graph(&[("deploy", "[D] Deploy"), ("rollback", "[R] Rollback")]);
    let node = g.get_node("gate").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "gate".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g, tmp.path()).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    assert!(outcome.suggested_next_ids.contains(&"rollback".to_string()));
    Ok(())
}

/// §4.6 deviation: unmatched answer returns FAIL instead of falling back
/// to first choice (spec pseudocode line: `selected = choices[0]`).
/// See README.md Deviations for rationale.
#[tokio::test]
async fn wait_human_unmatched_answer_fails() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    // Answer with text that matches no choice key or label
    let interviewer = Arc::new(QueueInterviewer::new(vec![Answer::new(AnswerValue::Text(
        "banana".into(),
    ))]));
    let handler = WaitForHumanHandler::new(interviewer);

    let g = human_gate_graph(&[("deploy", "[D] Deploy"), ("rollback", "[R] Rollback")]);
    let node = g.get_node("gate").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "gate".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g, tmp.path()).await?;

    // Deviation from spec: returns FAIL rather than falling back to choices[0]
    assert_eq!(outcome.status, StageStatus::Fail);
    Ok(())
}

#[tokio::test]
async fn wait_human_recording_captures_interaction() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let inner = AutoApproveInterviewer;
    let recording = Arc::new(RecordingInterviewer::new(inner));
    let handler = WaitForHumanHandler::new(recording.clone());

    let g = human_gate_graph(&[("deploy", "[D] Deploy")]);
    let node = g.get_node("gate").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "gate".into(),
        }
    })?;
    let ctx = Context::new();

    let _outcome = handler.execute(node, &ctx, &g, tmp.path()).await?;

    let recs = recording.recordings();
    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0].question_text, "Choose an option:");
    Ok(())
}

#[tokio::test]
async fn wait_human_edge_label_fallback_to_target() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let interviewer = Arc::new(AutoApproveInterviewer);
    let handler = WaitForHumanHandler::new(interviewer);

    // Edge with no label — should use target node ID as label
    let mut g = Graph::new("test");
    let mut gate = Node::new("gate");
    gate.attrs
        .insert("shape".into(), AttrValue::from("hexagon"));
    g.add_node(gate);
    g.add_node(Node::new("deploy"));
    g.add_edge(Edge::new("gate", "deploy")); // No label

    let node = g.get_node("gate").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "gate".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g, tmp.path()).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    assert!(outcome.suggested_next_ids.contains(&"deploy".to_string()));
    // Key should be first char of "deploy" → "D"
    let selected = outcome
        .context_updates
        .get("human.gate.selected")
        .and_then(|v| v.as_str());
    assert_eq!(selected, Some("D"));
    Ok(())
}
