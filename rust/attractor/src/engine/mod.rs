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
}

impl std::fmt::Debug for EngineConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EngineConfig")
            .field("logs_root", &self.logs_root)
            .field("registry", &self.registry)
            .field("transforms", &"TransformRegistry { .. }")
            .finish_non_exhaustive()
    }
}

impl EngineConfig {
    /// Create a new engine configuration with the given logs root,
    /// default handler registry, default transforms, and no-op event emitter.
    #[must_use]
    pub fn new(logs_root: impl Into<PathBuf>) -> Self {
        Self {
            logs_root: logs_root.into(),
            registry: HandlerRegistry::with_defaults(),
            transforms: TransformRegistry::with_defaults(),
            emitter: Arc::new(NoOpEmitter),
        }
    }
}

/// Run a pipeline graph to completion.
///
/// Validates that both start and exit nodes exist, creates a run
/// directory, executes nodes in traversal order, and returns the
/// final outcome.
///
/// # Errors
///
/// Returns an error if:
/// - The graph has no start node (shape `Mdiamond` or id `start`)
/// - The graph has no exit node (shape `Msquare` or id `exit`)
/// - A handler cannot be resolved for a node
/// - An I/O error occurs writing run artifacts
pub async fn run(graph: &Graph, config: EngineConfig) -> crate::error::AttractorResult<Outcome> {
    // Apply transforms (variable expansion, etc.) before execution.
    let mut graph = graph.clone();
    config.transforms.apply_all(&mut graph)?;

    loop_core::run_loop(&graph, config).await
}
