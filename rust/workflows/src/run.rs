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
use chrono::{DateTime, TimeDelta, Utc};
use chrono_humanize::{Accuracy, HumanTime, Tense};
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
        run_id_out: None,
        gate_timeout: GateTimeoutConfig::default(),
    };
    run_workflow_with_options(workflow, options).await
}

/// Options for running a workflow from external callers (e.g. TUI).
pub struct RunOptions {
    pub emitter: Arc<dyn EventEmitter>,
    pub interviewer: Option<Arc<dyn Interviewer>>,
    /// When set, the run ID is published here as soon as it is known.
    /// Callers can use this to mark the run as cancelled if the task is
    /// aborted before the engine finishes.
    pub run_id_out: Option<Arc<Mutex<Option<String>>>>,
    /// Controls how human gates (hexagon nodes) behave during execution.
    pub gate_timeout: GateTimeoutConfig,
}

/// Controls how human gates behave during workflow execution.
#[derive(Clone, Debug, Default)]
pub enum GateTimeoutConfig {
    /// Wait indefinitely for human input (default).
    #[default]
    Interactive,
    /// Automatically approve all gates with zero timeout.
    AutoApprove,
    /// Auto-approve gates after the given number of seconds.
    Timed { seconds: f64 },
}

impl GateTimeoutConfig {
    /// Serialize this config to a JSON value suitable for writing into the
    /// `internal.gate_timeouts` context key.
    ///
    /// Returns `None` for `Interactive` (meaning: omit from context).
    pub fn to_context_json(&self) -> Option<serde_json::Value> {
        match self {
            Self::Interactive => None,
            Self::AutoApprove => Some(serde_json::json!({"*": 0})),
            Self::Timed { seconds } => {
                let value = if seconds.fract() == 0.0 {
                    serde_json::json!(*seconds as i64)
                } else {
                    serde_json::json!(*seconds)
                };
                Some(serde_json::json!({"*": value}))
            }
        }
    }
}

/// Write `internal.gate_timeouts` into the pipeline context when the
/// gate-timeout config is non-interactive.
fn propagate_gate_timeout(config: &GateTimeoutConfig, context: &Context) {
    if let Some(value) = config.to_context_json() {
        context.set("internal.gate_timeouts", value);
    }
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

    // Publish the run ID so callers (e.g. the TUI) can mark it as
    // cancelled if the task is aborted before the engine finishes.
    if let Some(out) = &options.run_id_out {
        let mut guard = out
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *guard = Some(run_id.clone());
    }

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

    propagate_gate_timeout(&options.gate_timeout, &context);

    let agent_metadata = collect_agent_metadata(&resolved);

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
    let session_pool = crate::session_pool::SessionPool::new();
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
        session_pool,
        options.gate_timeout.clone(),
    );

    let result = stencila_attractor::engine::run_with_context(&graph, config, context)
        .await
        .map_err(|e| eyre::eyre!("Pipeline execution failed: {e}"));

    // Finalize the run record regardless of success or failure.
    finalize_run_record(&db_conn, &run_id, &result, &run_metrics);

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

/// Filter applied when listing workflow runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunListFilter {
    /// Return all top-level workflow runs.
    All,
    /// Return only failed, cancelled, or still-running top-level runs that may be resumed.
    Resumable,
}

