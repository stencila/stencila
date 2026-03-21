//! Tests for human-in-the-loop (§6, §4.6).
//!
//! Covers interviewer types, all four interviewer implementations,
//! the WaitForHuman handler, accelerator key parsing, and end-to-end
//! pipeline execution with a human gate node.

mod common;

use std::sync::Arc;

use stencila_attractor::context::{Context, ctx};
use stencila_attractor::error::AttractorResult;
use stencila_attractor::graph::{AttrValue, Edge, Graph, Node, attr};
use stencila_attractor::handler::Handler;
use stencila_attractor::handlers::{WaitForHumanHandler, parse_accelerator_label};
use stencila_attractor::interviewer::{
    Answer, AnswerValue, InterviewError, Interviewer, Question, QuestionOption, QuestionType,
};
use stencila_attractor::interviewers::{
    AutoApproveInterviewer, CallbackInterviewer, QueueInterviewer, RecordingInterviewer,
};
use stencila_attractor::types::StageStatus;

// ===========================================================================
// §6.2 — Question types
// ===========================================================================

#[test]
fn question_yes_no_builder() {
    let q = Question::yes_no("Deploy?");
    assert_eq!(q.r#type, QuestionType::YesNo);
    assert_eq!(q.text, "Deploy?");
    assert!(q.options.is_empty());
    assert!(q.default.is_none());
    assert!(q.timeout_seconds.is_none());
}

#[test]
fn question_confirm_builder() {
    let q = Question::confirm("Are you sure?");
    assert_eq!(q.r#type, QuestionType::Confirm);
    assert_eq!(q.text, "Are you sure?");
}

#[test]
fn question_single_select_builder() {
    let opts = vec![
        QuestionOption {
            key: "A".into(),
            label: "Option A".into(),
            description: None,
        },
        QuestionOption {
            key: "B".into(),
            label: "Option B".into(),
            description: None,
        },
    ];
    let q = Question::single_select("Choose:", opts.clone());
    assert_eq!(q.r#type, QuestionType::SingleSelect);
    assert_eq!(q.options.len(), 2);
    assert_eq!(q.options[0].key, "A");
    assert_eq!(q.options[1].label, "Option B");
}

#[test]
fn question_freeform_builder() {
    let q = Question::freeform("Enter a name:");
    assert_eq!(q.r#type, QuestionType::Freeform);
    assert_eq!(q.text, "Enter a name:");
}

#[test]
fn question_multi_select_builder() {
    let opts = vec![
        QuestionOption {
            key: "X".into(),
            label: "Option X".into(),
            description: Some("First".into()),
        },
        QuestionOption {
            key: "Y".into(),
            label: "Option Y".into(),
            description: None,
        },
    ];
    let q = Question::multi_select("Select all:", opts.clone());
    assert_eq!(q.r#type, QuestionType::MultiSelect);
    assert_eq!(q.text, "Select all:");
    assert_eq!(q.options.len(), 2);
    assert_eq!(q.options[0].description, Some("First".into()));
    assert_eq!(q.options[1].description, None);
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
        description: None,
    };
    let answer = Answer::with_option(AnswerValue::Selected("X".into()), opt.clone());
    assert_eq!(answer.selected_option, Some(opt));
}

// ===========================================================================
// §6.4 — AutoApproveInterviewer
// ===========================================================================

#[tokio::test]
async fn auto_approve_yes_no() -> Result<(), InterviewError> {
    let interviewer = AutoApproveInterviewer;
    let q = Question::yes_no("Proceed?");
    let answer = interviewer.ask(&q).await?;
    assert_eq!(answer.value, AnswerValue::Yes);
    Ok(())
}

#[tokio::test]
async fn auto_approve_confirm() -> Result<(), InterviewError> {
    let interviewer = AutoApproveInterviewer;
    let q = Question::confirm("Confirm?");
    let answer = interviewer.ask(&q).await?;
    assert_eq!(answer.value, AnswerValue::Yes);
    Ok(())
}

#[tokio::test]
async fn auto_approve_single_select_selects_first() -> Result<(), InterviewError> {
    let interviewer = AutoApproveInterviewer;
    let opts = vec![
        QuestionOption {
            key: "A".into(),
            label: "Alpha".into(),
            description: None,
        },
        QuestionOption {
            key: "B".into(),
            label: "Beta".into(),
            description: None,
        },
    ];
    let q = Question::single_select("Pick:", opts);
    let answer = interviewer.ask(&q).await?;
    assert_eq!(answer.value, AnswerValue::Selected("A".into()));
    assert!(answer.selected_option.is_some());
    assert_eq!(
        answer.selected_option.as_ref().map(|o| &o.key),
        Some(&"A".to_string())
    );
    Ok(())
}

#[tokio::test]
async fn auto_approve_single_select_no_options() -> Result<(), InterviewError> {
    let interviewer = AutoApproveInterviewer;
    let q = Question::single_select("Pick:", vec![]);
    let answer = interviewer.ask(&q).await?;
    assert_eq!(answer.value, AnswerValue::Text("auto-approved".into()));
    Ok(())
}

#[tokio::test]
async fn auto_approve_freeform() -> Result<(), InterviewError> {
    let interviewer = AutoApproveInterviewer;
    let q = Question::freeform("Name?");
    let answer = interviewer.ask(&q).await?;
    assert_eq!(answer.value, AnswerValue::Text("auto-approved".into()));
    Ok(())
}

// ===========================================================================
// §6.4 — QueueInterviewer
// ===========================================================================

#[tokio::test]
async fn queue_fifo_order() -> Result<(), InterviewError> {
    let q1 = Answer::new(AnswerValue::Yes);
    let q2 = Answer::new(AnswerValue::No);
    let q3 = Answer::new(AnswerValue::Text("hello".into()));

    let interviewer = QueueInterviewer::new(vec![q1, q2, q3]);
    assert_eq!(interviewer.remaining(), 3);

    let q = Question::yes_no("Q1?");
    let a1 = interviewer.ask(&q).await?;
    assert_eq!(a1.value, AnswerValue::Yes);

    let a2 = interviewer.ask(&q).await?;
    assert_eq!(a2.value, AnswerValue::No);

    let a3 = interviewer.ask(&q).await?;
    assert_eq!(a3.value, AnswerValue::Text("hello".into()));

    assert_eq!(interviewer.remaining(), 0);
    Ok(())
}

#[tokio::test]
async fn queue_returns_skipped_when_empty() -> Result<(), InterviewError> {
    let interviewer = QueueInterviewer::new(vec![]);
    let q = Question::yes_no("Q?");
    let answer = interviewer.ask(&q).await?;
    assert_eq!(answer.value, AnswerValue::Skipped);
    Ok(())
}

#[tokio::test]
async fn queue_exhaustion_returns_skipped() -> Result<(), InterviewError> {
    let interviewer = QueueInterviewer::new(vec![Answer::new(AnswerValue::Yes)]);
    let q = Question::yes_no("Q?");

    let _ = interviewer.ask(&q).await?; // consume the one answer
    let answer = interviewer.ask(&q).await?; // now empty
    assert_eq!(answer.value, AnswerValue::Skipped);
    Ok(())
}

// ===========================================================================
// §6.4 — CallbackInterviewer
// ===========================================================================

#[tokio::test]
async fn callback_delegates_to_function() -> Result<(), InterviewError> {
    let interviewer = CallbackInterviewer::new(|q: &Question| {
        if q.r#type == QuestionType::YesNo {
            Answer::new(AnswerValue::No)
        } else {
            Answer::new(AnswerValue::Text("callback".into()))
        }
    });

    let yes_no = Question::yes_no("Q?");
    assert_eq!(interviewer.ask(&yes_no).await?.value, AnswerValue::No);

    let freeform = Question::freeform("Q?");
    assert_eq!(
        interviewer.ask(&freeform).await?.value,
        AnswerValue::Text("callback".into())
    );
    Ok(())
}

// ===========================================================================
// §6.4 — RecordingInterviewer
// ===========================================================================

#[tokio::test]
async fn recording_records_interactions() -> Result<(), InterviewError> {
    let inner = AutoApproveInterviewer;
    let recording = RecordingInterviewer::new(inner);

    let q1 = Question::yes_no("Deploy?");
    let a1 = recording.ask(&q1).await?;
    assert_eq!(a1.value, AnswerValue::Yes);

    let q2 = Question::freeform("Name?");
    let a2 = recording.ask(&q2).await?;
    assert_eq!(a2.value, AnswerValue::Text("auto-approved".into()));

    let recs = recording.recordings();
    assert_eq!(recs.len(), 2);
    assert_eq!(recs[0].question_text, "Deploy?");
    assert_eq!(recs[0].answer.value, AnswerValue::Yes);
    assert_eq!(recs[1].question_text, "Name?");
    Ok(())
}

#[tokio::test]
async fn recording_delegates_to_inner() -> Result<(), InterviewError> {
    let inner = QueueInterviewer::new(vec![
        Answer::new(AnswerValue::Text("first".into())),
        Answer::new(AnswerValue::Text("second".into())),
    ]);
    let recording = RecordingInterviewer::new(inner);

    let q = Question::freeform("Q?");
    assert_eq!(
        recording.ask(&q).await?.value,
        AnswerValue::Text("first".into())
    );
    assert_eq!(
        recording.ask(&q).await?.value,
        AnswerValue::Text("second".into())
    );

    let recs = recording.recordings();
    assert_eq!(recs.len(), 2);
    Ok(())
}

// ===========================================================================
// Interviewer default methods
// ===========================================================================

#[test]
fn inform_default_is_noop() {
    // Just verify it doesn't panic
    let interviewer = AutoApproveInterviewer;
    interviewer.inform("hello", "stage");
}

// ===========================================================================
// §4.6 — Accelerator label parsing
// ===========================================================================

#[test]
fn accelerator_bracket_format() {
    assert_eq!(
        parse_accelerator_label("[Y] Yes"),
        ("Y".into(), "Yes".into())
    );
    assert_eq!(parse_accelerator_label("[n] No"), ("N".into(), "No".into()));
    assert_eq!(
        parse_accelerator_label("[AB] Option AB"),
        ("AB".into(), "Option AB".into())
    );
}

#[test]
fn accelerator_bracket_no_space() {
    assert_eq!(
        parse_accelerator_label("[Y]Yes"),
        ("Y".into(), "Yes".into())
    );
    assert_eq!(
        parse_accelerator_label("[OK]Continue"),
        ("OK".into(), "Continue".into())
    );
}

#[test]
fn accelerator_paren_format() {
    assert_eq!(
        parse_accelerator_label("A) Option A"),
        ("A".into(), "Option A".into())
    );
    assert_eq!(
        parse_accelerator_label("b) option b"),
        ("B".into(), "option b".into())
    );
}

#[test]
fn accelerator_paren_no_space() {
    assert_eq!(
        parse_accelerator_label("A)Option"),
        ("A".into(), "Option".into())
    );
}

#[test]
fn accelerator_dash_format() {
    assert_eq!(
        parse_accelerator_label("X - Choice X"),
        ("X".into(), "Choice X".into())
    );
    assert_eq!(
        parse_accelerator_label("q - quit"),
        ("Q".into(), "quit".into())
    );
}

#[test]
fn accelerator_fallback_first_char() {
    assert_eq!(
        parse_accelerator_label("Deploy"),
        ("D".into(), "Deploy".into())
    );
    assert_eq!(
        parse_accelerator_label("review"),
        ("R".into(), "review".into())
    );
}

#[test]
fn accelerator_empty_string() {
    assert_eq!(parse_accelerator_label(""), ("".into(), "".into()));
}

#[test]
fn accelerator_whitespace_trimmed() {
    assert_eq!(
        parse_accelerator_label("  [Y] Yes  "),
        ("Y".into(), "Yes".into())
    );
    assert_eq!(
        parse_accelerator_label("  Deploy  "),
        ("D".into(), "Deploy".into())
    );
}

// ===========================================================================
// §4.6 — WaitForHumanHandler
// ===========================================================================

/// Build a graph with a human gate node and outgoing edges.
fn human_gate_graph(choices: &[(&str, &str)]) -> Graph {
    let mut g = Graph::new("test");

    let mut gate = Node::new("gate");
    gate.attrs
        .insert(attr::SHAPE.into(), Graph::HUMAN_SHAPE.into());
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
    let interviewer = Arc::new(AutoApproveInterviewer);
    let handler = WaitForHumanHandler::new(interviewer);

    let g = human_gate_graph(&[("deploy", "[D] Deploy"), ("rollback", "[R] Rollback")]);
    let node = g.get_node("gate").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "gate".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

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

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    assert!(outcome.suggested_next_ids.contains(&"rollback".to_string()));
    Ok(())
}

#[tokio::test]
async fn wait_human_timeout_with_default() -> AttractorResult<()> {
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

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    assert!(outcome.suggested_next_ids.contains(&"rollback".to_string()));
    Ok(())
}

#[tokio::test]
async fn wait_human_timeout_no_default_auto_approves_first_route() -> AttractorResult<()> {
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

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    assert!(outcome.suggested_next_ids.contains(&"deploy".to_string()));
    assert!(
        outcome.notes.contains("[timed out; auto-approved]"),
        "expected timed-out auto-approval marker in notes, got: {:?}",
        outcome.notes
    );
    Ok(())
}

#[tokio::test]
async fn wait_human_timeout_attr_propagated_to_question() -> AttractorResult<()> {
    use std::sync::Mutex;
    use stencila_attractor::types::Duration;

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
    handler.execute(node, &Context::new(), &g).await?;
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
    handler.execute(node, &Context::new(), &g).await?;
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

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Fail);
    Ok(())
}

#[tokio::test]
async fn wait_human_no_edges_fails() -> AttractorResult<()> {
    let interviewer = Arc::new(AutoApproveInterviewer);
    let handler = WaitForHumanHandler::new(interviewer);

    let mut g = Graph::new("test");
    let mut gate = Node::new("gate");
    gate.attrs
        .insert(attr::SHAPE.into(), Graph::HUMAN_SHAPE.into());
    g.add_node(gate);
    // No outgoing edges

    let node = g.get_node("gate").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "gate".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Fail);
    Ok(())
}

#[tokio::test]
async fn wait_human_text_match_by_key() -> AttractorResult<()> {
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

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    assert!(outcome.suggested_next_ids.contains(&"rollback".to_string()));
    Ok(())
}

/// §4.6 deviation: unmatched answer returns FAIL instead of falling back
/// to first choice (spec pseudocode line: `selected = choices[0]`).
/// See README.md Deviations for rationale.
#[tokio::test]
async fn wait_human_unmatched_answer_fails() -> AttractorResult<()> {
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

    let outcome = handler.execute(node, &ctx, &g).await?;

    // Deviation from spec: returns FAIL rather than falling back to choices[0]
    assert_eq!(outcome.status, StageStatus::Fail);
    Ok(())
}

#[tokio::test]
async fn wait_human_recording_captures_interaction() -> AttractorResult<()> {
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

    let _outcome = handler.execute(node, &ctx, &g).await?;

    let recs = recording.recordings();
    assert_eq!(recs.len(), 1);
    assert_eq!(recs[0].question_text, "Choose an option:");
    Ok(())
}

#[tokio::test]
async fn wait_human_edge_label_fallback_to_target() -> AttractorResult<()> {
    let interviewer = Arc::new(AutoApproveInterviewer);
    let handler = WaitForHumanHandler::new(interviewer);

    // Edge with no label — should use target node ID as label
    let mut g = Graph::new("test");
    let mut gate = Node::new("gate");
    gate.attrs
        .insert(attr::SHAPE.into(), Graph::HUMAN_SHAPE.into());
    g.add_node(gate);
    g.add_node(Node::new("deploy"));
    g.add_edge(Edge::new("gate", "deploy")); // No label

    let node = g.get_node("gate").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "gate".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

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

// ===========================================================================
// §4.6 — Extended question types (question_type attribute)
// ===========================================================================

/// Build a human node with a specific question_type and optional store key.
fn human_node_with_type(
    question_type: &str,
    label: &str,
    store: Option<&str>,
    targets: &[&str],
) -> Graph {
    let mut g = Graph::new("test");

    let mut gate = Node::new("gate");
    gate.attrs
        .insert(attr::SHAPE.into(), Graph::HUMAN_SHAPE.into());
    gate.attrs.insert("label".into(), AttrValue::from(label));
    gate.attrs
        .insert("question_type".into(), AttrValue::from(question_type));
    if let Some(key) = store {
        gate.attrs.insert("store".into(), AttrValue::from(key));
    }
    g.add_node(gate);

    for target in targets {
        g.add_node(Node::new(*target));
        g.add_edge(Edge::new("gate", *target));
    }

    g
}

fn get_gate_node(g: &Graph) -> AttractorResult<&Node> {
    g.get_node("gate")
        .ok_or_else(|| stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "gate".into(),
        })
}

#[tokio::test]
async fn wait_human_freeform_follows_first_edge() -> AttractorResult<()> {
    let interviewer = Arc::new(QueueInterviewer::new(vec![Answer::new(AnswerValue::Text(
        "Fix the formatting".into(),
    ))]));
    let handler = WaitForHumanHandler::new(interviewer);

    let g = human_node_with_type("freeform", "What should change?", None, &["next"]);
    let node = get_gate_node(&g)?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    assert!(outcome.suggested_next_ids.contains(&"next".to_string()));
    // Freeform should NOT set human.gate.selected (no choice matching)
    assert!(outcome.context_updates.get("human.gate.selected").is_none());
    Ok(())
}

#[tokio::test]
async fn wait_human_freeform_with_store() -> AttractorResult<()> {
    let interviewer = Arc::new(QueueInterviewer::new(vec![Answer::new(AnswerValue::Text(
        "Add error handling".into(),
    ))]));
    let handler = WaitForHumanHandler::new(interviewer);

    let g = human_node_with_type(
        "freeform",
        "Describe what must be improved",
        Some("human.feedback"),
        &["create"],
    );
    let node = get_gate_node(&g)?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    assert!(outcome.suggested_next_ids.contains(&"create".to_string()));

    let stored = outcome
        .context_updates
        .get("human.feedback")
        .and_then(|v| v.as_str());
    assert_eq!(stored, Some("Add error handling"));
    Ok(())
}

#[tokio::test]
async fn wait_human_yes_no_with_store() -> AttractorResult<()> {
    let interviewer = Arc::new(AutoApproveInterviewer);
    let handler = WaitForHumanHandler::new(interviewer);

    let g = human_node_with_type("yes_no", "Continue?", Some("human.decision"), &["next"]);
    let node = get_gate_node(&g)?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);

    let stored = outcome
        .context_updates
        .get("human.decision")
        .and_then(|v| v.as_str());
    assert_eq!(stored, Some("yes"));
    Ok(())
}

