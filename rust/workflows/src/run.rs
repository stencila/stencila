//! Workflow execution integration.
//!
//! Bridges `Workflow` + `attractor::engine::run()` + agent resolution.
//! Converts workflow-level attributes (goal, overrides, etc.) into
//! graph attributes, resolves agent references, sets up a codergen backend
//! that delegates to Stencila's agent session system, and runs the pipeline.
//!
//! Also provides [`resume_workflow_with_options`] for resuming a previously
//! failed or interrupted run from its persisted SQLite state.

use std::collections::HashMap;
use std::io::Cursor;
use std::path::Path;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use eyre::Result;
use indexmap::IndexMap;
use stencila_agents::types::Turn;
use stencila_db::rusqlite::Connection;

use stencila_attractor::context::Context;
use stencila_attractor::definition_snapshot;
use stencila_attractor::engine::EngineConfig;
use stencila_attractor::events::{EventEmitter, ObserverEmitter, PipelineEvent};
use stencila_attractor::graph::{AttrValue, Graph};
use stencila_attractor::handler::HandlerRegistry;
use stencila_attractor::handlers::{
    CodergenBackend, CodergenHandler, CodergenOutput, ParallelHandler, ShellHandler,
    WaitForHumanHandler,
};
use stencila_attractor::interviewer::Interviewer;
use stencila_attractor::types::Outcome;
use stencila_interviews::PersistentInterviewer;

use crate::WorkflowInstance;
use crate::handler::WorkflowHandler;

/// Run a workflow pipeline to completion using a stderr event emitter.
///
/// Convenience wrapper around [`run_workflow_with_options`] for CLI usage.
pub async fn run_workflow(workflow: &WorkflowInstance) -> Result<Outcome> {
    let options = RunOptions {
        emitter: stderr_event_emitter(),
        interviewer: None,
    };
    run_workflow_with_options(workflow, options).await
}

/// Options for running a workflow from external callers (e.g. TUI).
pub struct RunOptions {
    pub emitter: Arc<dyn EventEmitter>,
    pub interviewer: Option<Arc<dyn Interviewer>>,
}

#[derive(Clone, Default)]
pub(crate) struct ParentRun {
    pub run_id: String,
    pub node_id: String,
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

/// Run a workflow pipeline to completion with the given [`RunOptions`].
///
/// 1. Parses `Workflow.pipeline` → `attractor::Graph`
/// 2. Merges workflow-level attributes (goal, overrides, etc.) into graph attrs
/// 3. Resolves `agent=` references against workspace/user agent definitions
/// 4. Opens (or creates) the workspace SQLite database for persistent state
/// 5. Sets up the `AgentCodergenBackend` that delegates to Stencila's agent sessions
/// 6. Registers `parallel` and optionally `wait.human` handlers
/// 7. Calls `attractor::engine::run()`
///
/// Returns the final `Outcome` from the pipeline engine.
pub async fn run_workflow_with_options(
    workflow: &WorkflowInstance,
    options: RunOptions,
) -> Result<Outcome> {
    run_workflow_with_options_and_parent(workflow, options, None, None).await
}

pub(crate) async fn run_workflow_with_options_and_parent(
    workflow: &WorkflowInstance,
    options: RunOptions,
    parent_run: Option<ParentRun>,
    initial_context: Option<IndexMap<String, serde_json::Value>>,
) -> Result<Outcome> {
    let mut graph = workflow.graph()?;

    merge_workflow_attrs(workflow, &mut graph);
    insert_execution_context(&mut graph, workflow, parent_run.clone(), initial_context)?;

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
    let db_conn_for_parent = workspace_db.connection().clone();
    if let Some(parent) = &parent_run {
        let conn = db_conn_for_parent
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM workflow_runs WHERE run_id = ?1",
                (&parent.run_id,),
                |row| row.get(0),
            )
            .unwrap_or(0);
        if exists == 0 {
            drop(conn);
            stencila_attractor::sqlite_backend::SqliteBackend::from_shared(
                db_conn_for_parent.clone(),
                parent.run_id.clone(),
            )
            .insert_run("parent-workflow", "", env!("CARGO_PKG_VERSION"))
            .map_err(|e| eyre::eyre!("Failed to insert parent run placeholder: {e}"))?;
        }
    }

    let context = Context::with_sqlite(&workspace_db, &run_id)
        .map_err(|e| eyre::eyre!("Failed to initialize workflow context: {e}"))?;

