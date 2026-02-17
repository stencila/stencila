//! Workflow execution integration.
//!
//! Bridges `Workflow` + `attractor::engine::run()` + agent resolution.
//! Converts workflow-level attributes (goal, modelStylesheet, etc.) into
//! graph attributes, resolves agent references, sets up a codergen backend
//! that delegates to Stencila's agent session system, and runs the pipeline.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use eyre::Result;

use stencila_attractor::context::Context;
use stencila_attractor::engine::EngineConfig;
use stencila_attractor::events::{EventEmitter, ObserverEmitter, PipelineEvent};
use stencila_attractor::graph::{AttrValue, Graph};
use stencila_attractor::handler::HandlerRegistry;
use stencila_attractor::handlers::{
    CodergenBackend, CodergenHandler, CodergenResponse, ParallelHandler, WaitForHumanHandler,
};
use stencila_attractor::interviewer::Interviewer;
use stencila_attractor::types::Outcome;

use crate::WorkflowInstance;

/// Run a workflow pipeline to completion using a stderr event emitter.
///
/// Convenience wrapper around [`run_workflow_with_options`] for CLI usage.
pub async fn run_workflow(workflow: &WorkflowInstance, logs_dir: &Path) -> Result<Outcome> {
    let options = RunOptions {
        emitter: stderr_event_emitter(),
        interviewer: None,
    };
    run_workflow_with_options(workflow, logs_dir, options).await
}

/// Options for running a workflow from external callers (e.g. TUI).
pub struct RunOptions {
    pub emitter: Arc<dyn EventEmitter>,
    pub interviewer: Option<Arc<dyn Interviewer>>,
}

/// Run a workflow pipeline to completion with the given [`RunOptions`].
///
/// 1. Parses `Workflow.pipeline` → `attractor::Graph`
/// 2. Merges workflow-level attributes (goal, modelStylesheet, etc.) into graph attrs
/// 3. Resolves `agent=` references against workspace/user agent definitions
/// 4. Sets up the `AgentCodergenBackend` that delegates to Stencila's agent sessions
/// 5. Registers `parallel` and optionally `wait.human` handlers
/// 6. Calls `attractor::engine::run()`
///
/// Returns the final `Outcome` from the pipeline engine.
pub async fn run_workflow_with_options(
    workflow: &WorkflowInstance,
    logs_dir: &Path,
    options: RunOptions,
) -> Result<Outcome> {
    let mut graph = workflow.graph()?;

    merge_workflow_attrs(workflow, &mut graph);

    let resolved = resolve_agent_references(workflow).await;
    let total_refs = workflow.agent_references().len();
    if resolved.len() < total_refs {
        tracing::warn!(
            "Only {}/{} agent references resolved for workflow `{}`",
            resolved.len(),
            total_refs,
            workflow.name
        );
    }

    let config = build_engine_config(logs_dir, options.emitter, options.interviewer);

    let outcome = stencila_attractor::engine::run(&graph, config)
        .await
        .map_err(|e| eyre::eyre!("Pipeline execution failed: {e}"))?;

    Ok(outcome)
}

/// Merge workflow-level metadata into attractor graph attributes.
///
/// Copies `goal`, `model_stylesheet`, `default_max_retry`, `retry_target`,
/// `fallback_retry_target`, and `default_fidelity` from the `Workflow` schema
/// type into the parsed `attractor::Graph`'s `graph_attrs` so that the
/// attractor engine transforms and validators can see them.
fn merge_workflow_attrs(workflow: &WorkflowInstance, graph: &mut Graph) {
    if let Some(ref goal) = workflow.goal {
        graph
            .graph_attrs
            .entry("goal".to_string())
            .or_insert_with(|| AttrValue::String(goal.clone()));
    }

    if let Some(ref stylesheet) = workflow.options.model_stylesheet {
        graph
            .graph_attrs
            .entry("model_stylesheet".to_string())
            .or_insert_with(|| AttrValue::String(stylesheet.clone()));
    }

    if let Some(max_retry) = workflow.options.default_max_retry {
        graph
            .graph_attrs
            .entry("default_max_retry".to_string())
            .or_insert_with(|| AttrValue::Integer(max_retry));
    }

    if let Some(ref target) = workflow.options.retry_target {
        graph
            .graph_attrs
            .entry("retry_target".to_string())
            .or_insert_with(|| AttrValue::String(target.clone()));
    }

    if let Some(ref target) = workflow.options.fallback_retry_target {
        graph
            .graph_attrs
            .entry("fallback_retry_target".to_string())
            .or_insert_with(|| AttrValue::String(target.clone()));
    }

    if let Some(ref fidelity) = workflow.options.default_fidelity {
        graph
            .graph_attrs
            .entry("default_fidelity".to_string())
            .or_insert_with(|| AttrValue::String(fidelity.clone()));
    }
}