#[tokio::test]
async fn wait_human_yes_no_no_answer() -> AttractorResult<()> {
    let interviewer = Arc::new(QueueInterviewer::new(vec![Answer::new(AnswerValue::No)]));
    let handler = WaitForHumanHandler::new(interviewer);

    let g = human_node_with_type("yes_no", "Continue?", Some("human.decision"), &["next"]);
    let node = get_gate_node(&g)?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);

    let stored = outcome
        .context_updates
        .get("human.decision")
        .and_then(|v| v.as_str());
    assert_eq!(stored, Some("no"));
    Ok(())
}

#[tokio::test]
async fn wait_human_confirm_with_store() -> AttractorResult<()> {
    let interviewer = Arc::new(AutoApproveInterviewer);
    let handler = WaitForHumanHandler::new(interviewer);

    let g = human_node_with_type(
        "confirm",
        "Are you sure?",
        Some("human.confirmed"),
        &["next"],
    );
    let node = get_gate_node(&g)?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);

    let stored = outcome
        .context_updates
        .get("human.confirmed")
        .and_then(|v| v.as_str());
    assert_eq!(stored, Some("yes"));
    Ok(())
}

#[tokio::test]
async fn wait_human_single_select_with_store() -> AttractorResult<()> {
    // Auto-approve selects first option
    let interviewer = Arc::new(AutoApproveInterviewer);
    let handler = WaitForHumanHandler::new(interviewer);

    let mut g = Graph::new("test");
    let mut gate = Node::new("gate");
    gate.attrs
        .insert(attr::SHAPE.into(), Graph::HUMAN_SHAPE.into());
    gate.attrs
        .insert("label".into(), AttrValue::from("Pick action:"));
    gate.attrs
        .insert("store".into(), AttrValue::from("human.choice"));
    g.add_node(gate);

    for (target, label) in &[("accept", "[A] Accept"), ("revise", "[R] Revise")] {
        g.add_node(Node::new(*target));
        let mut edge = Edge::new("gate", *target);
        edge.attrs.insert("label".into(), AttrValue::from(*label));
        g.add_edge(edge);
    }

    let node = get_gate_node(&g)?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    // Should have both the traditional gate keys AND the store key
    assert!(outcome.context_updates.get("human.gate.selected").is_some());
    assert!(outcome.context_updates.get("human.gate.label").is_some());

    let stored = outcome
        .context_updates
        .get("human.choice")
        .and_then(|v| v.as_str());
    assert_eq!(stored, Some("Accept"));
    Ok(())
}