    if let Some(backend) = context.sqlite_backend() {
        let goal = workflow.goal.as_deref().unwrap_or("");
        let workflow_name = &workflow.name;
        backend
            .insert_run_with_parent(
                workflow_name,
                goal,
                env!("CARGO_PKG_VERSION"),
                parent_run.as_ref().map(|parent| parent.run_id.as_str()),
                parent_run.as_ref().map(|parent| parent.node_id.as_str()),
            )
            .map_err(|e| eyre::eyre!("Failed to insert run record: {e}"))?;

        capture_definition_snapshots(workflow, &resolved, backend).await;
    }

    // Set the run_id in context *after* the workflow_runs row exists so
    // the INSERT into workflow_context doesn't violate the FK constraint.
    context.set("internal.run_id", serde_json::json!(run_id.clone()));

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
            Arc::new(PersistentInterviewer::new(
                inner,
                conn.clone(),
                "workflow",
                run_id.clone(),
            )) as Arc<dyn Interviewer>
        } else {
            inner
        }
    });
    // The same interviewer serves both pipeline gates (WaitForHumanHandler)
    // and agent-level questions (ask_user tool) within the same run.
    let agent_interviewer = interviewer.clone();
    let config = build_engine_config(
        workflow.home().to_path_buf(),
        options.emitter,
        interviewer,
        db_conn.clone(),
        Some(run_id.clone()),
        run_metrics.clone(),
        agent_metadata,
        Some(artifacts_dir),
        Some(workspace_root),
        agent_interviewer,
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

/// Summary of a past workflow run, returned by [`list_runs`] and [`get_run`].
#[derive(Debug, Clone)]
pub struct RunInfo {
    pub run_id: String,
    pub workflow_name: String,
    pub goal: String,
    pub status: String,
    pub started_at: String,
    pub completed_at: Option<String>,
    pub node_count: i64,
    pub total_tokens: i64,
    pub is_child: bool,
}

/// List recent workflow runs from the workspace database.
///
/// Returns up to `limit` runs ordered by most recent first. Only
/// top-level runs are included (those without a parent).
///
/// # Errors
///
/// Returns an error if the workspace database cannot be opened.
pub async fn list_runs(workspace_path: &Path, limit: u32) -> Result<Vec<RunInfo>> {
    let stencila_dir = stencila_dirs::closest_stencila_dir(workspace_path, false).await?;
    let db_path = stencila_dir.join(stencila_dirs::DB_SQLITE_FILE);

    if !db_path.exists() {
        return Ok(Vec::new());
    }

    let workspace_db = stencila_db::WorkspaceDb::open(&db_path)
        .map_err(|e| eyre::eyre!("Failed to open workspace database: {e}"))?;

    // Ensure migrations are applied so tables exist.
    workspace_db
        .migrate_all(&[
            (
                "workflows",
                stencila_attractor::sqlite_backend::WORKFLOW_MIGRATIONS,
            ),
            ("interviews", stencila_interviews::INTERVIEW_MIGRATIONS),
        ])
        .map_err(|e| eyre::eyre!("Failed to apply workflow migrations: {e}"))?;

    let conn = workspace_db.connection();
    let conn = conn
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    let mut stmt = conn
        .prepare(
            "SELECT run_id, workflow_name, goal, status, started_at,
                    completed_at, node_count, total_tokens
             FROM workflow_runs
             WHERE parent_run_id IS NULL
             ORDER BY started_at DESC
             LIMIT ?1",
        )
        .map_err(|e| eyre::eyre!("Failed to prepare runs query: {e}"))?;

    let rows = stmt
        .query_map((limit,), |row| {
            Ok(RunInfo {
                run_id: row.get(0)?,
                workflow_name: row.get(1)?,
                goal: row.get(2)?,
                status: row.get(3)?,
                started_at: row.get(4)?,
                completed_at: row.get(5)?,
                node_count: row.get::<_, Option<i64>>(6)?.unwrap_or(0),
                total_tokens: row.get::<_, Option<i64>>(7)?.unwrap_or(0),
                is_child: false,
            })
        })
        .map_err(|e| eyre::eyre!("Failed to query runs: {e}"))?;

    let mut runs = Vec::new();
    for row in rows {
        runs.push(row.map_err(|e| eyre::eyre!("Failed to read run row: {e}"))?);
    }

    Ok(runs)
}

/// Find the most recent resumable run (failed or still marked running).
///
/// # Errors
///
/// Returns an error if the workspace database cannot be opened.
pub async fn last_resumable_run(workspace_path: &Path) -> Result<Option<RunInfo>> {
    let runs = list_runs(workspace_path, 50).await?;
    Ok(runs
        .into_iter()
        .find(|r| r.status == "failed" || r.status == "fail" || r.status == "running"))
}

/// Look up a single run by its full ID.
///
/// Returns the run regardless of whether it is a top-level or child run.
///
/// # Errors
///
/// Returns an error if the database cannot be opened or the run is not found.
pub async fn get_run(workspace_path: &Path, run_id: &str) -> Result<RunInfo> {
    let stencila_dir = stencila_dirs::closest_stencila_dir(workspace_path, false).await?;
    let db_path = stencila_dir.join(stencila_dirs::DB_SQLITE_FILE);

    let workspace_db = stencila_db::WorkspaceDb::open(&db_path)
        .map_err(|e| eyre::eyre!("Failed to open workspace database: {e}"))?;
    workspace_db
        .migrate_all(&[
            (
                "workflows",
                stencila_attractor::sqlite_backend::WORKFLOW_MIGRATIONS,
            ),
            ("interviews", stencila_interviews::INTERVIEW_MIGRATIONS),
        ])
        .map_err(|e| eyre::eyre!("Failed to apply workflow migrations: {e}"))?;

    let conn = workspace_db.connection();
    let conn = conn
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    conn.query_row(
        "SELECT run_id, workflow_name, goal, status, started_at,
                completed_at, node_count, total_tokens, parent_run_id
         FROM workflow_runs
         WHERE run_id = ?1",
        (run_id,),
        |row| {
            let parent: Option<String> = row.get(8)?;
            Ok(RunInfo {
                run_id: row.get(0)?,
                workflow_name: row.get(1)?,
                goal: row.get(2)?,
                status: row.get(3)?,
                started_at: row.get(4)?,
                completed_at: row.get(5)?,
                node_count: row.get::<_, Option<i64>>(6)?.unwrap_or(0),
                total_tokens: row.get::<_, Option<i64>>(7)?.unwrap_or(0),
                is_child: parent.is_some(),
            })
        },
    )
    .map_err(|e| eyre::eyre!("Run `{run_id}` not found: {e}"))
}

/// Resolve a run ID or prefix against the database.
///
/// Searches all runs (including child runs) by exact match first, then
/// by prefix. Returns an error if no match is found or the prefix is
/// ambiguous.
///
/// # Errors
///
/// Returns an error if the database cannot be opened, no match is found,
/// or the prefix matches multiple runs.
pub async fn resolve_run_id_from_db(workspace_path: &Path, id: &str) -> Result<String> {
    let stencila_dir = stencila_dirs::closest_stencila_dir(workspace_path, false).await?;
    let db_path = stencila_dir.join(stencila_dirs::DB_SQLITE_FILE);

    if !db_path.exists() {
        eyre::bail!("No workflow run found matching `{id}`");
    }

    let workspace_db = stencila_db::WorkspaceDb::open(&db_path)
        .map_err(|e| eyre::eyre!("Failed to open workspace database: {e}"))?;
    workspace_db
        .migrate_all(&[
            (
                "workflows",
                stencila_attractor::sqlite_backend::WORKFLOW_MIGRATIONS,
            ),
            ("interviews", stencila_interviews::INTERVIEW_MIGRATIONS),
        ])
        .map_err(|e| eyre::eyre!("Failed to apply workflow migrations: {e}"))?;

    let conn = workspace_db.connection();
    let conn = conn
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    // Exact match.
    let exact: Result<String, _> = conn.query_row(
        "SELECT run_id FROM workflow_runs WHERE run_id = ?1",
        (id,),
        |row| row.get(0),
    );
    if let Ok(run_id) = exact {
        return Ok(run_id);
    }

    // Prefix match using LIKE (the id is user-supplied but only used in
    // a parameterised query so there is no injection risk; we escape any
    // embedded `%` / `_` characters for correctness).
    let escaped = id.replace('%', r"\%").replace('_', r"\_");
    let pattern = format!("{escaped}%");
    let mut stmt = conn
        .prepare("SELECT run_id FROM workflow_runs WHERE run_id LIKE ?1 ESCAPE '\\'")
        .map_err(|e| eyre::eyre!("Failed to prepare prefix query: {e}"))?;
    let matches: Vec<String> = stmt
        .query_map((&pattern,), |row| row.get(0))
        .map_err(|e| eyre::eyre!("Failed to query runs: {e}"))?
        .filter_map(Result::ok)
        .collect();

    match matches.len() {
        0 => eyre::bail!("No workflow run found matching `{id}`"),
        1 => Ok(matches.into_iter().next().expect("length checked")),
        n => {
            eyre::bail!("Ambiguous run ID prefix `{id}` matches {n} runs; provide more characters")
        }
    }
}

/// Resume a previously failed or interrupted workflow run.
///
/// Looks up the original run's workflow name from the database, loads the
/// current workflow definition, rebuilds the pipeline graph and engine
/// configuration, then calls [`attractor::engine::resume_with_sqlite`] to
/// continue execution from where the run left off.
///
/// The resumed execution reuses the original `run_id` so that node records,
/// edges, and context accumulate in the same run scope.
///
/// Set `force` to `true` to allow resuming a run that is still marked as
/// `"running"` (which may indicate a stale process or a concurrent run).
///
/// # Errors
///
/// Returns an error if:
/// - The run ID is not found in the database
/// - The run is a nested/child run (resume the parent instead)
/// - The run has already completed successfully
/// - The run is still marked as running and `force` is `false`
/// - The workflow definition cannot be loaded
/// - The pipeline engine fails during resumed execution
pub async fn resume_workflow_with_options(
    run_id: &str,
    workspace_path: &Path,
    options: RunOptions,
    force: bool,
) -> Result<Outcome> {
    let stencila_dir = stencila_dirs::closest_stencila_dir(workspace_path, false).await?;
    let workspace_root = stencila_dirs::workspace_dir(&stencila_dir)?;
    let db_path = stencila_dir.join(stencila_dirs::DB_SQLITE_FILE);

    let workspace_db = stencila_db::WorkspaceDb::open(&db_path)
        .map_err(|e| eyre::eyre!("Failed to open workspace database: {e}"))?;

    workspace_db
        .migrate_all(&[
            (
                "workflows",
                stencila_attractor::sqlite_backend::WORKFLOW_MIGRATIONS,
            ),
            ("interviews", stencila_interviews::INTERVIEW_MIGRATIONS),
        ])
        .map_err(|e| eyre::eyre!("Failed to apply workflow migrations: {e}"))?;

    // Look up the original run to find the workflow name, goal, and status.
    let (workflow_name, goal, status, is_child) = {
        let conn = workspace_db.connection();
        let conn = conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        conn.query_row(
            "SELECT workflow_name, goal, status, parent_run_id FROM workflow_runs WHERE run_id = ?1",
            (run_id,),
            |row| {
                let parent: Option<String> = row.get(3)?;
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    parent.is_some(),
                ))
            },
        )
        .map_err(|e| eyre::eyre!("Run `{run_id}` not found in workspace database: {e}"))?
    };

    // Validate that the run is resumable.
    if is_child {
        eyre::bail!("Cannot resume nested workflow run `{run_id}`; resume the parent run instead");
    }
    if status == "success" {
        eyre::bail!(
            "Run `{run_id}` already completed successfully; use `stencila workflows run` to start a new run"
        );
    }
    if status == "running" && !force {
        eyre::bail!(
            "Run `{run_id}` is still marked as running (possibly stale or active in another process); \
             use --force to resume it anyway"
        );
    }

    // Load the current workflow definition.
    let cwd = std::env::current_dir()?;
    let mut wf = crate::definition::get_by_name(&cwd, &workflow_name)
        .await
        .map_err(|e| {
            eyre::eyre!("Cannot resume run `{run_id}`: workflow `{workflow_name}` not found: {e}")
        })?;

    // Restore the goal from the original run.
    if !goal.is_empty() {
        wf.inner.goal = Some(goal);
    }

    let mut graph = wf.graph()?;
    merge_workflow_attrs(&wf, &mut graph);
    insert_execution_context(&mut graph, &wf, None, None)?;

    let resolved = resolve_agent_references(&wf).await;

    let db_conn = workspace_db.connection().clone();

    // Reset status to 'running' for the resumed execution.
    {
        let conn = db_conn
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let _ = conn.execute(
            "UPDATE workflow_runs SET status = 'running', completed_at = NULL WHERE run_id = ?1",
            (run_id,),
        );
    }

    let stencila_artifacts_dir =
        stencila_dirs::stencila_artifacts_dir(&stencila_dir, false).await?;
    let artifacts_dir = stencila_artifacts_dir.join(format!("workflows/{run_id}"));

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

    let interviewer = options.interviewer.map(|inner| {
        Arc::new(PersistentInterviewer::new(
            inner,
            db_conn.clone(),
            "workflow",
            run_id.to_string(),
        )) as Arc<dyn Interviewer>
    });
    let agent_interviewer = interviewer.clone();

    let config = build_engine_config(
        wf.home().to_path_buf(),
        options.emitter,
        interviewer,
        Some(db_conn.clone()),
        Some(run_id.to_string()),
        run_metrics.clone(),
        agent_metadata,
        Some(artifacts_dir),
        Some(workspace_root),
        agent_interviewer,
    );

    let result =
        stencila_attractor::engine::resume_with_sqlite(&graph, config, db_conn.clone(), run_id)
            .await
            .map_err(|e| eyre::eyre!("Pipeline resume failed: {e}"));

    // Finalize the run record.
    let backend = stencila_attractor::sqlite_backend::SqliteBackend::from_shared(
        db_conn.clone(),
        run_id.to_string(),
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
        tracing::warn!("Failed to finalize resumed run record: {e}");
    }

    result
}

