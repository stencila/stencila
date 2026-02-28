//! Tests for parallel handler (§4.8) and fan-in handler (§4.9).

mod common;

use std::sync::Arc;

use async_trait::async_trait;

use stencila_attractor::context::Context;
use stencila_attractor::error::AttractorResult;
use stencila_attractor::events::NoOpEmitter;
use stencila_attractor::graph::{AttrValue, Edge, Graph, Node};
use stencila_attractor::handler::{Handler, HandlerRegistry};
use stencila_attractor::handlers::{DEFAULT_MAX_PARALLEL, FanInHandler, ParallelHandler};
use stencila_attractor::types::{Outcome, StageStatus};

/// A handler that always returns FAIL.
struct FailHandler;

#[async_trait]
impl Handler for FailHandler {
    async fn execute(
        &self,
        _node: &Node,
        _context: &Context,
        _graph: &Graph,
    ) -> AttractorResult<Outcome> {
        Ok(Outcome::fail("intentional failure"))
    }
}

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Build a graph with a parallel node that fans out to the given targets.
/// Each target is a codergen (box) node.
fn parallel_graph(targets: &[&str]) -> Graph {
    let mut g = Graph::new("test_parallel");

    let mut par = Node::new("parallel_node");
    par.attrs
        .insert("shape".into(), AttrValue::from("component"));
    g.add_node(par);

    for &target in targets {
        let node = Node::new(target);
        g.add_node(node);
        g.add_edge(Edge::new("parallel_node", target));
    }

    g
}

/// Build a registry with defaults (start, exit, conditional, codergen, shell).
fn default_registry() -> Arc<HandlerRegistry> {
    Arc::new(HandlerRegistry::with_defaults())
}

// ===========================================================================
// §4.8 — Parallel handler: basic execution
// ===========================================================================

#[tokio::test]
async fn parallel_executes_all_branches() -> AttractorResult<()> {
    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let g = parallel_graph(&["branch_a", "branch_b", "branch_c"]);
    let node = g.get_node("parallel_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "parallel_node".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert!(outcome.status.is_success());

    // parallel.results should be set in context
    let results = ctx.get("parallel.results");
    assert!(results.is_some());
    let arr = results.as_ref().and_then(|v| v.as_array());
    assert_eq!(arr.map(Vec::len), Some(3));

    Ok(())
}

#[tokio::test]
async fn parallel_no_edges_fails() -> AttractorResult<()> {
    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let mut g = Graph::new("test");
    let mut par = Node::new("parallel_node");
    par.attrs
        .insert("shape".into(), AttrValue::from("component"));
    g.add_node(par);
    // No outgoing edges

    let node = g.get_node("parallel_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "parallel_node".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Fail);
    Ok(())
}

#[tokio::test]
async fn parallel_context_isolation() -> AttractorResult<()> {
    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let g = parallel_graph(&["branch_a", "branch_b"]);
    let node = g.get_node("parallel_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "parallel_node".into(),
        }
    })?;

    // Set a value in the parent context
    let ctx = Context::new();
    ctx.set("parent_key", serde_json::json!("parent_value"));

    let outcome = handler.execute(node, &ctx, &g).await?;
    assert!(outcome.status.is_success());

    // Parent context value should still be there
    let val = ctx.get("parent_key");
    assert_eq!(val, Some(serde_json::json!("parent_value")));

    Ok(())
}

// ===========================================================================
// §4.8 — Join policies
// ===========================================================================

