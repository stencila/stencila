//! Core traversal loop (§3.2).
//!
//! Implements the main execution loop: find start → execute node →
//! select edge → advance → repeat until exit or failure.

use indexmap::IndexMap;
use serde_json::Value;

use crate::checkpoint::Checkpoint;
use crate::context::Context;
use crate::edge_selection::select_edge;
use crate::error::{AttractorError, AttractorResult};
use crate::events::PipelineEvent;
use crate::fidelity::{resolve_fidelity, resolve_thread_id};
use crate::graph::{AttrValue, Graph, Node};
use crate::retry::{build_retry_policy, execute_with_retry};
use crate::types::FidelityMode;
use crate::types::{HandlerType, Outcome, StageStatus};

use super::EngineConfig;
use super::routing::{check_goal_gates, find_fail_edge, get_retry_target};

/// Mutable state carried through the traversal loop.
struct LoopState {
    current_node_id: String,
    completed_nodes: Vec<String>,
    node_outcomes: IndexMap<String, Outcome>,
    /// Outcome status strings per node, stored in the checkpoint so
    /// resume can reconstruct accurate outcomes for goal-gate checks.
    node_statuses: IndexMap<String, String>,
    node_retries: IndexMap<String, u32>,
    stage_index: usize,
    last_outcome: Outcome,
    /// The (from, to) node IDs of the last edge selected by `advance()`.
    /// Used to resolve fidelity and `thread_id` from edge attributes before
    /// executing the next node.
    last_selected_edge: Option<(String, String)>,
}

impl LoopState {
    /// Create fresh state for a new (non-resumed) run starting at the given node.
    fn new(start_node_id: String) -> Self {
        Self {
            current_node_id: start_node_id,
            completed_nodes: Vec::new(),
            node_outcomes: IndexMap::new(),
            node_statuses: IndexMap::new(),
            node_retries: IndexMap::new(),
            stage_index: 0,
            last_outcome: Outcome::success(),
            last_selected_edge: None,
        }
    }
}

/// Run the core traversal loop with a pre-created context.
///
/// Used when the caller needs to provide a custom context backend (e.g.
/// SQLite-backed). Populates the context with goal and graph attributes
/// just like [`run_loop`].
pub(crate) async fn run_loop_with_context(
    graph: &Graph,
    config: EngineConfig,
    context: Context,
) -> AttractorResult<Outcome> {
    let start_node = graph.find_start_node()?;
    graph.find_exit_node()?;

    populate_context(graph, &context);

    config.emitter.emit(PipelineEvent::PipelineStarted {
        pipeline_name: graph.name.clone(),
    });

    let state = LoopState::new(start_node.id.clone());
    execute_loop(graph, config, context, state).await
}

/// Run the core traversal loop for a pipeline graph.
pub(crate) async fn run_loop(graph: &Graph, config: EngineConfig) -> AttractorResult<Outcome> {
    // Validate both start and exit nodes exist before running (§3.2).
    let start_node = graph.find_start_node()?;
    graph.find_exit_node()?;

    let context = init_run(graph, None);

    config.emitter.emit(PipelineEvent::PipelineStarted {
        pipeline_name: graph.name.clone(),
    });

    let state = LoopState::new(start_node.id.clone());
    execute_loop(graph, config, context, state).await
}

