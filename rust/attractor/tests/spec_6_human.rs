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
fn question_type_display() {
    assert_eq!(QuestionType::YesNo.to_string(), "YES_NO");
    assert_eq!(QuestionType::MultipleChoice.to_string(), "MULTIPLE_CHOICE");
    assert_eq!(QuestionType::MultiSelect.to_string(), "MULTI_SELECT");
    assert_eq!(QuestionType::Freeform.to_string(), "FREEFORM");
    assert_eq!(QuestionType::Confirmation.to_string(), "CONFIRMATION");
}

#[test]
fn question_yes_no_builder() {
    let q = Question::yes_no("Deploy?");
    assert_eq!(q.question_type, QuestionType::YesNo);
    assert_eq!(q.text, "Deploy?");
    assert!(q.options.is_empty());
    assert!(q.default.is_none());
    assert!(q.timeout_seconds.is_none());
}

#[test]
fn question_confirmation_builder() {
    let q = Question::confirmation("Are you sure?");
    assert_eq!(q.question_type, QuestionType::Confirmation);
    assert_eq!(q.text, "Are you sure?");
}

#[test]
fn question_multiple_choice_builder() {
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
    let q = Question::multiple_choice("Choose:", opts.clone());
    assert_eq!(q.question_type, QuestionType::MultipleChoice);
    assert_eq!(q.options.len(), 2);
    assert_eq!(q.options[0].key, "A");
    assert_eq!(q.options[1].label, "Option B");
}

#[test]
fn question_freeform_builder() {
    let q = Question::freeform("Enter a name:");
    assert_eq!(q.question_type, QuestionType::Freeform);
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
    assert_eq!(q.question_type, QuestionType::MultiSelect);
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
async fn auto_approve_confirmation() -> Result<(), InterviewError> {
    let interviewer = AutoApproveInterviewer;
    let q = Question::confirmation("Confirm?");
    let answer = interviewer.ask(&q).await?;
    assert_eq!(answer.value, AnswerValue::Yes);
    Ok(())
}

#[tokio::test]
async fn auto_approve_multiple_choice_selects_first() -> Result<(), InterviewError> {
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
    let q = Question::multiple_choice("Pick:", opts);
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
async fn auto_approve_multiple_choice_no_options() -> Result<(), InterviewError> {
    let interviewer = AutoApproveInterviewer;
    let q = Question::multiple_choice("Pick:", vec![]);
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
        if q.question_type == QuestionType::YesNo {
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
async fn wait_human_timeout_no_default_retries() -> AttractorResult<()> {
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

    assert_eq!(outcome.status, StageStatus::Retry);
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
        .insert("shape".into(), AttrValue::from("hexagon"));
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
        .insert("shape".into(), AttrValue::from("hexagon"));
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
async fn wait_human_confirmation_with_store() -> AttractorResult<()> {
    let interviewer = Arc::new(AutoApproveInterviewer);
    let handler = WaitForHumanHandler::new(interviewer);

    let g = human_node_with_type(
        "confirmation",
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
async fn wait_human_multiple_choice_with_store() -> AttractorResult<()> {
    // Auto-approve selects first option
    let interviewer = Arc::new(AutoApproveInterviewer);
    let handler = WaitForHumanHandler::new(interviewer);

    let mut g = Graph::new("test");
    let mut gate = Node::new("gate");
    gate.attrs
        .insert("shape".into(), AttrValue::from("hexagon"));
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
    assert_eq!(stored, Some("A"));
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

    // Unknown question_type should be treated as default (multiple_choice)
    let mut g = Graph::new("test");
    let mut gate = Node::new("gate");
    gate.attrs
        .insert("shape".into(), AttrValue::from("hexagon"));
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
    // Should behave like multiple_choice
    assert!(outcome.context_updates.get("human.gate.selected").is_some());
    Ok(())
}
