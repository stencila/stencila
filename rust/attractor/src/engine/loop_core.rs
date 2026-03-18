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
                // `init_run` with a previous context uses
                // `clone_runtime_context_for_restart` which creates a fresh
                // context excluding all transient keys (`internal.*`,
                // `current_node`, `outcome`, `preferred_label`).
                context = init_run(graph, Some(&context));
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
    } else {
        context.set("internal.thread_id", Value::Null);
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
        state.last_selected_edge = None;
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

    /// Read `internal.fidelity` and `internal.thread_id` from the context.
    fn capture_fidelity(context: &Context) -> (Option<String>, Option<String>) {
        let fidelity = context
            .get("internal.fidelity")
            .and_then(|v| v.as_str().map(String::from));
        let thread_id = context
            .get("internal.thread_id")
            .and_then(|v| v.as_str().map(String::from));
        (fidelity, thread_id)
    }

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

    /// A test handler that captures `internal.fidelity` and `internal.thread_id`
    /// from the context at execution time, keyed by node ID. Optionally returns
    /// a `jump_target` for specific nodes.
    struct CapturingHandler {
        captured: Arc<Mutex<CapturedFidelity>>,
        jump_targets: HashMap<String, String>,
    }

    #[async_trait]
    impl Handler for CapturingHandler {
        async fn execute(
            &self,
            node: &Node,
            context: &Context,
            _graph: &Graph,
        ) -> AttractorResult<Outcome> {
            self.captured
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .insert(node.id.clone(), capture_fidelity(context));

            let mut outcome = Outcome::success();
            if let Some(target) = self.jump_targets.get(&node.id) {
                outcome.jump_target = Some(target.clone());
            }
            Ok(outcome)
        }
    }

    /// Build a minimal Start → A → B → Exit graph with a custom A→B edge.
    fn build_linear_graph(name: &str, edge_ab: Edge) -> Graph {
        let mut graph = Graph::new(name);
        graph.add_node(make_start_node());
        graph.add_node(Node::new("A"));
        graph.add_node(Node::new("B"));
        graph.add_node(make_exit_node());

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
            jump_targets: HashMap::new(),
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

    /// §5.4 / Slice 4: after a `jump_target` advance, `last_selected_edge`
    /// must be cleared so the jumped-to node resolves fidelity from
    /// node/graph/default — not stale edge data.
    ///
    /// Graph: Start → A → B → C → Exit
    /// Edge A→B has fidelity="full". Node A's handler returns a normal
    /// success, so `advance()` selects edge A→B (setting `last_selected_edge`).
    /// Node B's handler returns `jump_target="C`", so `advance()` jumps directly
    /// to C without edge selection. If `last_selected_edge` is NOT cleared,
    /// node C will incorrectly inherit fidelity="full" from the stale A→B edge.
    #[tokio::test]
    async fn jump_target_clears_last_selected_edge() -> AttractorResult<()> {
        let mut graph = Graph::new("jump_clears_edge");
        graph.add_node(make_start_node());
        graph.add_node(Node::new("A"));
        graph.add_node(Node::new("B"));
        graph.add_node(Node::new("C"));
        graph.add_node(make_exit_node());

        graph.add_edge(Edge::new("Start", "A"));
        // Edge A→B carries fidelity="full" — this is the stale data source
        let mut edge_ab = Edge::new("A", "B");
        edge_ab
            .attrs
            .insert("fidelity".into(), AttrValue::String("full".into()));
        graph.add_edge(edge_ab);
        graph.add_edge(Edge::new("B", "C"));
        graph.add_edge(Edge::new("C", "Exit"));

        let captured = Arc::new(Mutex::new(HashMap::new()));

        // Node B returns jump_target="C", bypassing normal edge selection
        let handler = CapturingHandler {
            captured: captured.clone(),
            jump_targets: HashMap::from([("B".to_string(), "C".to_string())]),
        };

        let mut config = EngineConfig::new();
        config.skip_validation = true;
        config.registry.register(HandlerType::Codergen, handler);

        crate::engine::run_with_context(&graph, config, Context::new()).await?;

        let map = captured
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();

        // Node B was reached via edge A→B with fidelity="full" — correct
        let (fidelity_b, _) = map.get("B").expect("node B should have been executed");
        assert_eq!(
            fidelity_b.as_deref(),
            Some("full"),
            "node B should see fidelity='full' from edge A→B"
        );

        // Node C was reached via jump_target from B (no edge selection).
        // last_selected_edge must have been cleared, so fidelity falls
        // through to the default "compact".
        let (fidelity_c, thread_id_c) = map.get("C").expect("node C should have been executed");
        assert_eq!(
            fidelity_c.as_deref(),
            Some("compact"),
            "node C should have default fidelity='compact' after jump_target, \
             not stale fidelity='full' from edge A→B"
        );
        assert_eq!(
            thread_id_c.as_deref(),
            None,
            "node C should have no thread_id after jump_target clears last_selected_edge"
        );

        Ok(())
    }

    /// (`node_id`, fidelity, `thread_id`) tuple captured per handler invocation.
    type FidelityCapture = (String, Option<String>, Option<String>);

    /// A test handler that captures fidelity on each invocation and returns
    /// outcomes from a pre-built sequence (keyed by call index). Used for
    /// loop restart tests where the same node is visited multiple times.
    struct LoopCapturingHandler {
        captured: Arc<Mutex<Vec<FidelityCapture>>>,
        /// Pre-built outcomes indexed by call order.
        outcomes: Mutex<Vec<Outcome>>,
        call_count: Mutex<usize>,
    }

    #[async_trait]
    impl Handler for LoopCapturingHandler {
        async fn execute(
            &self,
            node: &Node,
            context: &Context,
            _graph: &Graph,
        ) -> AttractorResult<Outcome> {
            let (fidelity, thread_id) = capture_fidelity(context);
            self.captured
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push((node.id.clone(), fidelity, thread_id));

            let mut count = self
                .call_count
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let idx = *count;
            *count += 1;
            let outcomes = self
                .outcomes
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if idx < outcomes.len() {
                Ok(outcomes[idx].clone())
            } else {
                Ok(outcomes.last().cloned().unwrap_or_else(Outcome::success))
            }
        }
    }

    /// §5.4 / Slice 5: after a `LoopRestart`, `last_selected_edge` must be
    /// None so the first node in the restarted iteration resolves fidelity
    /// from node attributes, not stale edge data from the prior iteration.
    ///
    /// Graph: Start → A → B with two outgoing edges from B:
    ///   - B → A [label="retry", `loop_restart=true`, fidelity="full"]
    ///   - B → Exit [label="done"]
    ///
    /// First iteration: Start → A (no fidelity) → B returns `preferred_label="retry`"
    ///   → edge B→A selected (fidelity="full", `loop_restart=true`) → `LoopRestart`
    /// Second iteration: fresh `LoopState` → Start → A
    ///   → A must see fidelity from node attributes (truncate), not stale "full"
    #[tokio::test]
    async fn loop_restart_clears_last_selected_edge() -> AttractorResult<()> {
        let mut graph = Graph::new("loop_restart_fidelity");
        graph.add_node(make_start_node());

        // Node A has explicit fidelity="truncate" — this should be visible
        // after restart (not stale edge fidelity from prior iteration).
        let mut node_a = Node::new("A");
        node_a
            .attrs
            .insert("fidelity".into(), AttrValue::String("truncate".into()));
        graph.add_node(node_a);

        graph.add_node(Node::new("B"));
        graph.add_node(make_exit_node());

        graph.add_edge(Edge::new("Start", "A"));
        graph.add_edge(Edge::new("A", "B"));

        // Loop restart edge with fidelity="full" — this is the stale data
        let mut e_loop = Edge::new("B", "A");
        e_loop
            .attrs
            .insert("label".into(), AttrValue::String("retry".into()));
        e_loop
            .attrs
            .insert("loop_restart".into(), AttrValue::Boolean(true));
        e_loop
            .attrs
            .insert("fidelity".into(), AttrValue::String("full".into()));
        graph.add_edge(e_loop);

        let mut e_exit = Edge::new("B", "Exit");
        e_exit
            .attrs
            .insert("label".into(), AttrValue::String("done".into()));
        graph.add_edge(e_exit);

        let captured = Arc::new(Mutex::new(Vec::new()));

        // Sequence: call 0 = A (iter 1), call 1 = B returns "retry",
        //           call 2 = A (iter 2), call 3 = B returns "done"
        let handler = LoopCapturingHandler {
            captured: captured.clone(),
            outcomes: Mutex::new(vec![
                Outcome::success(),
                {
                    let mut o = Outcome::success();
                    o.preferred_label = "retry".to_string();
                    o
                },
                Outcome::success(),
                {
                    let mut o = Outcome::success();
                    o.preferred_label = "done".to_string();
                    o
                },
            ]),
            call_count: Mutex::new(0),
        };

        let mut config = EngineConfig::new();
        config.skip_validation = true;
        config.registry.register(HandlerType::Codergen, handler);

        crate::engine::run_with_context(&graph, config, Context::new()).await?;

        let entries = captured
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();

        // Find the second visit to node A (after loop restart).
        // entries[0] = A (first iteration), entries[2] = A (second iteration)
        let second_a = entries
            .iter()
            .filter(|(id, _, _)| id == "A")
            .nth(1)
            .expect("node A should have been executed twice (loop restart)");

        // After loop restart, LoopState::new() sets last_selected_edge=None,
        // so node A's fidelity should come from the node attribute ("truncate"),
        // NOT from the stale loop_restart edge fidelity ("full").
        assert_eq!(
            second_a.1.as_deref(),
            Some("truncate"),
            "after loop restart, node A should resolve fidelity from node attribute ('truncate'), \
             not from stale loop_restart edge fidelity ('full')"
        );
        assert_eq!(
            second_a.2.as_deref(),
            None,
            "after loop restart, node A should have no thread_id since fidelity is not 'full'"
        );

        Ok(())
    }
}
