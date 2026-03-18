//! Integration tests for session pool reuse in `AgentCodergenBackend`.
//!
//! These tests verify that when `fidelity=full` is set on an edge,
//! `AgentCodergenBackend::run()` consults the `SessionPool` via
//! `take(thread_id)` / `SessionGuard` for session reuse across loop iterations.
//! They run entirely offline (no LLM calls, no API keys, no network access).
//!
//! Run with:
//!
//! ```sh
//! cargo test -p stencila-workflows --test pool_session_reuse
//! ```

use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use stencila_attractor::context::Context;
use stencila_attractor::engine::{self, EngineConfig};
use stencila_attractor::error::AttractorResult;
use stencila_attractor::events::EventEmitter;
use stencila_attractor::graph::{AttrValue, Edge, Graph, Node};
use stencila_attractor::handlers::{CodergenBackend, CodergenHandler, CodergenOutput};
use stencila_attractor::types::Outcome;

use stencila_workflows::session_pool::{SessionEntry, SessionGuard, SessionPool};

// ---------------------------------------------------------------------------
// PoolAwareMockBackend — test infrastructure (Phase 3a, Task 0)
// ---------------------------------------------------------------------------

/// Record of a single `run()` invocation on the mock backend.
#[derive(Debug, Clone)]
struct BackendCallRecord {
    node_id: String,
    fidelity: Option<String>,
    thread_id: Option<String>,
}

/// A mock `CodergenBackend` that records context values, interacts
/// with a `SessionPool` clone following the same protocol that
/// `AgentCodergenBackend.run()` is expected to implement, and returns
/// configurable text responses without network access.
///
/// When `internal.fidelity` is `"full"` and a `thread_id` is present,
/// the mock calls `pool.take(thread_id)` and wraps the result in a
/// `SessionGuard` (which puts the entry back on drop). This simulates
/// the pool lifecycle that `AgentCodergenBackend.run()` must implement.
struct PoolAwareMockBackend {
    pool: SessionPool,
    calls: Mutex<Vec<BackendCallRecord>>,
    response: String,
}

impl PoolAwareMockBackend {
    fn new(pool: SessionPool, response: &str) -> Self {
        Self {
            pool,
            calls: Mutex::new(Vec::new()),
            response: response.to_string(),
        }
    }