/// List recent workflow runs from the workspace database.
///
/// Returns up to `limit` runs ordered by most recent first. Only
/// top-level runs are included (those without a parent). When `filter`
/// is [`RunListFilter::Resumable`], only failed, cancelled, or still-running
/// runs are returned.
///
/// # Errors
///
/// Returns an error if the workspace database cannot be opened.
pub async fn list_runs(
    workspace_path: &Path,
    limit: u32,
    filter: RunListFilter,
) -> Result<Vec<RunInfo>> {
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

    let query = if matches!(filter, RunListFilter::Resumable) {
        "SELECT run_id, workflow_name, goal, status, started_at,
                completed_at, node_count, total_tokens
         FROM workflow_runs
         WHERE parent_run_id IS NULL
           AND status IN ('failed', 'fail', 'running', 'cancelled')
         ORDER BY started_at DESC
         LIMIT ?1"
    } else {
        "SELECT run_id, workflow_name, goal, status, started_at,
                completed_at, node_count, total_tokens
         FROM workflow_runs
         WHERE parent_run_id IS NULL
         ORDER BY started_at DESC
         LIMIT ?1"
    };

    let mut stmt = conn
        .prepare(query)
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

/// Humanize an RFC3339 timestamp for CLI and TUI display.
pub fn humanize_timestamp(iso_timestamp: &str) -> String {
    if let Ok(dt) = DateTime::parse_from_rfc3339(iso_timestamp) {
        let now = Utc::now();
        let duration = now.signed_duration_since(dt.with_timezone(&Utc));
        let delta = if duration < TimeDelta::zero() {
            -duration
        } else {
            duration
        };

        let mut text = HumanTime::from(delta).to_text_en(
            Accuracy::Rough,
            if duration < TimeDelta::zero() {
                Tense::Future
            } else {
                Tense::Present
            },
        );

        if duration < TimeDelta::zero() {
            return text;
        }

        if text == "now" {
            text.insert_str(0, "just ");
        } else {
            text.push_str(" ago");
        }
        return text;
    }

    if iso_timestamp.len() >= 16 {
        iso_timestamp[..16].to_string()
    } else {
        iso_timestamp.to_string()
    }
}

/// Find the most recent resumable run (failed, cancelled, or still marked running).
///
/// # Errors
///
/// Returns an error if the workspace database cannot be opened.
pub async fn last_resumable_run(workspace_path: &Path) -> Result<Option<RunInfo>> {
    let runs = list_runs(workspace_path, 1, RunListFilter::Resumable).await?;
    Ok(runs.into_iter().next())
}

/// Mark a workflow run as cancelled in the workspace database.
///
/// Updates the run's status to `"cancelled"` and sets the `completed_at`
/// timestamp. This should be called when a running workflow is aborted
/// (e.g. by the user pressing Ctrl-C in the TUI) so the run does not
/// remain stuck as `"running"` in the database.
///
/// No-op if the database does not exist or the run ID is not found.
///
/// # Errors
///
/// Returns an error if the workspace database cannot be opened or the
/// update fails.
pub async fn cancel_run(workspace_path: &Path, run_id: &str) -> Result<()> {
    let stencila_dir = match stencila_dirs::closest_stencila_dir(workspace_path, false).await {
        Ok(dir) => dir,
        Err(_) => return Ok(()),
    };
    let db_path = stencila_dir.join(stencila_dirs::DB_SQLITE_FILE);

    if !db_path.exists() {
        return Ok(());
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
    conn.execute(
        "UPDATE workflow_runs
         SET status = 'cancelled',
             completed_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now'),
             total_duration_ms = CAST(
                (julianday(strftime('%Y-%m-%dT%H:%M:%fZ', 'now')) - julianday(started_at))
                * 86400000.0 AS INTEGER
             )
         WHERE run_id = ?1 AND status = 'running'",
        (run_id,),
    )
    .map_err(|e| eyre::eyre!("Failed to cancel run `{run_id}`: {e}"))?;

    Ok(())
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
        1 => Ok(matches.into_iter().next().unwrap_or_default()),
        n => {
            eyre::bail!("Ambiguous run ID prefix `{id}` matches {n} runs; provide more characters")
        }
    }
}

/// Resume a previously failed, cancelled, or interrupted workflow run.
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

    let agent_metadata = collect_agent_metadata(&resolved);

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
    let session_pool = crate::session_pool::SessionPool::new();

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
        session_pool,
        options.gate_timeout.clone(),
    );

    let result =
        stencila_attractor::engine::resume_with_sqlite(&graph, config, db_conn.clone(), run_id)
            .await
            .map_err(|e| eyre::eyre!("Pipeline resume failed: {e}"));

    // Finalize the run record.
    finalize_run_record(&Some(db_conn), run_id, &result, &run_metrics);

    result
}

/// Collect agent model/provider metadata from resolved agent instances.
fn collect_agent_metadata(
    resolved: &HashMap<String, stencila_agents::definition::AgentInstance>,
) -> HashMap<String, AgentMetadata> {
    resolved
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
        .collect()
}