/// Resume the core traversal loop from a previously saved checkpoint.
///
/// Similar to [`run_loop`] but starts from the checkpoint's `next_node_id`
/// with the restored context and completed-node set. A fresh run directory
/// is created for the resumed run.
pub(crate) async fn resume_loop(
    graph: &Graph,
    config: EngineConfig,
    resume_state: crate::resume::ResumeState,
) -> AttractorResult<Outcome> {
    graph.find_exit_node()?;

    let context = resume_state.context;

    config.emitter.emit(PipelineEvent::PipelineStarted {
        pipeline_name: graph.name.clone(),
    });

    // Restore retry counts from checkpoint.
    let mut node_retries = IndexMap::new();
    for node_id in &resume_state.completed_nodes_ordered {
        if let Some(count) = context.get_i64(&format!("internal.retry_count.{node_id}")) {
            #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
            node_retries.insert(node_id.clone(), count as u32);
        }
    }

    // Restore accurate outcomes for pre-checkpoint nodes so goal-gate
    // checks correctly enforce §3.4. Use the checkpoint's node_statuses
    // when available; fall back to Outcome::success() for legacy
    // checkpoints that lack this field.
    let mut node_outcomes = IndexMap::new();
    let mut node_statuses = IndexMap::new();
    for node_id in &resume_state.completed_nodes_ordered {
        let status_str = resume_state
            .node_statuses
            .get(node_id)
            .map_or("success", String::as_str);
        let outcome = match status_str {
            "success" => Outcome::success(),
            "fail" => Outcome::fail("restored from checkpoint"),
            "partial_success" => {
                let mut o = Outcome::success();
                o.status = StageStatus::PartialSuccess;
                o
            }
            "retry" => {
                let mut o = Outcome::success();
                o.status = StageStatus::Retry;
                o
            }
            "skipped" => {
                let mut o = Outcome::success();
                o.status = StageStatus::Skipped;
                o
            }
            // Unknown/corrupted status — conservatively treat as fail
            // to prevent incorrectly satisfying goal gates.
            _ => Outcome::fail("unknown status in checkpoint"),
        };
        node_outcomes.insert(node_id.clone(), outcome);
        node_statuses.insert(node_id.clone(), status_str.to_string());
    }

    // Apply fidelity degradation marker (§5.3): when the checkpoint's
    // previous node used full fidelity, the first resumed hop must
    // degrade because in-memory LLM sessions can't be serialized.
    if resume_state.degrade_fidelity {
        context.set("internal.resume_degrade_fidelity", Value::Bool(true));
    }

    let stage_index = resume_state.completed_nodes_ordered.len();

    let state = LoopState {
        current_node_id: resume_state.next_node_id,
        completed_nodes: resume_state.completed_nodes_ordered,
        node_outcomes,
        node_statuses,
        node_retries,
        stage_index,
        last_outcome: Outcome::success(),
        last_selected_edge: None,
    };

    execute_loop(graph, config, context, state).await
}

/// Initialize the context from graph attributes.
///
/// Creates a new context populated with the graph's `goal` and other
/// graph-level attributes. Used for fresh runs and loop restarts
/// (§2.7/§3.2).
fn init_run(graph: &Graph, previous: Option<&Context>) -> Context {
    let context = previous.map_or_else(Context::new, clone_runtime_context_for_restart);
    populate_context(graph, &context);
    context
}

/// Clear transient execution keys that should not survive a loop restart.
fn clear_restart_transients(context: &Context) {
    context.set("current_node", Value::Null);
    context.set("outcome", Value::Null);
    context.set("preferred_label", Value::Null);

    for key in context.snapshot().keys() {
        if key.starts_with("internal.") {
            context.set(key.clone(), Value::Null);
        }
    }
}

/// Clone only restart-safe runtime context into a fresh in-memory backend.
///
/// Loop restarts should preserve user-visible/runtime variables from prior
/// iterations (e.g. handler context updates) while dropping transient
/// execution bookkeeping such as `internal.*`, `current_node`, `outcome`,
/// and `preferred_label` so the next iteration starts cleanly.
fn clone_runtime_context_for_restart(previous: &Context) -> Context {
    let context = Context::new();

    for (key, value) in previous.snapshot() {
        if key == "current_node"
            || key == "outcome"
            || key == "preferred_label"
            || key.starts_with("internal.")
        {
            continue;
        }

        context.set(key, value);
    }

    context
}

