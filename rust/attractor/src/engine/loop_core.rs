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
use crate::graph::Graph;
use crate::retry::{build_retry_policy, execute_with_retry};
use crate::run_directory::{Manifest, RunDirectory};
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
}

/// Run the core traversal loop for a pipeline graph.
pub(crate) async fn run_loop(graph: &Graph, config: EngineConfig) -> AttractorResult<Outcome> {
    // Validate both start and exit nodes exist before running (§3.2).
    let start_node = graph.find_start_node()?;
    graph.find_exit_node()?;

    let (run_dir, context) = init_run(graph, &config)?;

    config.emitter.emit(PipelineEvent::PipelineStarted {
        pipeline_name: graph.name.clone(),
    });

    let state = LoopState {
        current_node_id: start_node.id.clone(),
        completed_nodes: Vec::new(),
        node_outcomes: IndexMap::new(),
        node_statuses: IndexMap::new(),
        node_retries: IndexMap::new(),
        stage_index: 0,
        last_outcome: Outcome::success(),
    };

    execute_loop(graph, config, run_dir, context, state).await
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

    let run_dir = create_run_dir(graph, &config)?;
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
    };

    execute_loop(graph, config, run_dir, context, state).await
}

/// Create a run directory with manifest for a pipeline run.
///
/// Used by both fresh runs (via [`init_run`]) and resumed runs which
/// supply their own restored context.
fn create_run_dir(graph: &Graph, config: &EngineConfig) -> AttractorResult<RunDirectory> {
    let run_id = chrono::Utc::now().format("%Y%m%dT%H%M%S%.6f").to_string();
    let run_dir = RunDirectory::create(config.logs_root.join(&run_id))?;

    let goal = graph
        .get_graph_attr("goal")
        .map(super::super::graph::AttrValue::to_string_value)
        .unwrap_or_default();

    run_dir.write_manifest(&Manifest {
        name: graph.name.clone(),
        goal,
        start_time: chrono::Utc::now().to_rfc3339(),
    })?;

    Ok(run_dir)
}

/// Initialize the run directory and context from graph attributes.
///
/// Creates a fresh run directory and a new context populated with the
/// graph's `goal` and other graph-level attributes. Used for fresh
/// runs and loop restarts (§2.7/§3.2).
fn init_run(graph: &Graph, config: &EngineConfig) -> AttractorResult<(RunDirectory, Context)> {
    let run_dir = create_run_dir(graph, config)?;

    let goal = graph
        .get_graph_attr("goal")
        .map(super::super::graph::AttrValue::to_string_value)
        .unwrap_or_default();

    let context = Context::new();
    if !goal.is_empty() {
        context.set("goal", Value::String(goal));
    }
    for (key, value) in &graph.graph_attrs {
        context.set(
            format!("graph.{key}"),
            Value::String(value.to_string_value()),
        );
    }

    Ok((run_dir, context))
}

/// The shared traversal loop used by both fresh runs and resumed runs.
///
/// Executes nodes in traversal order, selecting edges via the 5-step
/// algorithm, managing retries, checkpoints, and events.
#[allow(clippy::too_many_lines)]
async fn execute_loop(
    graph: &Graph,
    config: EngineConfig,
    mut run_dir: RunDirectory,
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
                    state.current_node_id = target;
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

            let outcome =
                execute_node(node, graph, &config, &run_dir, &context, state.stage_index).await?;
            record_and_checkpoint(node, &outcome, &run_dir, &context, &mut state, None)?;
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
        });

        let outcome =
            execute_node(node, graph, &config, &run_dir, &context, state.stage_index).await?;

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
            &run_dir,
            &context,
            &mut state,
            next_node_id.as_deref(),
        )?;
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
                // §2.7/§3.2: create fresh run directory and context
                let (new_run_dir, new_context) = init_run(graph, &config)?;
                run_dir = new_run_dir;
                context = new_context;
                state.completed_nodes.clear();
                state.node_outcomes.clear();
                state.node_statuses.clear();
                state.node_retries.clear();
                state.stage_index = 0;
                state.current_node_id = target;
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
    node: &crate::graph::Node,
    graph: &Graph,
    config: &EngineConfig,
    run_dir: &RunDirectory,
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
        run_dir.root(),
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
    node: &crate::graph::Node,
    outcome: &Outcome,
    run_dir: &RunDirectory,
    context: &Context,
    state: &mut LoopState,
    next_node_id: Option<&str>,
) -> AttractorResult<()> {
    run_dir.write_status(&node.id, outcome)?;
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

    let checkpoint = Checkpoint::from_context(
        context,
        &node.id,
        state.completed_nodes.clone(),
        state.node_statuses.clone(),
        state.node_retries.clone(),
    );
    let checkpoint = match next_node_id {
        Some(next) => checkpoint.with_next_node(next),
        None => checkpoint,
    };
    checkpoint.save(&run_dir.checkpoint_path())
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

/// Route a failed node to the next target, if possible.
fn route_failure(
    node: &crate::graph::Node,
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
    node: &crate::graph::Node,
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

    let is_loop_restart = edge
        .get_attr("loop_restart")
        .and_then(super::super::graph::AttrValue::as_bool)
        .unwrap_or(false);

    if is_loop_restart {
        return AdvanceResult::LoopRestart(edge.to.clone());
    }

    state.current_node_id.clone_from(&edge.to);
    AdvanceResult::Continue
}