/// Finalize a workflow run record in the database.
///
/// Computes the final status, total tokens, and node count, then calls
/// `complete_run` on the SQLite backend. Logs a warning on failure rather
/// than propagating the error, since the pipeline result is more important.
fn finalize_run_record(
    db_conn: &Option<Arc<Mutex<Connection>>>,
    run_id: &str,
    result: &Result<Outcome>,
    run_metrics: &Arc<Mutex<RunMetrics>>,
) {
    let Some(conn) = db_conn else { return };
    let backend = stencila_attractor::sqlite_backend::SqliteBackend::from_shared(
        conn.clone(),
        run_id.to_string(),
    );
    let status = match result {
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
    /// Session pool for reusing agent sessions across loop iterations.
    session_pool: crate::session_pool::SessionPool,
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

        // Session reuse: when fidelity is "full" and a thread_id is present,
        // attempt to take a pooled session for multi-turn conversation reuse.
        let fidelity = context
            .get("internal.fidelity")
            .and_then(|v| v.as_str().map(String::from));
        let thread_id = context
            .get("internal.thread_id")
            .and_then(|v| v.as_str().map(String::from));
        let is_full_fidelity = fidelity.as_deref() == Some("full");

        let max_session_turns: Option<u64> = node
            .get_str_attr("max_session_turns")
            .and_then(|s| s.parse().ok());

        let (pooled, prev_turn_count) = if is_full_fidelity && let Some(ref tid) = thread_id {
            match (self.session_pool.take(tid), max_session_turns) {
                (Some(entry), Some(limit)) if entry.turn_count >= limit => {
                    tracing::debug!(
                        thread_id = %tid,
                        turn_count = entry.turn_count,
                        max_session_turns = limit,
                        "session turn limit reached; creating fresh session"
                    );
                    (None, 0u64)
                }
                (Some(entry), _) if entry.session.is_some() => {
                    let tc = entry.turn_count;
                    tracing::debug!(
                        thread_id = %tid,
                        turn_count = tc,
                        "reusing pooled session"
                    );
                    (Some(entry), tc)
                }
                _ => (None, 0),
            }
        } else {
            (None, 0)
        };

        let (mut session, mut event_rx, is_new_session) = if let Some(mut entry) = pooled {
            let session = entry.session.take().expect("checked above");
            let event_rx = entry.event_rx.take().expect("session implies event_rx");
            (session, event_rx, false)
        } else {
            let (_agent, session, event_rx) = if has_overrides {
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
            (session, event_rx, true)
        };

        let is_cli_session = matches!(session, stencila_agents::session::AgentSession::Cli(_));

        // Register workflow-context tools if we have a DB connection.
        //
        // On reused sessions these tools are already registered from the
        // previous turn, but we re-register unconditionally because:
        //   - `register_tool` replaces by name (IndexMap insert), so the
        //     tool count stays constant — no context bloat from tool defs.
        //   - `allowed_tools` deduplicates, so no growth there either.
        //   - The executor closures capture per-node state (e.g.
        //     `context_writable`), which may differ between nodes sharing
        //     a thread, so a fresh executor is always correct.
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
        //
        // On reused sessions the routing tool is re-registered with the
        // current node's labels and a fresh `preferred_label` Arc. If the
        // current node has no outgoing edges, the old tool from a previous
        // node stays in the registry but is harmless: its prompt-level
        // routing instructions are absent, so the model won't call it, and
        // even if it did the old `preferred_label` Arc is no longer read.
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

        // Build the effective prompt by appending workflow instructions.
        //
        // On reused sessions these instructions become part of the
        // conversation history and are replayed on every subsequent
        // request. This is a minor per-turn overhead (~400-500 bytes)
        // that accumulates across loop iterations.  The
        // `max_session_turns` node attribute exists to bound this
        // growth by forcing a fresh session after N submissions.
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

        if !is_new_session {
            drain_stale_events(&mut event_rx);
        }

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
                    process_session_event(event, &mut collected_text, &*emitter, &node_id, stage_index);
                }

                result = &mut submit_fut, if !submit_done => {
                    submit_done = true;
                    submit_result = Some(result);
                }
            }

            if submit_done {
                while let Ok(event) = event_rx.try_recv() {
                    process_session_event(
                        event,
                        &mut collected_text,
                        &*emitter,
                        &node_id,
                        stage_index,
                    );
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

        // Return the session to the pool for reuse when fidelity is "full".
        // CLI sessions do not support multi-turn reuse, so they are not pooled.
        if is_full_fidelity
            && let Some(tid) = thread_id
            && !is_cli_session
        {
            let turn_count = prev_turn_count + 1;
            tracing::debug!(
                thread_id = %tid,
                turn_count,
                "returning session to pool"
            );
            self.session_pool.put_back(
                tid,
                crate::session_pool::SessionEntry {
                    agent_name: agent_name.to_string(),
                    turn_count,
                    session: Some(session),
                    event_rx: Some(event_rx),
                },
            );
        }

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

/// Process a single agent session event: accumulate assistant text deltas
/// and forward the event to the pipeline emitter.
fn process_session_event(
    event: stencila_agents::types::SessionEvent,
    collected_text: &mut String,
    emitter: &dyn EventEmitter,
    node_id: &str,
    stage_index: usize,
) {
    if event.kind == stencila_agents::types::EventKind::AssistantTextDelta
        && let Some(serde_json::Value::String(delta)) = event.data.get("delta")
    {
        collected_text.push_str(delta);
    }
    emitter.emit(PipelineEvent::StageSessionEvent {
        node_id: node_id.to_string(),
        stage_index,
        event,
    });
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
    session_pool: crate::session_pool::SessionPool,
    gate_timeout: GateTimeoutConfig,
) -> EngineConfig {
    let mut config = EngineConfig::new();
    config.emitter = emitter.clone();

    // Inner registry: used by ParallelHandler for branch execution.
    // Does not need `parallel` itself (branches don't recurse into parallel).
    let mut inner_registry = HandlerRegistry::with_defaults();
    inner_registry.register("shell", ShellHandler::with_emitter(emitter.clone()));
    inner_registry.register(
        "workflow",
        WorkflowHandler::new(
            workflow_home.clone(),
            emitter.clone(),
            interviewer.clone(),
            gate_timeout.clone(),
        ),
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
                session_pool: session_pool.clone(),
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
        WorkflowHandler::new(
            workflow_home,
            emitter.clone(),
            interviewer.clone(),
            gate_timeout,
        ),
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
                session_pool,
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
    session
        .history()
        .iter()
        .filter_map(|turn| match turn {
            Turn::Assistant { usage, .. } => Some(usage),
            _ => None,
        })
        .fold((0_i64, 0_i64), |(inp, out), usage| {
            (
                inp.saturating_add(i64_from_u64(usage.input_tokens)),
                out.saturating_add(i64_from_u64(usage.output_tokens)),
            )
        })
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

/// Drain stale events from an [`EventReceiver`](stencila_agents::events::EventReceiver)
/// by calling `try_recv()` in a loop until the channel is empty.
///
/// This should be called before each new `submit()` on a reused session to
/// prevent leftover events from a previous submission from leaking into the
/// new event loop.
fn drain_stale_events(receiver: &mut stencila_agents::events::EventReceiver) {
    while receiver.try_recv().is_ok() {}
}

#[cfg(test)]
mod tests {
    use super::*;

    /// AC-2: `AgentCodergenBackend` has a `session_pool: SessionPool` field.
    ///
    /// This test constructs an `AgentCodergenBackend` with a `session_pool`
    /// field. It will fail to compile until the field is added to the struct.
    #[test]
    fn agent_codergen_backend_has_session_pool_field() {
        let pool = crate::session_pool::SessionPool::new();
        let backend = AgentCodergenBackend {
            db_conn: None,
            run_id: None,
            run_metrics: Arc::new(Mutex::new(RunMetrics::default())),
            agent_metadata: HashMap::new(),
            artifacts_dir: None,
            workspace_root: None,
            interviewer: None,
            session_pool: pool.clone(),
        };

        // Verify the pool is accessible and is the same instance.
        backend.session_pool.put_back(
            "test-thread".to_string(),
            crate::session_pool::SessionEntry {
                agent_name: "test-agent".to_string(),
                ..Default::default()
            },
        );
        let entry = pool.take("test-thread");
        assert!(
            entry.is_some(),
            "the session_pool field on AgentCodergenBackend should share \
             state with the original SessionPool clone"
        );
    }

    /// AC-3: `build_engine_config()` creates a single `SessionPool` and
    /// passes clones to both inner and outer `AgentCodergenBackend` instances.
    ///
    /// Since `build_engine_config()` is private and returns an `EngineConfig`
    /// (which doesn't expose the pool), we verify this indirectly: the
    /// function signature must accept a `SessionPool` parameter so that the
    /// caller (in `run_workflow_with_options_and_parent`) can share one pool
    /// across both registries.
    ///
    /// This test will fail to compile until `build_engine_config()` accepts
    /// a `SessionPool` parameter.
    #[test]
    fn build_engine_config_accepts_session_pool() {
        let pool = crate::session_pool::SessionPool::new();
        let emitter: Arc<dyn stencila_attractor::events::EventEmitter> =
            Arc::new(stencila_attractor::events::NoOpEmitter);

        // Call build_engine_config with a SessionPool parameter.
        // This will fail to compile until the function signature is updated.
        let _config = build_engine_config(
            std::path::PathBuf::from("/tmp/test"),
            emitter,
            None, // interviewer
            None, // db_conn
            None, // run_id
            Arc::new(Mutex::new(RunMetrics::default())),
            HashMap::new(),
            None,                         // artifacts_dir
            None,                         // workspace_root
            None,                         // agent_interviewer
            pool,                         // session_pool
            GateTimeoutConfig::default(), // gate_timeout
        );
    }

    /// AC-4: `run()` reads `internal.fidelity` and `internal.thread_id`
    /// from context; when fidelity is Full, calls `session_pool.take(thread_id)`
    /// and wraps result in `SessionGuard`.
    ///
    /// This test constructs an `AgentCodergenBackend` with a pool, then
    /// pre-populates the pool with a session entry. After calling `run()`
    /// with fidelity="full", the pool should show that the entry was
    /// taken and then returned (via `SessionGuard::Drop`) with an
    /// incremented `turn_count`.
    ///
    /// **Expected to FAIL** because:
    /// 1. `AgentCodergenBackend` does not yet have a `session_pool` field
    ///    (compilation failure).
    /// 2. Even after adding the field, `run()` does not yet consult the
    ///    pool (runtime assertion failure on `turn_count`).
    #[tokio::test]
    #[ignore] // Requires agent session creation infrastructure; validated via integration test
    async fn run_uses_pool_when_fidelity_is_full() -> eyre::Result<()> {
        let pool = crate::session_pool::SessionPool::new();

        // Pre-populate the pool with a session entry for "thread-1".
        pool.put_back(
            "thread-1".to_string(),
            crate::session_pool::SessionEntry {
                agent_name: "pre-existing-agent".to_string(),
                turn_count: 5,
                ..Default::default()
            },
        );

        let backend = AgentCodergenBackend {
            db_conn: None,
            run_id: None,
            run_metrics: Arc::new(Mutex::new(RunMetrics::default())),
            agent_metadata: HashMap::new(),
            artifacts_dir: None,
            workspace_root: None,
            interviewer: None,
            session_pool: pool.clone(),
        };

        let node = stencila_attractor::graph::Node::new("test-node");
        let context = stencila_attractor::context::Context::new();
        context.set(
            "internal.fidelity",
            serde_json::Value::String("full".into()),
        );
        context.set(
            "internal.thread_id",
            serde_json::Value::String("thread-1".into()),
        );

        // This call will fail at runtime because create_session requires
        // real agent infrastructure. The test is #[ignore]d but documents
        // the expected behavior. The pool interaction is verified through
        // the integration test with PoolAwareMockBackend instead.
        let _result = backend
            .run(
                &node,
                "test prompt",
                &context,
                Arc::new(stencila_attractor::events::NoOpEmitter),
                0,
            )
            .await;

        // After run() with fidelity=full and thread_id="thread-1":
        // - run() should have called pool.take("thread-1") → Some(entry with turn_count=5)
        // - run() should have wrapped it in SessionGuard::from_pool(...)
        // - SessionGuard::Drop should have put it back with turn_count=6
        assert_eq!(
            pool.turn_count("thread-1"),
            Some(6),
            "after run() with fidelity=full, the pool entry should have \
             turn_count incremented by 1 (from 5 to 6) via SessionGuard::Drop"
        );

        Ok(())
    }

    /// AC-4 (cont): When fidelity is not "full", `run()` should NOT
    /// consult the session pool at all.
    ///
    /// **Expected to FAIL** because `AgentCodergenBackend` does not yet
    /// have a `session_pool` field (compilation failure).
    #[tokio::test]
    #[ignore] // Requires agent session creation infrastructure; validated via integration test
    async fn run_does_not_use_pool_when_fidelity_is_compact() -> eyre::Result<()> {
        let pool = crate::session_pool::SessionPool::new();

        // Pre-populate the pool — it should remain untouched.
        pool.put_back(
            "thread-1".to_string(),
            crate::session_pool::SessionEntry {
                agent_name: "pre-existing-agent".to_string(),
                turn_count: 3,
                ..Default::default()
            },
        );

        let backend = AgentCodergenBackend {
            db_conn: None,
            run_id: None,
            run_metrics: Arc::new(Mutex::new(RunMetrics::default())),
            agent_metadata: HashMap::new(),
            artifacts_dir: None,
            workspace_root: None,
            interviewer: None,
            session_pool: pool.clone(),
        };

        let node = stencila_attractor::graph::Node::new("test-node");
        let context = stencila_attractor::context::Context::new();
        context.set(
            "internal.fidelity",
            serde_json::Value::String("compact".into()),
        );

        let _result = backend
            .run(
                &node,
                "test prompt",
                &context,
                Arc::new(stencila_attractor::events::NoOpEmitter),
                0,
            )
            .await;

        // Pool should be untouched — entry should still be there with
        // original turn_count.
        assert_eq!(
            pool.turn_count("thread-1"),
            Some(3),
            "pool entry should be untouched when fidelity is not 'full'"
        );

        Ok(())
    }

    /// Slice 5 (CLI session fallback): `AgentCodergenBackend::run()` detects
    /// `AgentSession::Cli` and calls `guard.discard()` so the session is not
    /// pooled.
    ///
    /// This test verifies the real backend's behavior when the created
    /// session is a CLI variant. The pool should remain empty after
    /// `run()` completes because `guard.discard()` was called.
    ///
    /// **Expected to FAIL** until `AgentCodergenBackend::run()` implements
    /// CLI detection: `if let AgentSession::Cli(_) = session { guard.discard(); }`
    #[tokio::test]
    #[ignore] // Requires agent session creation infrastructure
    async fn run_discards_guard_for_cli_sessions() -> eyre::Result<()> {
        let pool = crate::session_pool::SessionPool::new();

        let backend = AgentCodergenBackend {
            db_conn: None,
            run_id: None,
            run_metrics: Arc::new(Mutex::new(RunMetrics::default())),
            agent_metadata: HashMap::new(),
            artifacts_dir: None,
            workspace_root: None,
            interviewer: None,
            session_pool: pool.clone(),
        };

        // Use a node with agent_type="cli" to trigger CLI session creation
        let mut node = stencila_attractor::graph::Node::new("cli-test-node");
        node.attrs.insert(
            "agent_type".into(),
            stencila_attractor::graph::AttrValue::String("cli".into()),
        );
        let context = stencila_attractor::context::Context::new();
        context.set(
            "internal.fidelity",
            serde_json::Value::String("full".into()),
        );
        context.set(
            "internal.thread_id",
            serde_json::Value::String("cli-thread".into()),
        );

        // This will fail at runtime because create_session requires real
        // agent infrastructure, but the #[ignore] attribute keeps it from
        // running in CI. The test documents the expected behavior.
        let _result = backend
            .run(
                &node,
                "test prompt",
                &context,
                Arc::new(stencila_attractor::events::NoOpEmitter),
                0,
            )
            .await;

        // After run() with a CLI session and fidelity=full:
        // - run() should detect AgentSession::Cli
        // - run() should log a tracing::warn about CLI sessions not supporting reuse
        // - run() should call guard.discard()
        // - The pool should remain empty
        assert!(
            pool.take("cli-thread").is_none(),
            "CLI sessions should not be pooled — guard.discard() should be \
             called when AgentSession::Cli is detected, preventing the entry \
             from being returned to the pool"
        );

        Ok(())
    }

    /// Slice 6 (Event drain): Before resubmitting on a reused session,
    /// stale events from the previous submission must be drained from the
    /// `EventReceiver` via `try_recv()` loop.
    ///
    /// This test creates an `EventReceiver` with pre-buffered events
    /// and verifies that calling `drain_stale_events()` (the helper
    /// function that `AgentCodergenBackend::run()` should use) consumes
    /// all stale events before the next submission begins.
    ///
    /// **Expected to FAIL** until a `drain_stale_events()` helper is
    /// added (or the drain logic is inlined in `run()`).
    #[tokio::test]
    async fn drain_stale_events_clears_buffered_events() -> eyre::Result<()> {
        let (emitter, mut receiver) = stencila_agents::events::channel();

        // Simulate stale events from a previous submission
        emitter.emit_assistant_text_delta("stale1");
        emitter.emit_assistant_text_delta("stale2");

        // Call the drain helper that should exist as a free function in
        // run.rs. This will fail to compile until the function is
        // implemented.
        drain_stale_events(&mut receiver);

        // After draining, try_recv should return Empty (no events left)
        let result = receiver.try_recv();
        assert!(
            result.is_err(),
            "after drain_stale_events(), the receiver should be empty, \
             but try_recv() returned Ok — stale events were not drained"
        );

        Ok(())
    }

    /// Slice 6 (cont): drain on an already-empty receiver is a no-op.
    ///
    /// **Expected to FAIL** until `drain_stale_events()` is implemented.
    #[tokio::test]
    async fn drain_stale_events_is_noop_on_empty_receiver() -> eyre::Result<()> {
        let (_emitter, mut receiver) = stencila_agents::events::channel();

        // Drain an empty receiver — should not panic or block.
        drain_stale_events(&mut receiver);

        let result = receiver.try_recv();
        assert!(
            result.is_err(),
            "draining an empty receiver should leave it empty"
        );

        Ok(())
    }

    // -- GateTimeoutConfig tests (Phase 1 / Slice 2) --

    /// AC-1: `GateTimeoutConfig` enum exists with `Interactive`, `AutoApprove`,
    /// and `Timed { seconds: f64 }` variants and derives `Clone`, `Debug`,
    /// `Default` (defaulting to `Interactive`).
    #[test]
    fn gate_timeout_config_has_expected_variants_and_derives() {
        // Construct each variant to verify they exist.
        let interactive = GateTimeoutConfig::Interactive;
        let auto_approve = GateTimeoutConfig::AutoApprove;
        let timed = GateTimeoutConfig::Timed { seconds: 30.0 };

        // Verify Default is Interactive.
        let default_config = GateTimeoutConfig::default();
        assert!(
            matches!(default_config, GateTimeoutConfig::Interactive),
            "GateTimeoutConfig::default() should be Interactive"
        );

        // Verify Clone.
        let _cloned = interactive.clone();
        let _cloned = auto_approve.clone();
        let _cloned = timed.clone();

        // Verify Debug.
        let debug_str = format!("{interactive:?}");
        assert!(
            !debug_str.is_empty(),
            "Debug should produce non-empty output"
        );
    }

    /// AC-2: `RunOptions` has a `gate_timeout: GateTimeoutConfig` field that
    /// defaults to `Interactive`.
    #[test]
    fn run_options_has_gate_timeout_field() {
        let options = RunOptions {
            emitter: Arc::new(stencila_attractor::events::NoOpEmitter),
            interviewer: None,
            run_id_out: None,
            gate_timeout: GateTimeoutConfig::AutoApprove,
        };

        assert!(
            matches!(options.gate_timeout, GateTimeoutConfig::AutoApprove),
            "RunOptions should accept gate_timeout field"
        );
    }

    /// AC-2 (cont): `RunOptions` with default gate_timeout (Interactive) still
    /// works at the `run_workflow` convenience wrapper construction site.
    #[test]
    fn run_options_defaults_gate_timeout_to_interactive() {
        let options = RunOptions {
            emitter: Arc::new(stencila_attractor::events::NoOpEmitter),
            interviewer: None,
            run_id_out: None,
            gate_timeout: GateTimeoutConfig::default(),
        };

        assert!(
            matches!(options.gate_timeout, GateTimeoutConfig::Interactive),
            "default gate_timeout should be Interactive"
        );
    }

    /// AC-4: `GateTimeoutConfig::Interactive` serializes to empty/omit for
    /// context — `to_context_json()` returns `None`.
    #[test]
    fn gate_timeout_interactive_serializes_to_none() {
        let config = GateTimeoutConfig::Interactive;
        let json = config.to_context_json();
        assert!(
            json.is_none(),
            "Interactive should serialize to None (omit from context), got {json:?}"
        );
    }

    /// AC-5: `GateTimeoutConfig::AutoApprove` serializes to `{{"*": 0}}` for
    /// context.
    #[test]
    fn gate_timeout_auto_approve_serializes_to_star_zero() {
        let config = GateTimeoutConfig::AutoApprove;
        let json = config.to_context_json();
        assert!(json.is_some(), "AutoApprove should produce Some(...)");
        let value = json.expect("should be Some");
        let expected: serde_json::Value = serde_json::json!({"*": 0});
        assert_eq!(
            value, expected,
            "AutoApprove should serialize to {{\"*\": 0}}, got {value}"
        );
    }

    /// AC-6: `GateTimeoutConfig::Timed { seconds: 30.0 }` serializes to
    /// `{{"*": 30}}` for context.
    #[test]
    fn gate_timeout_timed_serializes_to_star_seconds() {
        let config = GateTimeoutConfig::Timed { seconds: 30.0 };
        let json = config.to_context_json();
        assert!(json.is_some(), "Timed should produce Some(...)");
        let value = json.expect("should be Some");
        let expected: serde_json::Value = serde_json::json!({"*": 30});
        assert_eq!(
            value, expected,
            "Timed {{ seconds: 30.0 }} should serialize to {{\"*\": 30}}, got {value}"
        );
    }

    /// AC-6 (cont): Timed with fractional seconds preserves the value.
    #[test]
    fn gate_timeout_timed_fractional_seconds() {
        let config = GateTimeoutConfig::Timed { seconds: 15.5 };
        let json = config.to_context_json();
        assert!(json.is_some(), "Timed should produce Some(...)");
        let value = json.expect("should be Some");
        let expected: serde_json::Value = serde_json::json!({"*": 15.5});
        assert_eq!(
            value, expected,
            "Timed {{ seconds: 15.5 }} should serialize to {{\"*\": 15.5}}, got {value}"
        );
    }

    /// AC-7: `propagate_gate_timeout` writes `internal.gate_timeouts` into
    /// context for `AutoApprove` config.
    ///
    /// The helper function `propagate_gate_timeout(config, context)` should
    /// call `config.to_context_json()` and, when the result is `Some`, write
    /// it into the context under key `"internal.gate_timeouts"`.
    #[test]
    fn propagate_gate_timeout_writes_auto_approve_to_context() {
        let context = stencila_attractor::context::Context::new();
        let config = GateTimeoutConfig::AutoApprove;

        propagate_gate_timeout(&config, &context);

        let value = context.get("internal.gate_timeouts");
        assert!(
            value.is_some(),
            "AutoApprove should write internal.gate_timeouts to context"
        );
        let expected: serde_json::Value = serde_json::json!({"*": 0});
        assert_eq!(
            value.expect("should be Some"),
            expected,
            "internal.gate_timeouts should be {{\"*\": 0}} for AutoApprove"
        );
    }

    /// AC-7 (cont): `propagate_gate_timeout` writes `internal.gate_timeouts`
    /// into context for `Timed` config.
    #[test]
    fn propagate_gate_timeout_writes_timed_to_context() {
        let context = stencila_attractor::context::Context::new();
        let config = GateTimeoutConfig::Timed { seconds: 45.0 };

        propagate_gate_timeout(&config, &context);

        let value = context.get("internal.gate_timeouts");
        assert!(
            value.is_some(),
            "Timed should write internal.gate_timeouts to context"
        );
        let expected: serde_json::Value = serde_json::json!({"*": 45});
        assert_eq!(
            value.expect("should be Some"),
            expected,
            "internal.gate_timeouts should be {{\"*\": 45}} for Timed {{ seconds: 45.0 }}"
        );
    }

    /// AC-7 (cont): `propagate_gate_timeout` does NOT write
    /// `internal.gate_timeouts` for `Interactive` config (omit).
    #[test]
    fn propagate_gate_timeout_omits_interactive_from_context() {
        let context = stencila_attractor::context::Context::new();
        let config = GateTimeoutConfig::Interactive;

        propagate_gate_timeout(&config, &context);

        let value = context.get("internal.gate_timeouts");
        assert!(
            value.is_none(),
            "Interactive should not write internal.gate_timeouts to context, \
             but found {value:?}"
        );
    }

    // NOTE: AC-3 (all existing `RunOptions` construction sites compile) is
    // verified by `cargo check` / `cargo clippy` across the workspace. The
    // implementation must update run_workflow(), Run::run(), Resume::run(),
    // spawn_workflow(), spawn_resume_workflow(), WorkflowHandler::execute(),
    // and test helpers. This is a compilation-level concern, not a unit-test
    // concern. See AC-8/AC-9.

    // -- Phase 6 / Slice 1: build_engine_config accepts gate_timeout --

    /// AC-3 (Phase 6): `build_engine_config()` accepts a `gate_timeout:
    /// GateTimeoutConfig` parameter.
    ///
    /// This test will fail to compile until `build_engine_config` is
    /// updated to accept the new parameter.
    #[test]
    fn build_engine_config_accepts_gate_timeout() {
        let pool = crate::session_pool::SessionPool::new();
        let emitter: Arc<dyn stencila_attractor::events::EventEmitter> =
            Arc::new(stencila_attractor::events::NoOpEmitter);

        let _config = build_engine_config(
            std::path::PathBuf::from("/tmp/test"),
            emitter,
            None, // interviewer
            None, // db_conn
            None, // run_id
            Arc::new(Mutex::new(RunMetrics::default())),
            HashMap::new(),
            None,                           // artifacts_dir
            None,                           // workspace_root
            None,                           // agent_interviewer
            pool,                           // session_pool
            GateTimeoutConfig::AutoApprove, // gate_timeout — new parameter
        );
    }
}