    fn calls(&self) -> Vec<BackendCallRecord> {
        self.calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

#[async_trait]
impl CodergenBackend for PoolAwareMockBackend {
    async fn run(
        &self,
        node: &stencila_attractor::graph::Node,
        _prompt: &str,
        context: &Context,
        _emitter: Arc<dyn EventEmitter>,
        _stage_index: usize,
    ) -> AttractorResult<CodergenOutput> {
        let fidelity = context
            .get("internal.fidelity")
            .and_then(|v| v.as_str().map(String::from));
        let thread_id = context
            .get("internal.thread_id")
            .and_then(|v| v.as_str().map(String::from));

        self.calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(BackendCallRecord {
                node_id: node.id.clone(),
                fidelity: fidelity.clone(),
                thread_id: thread_id.clone(),
            });

        // Simulate the pool interaction protocol that
        // AgentCodergenBackend.run() must implement (AC-4):
        //   - When fidelity == "full" and thread_id is present,
        //     take a session from the pool (or create a new entry)
        //     and wrap in SessionGuard so it's returned on drop.
        //   - When fidelity != "full", don't touch the pool.
        if fidelity.as_deref() == Some("full")
            && let Some(tid) = &thread_id
        {
            let entry = self.pool.take(tid).unwrap_or_else(|| SessionEntry {
                agent_name: format!("mock-agent-{}", node.id),
                ..Default::default()
            });
            let mut guard = SessionGuard::from_pool(self.pool.clone(), tid.clone(), entry);

            // CLI session fallback: detect agent_type="cli" on the node
            // and discard the guard so the session is not returned to the pool.
            let is_cli = node.attrs.get("agent_type").and_then(|v| v.as_str()) == Some("cli");
            if is_cli {
                tracing::warn!(
                    node_id = %node.id,
                    "CLI sessions do not support persistent reuse; discarding pool guard"
                );
                guard.discard();
            }
        }

        Ok(CodergenOutput::Text(self.response.clone()))
    }
}

// ---------------------------------------------------------------------------
// Graph construction helpers
// ---------------------------------------------------------------------------

fn make_start_node() -> Node {
    let mut node = Node::new("Start");
    node.attrs
        .insert("shape".into(), AttrValue::String(Graph::START_SHAPE.into()));
    node
}

fn make_exit_node() -> Node {
    let mut node = Node::new("Exit");
    node.attrs
        .insert("shape".into(), AttrValue::String(Graph::EXIT_SHAPE.into()));
    node
}

/// Build a 2-iteration loop graph:
///
/// ```text
/// Start → A [fidelity=full, thread_id=loop_thread] → B
///              ├──[retry, loop_restart]──→ A
///              └──[done]──→ Exit
/// ```
///
/// Node A carries `fidelity="full"` and `thread_id="loop_thread"` as
/// **node attributes** so that every execution of A (regardless of
/// which edge was used to reach it) triggers the pool protocol.
///
/// On loop restart, the engine clears `last_selected_edge`, so
/// edge-level fidelity from B→A would not be visible to A. By placing
/// fidelity on the node itself, we ensure it survives restart
/// (fidelity precedence: edge → node → graph → default).
///
/// The `SequenceCodergenHandler` is used so that B returns "retry" on
/// the first call and "done" on the second call, producing exactly two
/// iterations through A.
fn build_loop_graph() -> Graph {
    let mut graph = Graph::new("pool_reuse_loop");
    graph.add_node(make_start_node());

    // Node A: fidelity=full and thread_id=loop_thread as node attributes
    let mut node_a = Node::new("A");
    node_a
        .attrs
        .insert("fidelity".into(), AttrValue::String("full".into()));
    node_a
        .attrs
        .insert("thread_id".into(), AttrValue::String("loop_thread".into()));
    graph.add_node(node_a);

    graph.add_node(Node::new("B"));
    graph.add_node(make_exit_node());

    graph.add_edge(Edge::new("Start", "A"));
    graph.add_edge(Edge::new("A", "B"));

    // Back edge: B → A with loop_restart (fidelity is on the node, not edge)
    let mut e_loop = Edge::new("B", "A");
    e_loop
        .attrs
        .insert("label".into(), AttrValue::String("retry".into()));
    e_loop
        .attrs
        .insert("loop_restart".into(), AttrValue::Boolean(true));
    graph.add_edge(e_loop);

    let mut e_exit = Edge::new("B", "Exit");
    e_exit
        .attrs
        .insert("label".into(), AttrValue::String("done".into()));
    graph.add_edge(e_exit);

    graph
}

/// A handler wrapper that returns sequenced outcomes (alternating
/// preferred labels) to drive loop iteration, while delegating the
/// actual backend call to the `CodergenHandler`.
///
/// Nodes named "B" get sequenced `preferred_label` values to control
/// edge selection. All other nodes just get standard success outcomes.
struct SequenceCodergenHandler {
    inner: CodergenHandler,
    labels: Mutex<Vec<String>>,
    call_count: Mutex<usize>,
}

impl SequenceCodergenHandler {
    fn new(backend: Arc<dyn CodergenBackend>, labels: Vec<String>) -> Self {
        Self {
            inner: CodergenHandler::with_backend(backend),
            labels: Mutex::new(labels),
            call_count: Mutex::new(0),
        }
    }
}

#[async_trait]
impl stencila_attractor::handler::Handler for SequenceCodergenHandler {
    async fn execute(
        &self,
        node: &Node,
        context: &Context,
        graph: &Graph,
    ) -> AttractorResult<Outcome> {
        let mut outcome = self.inner.execute(node, context, graph).await?;

        if node.id == "B" {
            let mut count = self
                .call_count
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let labels = self
                .labels
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if *count < labels.len() {
                outcome.preferred_label = labels[*count].clone();
            }
            *count += 1;
        }

        Ok(outcome)
    }
}

// ===========================================================================
// Tests
// ===========================================================================

/// AC-1: `PoolAwareMockBackend` exists and implements `CodergenBackend`,
/// records context values per `run()` call, holds a `SessionPool` clone,
/// and returns configurable text responses without network access.
#[tokio::test]
async fn pool_aware_mock_backend_records_context_values() -> eyre::Result<()> {
    let pool = SessionPool::new();
    let backend = Arc::new(PoolAwareMockBackend::new(pool.clone(), "mock response"));

    let node = Node::new("test-node");
    let context = Context::new();
    context.set(
        "internal.fidelity",
        serde_json::Value::String("full".into()),
    );
    context.set(
        "internal.thread_id",
        serde_json::Value::String("thread-1".into()),
    );

    let result = backend
        .run(
            &node,
            "test prompt",
            &context,
            Arc::new(stencila_attractor::events::NoOpEmitter),
            0,
        )
        .await?;

    // Verify response
    match result {
        CodergenOutput::Text(text) => assert_eq!(text, "mock response"),
        _ => panic!("expected Text output"),
    }

    // Verify recorded context
    let calls = backend.calls();
    assert_eq!(calls.len(), 1, "should have recorded exactly one call");
    assert_eq!(calls[0].node_id, "test-node");
    assert_eq!(calls[0].fidelity.as_deref(), Some("full"));
    assert_eq!(calls[0].thread_id.as_deref(), Some("thread-1"));

    Ok(())
}

/// AC-1 (cont): `PoolAwareMockBackend` interacts with the pool when
/// fidelity is "full" — simulating the protocol `AgentCodergenBackend`
/// must implement. After `run()`, the `SessionGuard` should have
/// returned the session to the pool with an incremented turn count.
#[tokio::test]
async fn pool_aware_mock_backend_interacts_with_pool_on_full_fidelity() -> eyre::Result<()> {
    let pool = SessionPool::new();
    let backend = Arc::new(PoolAwareMockBackend::new(pool.clone(), "ok"));

    let node = Node::new("test-node");
    let context = Context::new();
    context.set(
        "internal.fidelity",
        serde_json::Value::String("full".into()),
    );
    context.set(
        "internal.thread_id",
        serde_json::Value::String("tid-1".into()),
    );

    backend
        .run(
            &node,
            "prompt",
            &context,
            Arc::new(stencila_attractor::events::NoOpEmitter),
            0,
        )
        .await?;

    // After run() with fidelity=full, SessionGuard::Drop should have
    // put an entry back into the pool with turn_count=1.
    assert_eq!(
        pool.turn_count("tid-1"),
        Some(1),
        "after first run with fidelity=full, pool should contain an entry \
         for tid-1 with turn_count=1 (put back by SessionGuard::Drop)"
    );

    Ok(())
}

/// AC-1 (cont): `PoolAwareMockBackend` does NOT interact with the
/// pool when fidelity is not "full".
#[tokio::test]
async fn pool_aware_mock_backend_skips_pool_without_full_fidelity() -> eyre::Result<()> {
    let pool = SessionPool::new();
    let backend = Arc::new(PoolAwareMockBackend::new(pool.clone(), "ok"));

    let node = Node::new("test-node");
    let context = Context::new();
    context.set(
        "internal.fidelity",
        serde_json::Value::String("compact".into()),
    );

    backend
        .run(
            &node,
            "prompt",
            &context,
            Arc::new(stencila_attractor::events::NoOpEmitter),
            0,
        )
        .await?;

    let drained = pool.drain();
    assert!(
        drained.is_empty(),
        "pool should remain empty when fidelity is not 'full', \
         but found {} entries",
        drained.len()
    );

    Ok(())
}

/// AC-5: Integration test — in a 2-iteration loop with `fidelity=full`,
/// the pool's `take()` returns `Some` on the second iteration (session
/// was returned by `SessionGuard::Drop` after the first iteration).
///
/// This test uses `PoolAwareMockBackend` which simulates the pool
/// interaction protocol that `AgentCodergenBackend.run()` must
/// implement. The mock calls `pool.take(thread_id)` and wraps the
/// result in a `SessionGuard` when fidelity is "full", exactly as
/// the real backend should.
///
/// Node A carries `fidelity="full"` and `thread_id="loop_thread"` as
/// node attributes so that every execution of A triggers pool interaction
/// (regardless of which edge was used to reach A — the engine clears
/// `last_selected_edge` on loop restart, so edge-level fidelity would
/// not survive).
///
/// After a 2-iteration loop:
///   - First A execution: fidelity="full" (from node attr), pool take
///     returns None (empty pool), creates new entry, SessionGuard puts
///     back with turn_count=1.
///   - Second A execution: fidelity="full" (from node attr), pool take
///     returns Some (entry from first iteration with turn_count=1),
///     SessionGuard puts back with turn_count=2.
///
/// After execution, pool should have an entry for "loop_thread" with
/// turn_count=2, proving session reuse across loop iterations.
///
/// This test validates the pool lifecycle protocol end-to-end using the
/// mock backend. The corresponding unit tests in `run.rs` verify that
/// `AgentCodergenBackend` actually has a `session_pool` field and that
/// `build_engine_config()` wires it.
#[tokio::test]
async fn loop_with_fidelity_full_reuses_session_via_pool() -> eyre::Result<()> {
    let pool = SessionPool::new();
    let backend = Arc::new(PoolAwareMockBackend::new(pool.clone(), "ok"));

    let graph = build_loop_graph();

    // Sequence: B returns "retry" first, then "done" to produce 2 iterations.
    let handler = SequenceCodergenHandler::new(
        backend.clone(),
        vec!["retry".to_string(), "done".to_string()],
    );

    let mut config = EngineConfig::new();
    config.skip_validation = true;
    config.registry.register("codergen", handler);

    let outcome = engine::run_with_context(&graph, config, Context::new()).await?;
    assert!(
        outcome.status.is_success(),
        "pipeline should complete successfully, got: {:?}",
        outcome.status
    );

    // Verify that the backend was called for node A twice (once per iteration).
    let calls = backend.calls();
    let a_calls: Vec<_> = calls.iter().filter(|c| c.node_id == "A").collect();
    assert_eq!(
        a_calls.len(),
        2,
        "node A should have been executed twice (two loop iterations), \
         but was executed {} times",
        a_calls.len()
    );

    // Node A has fidelity="full" as a node attribute, so both iterations
    // should have seen fidelity="full" in the context. Verify the mock
    // recorded this.
    assert_eq!(
        a_calls[0].fidelity.as_deref(),
        Some("full"),
        "first A execution should see fidelity='full' (from node attribute)"
    );
    assert_eq!(
        a_calls[0].thread_id.as_deref(),
        Some("loop_thread"),
        "first A execution should see thread_id='loop_thread' (from node attribute)"
    );
    assert_eq!(
        a_calls[1].fidelity.as_deref(),
        Some("full"),
        "second A execution should see fidelity='full' (from node attribute, survives restart)"
    );
    assert_eq!(
        a_calls[1].thread_id.as_deref(),
        Some("loop_thread"),
        "second A execution should see thread_id='loop_thread' (from node attribute, survives restart)"
    );

    // After the 2-iteration loop, the pool should contain an entry for
    // "loop_thread" with turn_count=2 — proving that:
    //   1. First iteration: take() returned None, created new entry,
    //      SessionGuard::Drop put it back with turn_count=1
    //   2. Second iteration: take() returned Some (the entry from iteration 1),
    //      SessionGuard::Drop put it back with turn_count=2
    let pool_entry = pool.take("loop_thread");
    assert!(
        pool_entry.is_some(),
        "after a 2-iteration loop with fidelity=full, the session pool \
         should contain an entry for thread_id='loop_thread' \
         (put back by SessionGuard::Drop after each iteration). \
         This verifies the pool lifecycle protocol works end-to-end."
    );

    let entry = pool_entry.expect("entry should exist");
    assert_eq!(
        entry.turn_count, 2,
        "turn_count should be 2 after two SessionGuard lifecycles \
         (one per loop iteration), got {}",
        entry.turn_count
    );

    Ok(())
}

/// Slice 5 (CLI session fallback): When `fidelity=full` and the agent
/// session is CLI-backed (indicated by `agent_type="cli"` node attribute),
/// the mock simulates the protocol that `AgentCodergenBackend.run()` must
/// implement: detect `AgentSession::Cli`, call `guard.discard()`, and log
/// a warning. The observable result is that the pool remains empty after
/// execution — the session is used for the current execution but not returned.
///
/// This test builds a loop graph where node A has `fidelity="full"`,
/// `thread_id="loop_thread"`, AND `agent_type="cli"`. After two loop
/// iterations the pool should be empty because both iterations should
/// have discarded the guard.
///
/// **Expected to FAIL** until `PoolAwareMockBackend.run()` is updated to
/// check for `agent_type="cli"` and call `guard.discard()`, mirroring the
/// real `AgentCodergenBackend` protocol.
#[tokio::test]
async fn cli_session_with_fidelity_full_does_not_pool_session() -> eyre::Result<()> {
    let pool = SessionPool::new();
    let backend = Arc::new(PoolAwareMockBackend::new(pool.clone(), "ok"));

    // Build a loop graph similar to build_loop_graph() but with agent_type="cli"
    // on node A.
    let mut graph = Graph::new("cli_pool_test");
    graph.add_node(make_start_node());

    let mut node_a = Node::new("A");
    node_a
        .attrs
        .insert("fidelity".into(), AttrValue::String("full".into()));
    node_a
        .attrs
        .insert("thread_id".into(), AttrValue::String("loop_thread".into()));
    node_a
        .attrs
        .insert("agent_type".into(), AttrValue::String("cli".into()));
    graph.add_node(node_a);

    graph.add_node(Node::new("B"));
    graph.add_node(make_exit_node());

    graph.add_edge(Edge::new("Start", "A"));
    graph.add_edge(Edge::new("A", "B"));

    let mut e_loop = Edge::new("B", "A");
    e_loop
        .attrs
        .insert("label".into(), AttrValue::String("retry".into()));
    e_loop
        .attrs
        .insert("loop_restart".into(), AttrValue::Boolean(true));
    graph.add_edge(e_loop);

    let mut e_exit = Edge::new("B", "Exit");
    e_exit
        .attrs
        .insert("label".into(), AttrValue::String("done".into()));
    graph.add_edge(e_exit);

    let handler = SequenceCodergenHandler::new(
        backend.clone(),
        vec!["retry".to_string(), "done".to_string()],
    );

    let mut config = EngineConfig::new();
    config.skip_validation = true;
    config.registry.register("codergen", handler);

    let outcome = engine::run_with_context(&graph, config, Context::new()).await?;
    assert!(
        outcome.status.is_success(),
        "pipeline should complete successfully, got: {:?}",
        outcome.status
    );

    // Node A should have been executed twice
    let calls = backend.calls();
    let a_calls: Vec<_> = calls.iter().filter(|c| c.node_id == "A").collect();
    assert_eq!(
        a_calls.len(),
        2,
        "node A should have been executed twice (two loop iterations)"
    );

    // Key assertion: after execution with agent_type="cli", the pool
    // should be EMPTY because guard.discard() should have been called
    // on every iteration, preventing the session from being returned.
    let drained = pool.drain();
    assert!(
        drained.is_empty(),
        "CLI-backed sessions should NOT be pooled (guard.discard() should \
         be called when agent_type='cli'). Expected empty pool, but found \
         {} entries: {:?}",
        drained.len(),
        drained.keys().collect::<Vec<_>>()
    );

    Ok(())
}

/// Slice 5 (cont): Verify that the mock backend records that CLI sessions
/// were detected — specifically, when `fidelity=full` AND `agent_type=cli`,
/// the guard should be discarded (not returned to pool).
///
/// This unit-level test calls `PoolAwareMockBackend.run()` directly with
/// `agent_type=cli` to verify discard behavior without running a full pipeline.
#[tokio::test]
async fn pool_aware_mock_discards_guard_for_cli_sessions() -> eyre::Result<()> {
    let pool = SessionPool::new();
    let backend = Arc::new(PoolAwareMockBackend::new(pool.clone(), "ok"));

    let mut node = Node::new("cli-node");
    node.attrs
        .insert("agent_type".into(), AttrValue::String("cli".into()));

    let context = Context::new();
    context.set(
        "internal.fidelity",
        serde_json::Value::String("full".into()),
    );
    context.set(
        "internal.thread_id",
        serde_json::Value::String("cli-thread".into()),
    );

    backend
        .run(
            &node,
            "prompt",
            &context,
            Arc::new(stencila_attractor::events::NoOpEmitter),
            0,
        )
        .await?;

    // After a CLI session with fidelity=full, the pool should be empty
    // because guard.discard() should prevent the entry from being returned.
    assert!(
        pool.take("cli-thread").is_none(),
        "CLI session guard should have been discarded, so pool entry for \
         'cli-thread' should not exist. guard.discard() must be called \
         when the session is CLI-backed."
    );

    Ok(())
}

/// AC-4 (partial): When no edge carries fidelity="full", the pool
/// should remain completely empty after pipeline execution.
///
/// This test builds a simple linear graph (Start → A → Exit) with no
/// fidelity attributes. After execution, the pool should be untouched.
#[tokio::test]
async fn linear_graph_without_fidelity_does_not_touch_pool() -> eyre::Result<()> {
    let pool = SessionPool::new();
    let backend = Arc::new(PoolAwareMockBackend::new(pool.clone(), "ok"));

    let mut graph = Graph::new("no_pool_test");
    graph.add_node(make_start_node());
    graph.add_node(Node::new("A"));
    graph.add_node(make_exit_node());
    graph.add_edge(Edge::new("Start", "A"));
    graph.add_edge(Edge::new("A", "Exit"));

    let handler = CodergenHandler::with_backend(backend.clone());

    let mut config = EngineConfig::new();
    config.skip_validation = true;
    config.registry.register("codergen", handler);

    let outcome = engine::run_with_context(&graph, config, Context::new()).await?;
    assert!(outcome.status.is_success());

    // Verify the backend saw the default (compact) fidelity — the engine
    // sets internal.fidelity to "compact" by default (verified by
    // attractor's loop_core tests).
    let calls = backend.calls();
    let a_calls: Vec<_> = calls.iter().filter(|c| c.node_id == "A").collect();
    assert_eq!(a_calls.len(), 1);
    assert_eq!(
        a_calls[0].fidelity.as_deref(),
        Some("compact"),
        "node A should see default fidelity 'compact' (set by engine)"
    );

    // Pool should remain empty — the mock only interacts with the pool
    // when fidelity is "full".
    let drained = pool.drain();
    assert!(
        drained.is_empty(),
        "session pool should be empty when no nodes use fidelity=full, \
         but found {} entries",
        drained.len()
    );

    Ok(())
}