#[tokio::test]
async fn wait_human_freeform_timeout_no_store() -> AttractorResult<()> {
    let interviewer = Arc::new(QueueInterviewer::new(vec![Answer::new(
        AnswerValue::Timeout,
    )]));
    let handler = WaitForHumanHandler::new(interviewer);

    let g = human_node_with_type(
        "freeform",
        "What should change?",
        Some("human.feedback"),
        &["next"],
    );
    let node = get_gate_node(&g)?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    // Timeout should retry, not store anything
    assert_eq!(outcome.status, StageStatus::Retry);
    assert!(outcome.context_updates.get("human.feedback").is_none());
    Ok(())
}

#[tokio::test]
async fn wait_human_freeform_skipped_fails() -> AttractorResult<()> {
    let interviewer = Arc::new(QueueInterviewer::new(vec![Answer::new(
        AnswerValue::Skipped,
    )]));
    let handler = WaitForHumanHandler::new(interviewer);

    let g = human_node_with_type(
        "freeform",
        "What should change?",
        Some("human.feedback"),
        &["next"],
    );
    let node = get_gate_node(&g)?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Fail);
    assert!(outcome.context_updates.get("human.feedback").is_none());
    Ok(())
}

#[tokio::test]
async fn wait_human_no_store_attr_omits_key() -> AttractorResult<()> {
    let interviewer = Arc::new(QueueInterviewer::new(vec![Answer::new(AnswerValue::Text(
        "some feedback".into(),
    ))]));
    let handler = WaitForHumanHandler::new(interviewer);

    // Freeform without store attribute
    let g = human_node_with_type("freeform", "What should change?", None, &["next"]);
    let node = get_gate_node(&g)?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    // No store key → context_updates should be empty
    assert!(outcome.context_updates.is_empty());
    Ok(())
}

