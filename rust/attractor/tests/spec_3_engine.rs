//! Tests for engine execution (§3), edge selection (§3.3), goal gates (§3.4),
//! failure routing (§3.7), retry (§3.5-3.6), handler registry (§4.1-4.2),
//! and run directory (§5.6).

use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::Value;

use stencila_attractor::checkpoint::Checkpoint;
use stencila_attractor::context::Context;
use stencila_attractor::edge_selection::{
    best_by_weight_then_lexical, normalize_label, select_edge,
};
use stencila_attractor::engine::{self, EngineConfig, check_goal_gates, get_retry_target};
use stencila_attractor::error::{AttractorError, AttractorResult};
use stencila_attractor::events::{EventEmitter, NoOpEmitter, PipelineEvent};
use stencila_attractor::graph::{AttrValue, Edge, Graph, Node};
use stencila_attractor::handler::{Handler, HandlerRegistry};
use stencila_attractor::retry::{
    BackoffConfig, RetryPolicy, RetryPreset, build_retry_policy, delay_for_attempt,
    execute_with_retry,
};
use stencila_attractor::run_directory::{Manifest, RunDirectory};
use stencila_attractor::types::{Outcome, StageStatus};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Build a minimal start→exit graph.
fn linear_graph() -> Graph {
    let mut g = Graph::new("linear");
    let mut start = Node::new("start");
    start
        .attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);

    let mut exit = Node::new("exit");
    exit.attrs
        .insert("shape".into(), AttrValue::from("Msquare"));
    g.add_node(exit);

    g.add_edge(Edge::new("start", "exit"));
    g
}

/// Build a start→middle→exit graph.
fn three_node_graph() -> Graph {
    let mut g = Graph::new("three_node");
    let mut start = Node::new("start");
    start
        .attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);

    let middle = Node::new("middle");
    g.add_node(middle);

    let mut exit = Node::new("exit");
    exit.attrs
        .insert("shape".into(), AttrValue::from("Msquare"));
    g.add_node(exit);

    g.add_edge(Edge::new("start", "middle"));
    g.add_edge(Edge::new("middle", "exit"));
    g
}

/// A handler that returns a preconfigured outcome.
struct MockHandler {
    outcome: Outcome,
}

impl MockHandler {
    fn new(outcome: Outcome) -> Self {
        Self { outcome }
    }
}

#[async_trait]
impl Handler for MockHandler {
    async fn execute(
        &self,
        _node: &Node,
        _context: &Context,
        _graph: &Graph,
        _logs_root: &Path,
    ) -> AttractorResult<Outcome> {
        Ok(self.outcome.clone())
    }
}

/// A handler that returns different outcomes on successive calls.
struct SequenceHandler {
    outcomes: Vec<Outcome>,
    call_count: Mutex<usize>,
}

impl SequenceHandler {
    fn new(outcomes: Vec<Outcome>) -> Self {
        Self {
            outcomes,
            call_count: Mutex::new(0),
        }
    }
}

#[async_trait]
impl Handler for SequenceHandler {
    async fn execute(
        &self,
        _node: &Node,
        _context: &Context,
        _graph: &Graph,
        _logs_root: &Path,
    ) -> AttractorResult<Outcome> {
        let mut count = self
            .call_count
            .lock()
            .map_err(|_| AttractorError::HandlerFailed {
                node_id: "sequence".into(),
                reason: "lock poisoned".into(),
            })?;
        let idx = *count;
        *count += 1;
        if idx < self.outcomes.len() {
            Ok(self.outcomes[idx].clone())
        } else {
            Ok(self
                .outcomes
                .last()
                .cloned()
                .unwrap_or_else(Outcome::success))
        }
    }
}

/// A handler that always panics.
struct PanicHandler;

#[async_trait]
impl Handler for PanicHandler {
    async fn execute(
        &self,
        _node: &Node,
        _context: &Context,
        _graph: &Graph,
        _logs_root: &Path,
    ) -> AttractorResult<Outcome> {
        panic!("intentional panic for testing");
    }
}

/// A handler that returns a retryable error.
struct RetryableErrorHandler;

#[async_trait]
impl Handler for RetryableErrorHandler {
    async fn execute(
        &self,
        _node: &Node,
        _context: &Context,
        _graph: &Graph,
        _logs_root: &Path,
    ) -> AttractorResult<Outcome> {
        Err(AttractorError::TemporaryUnavailable {
            message: "temporarily unavailable".into(),
        })
    }
}

/// An event emitter that records all events.
#[derive(Default)]
struct RecordingEmitter {
    events: Mutex<Vec<String>>,
}

impl RecordingEmitter {
    fn event_names(&self) -> Vec<String> {
        self.events.lock().map(|e| e.clone()).unwrap_or_default()
    }
}

impl EventEmitter for RecordingEmitter {
    fn emit(&self, event: PipelineEvent) {
        let name = match &event {
            PipelineEvent::PipelineStarted { .. } => "PipelineStarted",
            PipelineEvent::PipelineCompleted { .. } => "PipelineCompleted",
            PipelineEvent::PipelineFailed { .. } => "PipelineFailed",
            PipelineEvent::StageStarted { .. } => "StageStarted",
            PipelineEvent::StageCompleted { .. } => "StageCompleted",
            PipelineEvent::StageFailed { .. } => "StageFailed",
            PipelineEvent::StageRetrying { .. } => "StageRetrying",
            PipelineEvent::CheckpointSaved { .. } => "CheckpointSaved",
            _ => "Other",
        };
        if let Ok(mut events) = self.events.lock() {
            events.push(name.to_string());
        }
    }
}

// ===========================================================================
// Edge Selection (~10 tests)
// ===========================================================================

#[test]
fn edge_selection_no_edges_returns_none() {
    let g = Graph::new("test");
    let outcome = Outcome::success();
    let ctx = Context::new();
    assert!(select_edge("node1", &outcome, &ctx, &g).is_none());
}

#[test]
fn edge_selection_condition_match_wins() {
    let mut g = Graph::new("test");
    g.add_node(Node::new("A"));
    g.add_node(Node::new("B"));
    g.add_node(Node::new("C"));

    let mut e1 = Edge::new("A", "B");
    e1.attrs
        .insert("condition".into(), AttrValue::from("outcome=success"));
    g.add_edge(e1);
    g.add_edge(Edge::new("A", "C"));

    let outcome = Outcome::success();
    let ctx = Context::new();
    let selected = select_edge("A", &outcome, &ctx, &g);
    assert_eq!(selected.map(|e| e.to.as_str()), Some("B"));
}