/// Resolve `agent=` references in the graph against discovered agents.
///
/// Looks up each referenced agent name via workspace/user agent discovery.
/// Returns a map of agent name → `AgentInstance` for agents that were found.
/// Logs warnings for agents that could not be resolved.
async fn resolve_agent_references(
    workflow: &WorkflowInstance,
) -> HashMap<String, stencila_agents::agent_def::AgentInstance> {
    let agent_names = workflow.agent_references();
    let mut resolved = HashMap::new();

    if agent_names.is_empty() {
        return resolved;
    }

    let cwd = std::env::current_dir().unwrap_or_default();

    for name in &agent_names {
        match stencila_agents::agent_def::get_by_name(&cwd, name).await {
            Ok(agent) => {
                tracing::debug!("Resolved agent `{name}` for workflow `{}`", workflow.name);
                resolved.insert(name.clone(), agent);
            }
            Err(e) => {
                tracing::warn!(
                    "Could not resolve agent `{name}` for workflow `{}`: {e}",
                    workflow.name
                );
            }
        }
    }

    resolved
}

/// A `CodergenBackend` that delegates LLM calls to Stencila's agent session
/// system.
///
/// For each node execution, looks up the `agent` attribute to determine which
/// agent to run. Creates an agent session via
/// [`stencila_agents::convenience::create_session`], submits the prompt, and
/// streams `StageSessionEvent` events back through the emitter while accumulating
/// the full response text.
///
/// Nodes without an `agent` attribute fall back to the `"default"` agent
/// (which resolves via `[agents].default` in `stencila.toml` or the agent
/// literally named `"default"`).
struct AgentCodergenBackend;

#[async_trait]
impl CodergenBackend for AgentCodergenBackend {
    async fn run(
        &self,
        node: &stencila_attractor::graph::Node,
        prompt: &str,
        _context: &Context,
        emitter: Arc<dyn EventEmitter>,
        stage_index: usize,
    ) -> stencila_attractor::AttractorResult<CodergenResponse> {
        let agent_name = node.get_str_attr("agent").unwrap_or("default");

        tracing::debug!(
            "Running agent `{agent_name}` for pipeline node `{}`",
            node.id
        );

        let (_agent, mut session, mut event_rx) =
            stencila_agents::convenience::create_session(agent_name)
                .await
                .map_err(|e| stencila_attractor::AttractorError::HandlerFailed {
                    node_id: node.id.clone(),
                    reason: format!("Agent `{agent_name}` session creation failed: {e}"),
                })?;

        let node_id = node.id.clone();
        let mut submit_fut = Box::pin(session.submit(prompt));
        let mut submit_done = false;
        let mut submit_result: Option<stencila_agents::error::AgentResult<()>> = None;
        let mut collected_text = String::new();

        loop {
            tokio::select! {
                biased;

                event = event_rx.recv() => {
                    let Some(event) = event else {
                        break;
                    };
                    if event.kind == stencila_agents::types::EventKind::AssistantTextDelta
                        && let Some(serde_json::Value::String(delta)) = event.data.get("delta") {
                            collected_text.push_str(delta);
                        }
                    emitter.emit(PipelineEvent::StageSessionEvent {
                        node_id: node_id.clone(),
                        stage_index,
                        event,
                    });
                }

                result = &mut submit_fut, if !submit_done => {
                    submit_done = true;
                    submit_result = Some(result);
                }
            }

            if submit_done {
                while let Ok(event) = event_rx.try_recv() {
                    if event.kind == stencila_agents::types::EventKind::AssistantTextDelta
                        && let Some(serde_json::Value::String(delta)) = event.data.get("delta")
                    {
                        collected_text.push_str(delta);
                    }
                    emitter.emit(PipelineEvent::StageSessionEvent {
                        node_id: node_id.clone(),
                        stage_index,
                        event,
                    });
                }
                break;
            }
        }

        if let Some(Err(e)) = submit_result {
            return Err(stencila_attractor::AttractorError::HandlerFailed {
                node_id: node_id.clone(),
                reason: format!("Agent `{agent_name}` failed: {e}"),
            });
        }

        Ok(CodergenResponse::Text(collected_text))
    }
}

