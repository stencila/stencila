//! Tests for parallel handler (§4.8) and fan-in handler (§4.9).

mod common;

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;

use stencila_attractor::context::Context;
use stencila_attractor::error::AttractorResult;
use stencila_attractor::events::NoOpEmitter;
use stencila_attractor::graph::{AttrValue, Edge, Graph, Node};
use stencila_attractor::handler::{Handler, HandlerRegistry};
use stencila_attractor::handlers::{DEFAULT_MAX_PARALLEL, FanInHandler, ParallelHandler};
use stencila_attractor::types::{Outcome, StageStatus};

use common::make_tempdir;

/// A handler that always returns FAIL.
struct FailHandler;

#[async_trait]
impl Handler for FailHandler {
    async fn execute(
        &self,
        _node: &Node,
        _context: &Context,
        _graph: &Graph,
        _logs_root: &Path,
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

/// Build a registry with defaults (start, exit, conditional, codergen, tool).
fn default_registry() -> Arc<HandlerRegistry> {
    Arc::new(HandlerRegistry::with_defaults())
}

// ===========================================================================
// §4.8 — Parallel handler: basic execution
// ===========================================================================

#[tokio::test]
async fn parallel_executes_all_branches() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let logs_root = tmp.path();
    std::fs::create_dir_all(logs_root.join("nodes"))?;

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

    let outcome = handler.execute(node, &ctx, &g, logs_root).await?;

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
    let tmp = make_tempdir()?;
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

    let outcome = handler.execute(node, &ctx, &g, tmp.path()).await?;

    assert_eq!(outcome.status, StageStatus::Fail);
    Ok(())
}

#[tokio::test]
async fn parallel_context_isolation() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let logs_root = tmp.path();
    std::fs::create_dir_all(logs_root.join("nodes"))?;

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

    let outcome = handler.execute(node, &ctx, &g, logs_root).await?;
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
    let tmp = make_tempdir()?;
    let logs_root = tmp.path();
    std::fs::create_dir_all(logs_root.join("nodes"))?;

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

    let outcome = handler.execute(node, &ctx, &g, logs_root).await?;

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
    let tmp = make_tempdir()?;
    let logs_root = tmp.path();
    std::fs::create_dir_all(logs_root.join("nodes"))?;

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

    let outcome = handler.execute(node, &ctx, &g, logs_root).await?;

    // At least one succeeded → overall success
    assert!(outcome.status.is_success());
    Ok(())
}

#[tokio::test]
async fn parallel_first_success_all_fail() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let logs_root = tmp.path();
    std::fs::create_dir_all(logs_root.join("nodes"))?;

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

    let outcome = handler.execute(node, &ctx, &g, logs_root).await?;

    // No branch succeeded → FAIL
    assert_eq!(outcome.status, StageStatus::Fail);
    Ok(())
}

#[tokio::test]
async fn parallel_fail_fast_stops_on_first_failure() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let logs_root = tmp.path();
    std::fs::create_dir_all(logs_root.join("nodes"))?;

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

    let outcome = handler.execute(node, &ctx, &g, logs_root).await?;

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
    let tmp = make_tempdir()?;
    let logs_root = tmp.path();
    std::fs::create_dir_all(logs_root.join("nodes"))?;

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

    let outcome = handler.execute(node, &ctx, &g, logs_root).await?;

    // With ignore policy, failures are hidden → overall SUCCESS
    assert!(outcome.status.is_success());
    Ok(())
}

#[tokio::test]
async fn parallel_wait_all_mixed_partial_success() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
    let logs_root = tmp.path();
    std::fs::create_dir_all(logs_root.join("nodes"))?;

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

    let outcome = handler.execute(node, &ctx, &g, logs_root).await?;

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
    let tmp = make_tempdir()?;
    let logs_root = tmp.path();
    std::fs::create_dir_all(logs_root.join("nodes"))?;

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

    let outcome = handler.execute(node, &ctx, &g, logs_root).await?;

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
    let tmp = make_tempdir()?;
    let logs_root = tmp.path();
    std::fs::create_dir_all(logs_root.join("nodes"))?;

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

    let outcome = handler.execute(node, &ctx, &g, logs_root).await?;

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
    let tmp = make_tempdir()?;
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

    let outcome = handler.execute(&node, &ctx, &g, tmp.path()).await?;

    let best_id = outcome
        .context_updates
        .get("parallel.fan_in.best_id")
        .and_then(|v| v.as_str());
    assert_eq!(best_id, Some("branch_b"));
    Ok(())
}

#[tokio::test]
async fn fan_in_tiebreak_by_id() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
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

    let outcome = handler.execute(&node, &ctx, &g, tmp.path()).await?;

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
    let tmp = make_tempdir()?;
    let handler = FanInHandler;
    let node = Node::new("fan_in");
    let ctx = Context::new();
    let g = Graph::new("test");

    let outcome = handler.execute(&node, &ctx, &g, tmp.path()).await?;

    assert_eq!(outcome.status, StageStatus::Fail);
    assert!(outcome.failure_reason.contains("No parallel results"));
    Ok(())
}

#[tokio::test]
async fn fan_in_selects_best_candidate() -> AttractorResult<()> {
    let tmp = make_tempdir()?;
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

    let outcome = handler.execute(&node, &ctx, &g, tmp.path()).await?;

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
    let tmp = make_tempdir()?;
    let handler = FanInHandler;
    let node = Node::new("fan_in");
    let ctx = Context::new();
    let g = Graph::new("test");

    let results = serde_json::json!([
        {"target": "a", "outcome": "fail", "notes": "err1"},
        {"target": "b", "outcome": "fail", "notes": "err2"},
    ]);
    ctx.set("parallel.results", results);

    let outcome = handler.execute(&node, &ctx, &g, tmp.path()).await?;

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
    let tmp = make_tempdir()?;
    let handler = FanInHandler;
    let node = Node::new("fan_in");
    let ctx = Context::new();
    let g = Graph::new("test");

    let results = serde_json::json!([
        {"target": "a", "outcome": "fail", "notes": "err"},
        {"target": "b", "outcome": "partial_success", "notes": "partial"},
    ]);
    ctx.set("parallel.results", results);

    let outcome = handler.execute(&node, &ctx, &g, tmp.path()).await?;

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
    let tmp = make_tempdir()?;
    let handler = FanInHandler;
    let node = Node::new("fan_in");
    let ctx = Context::new();
    let g = Graph::new("test");

    ctx.set("parallel.results", serde_json::json!([]));

    let outcome = handler.execute(&node, &ctx, &g, tmp.path()).await?;

    assert_eq!(outcome.status, StageStatus::Fail);
    Ok(())
}