#[tokio::test]
async fn wait_human_unknown_question_type_falls_back_to_choice() -> AttractorResult<()> {
    let interviewer = Arc::new(AutoApproveInterviewer);
    let handler = WaitForHumanHandler::new(interviewer);

    // Unknown question_type should be treated as default (single_select)
    let mut g = Graph::new("test");
    let mut gate = Node::new("gate");
    gate.attrs
        .insert(attr::SHAPE.into(), Graph::HUMAN_SHAPE.into());
    gate.attrs
        .insert("label".into(), AttrValue::from("Choose:"));
    gate.attrs
        .insert("question_type".into(), AttrValue::from("bogus"));
    g.add_node(gate);

    g.add_node(Node::new("next"));
    let mut edge = Edge::new("gate", "next");
    edge.attrs
        .insert("label".into(), AttrValue::from("[N] Next"));
    g.add_edge(edge);

    let node = get_gate_node(&g)?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    // Should behave like single_select
    assert!(outcome.context_updates.get("human.gate.selected").is_some());
    Ok(())
}

// ===========================================================================
// Interview-spec routing: option key ↔ edge key mismatch regression tests
// ===========================================================================

/// Build a graph with a human gate node carrying an `interview` YAML spec
/// and outgoing edges with the given labels.
fn interview_spec_graph(spec_yaml: &str, edges: &[(&str, &str)]) -> Graph {
    let mut g = Graph::new("test");

    let mut gate = Node::new("gate");
    gate.attrs
        .insert(attr::SHAPE.into(), Graph::HUMAN_SHAPE.into());
    gate.attrs
        .insert("interview".into(), AttrValue::from(spec_yaml));
    g.add_node(gate);

    for &(target, label) in edges {
        g.add_node(Node::new(target));
        let mut edge = Edge::new("gate", target);
        edge.attrs.insert("label".into(), AttrValue::from(label));
        g.add_edge(edge);
    }

    g
}

/// Regression: selecting the second option ("Revise", key B) must route
/// correctly even though the edge-derived key is "R" (first letter of
/// "Revise"), not "B".
#[tokio::test]
async fn interview_spec_routes_second_option_by_label_fallback() -> AttractorResult<()> {
    let spec = r#"
questions:
  - question: Is it acceptable?
    type: single-select
    options:
      - label: Accept
      - label: Revise
    store: human.decision
"#;
    // Edge labels produce keys A (Accept) and R (Revise).
    // The interview spec auto-assigns keys A and B.
    // Selecting "Revise" yields Selected("B") — must still route to "create".
    let g = interview_spec_graph(spec, &[("end", "Accept"), ("create", "Revise")]);
    let node = get_gate_node(&g)?;
    let ctx = Context::new();

    let interviewer = Arc::new(QueueInterviewer::new(vec![Answer::new(
        AnswerValue::Selected("B".into()),
    )]));
    let handler = WaitForHumanHandler::new(interviewer);

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    assert!(outcome.suggested_next_ids.contains(&"create".to_string()));
    let stored = outcome
        .context_updates
        .get("human.decision")
        .and_then(|v| v.as_str());
    assert_eq!(stored, Some("Revise"));
    Ok(())
}