/// Populate a context with goal and graph-level attributes.
fn populate_context(graph: &Graph, context: &Context) {
    let goal = graph
        .get_graph_attr("goal")
        .map(AttrValue::to_string_value)
        .unwrap_or_default();

    if !goal.is_empty() {
        context.set("goal", Value::String(goal));
    }
    for (key, value) in &graph.graph_attrs {
        context.set(
            format!("graph.{key}"),
            Value::String(value.to_string_value()),
        );
    }
}

/// The shared traversal loop used by both fresh runs and resumed runs.
///
/// Executes nodes in traversal order, selecting edges via the 5-step
/// algorithm, managing retries, checkpoints, and events.
#[allow(clippy::too_many_lines)]
async fn execute_loop(
    graph: &Graph,
    config: EngineConfig,
    mut context: Context,
    mut state: LoopState,
) -> AttractorResult<Outcome> {
    loop {
        let node =
            graph
                .get_node(&state.current_node_id)
                .ok_or_else(|| AttractorError::NodeNotFound {
                    node_id: state.current_node_id.clone(),
                })?;

        // Terminal check: exit node → goal gates → finish
        if node.handler_type() == HandlerType::Exit {
            let gate_result = check_goal_gates(graph, &state.node_outcomes);
            if !gate_result.satisfied {
                if let Some(target) = resolve_gate_retry(graph, &gate_result) {
                    if let Some(backend) = context.sqlite_backend()
                        && let Err(e) = backend.insert_edge(
                            #[allow(clippy::cast_possible_wrap)]
                            (state.stage_index as i64),
                            &node.id,
                            &target,
                            Some("goal_gate_retry"),
                        )
                    {
                        tracing::warn!(
                            "SQLite insert_edge({} -> {target}) for goal gate failed: {e}",
                            node.id
                        );
                    }
                    state.current_node_id = target;
                    state.stage_index += 1;
                    continue;
                }
                let reason = format!(
                    "goal gate not satisfied: {}",
                    gate_result.failed_node_id.as_deref().unwrap_or("unknown")
                );
                config.emitter.emit(PipelineEvent::PipelineFailed {
                    pipeline_name: graph.name.clone(),
                    reason: reason.clone(),
                });
                return Ok(Outcome::fail(reason));
            }

            let outcome = execute_node(node, graph, &config, &context, state.stage_index).await?;
            record_and_checkpoint(node, &outcome, &context, &mut state, None);
            config.emitter.emit(PipelineEvent::CheckpointSaved {
                node_id: node.id.clone(),
            });
            config.emitter.emit(PipelineEvent::PipelineCompleted {
                pipeline_name: graph.name.clone(),
                outcome: outcome.clone(),
            });
            return Ok(outcome);
        }

        context.set("current_node", Value::String(node.id.clone()));
        context.set(
            "internal.stage_index",
            Value::Number(serde_json::Number::from(state.stage_index as u64)),
        );
        config.emitter.emit(PipelineEvent::StageStarted {
            node_id: node.id.clone(),
            stage_index: state.stage_index,
            handler_type: node.handler_type().to_string(),
        });

        // Resolve fidelity and thread_id from the incoming edge (§5.4)
        // and store as context keys before execution so handlers can
        // observe them.
        apply_edge_fidelity(node, state.last_selected_edge.as_ref(), graph, &context);

        let outcome = execute_node(node, graph, &config, &context, state.stage_index).await?;

        // Clear one-shot fidelity degradation marker after the first
        // resumed hop (§5.3). For fresh runs the key is absent, so
        // the get returns None and no write occurs.
        if context
            .get("internal.resume_degrade_fidelity")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
        {
            context.set("internal.resume_degrade_fidelity", Value::Bool(false));
        }

        // §3.2 Step 4: Apply context updates from outcome before edge
        // selection so that routing conditions see the updated context.
        if !outcome.context_updates.is_empty() {
            context.apply_updates(&outcome.context_updates);
        }
        context.set(
            "outcome",
            Value::String(outcome.status.as_str().to_string()),
        );
        // Always overwrite to clear stale values from earlier stages (§5.1).
        context.set(
            "preferred_label",
            Value::String(outcome.preferred_label.clone()),
        );

        // Determine next node *before* saving the checkpoint so the
        // checkpoint contains the resolved next_node_id in a single
        // write (§5.3). This covers both success and failure paths.
        let (next_node_id, advance_result) = if outcome.status == StageStatus::Fail {
            let fail_next = route_failure(node, graph, &outcome, &context);
            (fail_next, None)
        } else {
            let ar = advance(node, &outcome, &context, graph, &mut state);
            let next = match &ar {
                AdvanceResult::Continue => Some(state.current_node_id.clone()),
                AdvanceResult::LoopRestart(target) => Some(target.clone()),
                AdvanceResult::End => None,
            };
            (next, Some(ar))
        };

        record_and_checkpoint(
            node,
            &outcome,
            &context,
            &mut state,
            next_node_id.as_deref(),
        );

        // Persist edge traversal to SQLite when available.
        if let (Some(next), Some(backend)) = (next_node_id.as_deref(), context.sqlite_backend())
            && let Err(e) = backend.insert_edge(
                #[allow(clippy::cast_possible_wrap)]
                (state.stage_index as i64),
                &node.id,
                next,
                None,
            )
        {
            tracing::warn!("SQLite insert_edge({} -> {next}) failed: {e}", node.id);
        }

        config.emitter.emit(PipelineEvent::CheckpointSaved {
            node_id: node.id.clone(),
        });

        if outcome.status == StageStatus::Fail {
            config.emitter.emit(PipelineEvent::StageFailed {
                node_id: node.id.clone(),
                stage_index: state.stage_index,
                reason: outcome.failure_reason.clone(),
            });
            if let Some(next) = next_node_id {
                state.current_node_id = next;
                state.stage_index += 1;
                continue;
            }
            config.emitter.emit(PipelineEvent::PipelineFailed {
                pipeline_name: graph.name.clone(),
                reason: outcome.failure_reason.clone(),
            });
            return Ok(outcome);
        }

        config.emitter.emit(PipelineEvent::StageCompleted {
            node_id: node.id.clone(),
            stage_index: state.stage_index,
            outcome: outcome.clone(),
        });
        state.last_outcome.clone_from(&outcome);

        match advance_result {
            Some(AdvanceResult::Continue) => {
                state.stage_index += 1;
            }
            Some(AdvanceResult::LoopRestart(target)) => {
                // §2.7/§3.2: preserve runtime context across loop restarts,
                // while clearing transient execution bookkeeping.
                let new_context = init_run(graph, Some(&context));
                clear_restart_transients(&new_context);
                context = new_context;
                state = LoopState::new(target);
            }
            Some(AdvanceResult::End) | None => {
                config.emitter.emit(PipelineEvent::PipelineCompleted {
                    pipeline_name: graph.name.clone(),
                    outcome: state.last_outcome.clone(),
                });
                return Ok(state.last_outcome);
            }
        }
    }
}

