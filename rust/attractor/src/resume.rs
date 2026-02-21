//! Checkpoint resume (§5.3).
//!
//! Restores pipeline execution state from a checkpoint, enabling
//! crash recovery and resume after interruption.

use std::path::Path;

use indexmap::IndexMap;

use crate::checkpoint::Checkpoint;
use crate::context::Context;
use crate::error::AttractorResult;
use crate::fidelity::resolve_fidelity;
use crate::graph::Graph;
use crate::types::{FidelityMode, HandlerType};

/// State restored from a checkpoint for resuming execution.
#[derive(Debug)]
pub struct ResumeState {
    /// The context restored from checkpoint values.
    pub context: Context,
    /// Ordered list of completed nodes (for checkpoint re-saving).
    pub completed_nodes_ordered: Vec<String>,
    /// Outcome status strings per completed node (e.g., `"success"`, `"fail"`).
    ///
    /// Empty for legacy checkpoints that pre-date this field; the engine
    /// falls back to assuming success in that case.
    pub node_statuses: IndexMap<String, String>,
    /// The node to start execution from (the one after `current_node`).
    pub next_node_id: String,
    /// Whether fidelity should be degraded for the first resumed hop.
    pub degrade_fidelity: bool,
}

/// Resume pipeline execution from a checkpoint file.
///
/// Loads the checkpoint, restores context and completed nodes, and
/// determines where to resume execution. If the previous node used
/// `full` fidelity (resolved via the §5.4 precedence chain), the
/// first resumed hop degrades to `summary:high` since in-memory LLM
/// sessions cannot be serialized (§5.3).
///
/// # Errors
///
/// Returns an error if the checkpoint file cannot be loaded or the
/// graph structure doesn't match the checkpoint state.
pub fn resume_from_checkpoint(
    checkpoint_path: &Path,
    graph: &Graph,
) -> AttractorResult<ResumeState> {
    let checkpoint = Checkpoint::load(checkpoint_path)?;

    // Restore context from checkpoint values
    let context = Context::new();
    for (key, value) in &checkpoint.context_values {
        context.set(key.clone(), value.clone());
    }
    for log in &checkpoint.logs {
        context.append_log(log.clone());
    }

    // Restore retry counters in context
    for (node_id, count) in &checkpoint.node_retries {
        context.set(
            format!("internal.retry_count.{node_id}"),
            serde_json::Value::Number(serde_json::Number::from(*count)),
        );
    }

    // Determine the next node to execute.
    // Prefer the explicit next_node_id stored at checkpoint time (set by the
    // engine after edge selection). Fall back to heuristic for legacy
    // checkpoints that lack this field.
    let next_node_id = if let Some(ref next) = checkpoint.next_node_id {
        next.clone()
    } else {
        determine_next_node(&checkpoint.current_node, graph)?
    };

    // Check if fidelity degradation is needed (§5.3):
    // Use the full §5.4 precedence chain (edge → node → graph → default)
    // to determine effective fidelity.
    let degrade_fidelity =
        should_degrade_fidelity(graph, &checkpoint.current_node, &checkpoint.completed_nodes);

    Ok(ResumeState {
        context,
        completed_nodes_ordered: checkpoint.completed_nodes,
        node_statuses: checkpoint.node_statuses,
        next_node_id,
        degrade_fidelity,
    })
}

/// Resume pipeline execution state from `SQLite` rows for a run.
///
/// Reconstructs loop state from `workflow_nodes` and `workflow_edges`.
/// Context and logs are already durable in `SQLite` and are provided by
/// a SQLite-backed [`Context`].
///
/// # Errors
///
/// Returns an error if `SQLite` queries fail or the graph structure
/// doesn't match the stored run state.
#[cfg(feature = "sqlite")]
pub fn resume_from_sqlite(
    conn: &std::sync::Arc<std::sync::Mutex<stencila_db::rusqlite::Connection>>,
    run_id: &str,
    graph: &Graph,
) -> AttractorResult<ResumeState> {
    let context = Context::with_backend(Box::new(
        crate::sqlite_backend::SqliteBackend::from_shared(conn.clone(), run_id.to_string()),
    ));

    let mut completed_nodes_ordered = Vec::new();
    let mut node_statuses = IndexMap::new();

    let edge = load_sqlite_run_state(
        conn,
        run_id,
        &context,
        &mut completed_nodes_ordered,
        &mut node_statuses,
    )?;

    let (current_node, next_node_id) = match edge {
        Ok((from, to)) => (from, to),
        Err(stencila_db::rusqlite::Error::QueryReturnedNoRows) => {
            let start = graph.find_start_node()?.id.clone();
            (start.clone(), start)
        }
        Err(error) => {
            return Err(crate::error::AttractorError::Io {
                message: format!("Failed to query workflow_edges for resume: {error}"),
            });
        }
    };

    let degrade_fidelity = should_degrade_fidelity(graph, &current_node, &completed_nodes_ordered);

    Ok(ResumeState {
        context,
        completed_nodes_ordered,
        node_statuses,
        next_node_id,
        degrade_fidelity,
    })
}