/// Selecting the first option still works when keys happen to match (A = A).
#[tokio::test]
async fn interview_spec_routes_first_option_by_direct_key_match() -> AttractorResult<()> {
    let spec = r#"
questions:
  - question: Is it acceptable?
    type: single-select
    options:
      - label: Accept
      - label: Revise
    store: human.decision
"#;
    let g = interview_spec_graph(spec, &[("end", "Accept"), ("create", "Revise")]);
    let node = get_gate_node(&g)?;
    let ctx = Context::new();

    let interviewer = Arc::new(QueueInterviewer::new(vec![Answer::new(
        AnswerValue::Selected("A".into()),
    )]));
    let handler = WaitForHumanHandler::new(interviewer);

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    assert!(outcome.suggested_next_ids.contains(&"end".to_string()));
    let stored = outcome
        .context_updates
        .get("human.decision")
        .and_then(|v| v.as_str());
    assert_eq!(stored, Some("Accept"));
    Ok(())
}

/// Conditional interview (finish-if) with second-option selection still
/// routes correctly — mirrors the real workflow pattern with Accept/Revise.
#[tokio::test]
async fn interview_spec_conditional_routes_second_option() -> AttractorResult<()> {
    let spec = r#"
questions:
  - question: Is it acceptable?
    type: single-select
    options:
      - label: Accept
      - label: Revise
    store: human.decision
    finish-if: Accept
  - question: What should change?
    type: freeform
    store: human.feedback
"#;
    let g = interview_spec_graph(spec, &[("end", "Accept"), ("create", "Revise")]);
    let node = get_gate_node(&g)?;
    let ctx = Context::new();

    // User picks "Revise" (key B), then enters feedback.
    let interviewer = Arc::new(QueueInterviewer::new(vec![
        Answer::new(AnswerValue::Selected("B".into())),
        Answer::new(AnswerValue::Text("Needs better error handling".into())),
    ]));
    let handler = WaitForHumanHandler::new(interviewer);

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    assert!(outcome.suggested_next_ids.contains(&"create".to_string()));
    let decision = outcome
        .context_updates
        .get("human.decision")
        .and_then(|v| v.as_str());
    assert_eq!(decision, Some("Revise"));
    let feedback = outcome
        .context_updates
        .get("human.feedback")
        .and_then(|v| v.as_str());
    assert_eq!(feedback, Some("Needs better error handling"));
    Ok(())
}

/// Conditional interview where finish-if triggers (Accept) — interview
/// ends after the first question and routes to the Accept edge.
#[tokio::test]
async fn interview_spec_conditional_finish_if_routes_first_option() -> AttractorResult<()> {
    let spec = r#"
questions:
  - question: Is it acceptable?
    type: single-select
    options:
      - label: Accept
      - label: Revise
    store: human.decision
    finish-if: Accept
  - question: What should change?
    type: freeform
    store: human.feedback
"#;
    let g = interview_spec_graph(spec, &[("end", "Accept"), ("create", "Revise")]);
    let node = get_gate_node(&g)?;
    let ctx = Context::new();

    // User picks "Accept" (key A) — finish-if triggers, feedback question skipped.
    let interviewer = Arc::new(QueueInterviewer::new(vec![Answer::new(
        AnswerValue::Selected("A".into()),
    )]));
    let handler = WaitForHumanHandler::new(interviewer);

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    assert!(outcome.suggested_next_ids.contains(&"end".to_string()));
    let decision = outcome
        .context_updates
        .get("human.decision")
        .and_then(|v| v.as_str());
    assert_eq!(decision, Some("Accept"));
    // Feedback should not be stored since the interview ended early.
    assert!(outcome.context_updates.get("human.feedback").is_none());
    Ok(())
}

/// Label fallback works when edge labels use accelerator syntax like
/// `[Y] Approve` — the key is "Y" but the option label "Approve"
/// should still match the choice label "Approve".
#[tokio::test]
async fn interview_spec_routes_via_label_when_edge_has_accelerator() -> AttractorResult<()> {
    let spec = r#"
questions:
  - question: Pick action
    type: single-select
    options:
      - label: Approve
      - label: Rollback
    store: human.action
"#;
    // Edge accelerators: [Y] Approve → key "Y", [N] Rollback → key "N".
    // Spec auto-assigns keys A and B.
    let g = interview_spec_graph(spec, &[("deploy", "[Y] Approve"), ("undo", "[N] Rollback")]);
    let node = get_gate_node(&g)?;
    let ctx = Context::new();

    // Select "Rollback" (spec key B, edge key N) — label fallback needed.
    let interviewer = Arc::new(QueueInterviewer::new(vec![Answer::new(
        AnswerValue::Selected("B".into()),
    )]));
    let handler = WaitForHumanHandler::new(interviewer);

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    assert!(outcome.suggested_next_ids.contains(&"undo".to_string()));
    let stored = outcome
        .context_updates
        .get("human.action")
        .and_then(|v| v.as_str());
    assert_eq!(stored, Some("Rollback"));
    Ok(())
}

// ===========================================================================
// Auto-approve fast path: internal.gate_timeouts context key
// ===========================================================================

/// When `internal.gate_timeouts` is set to `{"*": 0}` in context,
/// the WaitForHumanHandler should auto-approve by using an internal
/// AutoApproveInterviewer instead of `self.interviewer`.
#[tokio::test]
async fn auto_approve_via_gate_timeouts_zero() -> AttractorResult<()> {
    // Use a recording interviewer so we can verify it is NOT called
    let inner = QueueInterviewer::new(vec![Answer::new(AnswerValue::No)]);
    let recording = Arc::new(RecordingInterviewer::new(inner));
    let handler = WaitForHumanHandler::new(recording.clone());

    let g = human_gate_graph(&[("accept", "[A] Accept"), ("reject", "[R] Reject")]);
    let node = get_gate_node(&g)?;

    let ctx = Context::new();
    ctx.set("internal.gate_timeouts", serde_json::json!({"*": 0}));

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    // Auto-approve selects the first edge (Accept)
    assert!(outcome.suggested_next_ids.contains(&"accept".to_string()));
    // The normal interviewer should NOT have been called
    assert_eq!(recording.recordings().len(), 0);
    Ok(())
}

/// Auto-approved gates should emit `"[auto-approved]"` in outcome notes.
#[tokio::test]
async fn auto_approve_emits_marker_in_notes() -> AttractorResult<()> {
    let interviewer = Arc::new(QueueInterviewer::new(vec![]));
    let handler = WaitForHumanHandler::new(interviewer);

    let g = human_gate_graph(&[("accept", "[A] Accept"), ("reject", "[R] Reject")]);
    let node = get_gate_node(&g)?;

    let ctx = Context::new();
    ctx.set("internal.gate_timeouts", serde_json::json!({"*": 0}));

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    assert!(
        outcome.notes.contains("[auto-approved]"),
        "expected '[auto-approved]' in notes, got: {:?}",
        outcome.notes
    );
    Ok(())
}