#[test]
fn edge_selection_preferred_label_match() {
    let mut g = Graph::new("test");
    g.add_node(Node::new("A"));
    g.add_node(Node::new("B"));
    g.add_node(Node::new("C"));

    let mut e1 = Edge::new("A", "B");
    e1.attrs.insert("label".into(), AttrValue::from("Yes"));
    g.add_edge(e1);

    let mut e2 = Edge::new("A", "C");
    e2.attrs.insert("label".into(), AttrValue::from("No"));
    g.add_edge(e2);

    let mut outcome = Outcome::success();
    outcome.preferred_label = "yes".to_string(); // case-insensitive
    let ctx = Context::new();
    let selected = select_edge("A", &outcome, &ctx, &g);
    assert_eq!(selected.map(|e| e.to.as_str()), Some("B"));
}

#[test]
fn edge_selection_suggested_next_ids() {
    let mut g = Graph::new("test");
    g.add_node(Node::new("A"));
    g.add_node(Node::new("B"));
    g.add_node(Node::new("C"));

    g.add_edge(Edge::new("A", "B"));
    g.add_edge(Edge::new("A", "C"));

    let mut outcome = Outcome::success();
    outcome.suggested_next_ids = vec!["C".to_string()];
    let ctx = Context::new();
    let selected = select_edge("A", &outcome, &ctx, &g);
    assert_eq!(selected.map(|e| e.to.as_str()), Some("C"));
}

#[test]
fn edge_selection_highest_weight() {
    let mut g = Graph::new("test");
    g.add_node(Node::new("A"));
    g.add_node(Node::new("B"));
    g.add_node(Node::new("C"));

    let mut e1 = Edge::new("A", "B");
    e1.attrs.insert("weight".into(), AttrValue::Integer(1));
    g.add_edge(e1);

    let mut e2 = Edge::new("A", "C");
    e2.attrs.insert("weight".into(), AttrValue::Integer(5));
    g.add_edge(e2);

    let outcome = Outcome::success();
    let ctx = Context::new();
    let selected = select_edge("A", &outcome, &ctx, &g);
    assert_eq!(selected.map(|e| e.to.as_str()), Some("C"));
}

#[test]
fn edge_selection_lexical_tiebreak() {
    let mut g = Graph::new("test");
    g.add_node(Node::new("A"));
    g.add_node(Node::new("B"));
    g.add_node(Node::new("C"));

    // Same weight, different targets
    g.add_edge(Edge::new("A", "C"));
    g.add_edge(Edge::new("A", "B"));

    let outcome = Outcome::success();
    let ctx = Context::new();
    let selected = select_edge("A", &outcome, &ctx, &g);
    // B < C lexically
    assert_eq!(selected.map(|e| e.to.as_str()), Some("B"));
}

#[test]
fn edge_selection_condition_beats_label() {
    let mut g = Graph::new("test");
    g.add_node(Node::new("A"));
    g.add_node(Node::new("B"));
    g.add_node(Node::new("C"));

    // Condition edge
    let mut e1 = Edge::new("A", "B");
    e1.attrs
        .insert("condition".into(), AttrValue::from("outcome=success"));
    g.add_edge(e1);

    // Label edge
    let mut e2 = Edge::new("A", "C");
    e2.attrs
        .insert("label".into(), AttrValue::from("preferred"));
    g.add_edge(e2);

    let mut outcome = Outcome::success();
    outcome.preferred_label = "preferred".to_string();
    let ctx = Context::new();
    let selected = select_edge("A", &outcome, &ctx, &g);
    // Condition (step 1) beats label (step 2)
    assert_eq!(selected.map(|e| e.to.as_str()), Some("B"));
}

#[test]
fn edge_selection_fallback_unconditional() {
    let mut g = Graph::new("test");
    g.add_node(Node::new("A"));
    g.add_node(Node::new("B"));
    g.add_node(Node::new("C"));

    // Non-matching condition
    let mut e1 = Edge::new("A", "B");
    e1.attrs
        .insert("condition".into(), AttrValue::from("outcome=fail"));
    g.add_edge(e1);

    // Unconditional fallback
    g.add_edge(Edge::new("A", "C"));

    let outcome = Outcome::success();
    let ctx = Context::new();
    let selected = select_edge("A", &outcome, &ctx, &g);
    assert_eq!(selected.map(|e| e.to.as_str()), Some("C"));
}

#[test]
fn normalize_label_lowercases_and_trims() {
    assert_eq!(normalize_label("  Yes  "), "yes");
    assert_eq!(normalize_label("PROCEED"), "proceed");
}

#[test]
fn normalize_label_strips_accelerator_prefixes() {
    assert_eq!(normalize_label("[Y] Yes"), "yes");
    assert_eq!(normalize_label("Y) Yes"), "yes");
    assert_eq!(normalize_label("Y - Yes"), "yes");
    // Without accelerator
    assert_eq!(normalize_label("Continue"), "continue");
}

#[test]
fn edge_selection_any_edge_fallback() {
    // §3.3: when no unconditional edges exist, fallback to any edge.
    let mut g = Graph::new("test");
    g.add_node(Node::new("A"));
    g.add_node(Node::new("B"));
    g.add_node(Node::new("C"));

    // Only conditional edges, none matching
    let mut e1 = Edge::new("A", "B");
    e1.attrs
        .insert("condition".into(), AttrValue::from("outcome=fail"));
    e1.attrs.insert("weight".into(), AttrValue::Integer(5));
    g.add_edge(e1);

    let mut e2 = Edge::new("A", "C");
    e2.attrs
        .insert("condition".into(), AttrValue::from("outcome=fail"));
    g.add_edge(e2);

    let outcome = Outcome::success(); // Neither condition matches
    let ctx = Context::new();
    let selected = select_edge("A", &outcome, &ctx, &g);
    // Falls back to any edge by weight: B has weight 5
    assert_eq!(selected.map(|e| e.to.as_str()), Some("B"));
}

#[test]
fn edge_selection_preferred_label_matches_conditional_edge() {
    // §3.3 step 2: preferred label searches ALL edges, not just unconditional.
    let mut g = Graph::new("test");
    g.add_node(Node::new("A"));
    g.add_node(Node::new("B"));
    g.add_node(Node::new("C"));

    let mut e1 = Edge::new("A", "B");
    e1.attrs
        .insert("condition".into(), AttrValue::from("outcome=fail"));
    e1.attrs.insert("label".into(), AttrValue::from("Yes"));
    g.add_edge(e1);

    g.add_edge(Edge::new("A", "C")); // unconditional fallback

    let mut outcome = Outcome::success();
    outcome.preferred_label = "yes".to_string();
    let ctx = Context::new();
    let selected = select_edge("A", &outcome, &ctx, &g);
    // Preferred label match on conditional edge (step 2 searches all edges)
    assert_eq!(selected.map(|e| e.to.as_str()), Some("B"));
}

#[test]
fn best_by_weight_empty_returns_none() {
    let edges: Vec<&Edge> = vec![];
    assert!(best_by_weight_then_lexical(&edges).is_none());
}

