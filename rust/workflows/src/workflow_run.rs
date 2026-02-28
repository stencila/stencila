//! Workflow execution integration.
//!
//! Bridges `Workflow` + `attractor::engine::run()` + agent resolution.
//! Converts workflow-level attributes (goal, modelStylesheet, etc.) into
//! graph attributes, resolves agent references, sets up a codergen backend
//! that delegates to Stencila's agent session system, and runs the pipeline.

use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use eyre::Result;
use stencila_agents::types::Turn;
use stencila_db::rusqlite::Connection;

use stencila_attractor::context::Context;
use stencila_attractor::definition_snapshot;
use stencila_attractor::engine::EngineConfig;
use stencila_attractor::events::{EventEmitter, ObserverEmitter, PipelineEvent};
use stencila_attractor::graph::{AttrValue, Graph};
use stencila_attractor::handler::HandlerRegistry;
use stencila_attractor::handlers::{
    CodergenBackend, CodergenHandler, CodergenOutput, ParallelHandler, WaitForHumanHandler,
};
use stencila_attractor::interviewer::{AnswerValue, Interviewer, Question};
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

#[derive(Clone, Default)]
struct AgentMetadata {
    model: Option<String>,
    provider: Option<String>,
}

#[derive(Default)]
struct RunMetrics {
    input_tokens: i64,
    output_tokens: i64,
}

struct DbRecordingInterviewer {
    inner: Arc<dyn Interviewer>,
    db_conn: Arc<Mutex<Connection>>,
    run_id: String,
}

/// Run a workflow pipeline to completion with the given [`RunOptions`].
///
/// 1. Parses `Workflow.pipeline` → `attractor::Graph`
/// 2. Merges workflow-level attributes (goal, modelStylesheet, etc.) into graph attrs
/// 3. Resolves `agent=` references against workspace/user agent definitions
/// 4. Opens (or creates) the workspace SQLite database for persistent state
/// 5. Sets up the `AgentCodergenBackend` that delegates to Stencila's agent sessions
/// 6. Registers `parallel` and optionally `wait.human` handlers
/// 7. Calls `attractor::engine::run()`
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

    // Open SQLite database for persistent state.
    //
    // The DB lives in the workspace `.stencila/` directory (not the per-run
    // logs directory) so that data accumulates across runs for analytics and
    // definition-snapshot deduplication.
    //
    // Error policy: DB open and run-record insertion are *fatal* (returned as
    // `Err`) because downstream context writes rely on FK constraints. All
    // other DB writes (node records, edges, responses, finalization) are
    // *non-fatal* and logged as warnings so a DB glitch doesn't kill a
    // running pipeline.
    let stencila_dir = stencila_dirs::closest_stencila_dir(workflow.path(), true).await?;
    let workspace_root = stencila_dirs::workspace_dir(&stencila_dir)?;
    let effective_db_path = stencila_dir.join(stencila_dirs::DB_SQLITE_FILE);

    let run_id = uuid::Uuid::now_v7().to_string();
    let stencila_artifacts_dir =
        stencila_dirs::stencila_artifacts_dir(&stencila_dir, false).await?;
    let artifacts_dir = stencila_artifacts_dir.join(format!("workflows/{run_id}"));

    let workspace_db = stencila_db::WorkspaceDb::open(&effective_db_path)
        .map_err(|e| eyre::eyre!("Failed to open workspace database: {e}"))?;
    let context = Context::with_sqlite(&workspace_db, &run_id)
        .map_err(|e| eyre::eyre!("Failed to initialize workflow context: {e}"))?;

    if let Some(backend) = context.sqlite_backend() {
        let goal = workflow.goal.as_deref().unwrap_or("");
        let workflow_name = &workflow.name;
        backend
            .insert_run(workflow_name, goal, env!("CARGO_PKG_VERSION"))
            .map_err(|e| eyre::eyre!("Failed to insert run record: {e}"))?;

        capture_definition_snapshots(workflow, &resolved, backend).await;
    }

    let agent_metadata = resolved
        .iter()
        .map(|(name, instance)| {
            (
                name.clone(),
                AgentMetadata {
                    model: instance.model.clone(),
                    provider: instance.provider.clone(),
                },
            )
        })
        .collect::<HashMap<_, _>>();

    let run_metrics = Arc::new(Mutex::new(RunMetrics::default()));

    // Clone DB handle before context is moved into the engine so we can
    // finalize the run record after execution completes.
    let db_conn = context.sqlite_connection().cloned();
    let interviewer = options.interviewer.map(|inner| {
        if let Some(conn) = &db_conn {
            Arc::new(DbRecordingInterviewer {
                inner,
                db_conn: conn.clone(),
                run_id: run_id.clone(),
            }) as Arc<dyn Interviewer>
        } else {
            inner
        }
    });
    let config = build_engine_config(
        logs_dir,
        options.emitter,
        interviewer,
        db_conn.clone(),
        Some(run_id.clone()),
        run_metrics.clone(),
        agent_metadata,
        Some(artifacts_dir),
        Some(workspace_root),
    );

    let result = stencila_attractor::engine::run_with_context(&graph, config, context)
        .await
        .map_err(|e| eyre::eyre!("Pipeline execution failed: {e}"));

    // Finalize the run record regardless of success or failure.
    if let Some(conn) = &db_conn {
        let backend = stencila_attractor::sqlite_backend::SqliteBackend::from_shared(
            conn.clone(),
            run_id.clone(),
        );
        let status = match &result {
            Ok(outcome) => outcome.status.as_str(),
            Err(_) => "failed",
        };
        let (input_tokens, output_tokens) = {
            let metrics = run_metrics
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            (metrics.input_tokens, metrics.output_tokens)
        };
        let total_tokens = input_tokens.saturating_add(output_tokens);
        let node_count = backend.node_count().unwrap_or(0);
        if let Err(e) = backend.complete_run(status, total_tokens, node_count) {
            tracing::warn!("Failed to finalize run record: {e}");
        }
    }

    result
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
struct AgentCodergenBackend {
    /// SQLite connection shared with tool executors (None if no DB).
    db_conn: Option<Arc<Mutex<Connection>>>,
    /// Run ID for the current pipeline execution.
    run_id: Option<String>,
    /// Shared run metrics accumulated across all node executions.
    run_metrics: Arc<Mutex<RunMetrics>>,
    /// Resolved model/provider metadata by agent name.
    agent_metadata: HashMap<String, AgentMetadata>,
    /// Run-specific artifact directory.
    artifacts_dir: Option<std::path::PathBuf>,
    /// Workspace root for storing portable artifact paths.
    workspace_root: Option<std::path::PathBuf>,
}