/// Build an [`EngineConfig`] with all runtime handlers registered.
///
/// Creates two handler registries: an inner one (used by [`ParallelHandler`]
/// for branch execution) and an outer one (used by the main engine loop).
/// Both contain the default handlers plus `codergen` with the real agent
/// backend. The outer registry additionally has `parallel`. If an
/// `interviewer` is provided, both registries also get `wait.human`.
fn build_engine_config(
    logs_dir: &Path,
    emitter: Arc<dyn EventEmitter>,
    interviewer: Option<Arc<dyn Interviewer>>,
) -> EngineConfig {
    let mut config = EngineConfig::new(logs_dir);
    config.emitter = emitter.clone();

    // Inner registry: used by ParallelHandler for branch execution.
    // Does not need `parallel` itself (branches don't recurse into parallel).
    let mut inner_registry = HandlerRegistry::with_defaults();
    inner_registry.register(
        "codergen",
        CodergenHandler::with_backend_and_emitter(Arc::new(AgentCodergenBackend), emitter.clone()),
    );
    if let Some(ref iv) = interviewer {
        inner_registry.register(
            "wait.human",
            WaitForHumanHandler::with_emitter(iv.clone(), emitter.clone()),
        );
    }
    let inner_arc = Arc::new(inner_registry);

    // Outer registry: used by the main engine loop.
    config.registry.register(
        "codergen",
        CodergenHandler::with_backend_and_emitter(Arc::new(AgentCodergenBackend), emitter.clone()),
    );
    config
        .registry
        .register("parallel", ParallelHandler::new(inner_arc, emitter.clone()));
    if let Some(ref iv) = interviewer {
        config.registry.register(
            "wait.human",
            WaitForHumanHandler::with_emitter(iv.clone(), emitter),
        );
    }

    config
}

/// Create an `EventEmitter` that logs pipeline events to stderr.
#[allow(clippy::print_stderr)]
fn stderr_event_emitter() -> Arc<dyn EventEmitter> {
    Arc::new(ObserverEmitter::new(|event: &PipelineEvent| {
        match event {
            PipelineEvent::PipelineStarted { pipeline_name } => {
                eprintln!("[pipeline] Started: {pipeline_name}");
            }
            PipelineEvent::PipelineCompleted {
                pipeline_name,
                outcome,
            } => {
                eprintln!(
                    "[pipeline] Completed: {pipeline_name} (status={})",
                    outcome.status.as_str()
                );
            }
            PipelineEvent::PipelineFailed {
                pipeline_name,
                reason,
            } => {
                eprintln!("[pipeline] Failed: {pipeline_name}: {reason}");
            }
            PipelineEvent::StageStarted {
                node_id,
                stage_index,
            } => {
                eprintln!("[stage {stage_index}] Started: {node_id}");
            }
            PipelineEvent::StagePrompt {
                node_id,
                stage_index,
                prompt,
                agent_name,
            } => {
                let preview: String = prompt.chars().take(100).collect();
                eprintln!(
                    "[stage {stage_index}] Prompt for {node_id} (agent={agent_name}): {preview}"
                );
            }
            PipelineEvent::StageSessionEvent { .. } => {
                // Suppress streaming session events in CLI stderr output
            }
            PipelineEvent::StageResponse {
                node_id,
                stage_index,
                response,
            } => {
                let preview: String = response.chars().take(100).collect();
                eprintln!("[stage {stage_index}] Response from {node_id}: {preview}");
            }
            PipelineEvent::StageCompleted {
                node_id,
                stage_index,
                outcome,
            } => {
                eprintln!(
                    "[stage {stage_index}] Completed: {node_id} (status={})",
                    outcome.status.as_str()
                );
            }
            PipelineEvent::StageFailed {
                node_id,
                stage_index,
                reason,
            } => {
                eprintln!("[stage {stage_index}] Failed: {node_id}: {reason}");
            }
            PipelineEvent::StageRetrying {
                node_id,
                stage_index,
                attempt,
                max_attempts,
            } => {
                eprintln!(
                    "[stage {stage_index}] Retrying: {node_id} (attempt {attempt}/{max_attempts})"
                );
            }
            PipelineEvent::InterviewQuestionAsked { node_id } => {
                eprintln!("[interview] would present human question at node `{node_id}`");
            }
            PipelineEvent::InterviewAnswerReceived { node_id } => {
                eprintln!("[interview] received answer at node `{node_id}`");
            }
            PipelineEvent::InterviewTimedOut { node_id } => {
                eprintln!("[interview] timed out at node `{node_id}`");
            }
            PipelineEvent::CheckpointSaved { node_id } => {
                eprintln!("[checkpoint] Saved at: {node_id}");
            }
            PipelineEvent::ParallelStarted { node_id } => {
                eprintln!("[parallel] Started: {node_id}");
            }
            PipelineEvent::ParallelCompleted { node_id } => {
                eprintln!("[parallel] Completed: {node_id}");
            }
            PipelineEvent::ParallelBranchStarted {
                node_id,
                branch_index,
            } => {
                eprintln!("[parallel] Branch {branch_index} started: {node_id}");
            }
            PipelineEvent::ParallelBranchCompleted {
                node_id,
                branch_index,
            } => {
                eprintln!("[parallel] Branch {branch_index} completed: {node_id}");
            }
            PipelineEvent::ParallelBranchFailed {
                node_id,
                branch_index,
                reason,
            } => {
                eprintln!("[parallel] Branch {branch_index} failed: {node_id}: {reason}");
            }
        }
    }))
}