#[cfg(feature = "sqlite")]
fn load_sqlite_run_state(
    conn: &std::sync::Arc<std::sync::Mutex<stencila_db::rusqlite::Connection>>,
    run_id: &str,
    context: &Context,
    completed_nodes_ordered: &mut Vec<String>,
    node_statuses: &mut IndexMap<String, String>,
) -> AttractorResult<Result<(String, String), stencila_db::rusqlite::Error>> {
    let db = conn
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut stmt = db
        .prepare(
            "SELECT node_id, status, retry_count
             FROM workflow_nodes
             WHERE run_id = ?1
             ORDER BY COALESCE(completed_at, started_at, ''), node_id",
        )
        .map_err(|error| crate::error::AttractorError::Io {
            message: format!("Failed to prepare workflow_nodes resume query: {error}"),
        })?;

    let mut rows = stmt
        .query((run_id,))
        .map_err(|error| crate::error::AttractorError::Io {
            message: format!("Failed to query workflow_nodes for resume: {error}"),
        })?;
    while let Some(row) = rows
        .next()
        .map_err(|error| crate::error::AttractorError::Io {
            message: format!("Failed to read workflow_nodes resume row: {error}"),
        })?
    {
        let node_id: String = row
            .get(0)
            .map_err(|error| crate::error::AttractorError::Io {
                message: format!("Failed to read resumed node_id: {error}"),
            })?;
        let status: String = row
            .get(1)
            .map_err(|error| crate::error::AttractorError::Io {
                message: format!("Failed to read resumed node status: {error}"),
            })?;
        let retry_count: i64 = row
            .get(2)
            .map_err(|error| crate::error::AttractorError::Io {
                message: format!("Failed to read resumed node retry count: {error}"),
            })?;
        completed_nodes_ordered.push(node_id.clone());
        node_statuses.insert(node_id.clone(), status);
        context.set(
            format!("internal.retry_count.{node_id}"),
            serde_json::Value::Number(serde_json::Number::from(retry_count)),
        );
    }
    drop(rows);
    drop(stmt);

    let edge = db.query_row(
        "SELECT from_node, to_node
         FROM workflow_edges
         WHERE run_id = ?1
         ORDER BY step_index DESC
         LIMIT 1",
        (run_id,),
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
    );

    Ok(edge)
}

fn should_degrade_fidelity(
    graph: &Graph,
    current_node: &str,
    completed_nodes_ordered: &[String],
) -> bool {
    let predecessor = completed_nodes_ordered
        .len()
        .checked_sub(2)
        .and_then(|idx| completed_nodes_ordered.get(idx));

    graph.get_node(current_node).is_some_and(|node| {
        let incoming = graph.incoming_edges(current_node);
        let specific_edge =
            predecessor.and_then(|pred| incoming.iter().find(|e| e.from == *pred).copied());

        if let Some(edge) = specific_edge {
            resolve_fidelity(node, Some(edge), graph) == FidelityMode::Full
        } else if incoming.len() <= 1 {
            let incoming_edge = incoming.first().copied();
            resolve_fidelity(node, incoming_edge, graph) == FidelityMode::Full
        } else {
            incoming
                .iter()
                .any(|edge| resolve_fidelity(node, Some(edge), graph) == FidelityMode::Full)
        }
    })
}

/// Determine the next node after `current_node` in the graph.
///
/// This is the legacy fallback used when the checkpoint does not contain
/// an explicit `next_node_id`. Only safe for linear (non-branching) graphs.
///
/// # Errors
///
/// Returns an error if the current node has multiple outgoing edges
/// (ambiguous routing), or has none and is not an exit node.
fn determine_next_node(current_node: &str, graph: &Graph) -> AttractorResult<String> {
    let edges = graph.outgoing_edges(current_node);

    match edges.len() {
        0 => {
            // If the current node is an exit node, resume at exit
            if graph
                .get_node(current_node)
                .is_some_and(|n| n.handler_type() == HandlerType::Exit || Graph::is_exit_node(n))
            {
                Ok(current_node.to_string())
            } else {
                Err(crate::error::AttractorError::InvalidPipeline {
                    reason: format!("cannot resume: node '{current_node}' has no outgoing edges"),
                })
            }
        }
        1 => Ok(edges[0].to.clone()),
        _ => Err(crate::error::AttractorError::InvalidPipeline {
            reason: format!(
                "cannot resume: node '{current_node}' has {} outgoing edges; \
                 checkpoint lacks next_node_id for unambiguous routing",
                edges.len()
            ),
        }),
    }
}