#[async_trait]
impl CodergenBackend for AgentCodergenBackend {
    async fn run(
        &self,
        node: &stencila_attractor::graph::Node,
        prompt: &str,
        context: &Context,
        emitter: Arc<dyn EventEmitter>,
        stage_index: usize,
    ) -> stencila_attractor::AttractorResult<CodergenOutput> {
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

        // Register workflow-context tools if we have a DB connection.
        if let (Some(conn), Some(run_id), Some(artifacts_dir), Some(workspace_root)) = (
            &self.db_conn,
            &self.run_id,
            &self.artifacts_dir,
            &self.workspace_root,
        ) {
            let context_writable = node.get_str_attr("context_writable") == Some("true");
            if let Err(e) = crate::tools::register_workflow_tools(
                &mut session,
                conn.clone(),
                run_id.clone(),
                context_writable,
                artifacts_dir.clone(),
                workspace_root.clone(),
            ) {
                tracing::warn!("Failed to register workflow tools: {e}");
            }
        }

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
        drop(submit_fut);

        let (node_input_tokens, node_output_tokens) = aggregate_usage(&session);
        {
            let mut metrics = self
                .run_metrics
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            metrics.input_tokens = metrics.input_tokens.saturating_add(node_input_tokens);
            metrics.output_tokens = metrics.output_tokens.saturating_add(node_output_tokens);
        }
        context.set(
            format!("internal.input_tokens.{node_id}"),
            serde_json::json!(node_input_tokens),
        );
        context.set(
            format!("internal.output_tokens.{node_id}"),
            serde_json::json!(node_output_tokens),
        );

        // Persist the LLM response and node metrics to SQLite so workflow tools can retrieve them.
        if let (Some(conn), Some(run_id)) = (&self.db_conn, &self.run_id) {
            let backend = stencila_attractor::sqlite_backend::SqliteBackend::from_shared(
                conn.clone(),
                run_id.clone(),
            );
            let compressed =
                zstd::encode_all(Cursor::new(collected_text.as_bytes()), 3).map_err(|e| {
                    stencila_attractor::AttractorError::HandlerFailed {
                        node_id: node_id.clone(),
                        reason: format!("Failed to compress node output: {e}"),
                    }
                })?;
            if let Err(e) = backend.save_node_output(&node_id, &compressed) {
                tracing::warn!("SQLite save_node_output({node_id}) failed: {e}");
            }

            let metadata = self.agent_metadata.get(agent_name);
            if let Some(meta) = metadata {
                if let Some(model) = &meta.model {
                    context.set(
                        format!("internal.model.{}", node.id),
                        serde_json::json!(model),
                    );
                }
                if let Some(provider) = &meta.provider {
                    context.set(
                        format!("internal.provider.{}", node.id),
                        serde_json::json!(provider),
                    );
                }
            }
            let record = stencila_attractor::sqlite_backend::NodeRecord {
                node_id: &node_id,
                status: "running",
                model: metadata.and_then(|meta| meta.model.as_deref()),
                provider: metadata.and_then(|meta| meta.provider.as_deref()),
                duration_ms: None,
                input_tokens: Some(node_input_tokens),
                output_tokens: Some(node_output_tokens),
                retry_count: None,
                failure_reason: None,
            };
            if let Err(e) = backend.upsert_node(&record) {
                tracing::warn!("SQLite upsert_node({node_id}) from codergen failed: {e}");
            }
        }

        Ok(CodergenOutput::Text(collected_text))
    }
}

