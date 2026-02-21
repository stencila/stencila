//! Pipeline execution engine (§3.1–3.2).
//!
//! The engine takes a parsed [`Graph`] and runs it to completion,
//! executing each node through the appropriate [`Handler`], selecting
//! edges via the 5-step algorithm, and managing retries, checkpoints,
//! and events along the way.

mod loop_core;
mod routing;

use std::path::PathBuf;
use std::sync::Arc;

use crate::events::{EventEmitter, NoOpEmitter};
use crate::graph::Graph;
use crate::handler::HandlerRegistry;
use crate::transform::TransformRegistry;
use crate::types::Outcome;

pub use routing::{GoalGateResult, check_goal_gates, get_retry_target};

/// Configuration for the pipeline engine.
pub struct EngineConfig {
    /// Root directory for run artifacts (run directories are created inside).
    pub logs_root: PathBuf,
    /// Handler registry for resolving node types to handlers.
    pub registry: HandlerRegistry,
    /// Transform registry for graph preprocessing before execution.
    pub transforms: TransformRegistry,
    /// Event emitter for pipeline lifecycle events.
    pub emitter: Arc<dyn EventEmitter>,
    /// Skip validation before execution (default: `false`).
    ///
    /// When `false`, the engine runs [`validate_or_raise`](crate::validation::validate_or_raise)
    /// after transforms and refuses to execute pipelines with ERROR-level diagnostics (§7.1).
    /// Set to `true` only for testing or when validation has already been performed.
    pub skip_validation: bool,
}

impl std::fmt::Debug for EngineConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EngineConfig")
            .field("logs_root", &self.logs_root)
            .field("registry", &self.registry)
            .field("transforms", &"TransformRegistry { .. }")
            .field("skip_validation", &self.skip_validation)
            .finish_non_exhaustive()
    }
}

impl EngineConfig {
    /// Create a new engine configuration with the given logs root,
    /// default handler registry, default transforms, and no-op event emitter.
    ///
    /// The default registry includes handlers for `start`, `exit`,
    /// `conditional`, `codergen` (simulation mode), `shell`, and
    /// `parallel.fan_in`. Handlers that require runtime dependencies
    /// are **not** included: `parallel` (needs `Arc<HandlerRegistry>` +
    /// `Arc<dyn EventEmitter>`) and `wait.human` (needs `Arc<dyn
    /// Interviewer>`). Register them explicitly via
    /// `config.registry.register(...)` before calling [`run`].
    #[must_use]
    pub fn new(logs_root: impl Into<PathBuf>) -> Self {
        Self {
            logs_root: logs_root.into(),
            registry: HandlerRegistry::with_defaults(),
            transforms: TransformRegistry::with_defaults(),
            emitter: Arc::new(NoOpEmitter),
            skip_validation: false,
        }
    }
}

/// Run a pipeline graph to completion.
///
/// Applies transforms, validates the graph (refusing execution if any
/// ERROR-level diagnostics are found per §7.1), then executes nodes
/// in traversal order.
///
/// # Errors
///
/// Returns an error if:
/// - A transform fails
/// - Validation finds ERROR-level diagnostics
/// - The graph has no start node (shape `Mdiamond` or id `start`)
/// - The graph has no exit node (shape `Msquare` or id `exit`)
/// - A handler cannot be resolved for a node
/// - An I/O error occurs writing run artifacts
pub async fn run(graph: &Graph, config: EngineConfig) -> crate::error::AttractorResult<Outcome> {
    // Apply transforms (variable expansion, stylesheet, etc.) before execution.
    let mut graph = graph.clone();
    config.transforms.apply_all(&mut graph)?;

    // Validate the graph — refuse execution if any ERROR-level diagnostics (§7.1).
    if !config.skip_validation {
        crate::validation::validate_or_raise(&graph, &[])?;
    }

    loop_core::run_loop(&graph, config).await
}

/// Run a pipeline with a pre-created context.
///
/// Like [`run`] but accepts an externally-created [`Context`] (e.g. one
/// backed by `SQLite`) instead of creating a fresh in-memory context.
///
/// # Errors
///
/// Same as [`run`].
pub async fn run_with_context(
    graph: &Graph,
    config: EngineConfig,
    context: crate::context::Context,
) -> crate::error::AttractorResult<Outcome> {
    let mut graph = graph.clone();
    config.transforms.apply_all(&mut graph)?;

    if !config.skip_validation {
        crate::validation::validate_or_raise(&graph, &[])?;
    }

    loop_core::run_loop_with_context(&graph, config, context).await
}

/// Resume a pipeline from a checkpoint file (§5.3).
///
/// Loads execution state from the checkpoint, restores context and
/// completed nodes, then continues the traversal loop from the saved
/// next-node position. A fresh run directory is created for the resumed run.
///
/// # Errors
///
/// Returns an error if the checkpoint cannot be loaded, the graph
/// structure doesn't match, or any handler/I/O error occurs during
/// resumed execution.
pub async fn resume(
    graph: &Graph,
    config: EngineConfig,
    checkpoint_path: &std::path::Path,
) -> crate::error::AttractorResult<Outcome> {
    let mut graph = graph.clone();
    config.transforms.apply_all(&mut graph)?;

    if !config.skip_validation {
        crate::validation::validate_or_raise(&graph, &[])?;
    }

    let resume_state = crate::resume::resume_from_checkpoint(checkpoint_path, &graph)?;
    loop_core::resume_loop(&graph, config, resume_state).await
}

/// Resume a pipeline from SQLite-backed run state.
///
/// Reconstructs resume state from run-scoped tables (`workflow_nodes`,
/// `workflow_edges`, and SQLite-backed context/logs) instead of reading
/// `checkpoint.json`.
///
/// # Errors
///
/// Same as [`run`], plus `SQLite` query errors when loading run state.
#[cfg(feature = "sqlite")]
pub async fn resume_with_sqlite(
    graph: &Graph,
    config: EngineConfig,
    conn: std::sync::Arc<std::sync::Mutex<stencila_db::rusqlite::Connection>>,
    run_id: &str,
) -> crate::error::AttractorResult<Outcome> {
    let mut graph = graph.clone();
    config.transforms.apply_all(&mut graph)?;

    if !config.skip_validation {
        crate::validation::validate_or_raise(&graph, &[])?;
    }

    let resume_state = crate::resume::resume_from_sqlite(&conn, run_id, &graph)?;
    loop_core::resume_loop(&graph, config, resume_state).await
}