// ===========================================================================
// Goal Gate (~5 tests)
// ===========================================================================

#[test]
fn goal_gate_all_satisfied() {
    let mut g = Graph::new("test");
    let mut n = Node::new("task1");
    n.attrs.insert("goal_gate".into(), AttrValue::Boolean(true));
    g.add_node(n);

    let mut outcomes = HashMap::new();
    outcomes.insert("task1".to_string(), Outcome::success());

    let result = check_goal_gates(&g, &outcomes);
    assert!(result.satisfied);
    assert!(result.failed_node_id.is_none());
}

#[test]
fn goal_gate_unsatisfied_when_visited_with_fail() {
    let mut g = Graph::new("test");
    let mut n = Node::new("task1");
    n.attrs.insert("goal_gate".into(), AttrValue::Boolean(true));
    g.add_node(n);

    let mut outcomes = HashMap::new();
    outcomes.insert("task1".to_string(), Outcome::fail("bad"));
    let result = check_goal_gates(&g, &outcomes);
    assert!(!result.satisfied);
    assert_eq!(result.failed_node_id.as_deref(), Some("task1"));
}

#[test]
fn goal_gate_unvisited_does_not_block() {
    // Per §3.4: only visited nodes are checked, so an unvisited
    // goal-gate node should not prevent pipeline exit.
    let mut g = Graph::new("test");
    let mut n = Node::new("task1");
    n.attrs.insert("goal_gate".into(), AttrValue::Boolean(true));
    g.add_node(n);

    let outcomes = HashMap::new(); // No outcomes recorded
    let result = check_goal_gates(&g, &outcomes);
    assert!(result.satisfied);
}

#[test]
fn goal_gate_partial_success_counts() {
    let mut g = Graph::new("test");
    let mut n = Node::new("task1");
    n.attrs.insert("goal_gate".into(), AttrValue::Boolean(true));
    g.add_node(n);

    let mut outcomes = HashMap::new();
    outcomes.insert(
        "task1".to_string(),
        Outcome {
            status: StageStatus::PartialSuccess,
            ..Outcome::success()
        },
    );

    let result = check_goal_gates(&g, &outcomes);
    assert!(result.satisfied);
}

#[test]
fn goal_gate_no_gates_is_satisfied() {
    let mut g = Graph::new("test");
    g.add_node(Node::new("task1")); // No goal_gate attr
    let outcomes = HashMap::new();
    let result = check_goal_gates(&g, &outcomes);
    assert!(result.satisfied);
}

#[test]
fn goal_gate_skipped_outcome_not_satisfied() {
    // A goal-gate node visited but with Skipped status is not satisfied.
    let mut g = Graph::new("test");
    let mut n = Node::new("task1");
    n.attrs.insert("goal_gate".into(), AttrValue::Boolean(true));
    g.add_node(n);

    let mut outcomes = HashMap::new();
    outcomes.insert(
        "task1".to_string(),
        Outcome {
            status: StageStatus::Skipped,
            ..Outcome::success()
        },
    );
    let result = check_goal_gates(&g, &outcomes);
    assert!(!result.satisfied);
}

// ===========================================================================
// Failure Routing / Retry Target (~5 tests)
// ===========================================================================

#[test]
fn retry_target_node_level() -> AttractorResult<()> {
    let mut g = Graph::new("test");
    let mut n = Node::new("task1");
    n.attrs
        .insert("retry_target".into(), AttrValue::from("start"));
    g.add_node(n);
    g.add_node(Node::new("start")); // target must exist in graph

    let node = g.get_node("task1").ok_or(AttractorError::NodeNotFound {
        node_id: "task1".into(),
    })?;
    assert_eq!(get_retry_target(node, &g).as_deref(), Some("start"));
    Ok(())
}

#[test]
fn retry_target_fallback_node_level() -> AttractorResult<()> {
    let mut g = Graph::new("test");
    let mut n = Node::new("task1");
    n.attrs.insert(
        "fallback_retry_target".into(),
        AttrValue::from("retry_node"),
    );
    g.add_node(n);
    g.add_node(Node::new("retry_node")); // target must exist in graph

    let node = g.get_node("task1").ok_or(AttractorError::NodeNotFound {
        node_id: "task1".into(),
    })?;
    assert_eq!(get_retry_target(node, &g).as_deref(), Some("retry_node"));
    Ok(())
}

#[test]
fn retry_target_graph_level() -> AttractorResult<()> {
    let mut g = Graph::new("test");
    g.graph_attrs
        .insert("retry_target".into(), AttrValue::from("global_retry"));
    let n = Node::new("task1");
    g.add_node(n);
    g.add_node(Node::new("global_retry")); // target must exist in graph

    let node = g.get_node("task1").ok_or(AttractorError::NodeNotFound {
        node_id: "task1".into(),
    })?;
    assert_eq!(get_retry_target(node, &g).as_deref(), Some("global_retry"));
    Ok(())
}

#[test]
fn retry_target_graph_fallback() -> AttractorResult<()> {
    let mut g = Graph::new("test");
    g.graph_attrs.insert(
        "fallback_retry_target".into(),
        AttrValue::from("global_fallback"),
    );
    let n = Node::new("task1");
    g.add_node(n);
    g.add_node(Node::new("global_fallback")); // target must exist in graph

    let node = g.get_node("task1").ok_or(AttractorError::NodeNotFound {
        node_id: "task1".into(),
    })?;
    assert_eq!(
        get_retry_target(node, &g).as_deref(),
        Some("global_fallback")
    );
    Ok(())
}

#[test]
fn retry_target_none_when_absent() {
    let g = Graph::new("test");
    let n = Node::new("task1");
    assert!(get_retry_target(&n, &g).is_none());
}

#[test]
fn retry_target_skips_invalid_to_next_level() -> AttractorResult<()> {
    // §3.4: if retry_target points to a non-existent node, continue
    // down the chain to fallback_retry_target.
    let mut g = Graph::new("test");
    let mut n = Node::new("task1");
    n.attrs
        .insert("retry_target".into(), AttrValue::from("nonexistent"));
    n.attrs.insert(
        "fallback_retry_target".into(),
        AttrValue::from("valid_node"),
    );
    g.add_node(n);
    g.add_node(Node::new("valid_node"));

    let node = g.get_node("task1").ok_or(AttractorError::NodeNotFound {
        node_id: "task1".into(),
    })?;
    assert_eq!(get_retry_target(node, &g).as_deref(), Some("valid_node"));
    Ok(())
}