/// Build an [`EngineConfig`] with all runtime handlers registered.
///
/// Creates two handler registries: an inner one (used by [`ParallelHandler`]
/// for branch execution) and an outer one (used by the main engine loop).
/// Both contain the default handlers plus `codergen` with the real agent
/// backend. The outer registry additionally has `parallel`. If an
/// `interviewer` is provided, both registries also get `wait.human`.
#[allow(clippy::too_many_arguments)]
fn build_engine_config(
    logs_dir: &Path,
    emitter: Arc<dyn EventEmitter>,
    interviewer: Option<Arc<dyn Interviewer>>,
    db_conn: Option<Arc<Mutex<Connection>>>,
    run_id: Option<String>,
    run_metrics: Arc<Mutex<RunMetrics>>,
    agent_metadata: HashMap<String, AgentMetadata>,
    artifacts_dir: Option<std::path::PathBuf>,
    workspace_root: Option<std::path::PathBuf>,
) -> EngineConfig {
    let mut config = EngineConfig::new(logs_dir);
    config.emitter = emitter.clone();

    // Inner registry: used by ParallelHandler for branch execution.
    // Does not need `parallel` itself (branches don't recurse into parallel).
    let mut inner_registry = HandlerRegistry::with_defaults();
    inner_registry.register(
        "codergen",
        CodergenHandler::with_backend_and_emitter(
            Arc::new(AgentCodergenBackend {
                db_conn: db_conn.clone(),
                run_id: run_id.clone(),
                run_metrics: run_metrics.clone(),
                agent_metadata: agent_metadata.clone(),
                artifacts_dir: artifacts_dir.clone(),
                workspace_root: workspace_root.clone(),
            }),
            emitter.clone(),
        ),
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
        CodergenHandler::with_backend_and_emitter(
            Arc::new(AgentCodergenBackend {
                db_conn,
                run_id,
                run_metrics,
                agent_metadata,
                artifacts_dir,
                workspace_root,
            }),
            emitter.clone(),
        ),
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

async fn capture_definition_snapshots(
    workflow: &WorkflowInstance,
    resolved_agents: &HashMap<String, stencila_agents::agent_def::AgentInstance>,
    backend: &stencila_attractor::sqlite_backend::SqliteBackend,
) {
    save_snapshot_for_dir(
        workflow.home(),
        "workflow",
        &workflow.name,
        "workflow",
        backend,
    );

    let cwd = std::env::current_dir().unwrap_or_default();
    let mut captured_skills = std::collections::HashSet::new();

    for (agent_name, agent) in resolved_agents {
        save_snapshot_for_dir(
            agent.home(),
            "agent",
            &agent.name,
            &format!("agent:{agent_name}"),
            backend,
        );

        if let Some(skill_names) = &agent.allowed_skills {
            for skill_name in skill_names {
                if !captured_skills.insert(skill_name.clone()) {
                    continue;
                }

                match stencila_skills::get_from(
                    &cwd,
                    skill_name,
                    &stencila_skills::SkillSource::all(),
                )
                .await
                {
                    Ok(skill) => {
                        save_snapshot_for_dir(
                            skill.home(),
                            "skill",
                            &skill.name,
                            &format!("skill:{skill_name}"),
                            backend,
                        );
                    }
                    Err(error) => {
                        tracing::warn!(
                            "Could not resolve skill `{skill_name}` for snapshot: {error}"
                        );
                    }
                }
            }
        }
    }
}

fn save_snapshot_for_dir(
    dir: &Path,
    kind: &str,
    name: &str,
    role: &str,
    backend: &stencila_attractor::sqlite_backend::SqliteBackend,
) {
    match definition_snapshot::snapshot_dir(dir) {
        Ok((hash, blob)) => {
            if let Err(error) = backend.save_definition_snapshot(&hash, &blob, kind, name) {
                tracing::warn!(
                    "Failed to save definition snapshot for `{}` ({kind}): {error}",
                    dir.display()
                );
                return;
            }
            if let Err(error) = backend.link_run_definition(&hash, role) {
                tracing::warn!(
                    "Failed to link definition snapshot for `{}` with role `{role}`: {error}",
                    dir.display()
                );
            }
        }
        Err(error) => {
            tracing::warn!(
                "Failed to snapshot definition directory `{}`: {error}",
                dir.display()
            );
        }
    }
}

fn aggregate_usage(session: &stencila_agents::agent_session::AgentSession) -> (i64, i64) {
    let mut input = 0_i64;
    let mut output = 0_i64;

    for turn in session.history() {
        if let Turn::Assistant { usage, .. } = turn {
            input = input.saturating_add(i64_from_u64(usage.input_tokens));
            output = output.saturating_add(i64_from_u64(usage.output_tokens));
        }
    }

    (input, output)
}

fn i64_from_u64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

#[async_trait]
impl Interviewer for DbRecordingInterviewer {
    async fn ask(&self, question: &Question) -> stencila_attractor::interviewer::Answer {
        let started = chrono::Utc::now();
        let answer = self.inner.ask(question).await;
        let answered = chrono::Utc::now();
        let duration_ms = (answered - started).num_milliseconds();

        let answer_text = match &answer.value {
            AnswerValue::Yes => Some("yes".to_string()),
            AnswerValue::No => Some("no".to_string()),
            AnswerValue::Skipped => Some("skipped".to_string()),
            AnswerValue::Timeout => Some("timeout".to_string()),
            AnswerValue::Selected(key) => Some(key.clone()),
            AnswerValue::Text(text) => Some(text.clone()),
        };

        let selected_option = answer.selected_option.as_ref().map(|opt| opt.key.clone());
        let options_json = if question.options.is_empty() {
            None
        } else {
            let options = question
                .options
                .iter()
                .map(|opt| serde_json::json!({"key": opt.key, "label": opt.label}))
                .collect::<Vec<_>>();
            Some(serde_json::Value::Array(options).to_string())
        };
        let interview_id = uuid::Uuid::now_v7().to_string();
        let backend = stencila_attractor::sqlite_backend::SqliteBackend::from_shared(
            self.db_conn.clone(),
            self.run_id.clone(),
        );
        if let Err(error) = backend.insert_interview(
            &interview_id,
            &question.stage,
            &question.text,
            Some(&question.question_type.to_string()),
            options_json.as_deref(),
            answer_text.as_deref(),
            selected_option.as_deref(),
            &started.to_rfc3339(),
            Some(&answered.to_rfc3339()),
            Some(duration_ms),
        ) {
            tracing::warn!("Failed to persist interview record: {error}");
        }

        answer
    }

    fn inform(&self, message: &str, stage: &str) {
        self.inner.inform(message, stage);
    }
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
            PipelineEvent::StageInput {
                node_id,
                stage_index,
                input,
                agent_name,
            } => {
                let preview: String = input.chars().take(100).collect();
                eprintln!(
                    "[stage {stage_index}] Prompt for {node_id} (agent={agent_name}): {preview}"
                );
            }
            PipelineEvent::StageSessionEvent { .. } => {
                // Suppress streaming session events in CLI stderr output
            }
            PipelineEvent::StageOutput {
                node_id,
                stage_index,
                output,
            } => {
                let preview: String = output.chars().take(100).collect();
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