/// Execute a node through its handler with retry.
async fn execute_node(
    node: &Node,
    graph: &Graph,
    config: &EngineConfig,
    context: &Context,
    stage_index: usize,
) -> AttractorResult<Outcome> {
    let handler = config
        .registry
        .resolve(node)
        .ok_or_else(|| AttractorError::HandlerFailed {
            node_id: node.id.clone(),
            reason: format!("no handler registered for type '{}'", node.handler_type()),
        })?;

    let policy = build_retry_policy(node, graph);
    Ok(execute_with_retry(
        &handler,
        node,
        context,
        graph,
        &policy,
        config.emitter.as_ref(),
        stage_index,
    )
    .await)
}

/// Record outcome, sync retry counts, and save a checkpoint.
///
/// Context updates are applied earlier in the loop (before edge
/// selection per §3.2 Step 4), so this function only records the
/// outcome and persists checkpoint state.
///
/// The optional `next_node_id` is written into the checkpoint so that
/// resume routing is unambiguous even in branching graphs (§5.3).
fn record_and_checkpoint(
    node: &Node,
    outcome: &Outcome,
    context: &Context,
    state: &mut LoopState,
    next_node_id: Option<&str>,
) {
    state.completed_nodes.push(node.id.clone());
    state.node_outcomes.insert(node.id.clone(), outcome.clone());
    state
        .node_statuses
        .insert(node.id.clone(), outcome.status.as_str().to_string());

    // Sync retry count from context into LoopState so checkpoints
    // contain accurate retry metadata (§5.3).
    if let Some(count) = context.get_i64(&format!("internal.retry_count.{}", node.id)) {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        state.node_retries.insert(node.id.clone(), count as u32);
    }

    // Persist node record to SQLite when available.
    if let Some(backend) = context.sqlite_backend() {
        let retry_count = i64::from(state.node_retries.get(&node.id).copied().unwrap_or(0));
        let failure_reason = if outcome.status == StageStatus::Fail {
            Some(outcome.failure_reason.as_str())
        } else {
            None
        };
        let model = context
            .get(&format!("internal.model.{}", node.id))
            .and_then(|v| v.as_str().map(std::string::ToString::to_string));
        let provider = context
            .get(&format!("internal.provider.{}", node.id))
            .and_then(|v| v.as_str().map(std::string::ToString::to_string));
        let input_tokens = context.get_i64(&format!("internal.input_tokens.{}", node.id));
        let output_tokens = context.get_i64(&format!("internal.output_tokens.{}", node.id));
        let record = crate::sqlite_backend::NodeRecord {
            node_id: &node.id,
            status: outcome.status.as_str(),
            model: model.as_deref(),
            provider: provider.as_deref(),
            duration_ms: None,
            input_tokens,
            output_tokens,
            retry_count: Some(retry_count),
            failure_reason,
        };
        if let Err(e) = backend.upsert_node(&record) {
            tracing::warn!("SQLite upsert_node({}) failed: {e}", node.id);
        }
    }

    let checkpoint = Checkpoint::from_context(
        context,
        &node.id,
        state.completed_nodes.clone(),
        state.node_statuses.clone(),
        state.node_retries.clone(),
    );
    let _checkpoint = match next_node_id {
        Some(next) => checkpoint.with_next_node(next),
        None => checkpoint,
    };
}