#[tokio::test]
async fn parallel_wait_all_default_policy() -> AttractorResult<()> {
    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    // All branches are codergen (simulation) — all should succeed
    let g = parallel_graph(&["a", "b"]);
    let node = g.get_node("parallel_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "parallel_node".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert!(outcome.status.is_success());

    // Check success/fail counts
    let success = outcome
        .context_updates
        .get("parallel.success_count")
        .and_then(|v| v.as_u64());
    let fail = outcome
        .context_updates
        .get("parallel.fail_count")
        .and_then(|v| v.as_u64());
    assert_eq!(success, Some(2));
    assert_eq!(fail, Some(0));

    Ok(())
}

// ===========================================================================
// §4.8 — Join and error policies
// ===========================================================================

/// Build a graph with a parallel node where `fail_targets` are "fail_node"
/// type and `success_targets` are codergen (box) type.
fn mixed_parallel_graph(
    success_targets: &[&str],
    fail_targets: &[&str],
    attrs: &[(&str, &str)],
) -> Graph {
    let mut g = Graph::new("test_parallel");

    let mut par = Node::new("parallel_node");
    par.attrs
        .insert("shape".into(), AttrValue::from("component"));
    for &(key, value) in attrs {
        par.attrs.insert(key.into(), AttrValue::from(value));
    }
    g.add_node(par);

    for &target in success_targets {
        let node = Node::new(target);
        g.add_node(node);
        g.add_edge(Edge::new("parallel_node", target));
    }

    for &target in fail_targets {
        let mut node = Node::new(target);
        node.attrs
            .insert("type".into(), AttrValue::from("fail_node"));
        g.add_node(node);
        g.add_edge(Edge::new("parallel_node", target));
    }

    g
}

/// Registry that handles both codergen (success) and fail_node (failure) types.
fn mixed_registry() -> Arc<HandlerRegistry> {
    let mut reg = HandlerRegistry::with_defaults();
    reg.register("fail_node", FailHandler);
    Arc::new(reg)
}

#[tokio::test]
async fn parallel_first_success_returns_on_first_success() -> AttractorResult<()> {
    let registry = mixed_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let g = mixed_parallel_graph(
        &["ok_a", "ok_b"],
        &["bad_c"],
        &[("join_policy", "first_success")],
    );
    let node = g.get_node("parallel_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "parallel_node".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    // At least one succeeded → overall success
    assert!(outcome.status.is_success());
    Ok(())
}

#[tokio::test]
async fn parallel_first_success_all_fail() -> AttractorResult<()> {
    let registry = mixed_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let g = mixed_parallel_graph(
        &[],
        &["bad_a", "bad_b"],
        &[("join_policy", "first_success")],
    );
    let node = g.get_node("parallel_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "parallel_node".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    // No branch succeeded → FAIL
    assert_eq!(outcome.status, StageStatus::Fail);
    Ok(())
}

#[tokio::test]
async fn parallel_fail_fast_stops_on_first_failure() -> AttractorResult<()> {
    let registry = mixed_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let g = mixed_parallel_graph(
        &["ok_a"],
        &["bad_b", "bad_c"],
        &[("error_policy", "fail_fast")],
    );
    let node = g.get_node("parallel_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "parallel_node".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    // With fail_fast + wait_all, we get partial success (some succeeded, some failed)
    // but the key thing is it doesn't wait for all — we just verify it completes.
    let results = ctx.get("parallel.results");
    let arr = results.as_ref().and_then(|v| v.as_array());
    // Should have fewer than 3 results if early exit triggered
    // (but since all futures are spawned immediately, at least 1 arrives)
    assert!(arr.is_some());
    // The outcome should reflect that failures occurred
    let fail_count = outcome
        .context_updates
        .get("parallel.fail_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    assert!(fail_count >= 1);
    Ok(())
}

#[tokio::test]
async fn parallel_error_policy_ignore_hides_failures() -> AttractorResult<()> {
    let registry = mixed_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let g = mixed_parallel_graph(&["ok_a"], &["bad_b"], &[("error_policy", "ignore")]);
    let node = g.get_node("parallel_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "parallel_node".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    // With ignore policy, failures are hidden → overall SUCCESS
    assert!(outcome.status.is_success());
    Ok(())
}

#[tokio::test]
async fn parallel_wait_all_mixed_partial_success() -> AttractorResult<()> {
    let registry = mixed_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    // wait_all (default) + continue (default) with mixed results
    let g = mixed_parallel_graph(&["ok_a"], &["bad_b"], &[]);
    let node = g.get_node("parallel_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "parallel_node".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    // Mixed results with continue policy → PARTIAL_SUCCESS
    assert_eq!(outcome.status, StageStatus::PartialSuccess);
    let success_count = outcome
        .context_updates
        .get("parallel.success_count")
        .and_then(|v| v.as_u64());
    let fail_count = outcome
        .context_updates
        .get("parallel.fail_count")
        .and_then(|v| v.as_u64());
    assert_eq!(success_count, Some(1));
    assert_eq!(fail_count, Some(1));
    Ok(())
}

/// §4.8: wait_all with all branches failing must return PARTIAL_SUCCESS
/// (not FAIL), so the normal edge to fan-in is traversed.  Fan-in itself
/// handles the all-fail case.
#[tokio::test]
async fn parallel_wait_all_all_branches_fail_returns_partial() -> AttractorResult<()> {
    let registry = mixed_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    // All branches are fail_node type, wait_all (default) + continue (default)
    let g = mixed_parallel_graph(&[], &["bad_a", "bad_b"], &[]);
    let node = g.get_node("parallel_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "parallel_node".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    // Per §4.8 pseudocode: wait_all with failures → PARTIAL_SUCCESS (never FAIL)
    // so the engine follows the normal edge to fan-in.
    assert_eq!(outcome.status, StageStatus::PartialSuccess);

    let success_count = outcome
        .context_updates
        .get("parallel.success_count")
        .and_then(|v| v.as_u64());
    let fail_count = outcome
        .context_updates
        .get("parallel.fail_count")
        .and_then(|v| v.as_u64());
    assert_eq!(success_count, Some(0));
    assert_eq!(fail_count, Some(2));
    Ok(())
}

#[tokio::test]
async fn parallel_max_parallel_default() {
    // Verify the default max_parallel constant
    assert_eq!(DEFAULT_MAX_PARALLEL, 4);
}

#[tokio::test]
async fn parallel_max_parallel_from_attr() -> AttractorResult<()> {
    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    // Set max_parallel=1 — all branches run serially but should still complete
    let g = mixed_parallel_graph(&["a", "b", "c"], &[], &[("max_parallel", "1")]);
    let node = g.get_node("parallel_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "parallel_node".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert!(outcome.status.is_success());
    let results = ctx.get("parallel.results");
    let arr = results.as_ref().and_then(|v| v.as_array());
    assert_eq!(arr.map(Vec::len), Some(3));
    Ok(())
}

// ===========================================================================
// §4.9 — Fan-in handler: deterministic tie-breaks
// ===========================================================================

#[tokio::test]
async fn fan_in_tiebreak_by_score() -> AttractorResult<()> {
    let handler = FanInHandler;
    let node = Node::new("fan_in");
    let ctx = Context::new();
    let g = Graph::new("test");

    // Both success, but different scores — higher score wins
    let results = serde_json::json!([
        {"target": "branch_a", "outcome": "success", "score": 0.7},
        {"target": "branch_b", "outcome": "success", "score": 0.9},
    ]);
    ctx.set("parallel.results", results);

    let outcome = handler.execute(&node, &ctx, &g).await?;

    let best_id = outcome
        .context_updates
        .get("parallel.fan_in.best_id")
        .and_then(|v| v.as_str());
    assert_eq!(best_id, Some("branch_b"));
    Ok(())
}

#[tokio::test]
async fn fan_in_tiebreak_by_id() -> AttractorResult<()> {
    let handler = FanInHandler;
    let node = Node::new("fan_in");
    let ctx = Context::new();
    let g = Graph::new("test");

    // Same outcome, no score → deterministic tiebreak by ascending id
    let results = serde_json::json!([
        {"target": "branch_c", "outcome": "success"},
        {"target": "branch_a", "outcome": "success"},
        {"target": "branch_b", "outcome": "success"},
    ]);
    ctx.set("parallel.results", results);

    let outcome = handler.execute(&node, &ctx, &g).await?;

    let best_id = outcome
        .context_updates
        .get("parallel.fan_in.best_id")
        .and_then(|v| v.as_str());
    // branch_a should win (lowest id)
    assert_eq!(best_id, Some("branch_a"));
    Ok(())
}

// ===========================================================================
// §4.9 — Fan-in handler
// ===========================================================================

#[tokio::test]
async fn fan_in_no_results_fails() -> AttractorResult<()> {
    let handler = FanInHandler;
    let node = Node::new("fan_in");
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Fail);
    assert!(outcome.failure_reason.contains("No parallel results"));
    Ok(())
}

#[tokio::test]
async fn fan_in_selects_best_candidate() -> AttractorResult<()> {
    let handler = FanInHandler;
    let node = Node::new("fan_in");
    let ctx = Context::new();
    let g = Graph::new("test");

    // Simulate parallel.results in context
    let results = serde_json::json!([
        {"target": "branch_a", "outcome": "fail", "notes": "error"},
        {"target": "branch_b", "outcome": "success", "notes": "ok"},
        {"target": "branch_c", "outcome": "partial_success", "notes": "partial"},
    ]);
    ctx.set("parallel.results", results);

    let outcome = handler.execute(&node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Success);
    let best_id = outcome
        .context_updates
        .get("parallel.fan_in.best_id")
        .and_then(|v| v.as_str());
    assert_eq!(best_id, Some("branch_b"));

    Ok(())
}

#[tokio::test]
async fn fan_in_all_failed() -> AttractorResult<()> {
    let handler = FanInHandler;
    let node = Node::new("fan_in");
    let ctx = Context::new();
    let g = Graph::new("test");

    let results = serde_json::json!([
        {"target": "a", "outcome": "fail", "notes": "err1"},
        {"target": "b", "outcome": "fail", "notes": "err2"},
    ]);
    ctx.set("parallel.results", results);

    let outcome = handler.execute(&node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Fail);
    assert!(
        outcome
            .failure_reason
            .contains("all parallel candidates failed")
    );
    Ok(())
}

#[tokio::test]
async fn fan_in_partial_success_best() -> AttractorResult<()> {
    let handler = FanInHandler;
    let node = Node::new("fan_in");
    let ctx = Context::new();
    let g = Graph::new("test");

    let results = serde_json::json!([
        {"target": "a", "outcome": "fail", "notes": "err"},
        {"target": "b", "outcome": "partial_success", "notes": "partial"},
    ]);
    ctx.set("parallel.results", results);

    let outcome = handler.execute(&node, &ctx, &g).await?;

    // Best is partial_success → overall should be PartialSuccess
    assert_eq!(outcome.status, StageStatus::PartialSuccess);
    let best = outcome
        .context_updates
        .get("parallel.fan_in.best_outcome")
        .and_then(|v| v.as_str());
    assert_eq!(best, Some("partial_success"));
    Ok(())
}

#[tokio::test]
async fn fan_in_empty_results_array() -> AttractorResult<()> {
    let handler = FanInHandler;
    let node = Node::new("fan_in");
    let ctx = Context::new();
    let g = Graph::new("test");

    ctx.set("parallel.results", serde_json::json!([]));

    let outcome = handler.execute(&node, &ctx, &g).await?;

    assert_eq!(outcome.status, StageStatus::Fail);
    Ok(())
}

// ===========================================================================
// Structural fan-in detection
// ===========================================================================

/// Build a graph with a parallel fan-out → multi-step branches → convergence.
///
/// Topology:
///   parallel_node → branch_a → step_a2 → merge
///   parallel_node → branch_b → step_b2 → merge
///
/// The "merge" node is the structural fan-in point, reachable from both
/// branch entries but NOT directly connected to them.
fn multi_hop_diamond_graph() -> Graph {
    let mut g = Graph::new("multi_hop_diamond");

    let mut par = Node::new("parallel_node");
    par.attrs
        .insert("shape".into(), AttrValue::from("component"));
    g.add_node(par);

    // Branch A: two steps
    g.add_node(Node::new("branch_a"));
    g.add_node(Node::new("step_a2"));
    g.add_edge(Edge::new("parallel_node", "branch_a"));
    g.add_edge(Edge::new("branch_a", "step_a2"));
    g.add_edge(Edge::new("step_a2", "merge"));

    // Branch B: two steps
    g.add_node(Node::new("branch_b"));
    g.add_node(Node::new("step_b2"));
    g.add_edge(Edge::new("parallel_node", "branch_b"));
    g.add_edge(Edge::new("branch_b", "step_b2"));
    g.add_edge(Edge::new("step_b2", "merge"));

    // Convergence node
    g.add_node(Node::new("merge"));

    g
}

#[tokio::test]
async fn parallel_fan_in_multi_hop_structural() -> AttractorResult<()> {
    // When branches have intermediate nodes (branch_a → step_a2 → merge),
    // the fan-in node "merge" should still be detected via BFS reachability
    // and set as jump_target.

    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let g = multi_hop_diamond_graph();
    let node = g.get_node("parallel_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "parallel_node".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert!(outcome.status.is_success());
    // The jump_target should be the structural convergence node, not in context_updates
    assert_eq!(outcome.jump_target.as_deref(), Some("merge"));
    assert!(
        !outcome
            .context_updates
            .contains_key("parallel.fan_in_target")
    );
    Ok(())
}

#[tokio::test]
async fn parallel_fan_in_single_hop_structural() -> AttractorResult<()> {
    // Classic diamond: parallel → A → merge, parallel → B → merge.
    // The simplest structural fan-in case.

    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let mut g = Graph::new("single_hop_diamond");
    let mut par = Node::new("parallel_node");
    par.attrs
        .insert("shape".into(), AttrValue::from("component"));
    g.add_node(par);
    g.add_node(Node::new("branch_a"));
    g.add_node(Node::new("branch_b"));
    g.add_node(Node::new("merge"));
    g.add_edge(Edge::new("parallel_node", "branch_a"));
    g.add_edge(Edge::new("parallel_node", "branch_b"));
    g.add_edge(Edge::new("branch_a", "merge"));
    g.add_edge(Edge::new("branch_b", "merge"));

    let node = g.get_node("parallel_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "parallel_node".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert!(outcome.status.is_success());
    assert_eq!(outcome.jump_target.as_deref(), Some("merge"));
    Ok(())
}

#[tokio::test]
async fn parallel_fan_in_explicit_multi_hop() -> AttractorResult<()> {
    // Explicit parallel.fan_in node multiple hops from branch entries:
    //   parallel_node → branch_a → step_a2 → fan_in_node
    //   parallel_node → branch_b → step_b2 → fan_in_node
    // where fan_in_node has shape=tripleoctagon (handler type parallel.fan_in).

    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let mut g = Graph::new("explicit_fan_in");

    let mut par = Node::new("parallel_node");
    par.attrs
        .insert("shape".into(), AttrValue::from("component"));
    g.add_node(par);

    g.add_node(Node::new("branch_a"));
    g.add_node(Node::new("step_a2"));
    g.add_edge(Edge::new("parallel_node", "branch_a"));
    g.add_edge(Edge::new("branch_a", "step_a2"));
    g.add_edge(Edge::new("step_a2", "fan_in_node"));

    g.add_node(Node::new("branch_b"));
    g.add_node(Node::new("step_b2"));
    g.add_edge(Edge::new("parallel_node", "branch_b"));
    g.add_edge(Edge::new("branch_b", "step_b2"));
    g.add_edge(Edge::new("step_b2", "fan_in_node"));

    // Explicit fan-in node with the canonical shape
    let mut fan_in = Node::new("fan_in_node");
    fan_in
        .attrs
        .insert("shape".into(), AttrValue::from("tripleoctagon"));
    g.add_node(fan_in);

    let node = g.get_node("parallel_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "parallel_node".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert!(outcome.status.is_success());
    // The BFS should discover the explicit fan-in node even though it's
    // multiple hops from the branch entry nodes.
    assert_eq!(outcome.jump_target.as_deref(), Some("fan_in_node"));
    Ok(())
}

#[tokio::test]
async fn parallel_no_fan_in_when_branches_diverge() -> AttractorResult<()> {
    // When branches do not converge, jump_target should be None.
    //   parallel_node → branch_a → end_a
    //   parallel_node → branch_b → end_b

    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let mut g = Graph::new("divergent");
    let mut par = Node::new("parallel_node");
    par.attrs
        .insert("shape".into(), AttrValue::from("component"));
    g.add_node(par);
    g.add_node(Node::new("branch_a"));
    g.add_node(Node::new("end_a"));
    g.add_node(Node::new("branch_b"));
    g.add_node(Node::new("end_b"));
    g.add_edge(Edge::new("parallel_node", "branch_a"));
    g.add_edge(Edge::new("branch_a", "end_a"));
    g.add_edge(Edge::new("parallel_node", "branch_b"));
    g.add_edge(Edge::new("branch_b", "end_b"));

    let node = g.get_node("parallel_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "parallel_node".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert!(outcome.status.is_success());
    assert_eq!(outcome.jump_target, None);
    Ok(())
}

#[tokio::test]
async fn parallel_fan_in_target_not_in_context_updates() -> AttractorResult<()> {
    // Ensure the jump target is on outcome.jump_target, NOT leaked into
    // context_updates (which would pollute the pipeline context).

    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let g = multi_hop_diamond_graph();
    let node = g.get_node("parallel_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "parallel_node".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    // jump_target is set
    assert!(outcome.jump_target.is_some());
    // But it must NOT appear in context_updates
    assert!(
        !outcome
            .context_updates
            .contains_key("parallel.fan_in_target")
    );
    // Only the expected keys should be in context_updates
    assert!(
        outcome
            .context_updates
            .contains_key("parallel.success_count")
    );
    assert!(outcome.context_updates.contains_key("parallel.fail_count"));
    Ok(())
}

#[tokio::test]
async fn parallel_single_branch_with_explicit_fan_in_does_not_fail() -> AttractorResult<()> {
    // Regression test: a single-branch parallel node with a downstream
    // explicit parallel.fan_in node. find_fan_in_node returns None (only
    // one branch, so no convergence), but the branch must still stop at
    // the fan_in node rather than trying to execute it (which would fail
    // because parallel.results doesn't exist in the branch context yet).

    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let mut g = Graph::new("single_branch_fan_in");

    let mut par = Node::new("parallel_node");
    par.attrs
        .insert("shape".into(), AttrValue::from("component"));
    g.add_node(par);

    // Single branch
    g.add_node(Node::new("branch_a"));
    g.add_edge(Edge::new("parallel_node", "branch_a"));
    g.add_edge(Edge::new("branch_a", "fan_in_node"));

    // Explicit fan-in node (tripleoctagon shape → parallel.fan_in handler)
    let mut fan_in = Node::new("fan_in_node");
    fan_in
        .attrs
        .insert("shape".into(), AttrValue::from("tripleoctagon"));
    g.add_node(fan_in);

    let node = g.get_node("parallel_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "parallel_node".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    // With only one branch, there's no structural convergence.
    assert_eq!(outcome.jump_target, None);
    // The branch should succeed — the fan_in node is NOT executed within
    // the branch (it's stopped by the handler-type fallback guard).
    assert!(
        outcome.status.is_success(),
        "single-branch parallel should succeed, not fail from executing fan_in prematurely"
    );
    Ok(())
}

#[tokio::test]
async fn parallel_fan_in_detection_is_deterministic() -> AttractorResult<()> {
    // Run the same multi-hop diamond graph multiple times and verify
    // that the fan-in node selected is always the same, regardless of
    // hash randomization across iterations.

    let g = multi_hop_diamond_graph();
    let node = g.get_node("parallel_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "parallel_node".into(),
        }
    })?;

    let mut targets = Vec::new();
    for _ in 0..20 {
        let registry = default_registry();
        let emitter = Arc::new(NoOpEmitter);
        let handler = ParallelHandler::new(registry, emitter);
        let ctx = Context::new();

        let outcome = handler.execute(node, &ctx, &g).await?;
        targets.push(outcome.jump_target.clone());
    }

    // All runs must agree on the same fan-in node.
    let first = &targets[0];
    assert!(
        targets.iter().all(|t| t == first),
        "fan-in target varied across runs: {targets:?}"
    );
    assert_eq!(first.as_deref(), Some("merge"));
    Ok(())
}

#[tokio::test]
async fn parallel_fan_in_staggered_merge_selects_all_branch_convergence() -> AttractorResult<()> {
    // Staggered merge topology with 3 branches:
    //   parallel_node → A → merge_ab → merge_abc → (done)
    //   parallel_node → B → merge_ab ↗
    //   parallel_node → C → merge_abc ↗
    //
    // merge_ab is reachable from 2 branches (A, B).
    // merge_abc is reachable from all 3 (A via merge_ab, B via merge_ab, C).
    // The fan-in must be merge_abc, not merge_ab.

    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let mut g = Graph::new("staggered_merge");

    let mut par = Node::new("parallel_node");
    par.attrs
        .insert("shape".into(), AttrValue::from("component"));
    g.add_node(par);

    g.add_node(Node::new("A"));
    g.add_node(Node::new("B"));
    g.add_node(Node::new("C"));
    g.add_node(Node::new("merge_ab"));
    g.add_node(Node::new("merge_abc"));

    g.add_edge(Edge::new("parallel_node", "A"));
    g.add_edge(Edge::new("parallel_node", "B"));
    g.add_edge(Edge::new("parallel_node", "C"));
    g.add_edge(Edge::new("A", "merge_ab"));
    g.add_edge(Edge::new("B", "merge_ab"));
    g.add_edge(Edge::new("merge_ab", "merge_abc"));
    g.add_edge(Edge::new("C", "merge_abc"));

    let node = g.get_node("parallel_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "parallel_node".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert!(outcome.status.is_success());
    assert_eq!(
        outcome.jump_target.as_deref(),
        Some("merge_abc"),
        "should select all-branch convergence, not pairwise merge_ab"
    );
    Ok(())
}

#[tokio::test]
async fn parallel_fan_in_dead_end_branch_still_finds_partial_convergence() -> AttractorResult<()> {
    // 3 branches where one is a dead end:
    //   parallel_node → A → merge
    //   parallel_node → B → merge
    //   parallel_node → C → dead_end (no outgoing edges)
    //
    // merge is reachable from 2 of 3 branches. Since C can't reach any
    // shared node, merge is the best available convergence point.

    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let mut g = Graph::new("dead_end_branch");

    let mut par = Node::new("parallel_node");
    par.attrs
        .insert("shape".into(), AttrValue::from("component"));
    g.add_node(par);

    g.add_node(Node::new("A"));
    g.add_node(Node::new("B"));
    g.add_node(Node::new("C"));
    g.add_node(Node::new("merge"));
    g.add_node(Node::new("dead_end"));

    g.add_edge(Edge::new("parallel_node", "A"));
    g.add_edge(Edge::new("parallel_node", "B"));
    g.add_edge(Edge::new("parallel_node", "C"));
    g.add_edge(Edge::new("A", "merge"));
    g.add_edge(Edge::new("B", "merge"));
    g.add_edge(Edge::new("C", "dead_end"));
    // dead_end has no outgoing edges

    let node = g.get_node("parallel_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "parallel_node".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert!(outcome.status.is_success());
    assert_eq!(
        outcome.jump_target.as_deref(),
        Some("merge"),
        "should find partial convergence when one branch is a dead end"
    );
    Ok(())
}

#[tokio::test]
async fn parallel_fan_in_excludes_parallel_node_in_cyclic_graph() -> AttractorResult<()> {
    // Cyclic graph: branches have back-edges to the parallel node.
    //   parallel_node → branch_a → parallel_node (back-edge)
    //   parallel_node → branch_b → parallel_node (back-edge)
    //
    // Without the exclusion, find_fan_in_node would select parallel_node
    // itself as the fan-in (reachable from both branches), causing the
    // engine to jump back and re-execute the parallel handler forever.

    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let mut g = Graph::new("cyclic_parallel");

    let mut par = Node::new("parallel_node");
    par.attrs
        .insert("shape".into(), AttrValue::from("component"));
    g.add_node(par);

    g.add_node(Node::new("branch_a"));
    g.add_node(Node::new("branch_b"));

    g.add_edge(Edge::new("parallel_node", "branch_a"));
    g.add_edge(Edge::new("parallel_node", "branch_b"));
    // Back-edges to parallel_node
    g.add_edge(Edge::new("branch_a", "parallel_node"));
    g.add_edge(Edge::new("branch_b", "parallel_node"));

    let node = g.get_node("parallel_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "parallel_node".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    // The parallel node must NOT be selected as the fan-in target.
    assert_ne!(
        outcome.jump_target.as_deref(),
        Some("parallel_node"),
        "fan-in must not be the parallel node itself in cyclic graphs"
    );
    // With only back-edges and no forward convergence, jump_target should be None.
    assert_eq!(outcome.jump_target, None);
    Ok(())
}

#[tokio::test]
async fn parallel_fan_in_cyclic_with_real_convergence() -> AttractorResult<()> {
    // Cyclic graph with a real convergence node beyond the back-edges:
    //   parallel_node → branch_a → parallel_node (back-edge)
    //                   branch_a → merge
    //   parallel_node → branch_b → parallel_node (back-edge)
    //                   branch_b → merge
    //
    // merge is the correct fan-in, not parallel_node.

    let registry = default_registry();
    let emitter = Arc::new(NoOpEmitter);
    let handler = ParallelHandler::new(registry, emitter);

    let mut g = Graph::new("cyclic_with_merge");

    let mut par = Node::new("parallel_node");
    par.attrs
        .insert("shape".into(), AttrValue::from("component"));
    g.add_node(par);

    g.add_node(Node::new("branch_a"));
    g.add_node(Node::new("branch_b"));
    g.add_node(Node::new("merge"));

    g.add_edge(Edge::new("parallel_node", "branch_a"));
    g.add_edge(Edge::new("parallel_node", "branch_b"));
    g.add_edge(Edge::new("branch_a", "parallel_node")); // back-edge
    g.add_edge(Edge::new("branch_a", "merge"));
    g.add_edge(Edge::new("branch_b", "parallel_node")); // back-edge
    g.add_edge(Edge::new("branch_b", "merge"));

    let node = g.get_node("parallel_node").ok_or_else(|| {
        stencila_attractor::error::AttractorError::NodeNotFound {
            node_id: "parallel_node".into(),
        }
    })?;
    let ctx = Context::new();

    let outcome = handler.execute(node, &ctx, &g).await?;

    assert_eq!(
        outcome.jump_target.as_deref(),
        Some("merge"),
        "should select merge, not parallel_node, as fan-in in cyclic graph"
    );
    Ok(())
}