/// Merge workflow-level metadata into attractor graph attributes.
///
/// Copies `goal`, `overrides`, `default_max_retry`, `retry_target`,
/// `fallback_retry_target`, and `default_fidelity` from the `Workflow` schema
/// type into the parsed `attractor::Graph`'s `graph_attrs` so that the
/// attractor engine transforms and validators can see them.
fn merge_workflow_attrs(workflow: &WorkflowInstance, graph: &mut Graph) {
    // Override the DOT digraph name (typically "Workflow") with the actual
    // workflow name so that pipeline events report a meaningful identifier.
    graph.name.clone_from(&workflow.name);

    graph
        .graph_attrs
        .entry("label".to_string())
        .or_insert_with(|| AttrValue::String(workflow.name.clone()));

    if let Some(ref goal) = workflow.goal {
        graph
            .graph_attrs
            .entry("goal".to_string())
            .or_insert_with(|| AttrValue::String(goal.clone()));
    }

    // Inserted as "model_stylesheet" to match the Attractor spec's graph attribute name
    if let Some(ref stylesheet) = workflow.options.overrides {
        graph
            .graph_attrs
            .entry("model_stylesheet".to_string())
            .or_insert_with(|| AttrValue::String(stylesheet.clone()));
    }

    // Apply workflow-level default_max_retry, falling back to 3 when
    // neither the workflow frontmatter nor the DOT graph specifies one.
    // This ensures transient errors (network, rate-limit, server) are
    // retried automatically without requiring every workflow to opt in.
    let max_retry = workflow.options.default_max_retry.unwrap_or(3);
    graph
        .graph_attrs
        .entry("default_max_retry".to_string())
        .or_insert_with(|| AttrValue::Integer(max_retry));

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

fn insert_execution_context(
    graph: &mut Graph,
    workflow: &WorkflowInstance,
    parent_run: Option<ParentRun>,
    initial_context: Option<IndexMap<String, serde_json::Value>>,
) -> Result<()> {
    if let Some(existing_stack) = graph
        .graph_attrs
        .get("internal.workflow_stack")
        .and_then(AttrValue::as_str)
        && existing_stack.split('/').any(|name| name == workflow.name)
    {
        eyre::bail!(
            "Workflow composition cycle detected for `{}`",
            workflow.name
        );
    }

    graph.graph_attrs.insert(
        "internal.workflow_name".to_string(),
        AttrValue::String(workflow.name.clone()),
    );

    if let Some(parent) = parent_run {
        let mut stack = graph
            .graph_attrs
            .get("internal.workflow_stack")
            .and_then(AttrValue::as_str)
            .map(str::to_string)
            .unwrap_or_default();

        if stack.split('/').any(|name| name == workflow.name) {
            eyre::bail!(
                "Workflow composition cycle detected for `{}`",
                workflow.name
            );
        }

        if !stack.is_empty() {
            stack.push('/');
        }
        stack.push_str(&workflow.name);

        graph.graph_attrs.insert(
            "internal.workflow_stack".to_string(),
            AttrValue::String(stack),
        );
        graph.graph_attrs.insert(
            "internal.parent_node_id".to_string(),
            AttrValue::String(parent.node_id),
        );
        graph.graph_attrs.insert(
            "internal.parent_run_id".to_string(),
            AttrValue::String(parent.run_id),
        );
    } else {
        graph.graph_attrs.insert(
            "internal.workflow_stack".to_string(),
            AttrValue::String(workflow.name.clone()),
        );
    }

    if let Some(initial_context) = initial_context {
        for (key, value) in initial_context {
            graph
                .graph_attrs
                .insert(key, AttrValue::String(value.to_string()));
        }
    }

    Ok(())
}

/// Resolve `agent=` references in the graph against discovered agents.
///
/// Looks up each referenced agent name via workspace/user agent discovery.
/// Returns a map of agent name → `AgentInstance` for agents that were found.
/// Logs warnings for agents that could not be resolved.
async fn resolve_agent_references(
    workflow: &WorkflowInstance,
) -> HashMap<String, stencila_agents::definition::AgentInstance> {
    let agent_names = workflow.agent_references();
    let mut resolved = HashMap::new();

    if agent_names.is_empty() {
        return resolved;
    }

    let cwd = std::env::current_dir().unwrap_or_default();

    for name in &agent_names {
        match stencila_agents::definition::get_by_name(&cwd, name).await {
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
/// Nodes without an `agent` attribute fall back to
/// [`DEFAULT_WORKFLOW_AGENT_NAME`](stencila_agents::DEFAULT_WORKFLOW_AGENT_NAME)
/// (the general-purpose agent that performs work directly, rather than the
/// manager agent which always delegates).
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
    /// Interviewer for the `ask_user` tool in agent sessions.
    interviewer: Option<Arc<dyn Interviewer>>,
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
        let agent_name = node
            .get_str_attr("agent")
            .unwrap_or(stencila_agents::DEFAULT_WORKFLOW_AGENT_NAME);

        tracing::debug!(
            "Running agent `{agent_name}` for pipeline node `{}`",
            node.id
        );

        // Build overrides from node attributes. Two sources are merged:
        //
        //   1. `agent.*` dotted-key attributes set directly on the node
        //      (e.g. `agent.model="gpt-4o"`, `agent.provider="openai"`).
        //      These are the preferred user-facing syntax.
        //
        //   2. Overrides-derived attributes (`llm_model`, `llm_provider`,
        //      `reasoning_effort`, `trust_level`, `max_turns`) set by the
        //      attractor overrides transform.
        //
        // `agent.*` attributes take precedence over stylesheet attributes.
        //
        // All attribute keys are normalized to snake_case by the parser,
        // so only one lookup is needed per attribute regardless of the
        // casing the user wrote (kebab-case, snake_case, or camelCase).
        let overrides = stencila_agents::convenience::SessionOverrides {
            model: node
                .get_str_attr("agent.model")
                .or(node.get_str_attr("llm_model"))
                .map(String::from),
            provider: node
                .get_str_attr("agent.provider")
                .or(node.get_str_attr("llm_provider"))
                .map(String::from),
            reasoning_effort: node
                .get_str_attr("agent.reasoning_effort")
                .or(node.get_str_attr("reasoning_effort"))
                .map(String::from),
            trust_level: node
                .get_str_attr("agent.trust_level")
                .or(node.get_str_attr("trust_level"))
                .map(String::from),
            max_turns: node
                .get_str_attr("agent.max_turns")
                .or(node.get_str_attr("max_turns"))
                .and_then(|v| v.parse::<u32>().ok()),
        };
        let has_overrides = overrides.model.is_some()
            || overrides.provider.is_some()
            || overrides.reasoning_effort.is_some()
            || overrides.trust_level.is_some()
            || overrides.max_turns.is_some();

        let node_iv: Option<Arc<dyn Interviewer>> = self.interviewer.as_ref().map(|iv| {
            // Wrap with StageOverrideInterviewer so ask_user interviews
            // are attributed to this pipeline node, not generic "ask_user".
            Arc::new(
                stencila_interviews::interviewers::StageOverrideInterviewer::new(
                    iv.clone(),
                    &node.id,
                ),
            ) as Arc<dyn Interviewer>
        });

        let (_agent, mut session, mut event_rx) = if has_overrides {
            stencila_agents::convenience::create_session_with_overrides(
                agent_name, node_iv, &overrides,
            )
            .await
        } else if let Some(iv) = node_iv {
            stencila_agents::convenience::create_session_with_interviewer(agent_name, iv).await
        } else {
            stencila_agents::convenience::create_session(agent_name).await
        }
        .map_err(|e| stencila_attractor::AttractorError::AgentFailed {
            node_id: node.id.clone(),
            source: Box::new(e),
        })?;

        // Register workflow-context tools if we have a DB connection.
        let workflow_tools_available =
            if let (Some(conn), Some(run_id), Some(artifacts_dir), Some(workspace_root)) = (
                &self.db_conn,
                &self.run_id,
                &self.artifacts_dir,
                &self.workspace_root,
            ) {
                let context_writable = node.get_bool_attr("context_writable");
                match crate::tools::register_workflow_tools(
                    &mut session,
                    conn.clone(),
                    run_id.clone(),
                    context_writable,
                    artifacts_dir.clone(),
                    workspace_root.clone(),
                ) {
                    Ok(()) => true,
                    Err(e) => {
                        tracing::warn!("Failed to register workflow tools: {e}");
                        false
                    }
                }
            } else {
                false
            };

        // Collect outgoing edge labels from context (set fresh by
        // CodergenHandler before each backend call — intentionally transient
        // and node-local-in-time). When labels are present, register the
        // `workflow_set_route` tool so the agent can make a structured
        // routing decision. For sessions without tool support, fall back
        // to XML-block parsing from the response text.
        let outgoing_labels: Vec<String> = context
            .get("internal.outgoing_edge_labels")
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        let preferred_label: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

        let routing_tool_available = if !outgoing_labels.is_empty() {
            if let stencila_agents::session::AgentSession::Api(api_session) = &mut session {
                let tool = crate::tools::workflow_set_route::registered_tool(
                    outgoing_labels.clone(),
                    preferred_label.clone(),
                );
                match api_session.register_tool(tool) {
                    Ok(()) => true,
                    Err(e) => {
                        tracing::warn!("Failed to register workflow_set_route tool: {e}");
                        false
                    }
                }
            } else {
                false
            }
        } else {
            false
        };

        let mut effective_prompt = prompt.to_string();

        if workflow_tools_available {
            effective_prompt.push_str(
                "\n\n\
                 WORKFLOW CONTEXT TOOLS: You have access to workflow tools for retrieving \
                 prior state. Call `workflow_get_output` to fetch the full output from the \
                 previous pipeline node (e.g. reviewer feedback or a prior draft). Call \
                 `workflow_get_context` with a key (e.g. \"human.feedback\") to read stored \
                 values such as human revision notes. Use these tools to obtain context \
                 rather than assuming it is included in this prompt.",
            );
        }

        if !outgoing_labels.is_empty() {
            let labels_list = outgoing_labels.join(", ");
            if routing_tool_available {
                effective_prompt.push_str(&format!(
                    "\n\n\
                     WORKFLOW ROUTING: This node has multiple outgoing branches. \
                     After completing your main task, you MUST call the `workflow_set_route` tool \
                     with one of these labels to determine which branch the workflow takes next: \
                     {labels_list}"
                ));
            } else {
                effective_prompt.push_str(&format!(
                    "\n\n\
                     WORKFLOW ROUTING: This node has multiple outgoing branches. \
                     After completing your main task, you MUST end your response with an XML tag \
                     indicating which branch to take next. Use exactly one of these labels: \
                     {labels_list}\n\n\
                     Example: <preferred-label>Accept</preferred-label>"
                ));
            }
        }

        let node_id = node.id.clone();
        let mut submit_fut = Box::pin(session.submit(&effective_prompt));
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
            return Err(stencila_attractor::AttractorError::AgentFailed {
                node_id: node_id.clone(),
                source: Box::new(e),
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

            // Compute effective model/provider using the same precedence as
            // session creation: agent.* overrides > stylesheet > base agent metadata.
            let base_metadata = self.agent_metadata.get(agent_name);
            let effective_model = overrides
                .model
                .as_deref()
                .or(base_metadata.and_then(|m| m.model.as_deref()));
            let effective_provider = overrides
                .provider
                .as_deref()
                .or(base_metadata.and_then(|m| m.provider.as_deref()));

            if let Some(model) = effective_model {
                context.set(
                    format!("internal.model.{}", node.id),
                    serde_json::json!(model),
                );
            }
            if let Some(provider) = effective_provider {
                context.set(
                    format!("internal.provider.{}", node.id),
                    serde_json::json!(provider),
                );
            }
            let record = stencila_attractor::sqlite_backend::NodeRecord {
                node_id: &node_id,
                status: "running",
                model: effective_model,
                provider: effective_provider,
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

        // If the agent called workflow_set_route, propagate the choice
        // into the outcome so the edge selection algorithm (§3.3 Step 2)
        // can route on it. For sessions without tool support, fall back
        // to parsing a <preferred-label> XML block from the response text.
        let chosen_label = preferred_label
            .lock()
            // Poison recovery is safe: the guarded value is a simple Option<String>
            // with no invariants that could be violated by a panicking thread.
            .unwrap_or_else(|e| e.into_inner())
            .take()
            .or_else(|| {
                // XML-block fallback: parse <preferred-label>...</preferred-label>
                // from the agent's text response.
                parse_preferred_label_xml(&collected_text, &outgoing_labels)
            });

        if let Some(label) = chosen_label {
            let mut outcome = stencila_attractor::handlers::build_output_outcome(
                &node_id,
                &collected_text,
                context,
            );
            outcome.preferred_label = label;
            outcome.notes = format!("Codergen completed for node '{node_id}'");
            Ok(CodergenOutput::FullOutcome(outcome))
        } else {
            Ok(CodergenOutput::Text(collected_text))
        }
    }
}

/// Parse a `<preferred-label>` XML block from agent text output.
///
/// Returns the canonical (crate-defined) label if the extracted value
/// matches one of the `valid_labels` case-insensitively, or `None`
/// if no valid block is found.
fn parse_preferred_label_xml(text: &str, valid_labels: &[String]) -> Option<String> {
    // Look for <preferred-label>LABEL</preferred-label> (last occurrence wins)
    let open = text.rfind("<preferred-label>")?;
    let content_start = open + "<preferred-label>".len();
    let close = text[content_start..].find("</preferred-label>")?;
    let raw_label = text[content_start..content_start + close].trim();
    // Return the canonical label form (preserving the casing from edge definitions)
    valid_labels
        .iter()
        .find(|l| l.eq_ignore_ascii_case(raw_label))
        .cloned()
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
    workflow_home: std::path::PathBuf,
    emitter: Arc<dyn EventEmitter>,
    interviewer: Option<Arc<dyn Interviewer>>,
    db_conn: Option<Arc<Mutex<Connection>>>,
    run_id: Option<String>,
    run_metrics: Arc<Mutex<RunMetrics>>,
    agent_metadata: HashMap<String, AgentMetadata>,
    artifacts_dir: Option<std::path::PathBuf>,
    workspace_root: Option<std::path::PathBuf>,
    agent_interviewer: Option<Arc<dyn Interviewer>>,
) -> EngineConfig {
    let mut config = EngineConfig::new();
    config.emitter = emitter.clone();

    // Inner registry: used by ParallelHandler for branch execution.
    // Does not need `parallel` itself (branches don't recurse into parallel).
    let mut inner_registry = HandlerRegistry::with_defaults();
    inner_registry.register("shell", ShellHandler::with_emitter(emitter.clone()));
    inner_registry.register(
        "workflow",
        WorkflowHandler::new(workflow_home.clone(), emitter.clone(), interviewer.clone()),
    );
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
                interviewer: agent_interviewer.clone(),
            }),
            emitter.clone(),
        ),
    );
    if let Some(ref iv) = interviewer {
        let mut handler = WaitForHumanHandler::with_emitter(iv.clone(), emitter.clone());
        if let Some(ref conn) = db_conn
            && let Some(ref rid) = run_id
        {
            handler = handler.with_db(conn.clone(), "workflow", rid.clone());
        }
        inner_registry.register("wait.human", handler);
    }
    let inner_arc = Arc::new(inner_registry);

    // Outer registry: used by the main engine loop.
    config
        .registry
        .register("shell", ShellHandler::with_emitter(emitter.clone()));
    config.registry.register(
        "workflow",
        WorkflowHandler::new(workflow_home, emitter.clone(), interviewer.clone()),
    );
    config.registry.register(
        "codergen",
        CodergenHandler::with_backend_and_emitter(
            Arc::new(AgentCodergenBackend {
                db_conn: db_conn.clone(),
                run_id: run_id.clone(),
                run_metrics,
                agent_metadata,
                artifacts_dir,
                workspace_root,
                interviewer: agent_interviewer,
            }),
            emitter.clone(),
        ),
    );
    config
        .registry
        .register("parallel", ParallelHandler::new(inner_arc, emitter.clone()));
    if let Some(ref iv) = interviewer {
        let mut handler = WaitForHumanHandler::with_emitter(iv.clone(), emitter);
        if let Some(ref conn) = db_conn
            && let Some(ref rid) = run_id
        {
            handler = handler.with_db(conn.clone(), "workflow", rid.clone());
        }
        config.registry.register("wait.human", handler);
    }

    config
}

async fn capture_definition_snapshots(
    workflow: &WorkflowInstance,
    resolved_agents: &HashMap<String, stencila_agents::definition::AgentInstance>,
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

fn aggregate_usage(session: &stencila_agents::session::AgentSession) -> (i64, i64) {
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
                ..
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
            PipelineEvent::InterviewQuestionAsked {
                interview_id,
                node_id,
                ..
            } => {
                eprintln!(
                    "[interview {interview_id}] would present human question at node `{node_id}`"
                );
            }
            PipelineEvent::InterviewAnswerReceived {
                interview_id,
                node_id,
            } => {
                eprintln!("[interview {interview_id}] received answer at node `{node_id}`");
            }
            PipelineEvent::InterviewTimedOut {
                interview_id,
                node_id,
            } => {
                eprintln!("[interview {interview_id}] timed out at node `{node_id}`");
            }
            PipelineEvent::CheckpointSaved { node_id } => {
                eprintln!("[checkpoint] Saved at: {node_id}");
            }
            PipelineEvent::ParallelStarted { node_id, .. } => {
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

pub fn stderr_event_emitter_for_testing() -> Arc<dyn EventEmitter> {
    stderr_event_emitter()
}