/// Try to find a retry target for a failed goal gate.
fn resolve_gate_retry(
    graph: &Graph,
    gate_result: &super::routing::GoalGateResult,
) -> Option<String> {
    let failed_id = gate_result.failed_node_id.as_deref()?;
    let failed_node = graph.get_node(failed_id)?;
    get_retry_target(failed_node, graph)
}

/// Resolve fidelity mode and thread ID from the last selected edge and
/// write them into the context as `internal.fidelity` / `internal.thread_id`.
fn apply_edge_fidelity(
    node: &Node,
    last_selected_edge: Option<&(String, String)>,
    graph: &Graph,
    context: &Context,
) {
    let incoming_edge = last_selected_edge.and_then(|(from, to)| {
        graph
            .incoming_edges(to)
            .into_iter()
            .find(|e| e.from == *from)
    });
    let fidelity = resolve_fidelity(node, incoming_edge, graph);
    context.set("internal.fidelity", Value::String(fidelity.to_string()));

    if fidelity == FidelityMode::Full {
        let previous_node_id = last_selected_edge.map_or("", |(from, _)| from.as_str());
        let thread_id = resolve_thread_id(node, incoming_edge, graph, previous_node_id);
        context.set("internal.thread_id", Value::String(thread_id));
    }
}

/// Route a failed node to the next target, if possible.
fn route_failure(
    node: &Node,
    graph: &Graph,
    outcome: &Outcome,
    context: &Context,
) -> Option<String> {
    if let Some(fail_edge) = find_fail_edge(&node.id, graph, outcome, context) {
        return Some(fail_edge.to.clone());
    }
    get_retry_target(node, graph)
}

