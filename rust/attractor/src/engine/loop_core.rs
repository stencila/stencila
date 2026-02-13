//! Core traversal loop (§3.2).
//!
//! Implements the main execution loop: find start → execute node →
//! select edge → advance → repeat until exit or failure.

use std::collections::HashMap;

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
use crate::types::{Outcome, StageStatus};

use super::EngineConfig;
use super::routing::{check_goal_gates, find_fail_edge, get_retry_target};

/// Mutable state carried through the traversal loop.
struct LoopState {
    current_node_id: String,
    completed_nodes: Vec<String>,
    node_outcomes: HashMap<String, Outcome>,
    node_retries: IndexMap<String, u32>,
    stage_index: usize,
    last_outcome: Outcome,
}

/// Run the core traversal loop for a pipeline graph.
#[allow(clippy::too_many_lines)]
pub(crate) async fn run_loop(graph: &Graph, config: EngineConfig) -> AttractorResult<Outcome> {
    // Validate both start and exit nodes exist before running (§3.2).
    let start_node = graph.find_start_node()?;
    graph.find_exit_node()?;

    let (mut run_dir, mut context) = init_run(graph, &config)?;

    config.emitter.emit(PipelineEvent::PipelineStarted {
        pipeline_name: graph.name.clone(),
    });

    let mut state = LoopState {
        current_node_id: start_node.id.clone(),
        completed_nodes: Vec::new(),
        node_outcomes: HashMap::new(),
        node_retries: IndexMap::new(),
        stage_index: 0,
        last_outcome: Outcome::success(),
    };

    loop {
        let node =
            graph
                .get_node(&state.current_node_id)
                .ok_or_else(|| AttractorError::NodeNotFound {
                    node_id: state.current_node_id.clone(),
                })?;

        // Terminal check: exit node → goal gates → finish
        if node.handler_type() == "exit" {
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
            record_and_checkpoint(node, &outcome, &run_dir, &context, &mut state, false)?;
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
        config.emitter.emit(PipelineEvent::StageStarted {
            node_id: node.id.clone(),
            stage_index: state.stage_index,
        });

        let outcome =
            execute_node(node, graph, &config, &run_dir, &context, state.stage_index).await?;
        record_and_checkpoint(node, &outcome, &run_dir, &context, &mut state, true)?;
        config.emitter.emit(PipelineEvent::CheckpointSaved {
            node_id: node.id.clone(),
        });

        if outcome.status == StageStatus::Fail {
            config.emitter.emit(PipelineEvent::StageFailed {
                node_id: node.id.clone(),
                stage_index: state.stage_index,
                reason: outcome.failure_reason.clone(),
            });
            if let Some(next) = route_failure(node, graph, &outcome, &context) {
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

        match advance(node, &outcome, &context, graph, &mut state) {
            AdvanceResult::Continue => state.stage_index += 1,
            AdvanceResult::LoopRestart(target) => {
                // §2.7/§3.2: create fresh run directory and context
                let (new_run_dir, new_context) = init_run(graph, &config)?;
                run_dir = new_run_dir;
                context = new_context;
                state.completed_nodes.clear();
                state.node_outcomes.clear();
                state.node_retries.clear();
                state.stage_index = 0;
                state.current_node_id = target;
            }
            AdvanceResult::End => {
                config.emitter.emit(PipelineEvent::PipelineCompleted {
                    pipeline_name: graph.name.clone(),
                    outcome: state.last_outcome.clone(),
                });
                return Ok(state.last_outcome);
            }
        }
    }
}

/// Initialize the run directory and context from graph attributes.
fn init_run(graph: &Graph, config: &EngineConfig) -> AttractorResult<(RunDirectory, Context)> {
    let run_id = chrono::Utc::now().format("%Y%m%dT%H%M%S%.6f").to_string();
    let run_dir = RunDirectory::create(config.logs_root.join(&run_id))?;

    let goal = graph
        .get_graph_attr("goal")
        .map(super::super::graph::AttrValue::to_string_value)
        .unwrap_or_default();

    run_dir.write_manifest(&Manifest {
        name: graph.name.clone(),
        goal: goal.clone(),
        start_time: chrono::Utc::now().to_rfc3339(),
    })?;

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

/// Record outcome, optionally apply context updates, sync retry counts,
/// and save a checkpoint.
///
/// When `apply_context_updates` is true, the outcome's context updates
/// are applied and `outcome`/`preferred_label` keys are set. This is
/// used for regular nodes. Exit nodes pass `false` to skip context
/// updates since the pipeline is about to finish.
fn record_and_checkpoint(
    node: &crate::graph::Node,
    outcome: &Outcome,
    run_dir: &RunDirectory,
    context: &Context,
    state: &mut LoopState,
    apply_context_updates: bool,
) -> AttractorResult<()> {
    run_dir.write_status(&node.id, outcome)?;
    state.completed_nodes.push(node.id.clone());
    state.node_outcomes.insert(node.id.clone(), outcome.clone());

    // Sync retry count from context into LoopState so checkpoints
    // contain accurate retry metadata (§5.3).
    if let Some(count) = context.get_i64(&format!("internal.retry_count.{}", node.id)) {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        state.node_retries.insert(node.id.clone(), count as u32);
    }

    if apply_context_updates {
        if !outcome.context_updates.is_empty() {
            context.apply_updates(&outcome.context_updates);
        }
        context.set(
            "outcome",
            Value::String(outcome.status.as_str().to_string()),
        );
        if !outcome.preferred_label.is_empty() {
            context.set(
                "preferred_label",
                Value::String(outcome.preferred_label.clone()),
            );
        }
    }

    let checkpoint = Checkpoint::from_context(
        context,
        &node.id,
        state.completed_nodes.clone(),
        state.node_retries.clone(),
    );
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