/// Sub-second timeout (e.g. 0.5) should also trigger auto-approve.
#[tokio::test]
async fn auto_approve_via_gate_timeouts_sub_second() -> AttractorResult<()> {
    let inner = QueueInterviewer::new(vec![Answer::new(AnswerValue::No)]);
    let recording = Arc::new(RecordingInterviewer::new(inner));
    let handler = WaitForHumanHandler::new(recording.clone());

    let g = human_gate_graph(&[("accept", "[A] Accept"), ("reject", "[R] Reject")]);
    let node = get_gate_node(&g)?;

    let ctx = Context::new();
    ctx.set("internal.gate_timeouts", serde_json::json!({"*": 0.5}));

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    assert!(outcome.suggested_next_ids.contains(&"accept".to_string()));
    assert_eq!(recording.recordings().len(), 0);
    Ok(())
}

/// When `internal.gate_timeouts` has a value >= 1.0, auto-approve
/// should NOT trigger — the normal interviewer should be used.
#[tokio::test]
async fn no_auto_approve_when_timeout_ge_one() -> AttractorResult<()> {
    let recording = Arc::new(RecordingInterviewer::new(AutoApproveInterviewer));
    let handler = WaitForHumanHandler::new(recording.clone());

    let g = human_gate_graph(&[("accept", "[A] Accept"), ("reject", "[R] Reject")]);
    let node = get_gate_node(&g)?;

    let ctx = Context::new();
    ctx.set("internal.gate_timeouts", serde_json::json!({"*": 30}));

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    // The normal interviewer SHOULD have been called
    assert!(!recording.recordings().is_empty());
    // Notes should NOT contain auto-approved marker
    assert!(
        !outcome.notes.contains("[auto-approved]"),
        "expected no '[auto-approved]' marker when timeout >= 1.0"
    );
    Ok(())
}

/// When `internal.gate_timeouts` is absent from context, normal
/// interviewer behavior is unchanged (no regression).
#[tokio::test]
async fn no_auto_approve_when_gate_timeouts_absent() -> AttractorResult<()> {
    let recording = Arc::new(RecordingInterviewer::new(AutoApproveInterviewer));
    let handler = WaitForHumanHandler::new(recording.clone());

    let g = human_gate_graph(&[("accept", "[A] Accept"), ("reject", "[R] Reject")]);
    let node = get_gate_node(&g)?;

    let ctx = Context::new();
    // No gate_timeouts set

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    // Normal interviewer should be called
    assert!(!recording.recordings().is_empty());
    Ok(())
}

/// Auto-approve should still produce the standard human gate context
/// updates (human.gate.selected, human.gate.label) so downstream
/// nodes can inspect the routing decision.
#[tokio::test]
async fn auto_approve_produces_gate_context_updates() -> AttractorResult<()> {
    let interviewer = Arc::new(QueueInterviewer::new(vec![]));
    let handler = WaitForHumanHandler::new(interviewer);

    let g = human_gate_graph(&[("accept", "[A] Accept"), ("reject", "[R] Reject")]);
    let node = get_gate_node(&g)?;

    let ctx = Context::new();
    ctx.set("internal.gate_timeouts", serde_json::json!({"*": 0}));

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    assert!(outcome.suggested_next_ids.contains(&"accept".to_string()));

    let selected = outcome
        .context_updates
        .get("human.gate.selected")
        .and_then(|v| v.as_str());
    assert_eq!(selected, Some("A"));

    let label = outcome
        .context_updates
        .get("human.gate.label")
        .and_then(|v| v.as_str());
    assert_eq!(label, Some("Accept"));
    Ok(())
}

// ===========================================================================
// Phase 2 / Slice 2 — Interview-spec auto-approve and node-level precedence
// ===========================================================================

/// When `internal.gate_timeouts` is set to `{"*": 0}` and a node has an
/// `interview` attribute (no explicit `timeout`), the handler should
/// auto-approve using an internal AutoApproveInterviewer — the normal
/// interviewer should NOT be called.
#[tokio::test]
async fn interview_spec_auto_approves_via_gate_timeouts_zero() -> AttractorResult<()> {
    let spec = r#"
questions:
  - question: Is it acceptable?
    type: single-select
    options:
      - label: Accept
      - label: Revise
    store: human.decision
"#;
    let g = interview_spec_graph(spec, &[("end", "Accept"), ("create", "Revise")]);
    let node = get_gate_node(&g)?;

    // Use a recording interviewer so we can verify it is NOT called.
    let inner = QueueInterviewer::new(vec![Answer::new(AnswerValue::Selected("B".into()))]);
    let recording = Arc::new(RecordingInterviewer::new(inner));
    let handler = WaitForHumanHandler::new(recording.clone());

    let ctx = Context::new();
    ctx.set("internal.gate_timeouts", serde_json::json!({"*": 0}));

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    // Auto-approve selects the first edge (end / Accept)
    assert!(
        outcome.suggested_next_ids.contains(&"end".to_string()),
        "expected route to 'end', got: {:?}",
        outcome.suggested_next_ids
    );
    // The normal interviewer should NOT have been called
    assert_eq!(
        recording.recordings().len(),
        0,
        "normal interviewer should not be called during auto-approve"
    );
    Ok(())
}

/// Interview-spec auto-approved gates should emit `"[auto-approved]"` in
/// outcome notes, just like the regular (non-interview) auto-approve path.
#[tokio::test]
async fn interview_spec_auto_approve_emits_marker_in_notes() -> AttractorResult<()> {
    let spec = r#"
questions:
  - question: Proceed?
    type: single-select
    options:
      - label: Yes
      - label: No
    store: human.proceed
"#;
    let g = interview_spec_graph(spec, &[("next", "Yes"), ("abort", "No")]);
    let node = get_gate_node(&g)?;

    let interviewer = Arc::new(QueueInterviewer::new(vec![]));
    let handler = WaitForHumanHandler::new(interviewer);

    let ctx = Context::new();
    ctx.set("internal.gate_timeouts", serde_json::json!({"*": 0}));

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    assert!(
        outcome.notes.contains("[auto-approved]"),
        "expected '[auto-approved]' in notes, got: {:?}",
        outcome.notes
    );
    Ok(())
}