#[test]
fn retry_target_skips_all_invalid_to_graph_level() -> AttractorResult<()> {
    // If both node-level targets are invalid, fall through to graph-level.
    let mut g = Graph::new("test");
    let mut n = Node::new("task1");
    n.attrs
        .insert("retry_target".into(), AttrValue::from("missing1"));
    n.attrs
        .insert("fallback_retry_target".into(), AttrValue::from("missing2"));
    g.add_node(n);

    g.graph_attrs
        .insert("retry_target".into(), AttrValue::from("graph_target"));
    g.add_node(Node::new("graph_target"));

    let node = g.get_node("task1").ok_or(AttractorError::NodeNotFound {
        node_id: "task1".into(),
    })?;
    assert_eq!(get_retry_target(node, &g).as_deref(), Some("graph_target"));
    Ok(())
}

// ===========================================================================
// Retry (~8 tests)
// ===========================================================================

#[test]
fn retry_preset_none_policy() {
    let policy = RetryPreset::None.to_policy();
    assert_eq!(policy.max_attempts, 1);
}

#[test]
fn retry_preset_standard_policy() {
    let policy = RetryPreset::Standard.to_policy();
    assert_eq!(policy.max_attempts, 5);
    assert_eq!(policy.backoff.initial_delay_ms, 200);
    assert!((policy.backoff.backoff_factor - 2.0).abs() < f64::EPSILON);
    assert!(policy.backoff.jitter);
}

#[test]
fn retry_preset_aggressive_policy() {
    let policy = RetryPreset::Aggressive.to_policy();
    assert_eq!(policy.max_attempts, 5);
    assert_eq!(policy.backoff.initial_delay_ms, 500);
    assert!((policy.backoff.backoff_factor - 2.0).abs() < f64::EPSILON);
    assert!(policy.backoff.jitter);
}

#[test]
fn retry_preset_linear_policy() {
    let policy = RetryPreset::Linear.to_policy();
    assert_eq!(policy.max_attempts, 3);
    assert_eq!(policy.backoff.initial_delay_ms, 500);
    assert!((policy.backoff.backoff_factor - 1.0).abs() < f64::EPSILON);
    assert!(!policy.backoff.jitter);
}

#[test]
fn retry_preset_patient_policy() {
    let policy = RetryPreset::Patient.to_policy();
    assert_eq!(policy.max_attempts, 3);
    assert_eq!(policy.backoff.initial_delay_ms, 2000);
    assert!((policy.backoff.backoff_factor - 3.0).abs() < f64::EPSILON);
    assert!(policy.backoff.jitter);
}

#[test]
fn retry_backoff_formula() {
    let config = BackoffConfig {
        initial_delay_ms: 1000,
        backoff_factor: 2.0,
        max_delay_ms: 60_000,
        jitter: false,
    };
    // attempt 1: 1000 * 2^0 = 1000ms
    let d1 = delay_for_attempt(1, &config);
    assert_eq!(d1.as_millis(), 1000);

    // attempt 2: 1000 * 2^1 = 2000ms
    let d2 = delay_for_attempt(2, &config);
    assert_eq!(d2.as_millis(), 2000);

    // attempt 3: 1000 * 2^2 = 4000ms
    let d3 = delay_for_attempt(3, &config);
    assert_eq!(d3.as_millis(), 4000);
}

#[test]
fn retry_max_delay_cap() {
    let config = BackoffConfig {
        initial_delay_ms: 1000,
        backoff_factor: 10.0,
        max_delay_ms: 5000,
        jitter: false,
    };
    // attempt 3: 1000 * 10^2 = 100_000, capped to 5000
    let d = delay_for_attempt(3, &config);
    assert_eq!(d.as_millis(), 5000);
}

#[test]
fn retry_jitter_range() {
    let config = BackoffConfig {
        initial_delay_ms: 1000,
        backoff_factor: 1.0,
        max_delay_ms: 60_000,
        jitter: true,
    };
    // With jitter, delay should be in [500, 1500)
    for _ in 0..20 {
        let d = delay_for_attempt(1, &config);
        let ms = d.as_millis();
        assert!(ms >= 500, "jitter too low: {ms}");
        assert!(ms < 1500, "jitter too high: {ms}");
    }
}

#[test]
fn build_retry_policy_from_node() -> AttractorResult<()> {
    let mut g = Graph::new("test");
    let mut n = Node::new("task1");
    n.attrs.insert("max_retries".into(), AttrValue::Integer(3));
    g.add_node(n);

    let node = g.get_node("task1").ok_or(AttractorError::NodeNotFound {
        node_id: "task1".into(),
    })?;
    let policy = build_retry_policy(node, &g);
    assert_eq!(policy.max_attempts, 4); // 3 retries + 1 initial
    Ok(())
}

#[test]
fn build_retry_policy_from_graph_default() -> AttractorResult<()> {
    let mut g = Graph::new("test");
    g.graph_attrs
        .insert("default_max_retry".into(), AttrValue::Integer(2));
    let n = Node::new("task1");
    g.add_node(n);

    let node = g.get_node("task1").ok_or(AttractorError::NodeNotFound {
        node_id: "task1".into(),
    })?;
    let policy = build_retry_policy(node, &g);
    assert_eq!(policy.max_attempts, 3);
    Ok(())
}

#[test]
fn build_retry_policy_defaults_to_zero() {
    let g = Graph::new("test");
    let n = Node::new("task1");
    let policy = build_retry_policy(&n, &g);
    assert_eq!(policy.max_attempts, 1);
}

#[tokio::test]
async fn retry_success_on_second_attempt() {
    let handler: Arc<dyn Handler> = Arc::new(SequenceHandler::new(vec![
        Outcome::retry("first attempt"),
        Outcome::success(),
    ]));
    let node = Node::new("test");
    let ctx = Context::new();
    let g = Graph::new("test");
    let policy = RetryPolicy {
        max_attempts: 3,
        backoff: BackoffConfig {
            initial_delay_ms: 1,
            backoff_factor: 1.0,
            max_delay_ms: 10,
            jitter: false,
        },
    };

    let outcome = execute_with_retry(
        &handler,
        &node,
        &ctx,
        &g,
        Path::new("/tmp"),
        &policy,
        &NoOpEmitter,
        0,
    )
    .await;

    assert_eq!(outcome.status, StageStatus::Success);
}

#[tokio::test]
async fn retry_exhausted_returns_fail() {
    let handler: Arc<dyn Handler> = Arc::new(MockHandler::new(Outcome::retry("always retry")));
    let node = Node::new("test");
    let ctx = Context::new();
    let g = Graph::new("test");
    let policy = RetryPolicy {
        max_attempts: 2,
        backoff: BackoffConfig {
            initial_delay_ms: 1,
            backoff_factor: 1.0,
            max_delay_ms: 10,
            jitter: false,
        },
    };

    let outcome = execute_with_retry(
        &handler,
        &node,
        &ctx,
        &g,
        Path::new("/tmp"),
        &policy,
        &NoOpEmitter,
        0,
    )
    .await;

    assert_eq!(outcome.status, StageStatus::Fail);
}