/// Result of advancing to the next node.
enum AdvanceResult {
    Continue,
    /// Loop restart with the target node ID to start from.
    LoopRestart(String),
    End,
}

/// Select the next edge and advance.
fn advance(
    node: &Node,
    outcome: &Outcome,
    context: &Context,
    graph: &Graph,
    state: &mut LoopState,
) -> AdvanceResult {
    // If the handler set an explicit jump target (e.g. the parallel
    // handler routing to the structural fan-in node), skip normal edge
    // selection and advance directly to that node.
    if let Some(target) = &outcome.jump_target {
        state.current_node_id.clone_from(target);
        return AdvanceResult::Continue;
    }

    // A parallel handler with no jump target means all branches were
    // executed internally and no convergence point exists. Falling
    // through to select_edge would re-enter an already-executed branch
    // (the parallel node's outgoing edges ARE the branch entries).
    // Treat this as a terminal node — the pipeline ends here.
    if node.handler_type() == HandlerType::Parallel {
        return AdvanceResult::End;
    }

    let Some(edge) = select_edge(&node.id, outcome, context, graph) else {
        return AdvanceResult::End;
    };

    state.last_selected_edge = Some((edge.from.clone(), edge.to.clone()));

    let is_loop_restart = edge
        .get_attr("loop_restart")
        .and_then(AttrValue::as_bool)
        .unwrap_or(false);

    if is_loop_restart {
        return AdvanceResult::LoopRestart(edge.to.clone());
    }

    state.current_node_id.clone_from(&edge.to);
    AdvanceResult::Continue
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    use async_trait::async_trait;

    use crate::Edge;
    use crate::handler::Handler;

    use super::*;

    /// Fidelity and `thread_id` captured from context, keyed by node ID.
    type CapturedFidelity = HashMap<String, (Option<String>, Option<String>)>;

    /// A test handler that captures `internal.fidelity` and `internal.thread_id`
    /// from the context at execution time, keyed by node ID.
    struct CapturingHandler {
        captured: Arc<Mutex<CapturedFidelity>>,
    }

    #[async_trait]
    impl Handler for CapturingHandler {
        async fn execute(
            &self,
            node: &Node,
            context: &Context,
            _graph: &Graph,
        ) -> AttractorResult<Outcome> {
            let fidelity = context
                .get("internal.fidelity")
                .and_then(|v| v.as_str().map(String::from));
            let thread_id = context
                .get("internal.thread_id")
                .and_then(|v| v.as_str().map(String::from));
            self.captured
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert(node.id.clone(), (fidelity, thread_id));
            Ok(Outcome::success())
        }
    }

    /// Build a minimal Start → A → B → Exit graph with a custom A→B edge.
    fn build_linear_graph(name: &str, edge_ab: Edge) -> Graph {
        let mut graph = Graph::new(name);

        let mut start = Node::new("Start");
        start
            .attrs
            .insert("shape".into(), AttrValue::String(Graph::START_SHAPE.into()));
        graph.add_node(start);

        graph.add_node(Node::new("A"));
        graph.add_node(Node::new("B"));

        let mut exit = Node::new("Exit");
        exit.attrs
            .insert("shape".into(), AttrValue::String(Graph::EXIT_SHAPE.into()));
        graph.add_node(exit);

        graph.add_edge(Edge::new("Start", "A"));
        graph.add_edge(edge_ab);
        graph.add_edge(Edge::new("B", "Exit"));

        graph
    }

    /// Run a graph with a `CapturingHandler` and return the captured fidelity map.
    async fn run_capturing(graph: &Graph) -> AttractorResult<CapturedFidelity> {
        let captured = Arc::new(Mutex::new(HashMap::new()));

        let handler = CapturingHandler {
            captured: captured.clone(),
        };

        let mut config = EngineConfig::new();
        config.skip_validation = true;
        config.registry.register(HandlerType::Codergen, handler);

        crate::engine::run_with_context(graph, config, Context::new()).await?;

        let map = captured
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        Ok(map)
    }

    #[tokio::test]
    async fn default_fidelity_is_compact_and_thread_id_is_absent() -> AttractorResult<()> {
        let graph = build_linear_graph("no_fidelity_test", Edge::new("A", "B"));
        let map = run_capturing(&graph).await?;

        // Node A: traversed from Start with no fidelity attributes.
        let (fidelity_a, thread_id_a) = map.get("A").expect("node A should have been executed");
        assert_eq!(
            fidelity_a.as_deref(),
            Some("compact"),
            "internal.fidelity should default to 'compact' for node A"
        );
        assert_eq!(
            thread_id_a.as_deref(),
            None,
            "internal.thread_id should be absent when fidelity is not 'full' (node A)"
        );

        // Node B: traversed from A with no fidelity attributes.
        let (fidelity_b, thread_id_b) = map.get("B").expect("node B should have been executed");
        assert_eq!(
            fidelity_b.as_deref(),
            Some("compact"),
            "internal.fidelity should default to 'compact' for node B"
        );
        assert_eq!(
            thread_id_b.as_deref(),
            None,
            "internal.thread_id should be absent when fidelity is not 'full' (node B)"
        );

        Ok(())
    }

    #[tokio::test]
    async fn edge_fidelity_and_thread_id_propagate_to_context() -> AttractorResult<()> {
        let mut edge_ab = Edge::new("A", "B");
        edge_ab
            .attrs
            .insert("fidelity".into(), AttrValue::String("full".into()));
        edge_ab
            .attrs
            .insert("thread_id".into(), AttrValue::String("t1".into()));

        let graph = build_linear_graph("fidelity_test", edge_ab);
        let map = run_capturing(&graph).await?;

        // After traversing edge A→B (which has fidelity="full", thread_id="t1"),
        // node B should see these values in the context.
        let (fidelity, thread_id) = map.get("B").expect("node B should have been executed");
        assert_eq!(
            fidelity.as_deref(),
            Some("full"),
            "internal.fidelity should be 'full' for node B"
        );
        assert_eq!(
            thread_id.as_deref(),
            Some("t1"),
            "internal.thread_id should be 't1' for node B"
        );

        Ok(())
    }

    /// §5.4 edge-level fidelity override: when node B has `fidelity="truncate"`
    /// but the incoming edge A→B has `fidelity="full"`, the edge wins.
    ///
    /// This tests the precedence chain (edge > node > graph > default) through
    /// the engine's `last_selected_edge` mechanism, not just the pure
    /// `resolve_fidelity` function.
    #[tokio::test]
    async fn edge_fidelity_overrides_node_fidelity_via_last_selected_edge() -> AttractorResult<()> {
        // Edge A→B overrides with fidelity="full"
        let mut edge_ab = Edge::new("A", "B");
        edge_ab
            .attrs
            .insert("fidelity".into(), AttrValue::String("full".into()));

        let mut graph = build_linear_graph("edge_override_node_fidelity", edge_ab);

        // Node B has its own fidelity="truncate"
        graph
            .get_node_mut("B")
            .expect("node B exists")
            .attrs
            .insert("fidelity".into(), AttrValue::String("truncate".into()));

        let map = run_capturing(&graph).await?;

        // Node B: the edge fidelity="full" must override node fidelity="truncate"
        let (fidelity_b, thread_id_b) = map.get("B").expect("node B should have been executed");
        assert_eq!(
            fidelity_b.as_deref(),
            Some("full"),
            "edge fidelity='full' should override node fidelity='truncate' for node B"
        );
        // With fidelity="full" and no explicit thread_id on node or edge,
        // thread_id falls back to the previous node ID ("A") per §5.4 step 5.
        assert_eq!(
            thread_id_b.as_deref(),
            Some("A"),
            "thread_id should fall back to previous node ID 'A' when no explicit thread_id is set"
        );

        Ok(())
    }

    /// §5.4 edge-level fidelity override with explicit `thread_id` on the edge:
    /// when the edge carries both `fidelity="full"` and `thread_id="edge_thread"`,
    /// those values must appear in the context — even if the node has a different
    /// fidelity and its own `thread_id`.
    #[tokio::test]
    async fn edge_fidelity_full_with_edge_thread_id_overrides_node() -> AttractorResult<()> {
        // Edge A→B carries fidelity="full" and thread_id="edge_thread"
        let mut edge_ab = Edge::new("A", "B");
        edge_ab
            .attrs
            .insert("fidelity".into(), AttrValue::String("full".into()));
        edge_ab
            .attrs
            .insert("thread_id".into(), AttrValue::String("edge_thread".into()));

        let mut graph = build_linear_graph("edge_fidelity_thread_override", edge_ab);

        // Node B has fidelity="compact" and thread_id="node_thread"
        let node_b = graph.get_node_mut("B").expect("node B exists");
        node_b
            .attrs
            .insert("fidelity".into(), AttrValue::String("compact".into()));
        node_b
            .attrs
            .insert("thread_id".into(), AttrValue::String("node_thread".into()));

        let map = run_capturing(&graph).await?;

        let (fidelity_b, thread_id_b) = map.get("B").expect("node B should have been executed");
        assert_eq!(
            fidelity_b.as_deref(),
            Some("full"),
            "edge fidelity='full' should override node fidelity='compact'"
        );
        // thread_id resolution §5.4: node thread_id (step 1) has highest priority,
        // so even though the edge also has thread_id="edge_thread", the node's
        // thread_id="node_thread" wins.
        assert_eq!(
            thread_id_b.as_deref(),
            Some("node_thread"),
            "node thread_id should take priority over edge thread_id per §5.4 step 1"
        );

        Ok(())
    }

    /// §5.4 precedence: graph-level `default_fidelity` is overridden by
    /// edge-level fidelity. Node B has no fidelity, graph has
    /// `default_fidelity="summary:high`", but edge A→B has fidelity="full".
    #[tokio::test]
    async fn edge_fidelity_overrides_graph_default() -> AttractorResult<()> {
        let mut edge_ab = Edge::new("A", "B");
        edge_ab
            .attrs
            .insert("fidelity".into(), AttrValue::String("full".into()));

        let mut graph = build_linear_graph("edge_override_graph_default", edge_ab);

        graph.graph_attrs.insert(
            "default_fidelity".into(),
            AttrValue::String("summary:high".into()),
        );

        let map = run_capturing(&graph).await?;

        // Node A has no edge fidelity and no node fidelity — should use graph default
        let (fidelity_a, _) = map.get("A").expect("node A should have been executed");
        assert_eq!(
            fidelity_a.as_deref(),
            Some("summary:high"),
            "node A should inherit graph default_fidelity='summary:high'"
        );

        // Node B's incoming edge has fidelity="full" — edge overrides graph default
        let (fidelity_b, thread_id_b) = map.get("B").expect("node B should have been executed");
        assert_eq!(
            fidelity_b.as_deref(),
            Some("full"),
            "edge fidelity='full' should override graph default_fidelity='summary:high'"
        );
        assert_eq!(
            thread_id_b.as_deref(),
            Some("A"),
            "thread_id should fall back to previous node ID 'A'"
        );

        Ok(())
    }
}