/// A node with an explicit `timeout="2m"` attribute should NOT be
/// auto-approved even when `internal.gate_timeouts` is `{"*": 0}`.
/// Node-level `timeout` takes precedence over context-based timeout.
#[tokio::test]
async fn node_timeout_takes_precedence_over_gate_timeouts_zero() -> AttractorResult<()> {
    // Build a gate graph with an explicit timeout on the gate node.
    let mut g = Graph::new("test");

    let mut gate = Node::new("gate");
    gate.attrs
        .insert(attr::SHAPE.into(), Graph::HUMAN_SHAPE.into());
    gate.attrs
        .insert("label".into(), AttrValue::from("Choose an option:"));
    // Set an explicit node-level timeout of 2 minutes.
    gate.attrs
        .insert(attr::TIMEOUT.into(), AttrValue::from("2m"));
    g.add_node(gate);

    for (target, label) in &[("accept", "[A] Accept"), ("reject", "[R] Reject")] {
        g.add_node(Node::new(*target));
        let mut edge = Edge::new("gate", *target);
        edge.attrs.insert("label".into(), AttrValue::from(*label));
        g.add_edge(edge);
    }

    let node = get_gate_node(&g)?;

    // Use a recording interviewer — it SHOULD be called because auto-approve
    // must not trigger when the node has its own timeout.
    let recording = Arc::new(RecordingInterviewer::new(AutoApproveInterviewer));
    let handler = WaitForHumanHandler::new(recording.clone());

    let ctx = Context::new();
    ctx.set("internal.gate_timeouts", serde_json::json!({"*": 0}));

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    // The normal interviewer SHOULD have been called
    assert!(
        !recording.recordings().is_empty(),
        "normal interviewer should be called when node has explicit timeout"
    );
    // Notes should NOT contain auto-approved marker
    assert!(
        !outcome.notes.contains("[auto-approved]"),
        "expected no '[auto-approved]' marker when node has explicit timeout"
    );
    Ok(())
}

/// Same as above but with the interview-spec path: a node with both an
/// `interview` attribute and an explicit `timeout="2m"` attribute should
/// NOT be auto-approved even when `internal.gate_timeouts` is `{"*": 0}`.
#[tokio::test]
async fn interview_spec_node_timeout_takes_precedence_over_gate_timeouts() -> AttractorResult<()> {
    let spec = r#"
questions:
  - question: Is it acceptable?
    type: single-select
    options:
      - label: Accept
      - label: Revise
    store: human.decision
"#;

    // Build graph with interview spec AND explicit timeout on the gate node.
    let mut g = Graph::new("test");

    let mut gate = Node::new("gate");
    gate.attrs
        .insert(attr::SHAPE.into(), Graph::HUMAN_SHAPE.into());
    gate.attrs.insert("interview".into(), AttrValue::from(spec));
    // Explicit node-level timeout of 2 minutes — should take precedence.
    gate.attrs
        .insert(attr::TIMEOUT.into(), AttrValue::from("2m"));
    g.add_node(gate);

    for &(target, label) in &[("end", "Accept"), ("create", "Revise")] {
        g.add_node(Node::new(target));
        let mut edge = Edge::new("gate", target);
        edge.attrs.insert("label".into(), AttrValue::from(label));
        g.add_edge(edge);
    }

    let node = get_gate_node(&g)?;

    // Use a recording interviewer — it SHOULD be called.
    let inner = QueueInterviewer::new(vec![Answer::new(AnswerValue::Selected("A".into()))]);
    let recording = Arc::new(RecordingInterviewer::new(inner));
    let handler = WaitForHumanHandler::new(recording.clone());

    let ctx = Context::new();
    ctx.set("internal.gate_timeouts", serde_json::json!({"*": 0}));

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    // The normal interviewer SHOULD have been called
    assert!(
        !recording.recordings().is_empty(),
        "normal interviewer should be called when node has explicit timeout"
    );
    // Notes should NOT contain auto-approved marker
    assert!(
        !outcome.notes.contains("[auto-approved]"),
        "expected no '[auto-approved]' marker when node has explicit timeout"
    );
    Ok(())
}

// ===========================================================================
// Phase 3 / Slice 1 — Timeout application and expiry routing
// ===========================================================================

/// When `internal.gate_timeouts` has a non-zero value (>= 1.0), the resolved
/// timeout should be injected as `timeout_seconds` on the question so the
/// interviewer's timeout machinery fires after the configured duration.
#[tokio::test]
async fn context_gate_timeout_injected_as_timeout_seconds() -> AttractorResult<()> {
    use std::sync::Mutex;

    let captured: Arc<Mutex<Option<f64>>> = Arc::new(Mutex::new(None));
    let captured_clone = Arc::clone(&captured);
    let interviewer = Arc::new(CallbackInterviewer::new(move |q: &Question| {
        *captured_clone
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = q.timeout_seconds;
        Answer::new(AnswerValue::Yes)
    }));
    let handler = WaitForHumanHandler::new(interviewer);

    let g = human_gate_graph(&[("deploy", "[D] Deploy")]);
    let node = get_gate_node(&g)?;

    let ctx = Context::new();
    ctx.set(ctx::GATE_TIMEOUTS, serde_json::json!({"*": 30}));

    handler.execute(node, &ctx, &g).await?;

    let secs = captured
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .ok_or_else(|| stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "timeout_seconds not set".into(),
        })?;
    assert!(
        (secs - 30.0).abs() < f64::EPSILON,
        "expected timeout_seconds=30.0 from context gate_timeouts, got {secs}"
    );
    Ok(())
}

/// When the node already has an explicit `timeout` attribute AND context has
/// a `gate_timeouts` value, the node-level `timeout` should take precedence
/// for `timeout_seconds` — the context value must NOT override it.
#[tokio::test]
async fn node_timeout_attr_takes_precedence_over_context_for_timeout_seconds() -> AttractorResult<()>
{
    use std::sync::Mutex;
    use stencila_attractor::types::Duration;

    let captured: Arc<Mutex<Option<f64>>> = Arc::new(Mutex::new(None));
    let captured_clone = Arc::clone(&captured);
    let interviewer = Arc::new(CallbackInterviewer::new(move |q: &Question| {
        *captured_clone
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = q.timeout_seconds;
        Answer::new(AnswerValue::Yes)
    }));
    let handler = WaitForHumanHandler::new(interviewer);

    // Node has explicit timeout="5m" (300 seconds)
    let mut g = human_gate_graph(&[("deploy", "[D] Deploy")]);
    g.get_node_mut("gate")
        .ok_or_else(|| stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "gate".into(),
        })?
        .attrs
        .insert(
            "timeout".into(),
            AttrValue::Duration(Duration::from_spec_str("5m")?),
        );
    let node = get_gate_node(&g)?;

    // Context has a different gate_timeout of 10 seconds
    let ctx = Context::new();
    ctx.set(ctx::GATE_TIMEOUTS, serde_json::json!({"*": 10}));

    handler.execute(node, &ctx, &g).await?;

    let secs = captured
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .ok_or_else(|| stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "timeout_seconds not set".into(),
        })?;
    // Node-level 5m=300s should win over context 10s
    assert!(
        (secs - 300.0).abs() < f64::EPSILON,
        "expected node-level timeout_seconds=300.0, got {secs} (context value leaked)"
    );
    Ok(())
}