#[tokio::test]
async fn retry_exhausted_allow_partial() {
    let handler: Arc<dyn Handler> = Arc::new(MockHandler::new(Outcome::retry("always retry")));
    let mut node = Node::new("test");
    node.attrs
        .insert("allow_partial".into(), AttrValue::Boolean(true));
    let ctx = Context::new();
    let g = Graph::new("test");
    let policy = RetryPolicy {
        max_attempts: 2,
        backoff: BackoffConfig {
            initial_delay_ms: 1,
            backoff_factor: 1.0,
            max_delay_ms: 10,
            jitter: false,
        },
    };

    let outcome = execute_with_retry(
        &handler,
        &node,
        &ctx,
        &g,
        Path::new("/tmp"),
        &policy,
        &NoOpEmitter,
        0,
    )
    .await;

    assert_eq!(outcome.status, StageStatus::PartialSuccess);
}

#[tokio::test]
async fn retry_sets_context_key() {
    let handler: Arc<dyn Handler> = Arc::new(SequenceHandler::new(vec![
        Outcome::retry("first"),
        Outcome::success(),
    ]));
    let node = Node::new("mynode");
    let ctx = Context::new();
    let g = Graph::new("test");
    let policy = RetryPolicy {
        max_attempts: 3,
        backoff: BackoffConfig {
            initial_delay_ms: 1,
            backoff_factor: 1.0,
            max_delay_ms: 10,
            jitter: false,
        },
    };

    execute_with_retry(
        &handler,
        &node,
        &ctx,
        &g,
        Path::new("/tmp"),
        &policy,
        &NoOpEmitter,
        0,
    )
    .await;

    let retry_count = ctx.get_i64("internal.retry_count.mynode");
    assert_eq!(retry_count, Some(1));
}

#[tokio::test]
async fn retry_panic_converted_to_fail() {
    let handler: Arc<dyn Handler> = Arc::new(PanicHandler);
    let node = Node::new("test");
    let ctx = Context::new();
    let g = Graph::new("test");
    let policy = RetryPolicy {
        max_attempts: 1,
        backoff: BackoffConfig {
            initial_delay_ms: 1,
            backoff_factor: 1.0,
            max_delay_ms: 10,
            jitter: false,
        },
    };

    let outcome = execute_with_retry(
        &handler,
        &node,
        &ctx,
        &g,
        Path::new("/tmp"),
        &policy,
        &NoOpEmitter,
        0,
    )
    .await;

    assert_eq!(outcome.status, StageStatus::Fail);
    assert!(outcome.failure_reason.contains("panic"));
}

#[tokio::test]
async fn retry_retryable_error_retries() {
    let handler: Arc<dyn Handler> = Arc::new(RetryableErrorHandler);
    let node = Node::new("test");
    let ctx = Context::new();
    let g = Graph::new("test");
    let policy = RetryPolicy {
        max_attempts: 2,
        backoff: BackoffConfig {
            initial_delay_ms: 1,
            backoff_factor: 1.0,
            max_delay_ms: 10,
            jitter: false,
        },
    };

    let outcome = execute_with_retry(
        &handler,
        &node,
        &ctx,
        &g,
        Path::new("/tmp"),
        &policy,
        &NoOpEmitter,
        0,
    )
    .await;

    // After 2 attempts, should fail
    assert_eq!(outcome.status, StageStatus::Fail);
    assert!(outcome.failure_reason.contains("handler error"));
}

// ===========================================================================
// Handler Registry (~3 tests)
// ===========================================================================

#[test]
fn registry_resolve_by_type() {
    let registry = HandlerRegistry::with_defaults();
    let mut node = Node::new("s");
    node.attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    assert!(registry.resolve(&node).is_some());
}

#[test]
fn registry_resolve_by_shape_with_default_fallback() {
    let mut registry = HandlerRegistry::new();
    registry.set_default(MockHandler::new(Outcome::success()));

    // "box" shape → "codergen" handler type, not registered but default exists
    let node = Node::new("task");
    assert!(registry.resolve(&node).is_some());
}

#[test]
fn registry_register_replaces() {
    let mut registry = HandlerRegistry::with_defaults();
    // Re-register "start" with a custom handler
    registry.register("start", MockHandler::new(Outcome::fail("custom")));
    let mut node = Node::new("s");
    node.attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    assert!(registry.resolve(&node).is_some());
}

#[test]
fn registry_resolve_none_when_no_match() {
    let registry = HandlerRegistry::new(); // Empty, no default
    let node = Node::new("task");
    assert!(registry.resolve(&node).is_none());
}

// ===========================================================================
// Run Directory (~3 tests)
// ===========================================================================

#[test]
fn run_directory_create_and_manifest() -> AttractorResult<()> {
    let tmp = tempfile::tempdir().map_err(|e| AttractorError::Io {
        message: e.to_string(),
    })?;
    let run_dir = RunDirectory::create(tmp.path().join("run1"))?;

    let manifest = Manifest {
        name: "test".into(),
        goal: "test goal".into(),
        start_time: "2024-01-01T00:00:00Z".into(),
    };
    run_dir.write_manifest(&manifest)?;

    let data =
        std::fs::read_to_string(run_dir.manifest_path()).map_err(|e| AttractorError::Io {
            message: e.to_string(),
        })?;
    assert!(data.contains("test goal"));
    Ok(())
}

#[test]
fn run_directory_status_roundtrip() -> AttractorResult<()> {
    let tmp = tempfile::tempdir().map_err(|e| AttractorError::Io {
        message: e.to_string(),
    })?;
    let run_dir = RunDirectory::create(tmp.path().join("run1"))?;

    let outcome = Outcome::success();
    run_dir.write_status("node1", &outcome)?;

    let loaded = run_dir.read_status("node1")?;
    assert_eq!(loaded.status, StageStatus::Success);
    Ok(())
}

#[test]
fn run_directory_path_helpers() -> AttractorResult<()> {
    let tmp = tempfile::tempdir().map_err(|e| AttractorError::Io {
        message: e.to_string(),
    })?;
    let run_dir = RunDirectory::create(tmp.path().join("run1"))?;

    assert!(run_dir.manifest_path().ends_with("manifest.json"));
    assert!(run_dir.checkpoint_path().ends_with("checkpoint.json"));
    assert!(run_dir.node_dir("task1").ends_with("nodes/task1"));
    assert!(
        run_dir
            .status_path("task1")
            .ends_with("nodes/task1/status.json")
    );
    Ok(())
}

// ===========================================================================
// Engine Loop (~14 tests)
// ===========================================================================

#[tokio::test]
async fn engine_linear_start_exit() -> AttractorResult<()> {
    let tmp = tempfile::tempdir().map_err(|e| AttractorError::Io {
        message: e.to_string(),
    })?;
    let g = linear_graph();
    let config = EngineConfig::new(tmp.path());

    let outcome = engine::run(&g, config).await?;
    assert_eq!(outcome.status, StageStatus::Success);
    Ok(())
}

#[tokio::test]
async fn engine_three_node_linear() -> AttractorResult<()> {
    let tmp = tempfile::tempdir().map_err(|e| AttractorError::Io {
        message: e.to_string(),
    })?;
    let g = three_node_graph();
    let mut config = EngineConfig::new(tmp.path());
    // middle node has shape "box" → handler type "codergen", need to register
    config
        .registry
        .register("codergen", MockHandler::new(Outcome::success()));

    let outcome = engine::run(&g, config).await?;
    assert_eq!(outcome.status, StageStatus::Success);
    Ok(())
}

#[tokio::test]
async fn engine_conditional_success_path() -> AttractorResult<()> {
    let tmp = tempfile::tempdir().map_err(|e| AttractorError::Io {
        message: e.to_string(),
    })?;

    let mut g = Graph::new("cond_test");
    let mut start = Node::new("start");
    start
        .attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);

    let mut cond = Node::new("check");
    cond.attrs
        .insert("shape".into(), AttrValue::from("diamond"));
    g.add_node(cond);

    let mut good = Node::new("good");
    good.attrs
        .insert("shape".into(), AttrValue::from("Msquare"));
    g.add_node(good);

    g.add_node(Node::new("bad"));

    g.add_edge(Edge::new("start", "check"));

    let mut e_good = Edge::new("check", "good");
    e_good
        .attrs
        .insert("condition".into(), AttrValue::from("outcome=success"));
    g.add_edge(e_good);

    let mut e_bad = Edge::new("check", "bad");
    e_bad
        .attrs
        .insert("condition".into(), AttrValue::from("outcome=fail"));
    g.add_edge(e_bad);

    let config = EngineConfig::new(tmp.path());
    let outcome = engine::run(&g, config).await?;
    assert_eq!(outcome.status, StageStatus::Success);
    Ok(())
}

#[tokio::test]
async fn engine_dead_end_success_completes_normally() -> AttractorResult<()> {
    // §3.2 step 6: no outgoing edges + SUCCESS → BREAK (normal completion)
    let tmp = tempfile::tempdir().map_err(|e| AttractorError::Io {
        message: e.to_string(),
    })?;

    let mut g = Graph::new("dead_end");
    let mut start = Node::new("start");
    start
        .attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);

    let dead = Node::new("dead");
    g.add_node(dead);

    let mut exit = Node::new("exit");
    exit.attrs
        .insert("shape".into(), AttrValue::from("Msquare"));
    g.add_node(exit);

    g.add_edge(Edge::new("start", "dead"));
    // dead has no outgoing edges

    let mut config = EngineConfig::new(tmp.path());
    config
        .registry
        .register("codergen", MockHandler::new(Outcome::success()));

    let outcome = engine::run(&g, config).await?;
    // Pipeline ends without reaching exit, returns last outcome (SUCCESS)
    assert_eq!(outcome.status, StageStatus::Success);
    Ok(())
}

#[tokio::test]
async fn engine_dead_end_fail_returns_fail() -> AttractorResult<()> {
    // §3.2 step 6: FAIL + no outgoing fail edge → pipeline fails.
    let tmp = tempfile::tempdir().map_err(|e| AttractorError::Io {
        message: e.to_string(),
    })?;

    let mut g = Graph::new("dead_end_fail");
    let mut start = Node::new("start");
    start
        .attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);

    let dead = Node::new("dead");
    g.add_node(dead);

    let mut exit = Node::new("exit");
    exit.attrs
        .insert("shape".into(), AttrValue::from("Msquare"));
    g.add_node(exit);

    g.add_edge(Edge::new("start", "dead"));

    let mut config = EngineConfig::new(tmp.path());
    config
        .registry
        .register("codergen", MockHandler::new(Outcome::fail("task failed")));

    let outcome = engine::run(&g, config).await?;
    assert_eq!(outcome.status, StageStatus::Fail);
    Ok(())
}

#[tokio::test]
async fn engine_missing_exit_node_is_error() -> AttractorResult<()> {
    // run() should error when graph has no exit node.
    let tmp = tempfile::tempdir().map_err(|e| AttractorError::Io {
        message: e.to_string(),
    })?;

    let mut g = Graph::new("no_exit");
    let mut start = Node::new("start");
    start
        .attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);
    g.add_node(Node::new("middle"));
    g.add_edge(Edge::new("start", "middle"));

    let config = EngineConfig::new(tmp.path());
    let result = engine::run(&g, config).await;
    assert!(result.is_err(), "missing exit node should be an error");
    Ok(())
}

#[tokio::test]
async fn engine_context_propagation() -> AttractorResult<()> {
    let tmp = tempfile::tempdir().map_err(|e| AttractorError::Io {
        message: e.to_string(),
    })?;

    let mut g = Graph::new("ctx_test");
    let mut start = Node::new("start");
    start
        .attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);

    let middle = Node::new("middle");
    g.add_node(middle);

    let mut exit = Node::new("exit");
    exit.attrs
        .insert("shape".into(), AttrValue::from("Msquare"));
    g.add_node(exit);

    g.add_edge(Edge::new("start", "middle"));
    g.add_edge(Edge::new("middle", "exit"));

    // Middle handler sets a context update
    let mut outcome = Outcome::success();
    outcome
        .context_updates
        .insert("result".to_string(), Value::String("computed".into()));

    let mut config = EngineConfig::new(tmp.path());
    config
        .registry
        .register("codergen", MockHandler::new(outcome));

    let result = engine::run(&g, config).await?;
    assert_eq!(result.status, StageStatus::Success);
    Ok(())
}

#[tokio::test]
async fn engine_checkpoint_per_node() -> AttractorResult<()> {
    let tmp = tempfile::tempdir().map_err(|e| AttractorError::Io {
        message: e.to_string(),
    })?;

    let g = three_node_graph();
    let mut config = EngineConfig::new(tmp.path());
    config
        .registry
        .register("codergen", MockHandler::new(Outcome::success()));

    engine::run(&g, config).await?;

    // Find the run directory (should be the only subdir)
    let entries: Vec<_> = std::fs::read_dir(tmp.path())
        .map_err(|e| AttractorError::Io {
            message: e.to_string(),
        })?
        .filter_map(|e| e.ok())
        .collect();
    assert!(!entries.is_empty(), "run directory should exist");

    let run_dir = &entries[0].path();
    let checkpoint_path = run_dir.join("checkpoint.json");
    assert!(checkpoint_path.exists(), "checkpoint.json should exist");

    let checkpoint = Checkpoint::load(&checkpoint_path)?;
    assert!(
        checkpoint.completed_nodes.contains(&"start".to_string()),
        "start should be in completed_nodes"
    );
    assert!(
        checkpoint.completed_nodes.contains(&"middle".to_string()),
        "middle should be in completed_nodes"
    );
    Ok(())
}