/// For interview-spec paths, `timeout_seconds` from context should be
/// applied to each question in the spec individually.
#[tokio::test]
async fn interview_spec_context_timeout_injected_per_question() -> AttractorResult<()> {
    use std::sync::Mutex;

    let captured_timeouts: Arc<Mutex<Vec<Option<f64>>>> = Arc::new(Mutex::new(Vec::new()));
    let captured_clone = Arc::clone(&captured_timeouts);
    let interviewer = Arc::new(CallbackInterviewer::new(move |q: &Question| {
        captured_clone
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(q.timeout_seconds);
        // Return appropriate answer based on question type
        if q.r#type == QuestionType::SingleSelect {
            let key = q.options.first().map(|o| o.key.clone()).unwrap_or_default();
            Answer::new(AnswerValue::Selected(key))
        } else {
            Answer::new(AnswerValue::Text("auto".into()))
        }
    }));
    let handler = WaitForHumanHandler::new(interviewer);

    let spec = r#"
questions:
  - question: Is it acceptable?
    type: single-select
    options:
      - label: Accept
      - label: Revise
    store: human.decision
  - question: Any notes?
    type: freeform
    store: human.notes
"#;
    let g = interview_spec_graph(spec, &[("end", "Accept"), ("create", "Revise")]);
    let node = get_gate_node(&g)?;

    let ctx = Context::new();
    ctx.set(ctx::GATE_TIMEOUTS, serde_json::json!({"*": 15}));

    handler.execute(node, &ctx, &g).await?;

    let timeouts = captured_timeouts
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    assert!(
        timeouts.len() >= 2,
        "expected at least 2 questions asked, got {}",
        timeouts.len()
    );
    // Each question should have timeout_seconds=15.0 from context
    for (i, t) in timeouts.iter().enumerate() {
        assert_eq!(
            *t,
            Some(15.0),
            "question {i} expected timeout_seconds=15.0, got {t:?}"
        );
    }
    Ok(())
}

/// When a non-zero context timeout fires (i.e. interviewer returns
/// `AnswerValue::Timeout`), the handler should route via the existing
/// timeout path — using `human.default_choice` if available.
#[tokio::test]
async fn context_timeout_expiry_routes_via_default_choice() -> AttractorResult<()> {
    // Simulate a timeout by having the interviewer return AnswerValue::Timeout
    let interviewer = Arc::new(QueueInterviewer::new(vec![Answer::new(
        AnswerValue::Timeout,
    )]));
    let handler = WaitForHumanHandler::new(interviewer);

    let mut g = human_gate_graph(&[("accept", "[A] Accept"), ("reject", "[R] Reject")]);
    // Set default choice so timeout routes to "accept"
    g.get_node_mut("gate")
        .ok_or_else(|| stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "gate".into(),
        })?
        .attrs
        .insert("human.default_choice".into(), AttrValue::from("accept"));
    let node = get_gate_node(&g)?;

    // Non-zero context timeout (>= 1.0)
    let ctx = Context::new();
    ctx.set(ctx::GATE_TIMEOUTS, serde_json::json!({"*": 5}));

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    assert!(
        outcome.suggested_next_ids.contains(&"accept".to_string()),
        "expected timeout to route via default_choice to 'accept', got: {:?}",
        outcome.suggested_next_ids
    );
    Ok(())
}

/// When a non-zero context timeout fires and no `human.default_choice` is
/// set, the handler should produce a retry outcome (existing behavior).
#[tokio::test]
async fn context_timeout_expiry_retries_without_default_choice() -> AttractorResult<()> {
    let interviewer = Arc::new(QueueInterviewer::new(vec![Answer::new(
        AnswerValue::Timeout,
    )]));
    let handler = WaitForHumanHandler::new(interviewer);

    let g = human_gate_graph(&[("accept", "[A] Accept"), ("reject", "[R] Reject")]);
    let node = get_gate_node(&g)?;

    let ctx = Context::new();
    ctx.set(ctx::GATE_TIMEOUTS, serde_json::json!({"*": 5}));

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    assert!(
        outcome.suggested_next_ids.contains(&"accept".to_string()),
        "expected timeout without default_choice to auto-approve first route, got: {:?}",
        outcome.suggested_next_ids
    );
    assert!(
        outcome.notes.contains("[timed out; auto-approved]"),
        "expected timed-out auto-approval marker in notes, got: {:?}",
        outcome.notes
    );
    Ok(())
}

#[tokio::test]
async fn wait_human_timeout_with_default_marks_notes() -> AttractorResult<()> {
    let interviewer = Arc::new(QueueInterviewer::new(vec![Answer::new(
        AnswerValue::Timeout,
    )]));
    let handler = WaitForHumanHandler::new(interviewer);

    let mut g = human_gate_graph(&[("deploy", "[D] Deploy"), ("rollback", "[R] Rollback")]);
    g.get_node_mut("gate")
        .ok_or_else(|| stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "gate".into(),
        })?
        .attrs
        .insert("human.default_choice".into(), AttrValue::from("rollback"));

    let node = get_gate_node(&g)?;
    let ctx = Context::new();
    ctx.set(ctx::GATE_TIMEOUTS, serde_json::json!({"*": 5}));

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    assert!(outcome.suggested_next_ids.contains(&"rollback".to_string()));
    assert!(
        outcome.notes.contains("[timed out; used default_choice]"),
        "expected default_choice timeout marker in notes, got: {:?}",
        outcome.notes
    );
    Ok(())
}

/// When context has a non-zero gate timeout and the human responds in time,
/// the human's answer should be used normally — no auto-approve, no timeout.
#[tokio::test]
async fn human_responds_before_context_timeout_answer_used_normally() -> AttractorResult<()> {
    // Interviewer responds immediately with a specific selection
    let interviewer = Arc::new(QueueInterviewer::new(vec![Answer::new(
        AnswerValue::Selected("R".into()),
    )]));
    let handler = WaitForHumanHandler::new(interviewer);

    let g = human_gate_graph(&[("accept", "[A] Accept"), ("reject", "[R] Reject")]);
    let node = get_gate_node(&g)?;

    let ctx = Context::new();
    ctx.set(ctx::GATE_TIMEOUTS, serde_json::json!({"*": 10}));

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    // The human's explicit selection should win
    assert!(
        outcome.suggested_next_ids.contains(&"reject".to_string()),
        "expected human selection to route to 'reject', got: {:?}",
        outcome.suggested_next_ids
    );
    // No auto-approved marker
    assert!(
        !outcome.notes.contains("[auto-approved]"),
        "human answered in time — should not be auto-approved"
    );
    Ok(())
}