#[tokio::test]
async fn engine_checkpoint_includes_retry_counts() -> AttractorResult<()> {
    // §5.3: checkpoint node_retries should reflect actual retry attempts.
    let tmp = tempfile::tempdir().map_err(|e| AttractorError::Io {
        message: e.to_string(),
    })?;

    let mut g = Graph::new("retry_checkpoint");
    let mut start = Node::new("start");
    start
        .attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);

    let mut task = Node::new("task");
    task.attrs
        .insert("max_retries".into(), AttrValue::Integer(2));
    g.add_node(task);

    let mut exit = Node::new("exit");
    exit.attrs
        .insert("shape".into(), AttrValue::from("Msquare"));
    g.add_node(exit);

    g.add_edge(Edge::new("start", "task"));
    g.add_edge(Edge::new("task", "exit"));

    // First attempt returns RETRY, second returns SUCCESS
    let seq = SequenceHandler::new(vec![Outcome::retry("first attempt"), Outcome::success()]);
    let mut config = EngineConfig::new(tmp.path());
    config.registry.register("codergen", seq);

    engine::run(&g, config).await?;

    let entries: Vec<_> = std::fs::read_dir(tmp.path())
        .map_err(|e| AttractorError::Io {
            message: e.to_string(),
        })?
        .filter_map(|e| e.ok())
        .collect();
    let run_dir = &entries[0].path();
    let checkpoint = Checkpoint::load(&run_dir.join("checkpoint.json"))?;

    // task had 1 retry (attempt count stored in context)
    let retry_count = checkpoint.node_retries.get("task").copied();
    assert_eq!(
        retry_count,
        Some(1),
        "checkpoint should record retry count for task"
    );
    Ok(())
}

#[tokio::test]
async fn engine_run_dir_and_manifest() -> AttractorResult<()> {
    let tmp = tempfile::tempdir().map_err(|e| AttractorError::Io {
        message: e.to_string(),
    })?;

    let g = linear_graph();
    let config = EngineConfig::new(tmp.path());

    engine::run(&g, config).await?;

    let entries: Vec<_> = std::fs::read_dir(tmp.path())
        .map_err(|e| AttractorError::Io {
            message: e.to_string(),
        })?
        .filter_map(|e| e.ok())
        .collect();
    assert!(!entries.is_empty());

    let run_dir = &entries[0].path();
    let manifest_path = run_dir.join("manifest.json");
    assert!(manifest_path.exists());

    let manifest_data =
        std::fs::read_to_string(&manifest_path).map_err(|e| AttractorError::Io {
            message: e.to_string(),
        })?;
    assert!(manifest_data.contains("linear"));
    Ok(())
}

#[tokio::test]
async fn engine_status_json_per_node() -> AttractorResult<()> {
    let tmp = tempfile::tempdir().map_err(|e| AttractorError::Io {
        message: e.to_string(),
    })?;

    let g = three_node_graph();
    let mut config = EngineConfig::new(tmp.path());
    config
        .registry
        .register("codergen", MockHandler::new(Outcome::success()));

    engine::run(&g, config).await?;

    let entries: Vec<_> = std::fs::read_dir(tmp.path())
        .map_err(|e| AttractorError::Io {
            message: e.to_string(),
        })?
        .filter_map(|e| e.ok())
        .collect();
    let run_dir = &entries[0].path();

    // Check start status
    let start_status = run_dir.join("nodes/start/status.json");
    assert!(start_status.exists(), "start status.json should exist");

    // Check middle status
    let middle_status = run_dir.join("nodes/middle/status.json");
    assert!(middle_status.exists(), "middle status.json should exist");
    Ok(())
}

#[tokio::test]
async fn engine_panic_handler_fails() -> AttractorResult<()> {
    let tmp = tempfile::tempdir().map_err(|e| AttractorError::Io {
        message: e.to_string(),
    })?;

    let mut g = Graph::new("panic_test");
    let mut start = Node::new("start");
    start
        .attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);

    let middle = Node::new("middle");
    g.add_node(middle);

    let mut exit = Node::new("exit");
    exit.attrs
        .insert("shape".into(), AttrValue::from("Msquare"));
    g.add_node(exit);

    g.add_edge(Edge::new("start", "middle"));
    g.add_edge(Edge::new("middle", "exit"));

    let mut config = EngineConfig::new(tmp.path());
    config.registry.register("codergen", PanicHandler);

    let outcome = engine::run(&g, config).await?;
    // Panic is caught and converted to FAIL
    assert_eq!(outcome.status, StageStatus::Fail);
    Ok(())
}

#[tokio::test]
async fn engine_graph_attr_mirroring() -> AttractorResult<()> {
    let tmp = tempfile::tempdir().map_err(|e| AttractorError::Io {
        message: e.to_string(),
    })?;

    let mut g = linear_graph();
    g.graph_attrs
        .insert("goal".into(), AttrValue::from("Build something"));

    let config = EngineConfig::new(tmp.path());
    let outcome = engine::run(&g, config).await?;
    assert_eq!(outcome.status, StageStatus::Success);
    Ok(())
}

#[tokio::test]
async fn engine_loop_restart() -> AttractorResult<()> {
    // §2.7/§3.2: loop_restart creates a fresh run directory and context.
    let tmp = tempfile::tempdir().map_err(|e| AttractorError::Io {
        message: e.to_string(),
    })?;

    let mut g = Graph::new("loop_test");
    let mut start = Node::new("start");
    start
        .attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);

    let check = Node::new("check");
    g.add_node(check);

    let mut exit = Node::new("exit");
    exit.attrs
        .insert("shape".into(), AttrValue::from("Msquare"));
    g.add_node(exit);

    g.add_edge(Edge::new("start", "check"));

    // First iteration: check returns "retry" label → loops back to start
    // Second iteration: check returns "done" label → goes to exit
    let mut e_loop = Edge::new("check", "start");
    e_loop
        .attrs
        .insert("label".into(), AttrValue::from("retry"));
    e_loop
        .attrs
        .insert("loop_restart".into(), AttrValue::Boolean(true));
    g.add_edge(e_loop);

    let mut e_exit = Edge::new("check", "exit");
    e_exit.attrs.insert("label".into(), AttrValue::from("done"));
    g.add_edge(e_exit);

    // Use a sequence handler: first call returns "retry", second returns "done"
    let seq_handler = SequenceHandler::new(vec![
        {
            let mut o = Outcome::success();
            o.preferred_label = "retry".to_string();
            o
        },
        {
            let mut o = Outcome::success();
            o.preferred_label = "done".to_string();
            o
        },
    ]);

    let mut config = EngineConfig::new(tmp.path());
    config.registry.register("codergen", seq_handler);

    let outcome = engine::run(&g, config).await?;
    assert_eq!(outcome.status, StageStatus::Success);

    // Verify two run directories were created (one per run)
    let entries: Vec<_> = std::fs::read_dir(tmp.path())
        .map_err(|e| AttractorError::Io {
            message: e.to_string(),
        })?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .collect();
    assert!(
        entries.len() >= 2,
        "loop_restart should create a fresh run directory; found {}",
        entries.len()
    );
    Ok(())
}

#[tokio::test]
async fn engine_event_emission() -> AttractorResult<()> {
    let tmp = tempfile::tempdir().map_err(|e| AttractorError::Io {
        message: e.to_string(),
    })?;

    let g = linear_graph();
    let emitter = Arc::new(RecordingEmitter::default());
    let mut config = EngineConfig::new(tmp.path());
    config.emitter = emitter.clone();

    engine::run(&g, config).await?;

    let events = emitter.event_names();
    assert!(events.contains(&"PipelineStarted".to_string()));
    assert!(events.contains(&"StageStarted".to_string()));
    assert!(events.contains(&"CheckpointSaved".to_string()));
    assert!(events.contains(&"PipelineCompleted".to_string()));
    Ok(())
}

#[tokio::test]
async fn engine_unvisited_goal_gate_does_not_block() -> AttractorResult<()> {
    // §3.4: only visited nodes are checked, so an unvisited goal-gate
    // node should not prevent pipeline exit.
    let tmp = tempfile::tempdir().map_err(|e| AttractorError::Io {
        message: e.to_string(),
    })?;

    let mut g = Graph::new("gate_test");
    let mut start = Node::new("start");
    start
        .attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);

    // A goal gate node that we skip over (no edge leads to it)
    let mut gate = Node::new("important_task");
    gate.attrs
        .insert("goal_gate".into(), AttrValue::Boolean(true));
    g.add_node(gate);

    let mut exit = Node::new("exit");
    exit.attrs
        .insert("shape".into(), AttrValue::from("Msquare"));
    g.add_node(exit);

    g.add_edge(Edge::new("start", "exit"));

    let config = EngineConfig::new(tmp.path());
    let outcome = engine::run(&g, config).await?;
    // Unvisited goal gate does not block exit
    assert_eq!(outcome.status, StageStatus::Success);
    Ok(())
}

#[tokio::test]
async fn engine_visited_goal_gate_unsatisfied_fails() -> AttractorResult<()> {
    // §3.4: a visited goal-gate node with non-success outcome blocks exit.
    let tmp = tempfile::tempdir().map_err(|e| AttractorError::Io {
        message: e.to_string(),
    })?;

    let mut g = Graph::new("gate_test");
    let mut start = Node::new("start");
    start
        .attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);

    // A goal-gate node on the main path that fails
    let mut gate = Node::new("important_task");
    gate.attrs
        .insert("goal_gate".into(), AttrValue::Boolean(true));
    g.add_node(gate);

    let mut exit = Node::new("exit");
    exit.attrs
        .insert("shape".into(), AttrValue::from("Msquare"));
    g.add_node(exit);

    g.add_edge(Edge::new("start", "important_task"));
    g.add_edge(Edge::new("important_task", "exit"));

    let mut config = EngineConfig::new(tmp.path());
    config
        .registry
        .register("codergen", MockHandler::new(Outcome::fail("task failed")));

    let outcome = engine::run(&g, config).await?;
    // Visited goal-gate node with Fail status blocks exit via failure routing
    assert_eq!(outcome.status, StageStatus::Fail);
    Ok(())
}

#[tokio::test]
async fn engine_goal_gate_with_retry_target() -> AttractorResult<()> {
    // §3.4: a visited goal-gate node with fail outcome triggers retry_target
    // when the pipeline reaches exit.
    let tmp = tempfile::tempdir().map_err(|e| AttractorError::Io {
        message: e.to_string(),
    })?;

    let mut g = Graph::new("gate_retry_test");
    let mut start = Node::new("start");
    start
        .attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);

    // Goal gate node on the path — fails first, retry_target points to itself
    let mut gate = Node::new("gate_task");
    gate.attrs
        .insert("goal_gate".into(), AttrValue::Boolean(true));
    gate.attrs
        .insert("retry_target".into(), AttrValue::from("gate_task"));
    g.add_node(gate);

    let mut exit = Node::new("exit");
    exit.attrs
        .insert("shape".into(), AttrValue::from("Msquare"));
    g.add_node(exit);

    g.add_edge(Edge::new("start", "gate_task"));
    g.add_edge(Edge::new("gate_task", "exit"));

    // First call: fail (recorded as visited with fail status, triggers failure routing)
    // But wait — the fail status triggers route_failure before we even reach exit.
    // Instead, use a PartialSuccess-like status that isn't quite success.
    // Actually, let's use a sequence: first returns Skipped (not success), second returns Success.
    let seq = SequenceHandler::new(vec![
        Outcome {
            status: StageStatus::Skipped,
            ..Outcome::success()
        },
        Outcome::success(),
    ]);

    let mut config = EngineConfig::new(tmp.path());
    config.registry.register("codergen", seq);

    let outcome = engine::run(&g, config).await?;
    // gate_task returns Skipped → reaches exit → goal gate check finds Skipped
    // → retry to gate_task → gate_task returns Success → reaches exit → passes
    assert_eq!(outcome.status, StageStatus::Success);
    Ok(())
}

#[tokio::test]
async fn engine_failure_routing_via_fail_edge() -> AttractorResult<()> {
    let tmp = tempfile::tempdir().map_err(|e| AttractorError::Io {
        message: e.to_string(),
    })?;

    let mut g = Graph::new("fail_edge_test");
    let mut start = Node::new("start");
    start
        .attrs
        .insert("shape".into(), AttrValue::from("Mdiamond"));
    g.add_node(start);

    let task = Node::new("task");
    g.add_node(task);

    let recovery = Node::new("recovery");
    g.add_node(recovery);

    let mut exit = Node::new("exit");
    exit.attrs
        .insert("shape".into(), AttrValue::from("Msquare"));
    g.add_node(exit);

    g.add_edge(Edge::new("start", "task"));

    // Fail edge from task to recovery
    let mut fail_edge = Edge::new("task", "recovery");
    fail_edge
        .attrs
        .insert("condition".into(), AttrValue::from("outcome=fail"));
    g.add_edge(fail_edge);

    g.add_edge(Edge::new("task", "exit"));
    g.add_edge(Edge::new("recovery", "exit"));

    // Both task and recovery are "codergen" type — use a sequence handler
    // that fails first (task), then succeeds (recovery)
    let mut config = EngineConfig::new(tmp.path());
    config.registry.register(
        "codergen",
        SequenceHandler::new(vec![Outcome::fail("task failed"), Outcome::success()]),
    );

    let outcome = engine::run(&g, config).await?;
    assert_eq!(outcome.status, StageStatus::Success);
    Ok(())
}
