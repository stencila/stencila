//! Parallel handler (§4.8).
//!
//! Fans out execution to multiple branches concurrently. Each branch
//! receives an isolated clone of the parent context and runs independently.
//! The handler uses `FuturesUnordered` for policy-driven completion:
//! `FirstSuccess` returns as soon as one branch succeeds; `FailFast`
//! cancels remaining branches on the first failure.

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use indexmap::IndexMap;

use tokio::sync::Semaphore;

use crate::context::Context;
use crate::edge_selection::select_edge;
use crate::error::AttractorResult;
use crate::events::{EventEmitter, NoOpEmitter, PipelineEvent};
use crate::graph::{Graph, Node};
use crate::handler::{Handler, HandlerRegistry};
use crate::retry::{build_retry_policy, execute_with_retry};
use crate::types::{Outcome, StageStatus};

/// Default max concurrency when `max_parallel` is not set on the node (§4.8).
pub const DEFAULT_MAX_PARALLEL: usize = 4;

/// Join policy for parallel branches (§4.8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinPolicy {
    /// All branches must complete.
    WaitAll,
    /// Join satisfied as soon as one branch succeeds.
    FirstSuccess,
}

impl JoinPolicy {
    fn from_str_or_default(s: Option<&str>) -> Self {
        match s {
            Some("first_success") => Self::FirstSuccess,
            _ => Self::WaitAll,
        }
    }
}

/// Error policy for parallel branches (§4.8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorPolicy {
    /// Cancel remaining on first failure.
    FailFast,
    /// Continue remaining branches; collect all results.
    Continue,
    /// Ignore failures; return only successful results.
    Ignore,
}

impl ErrorPolicy {
    fn from_str_or_default(s: Option<&str>) -> Self {
        match s {
            Some("fail_fast") => Self::FailFast,
            Some("ignore") => Self::Ignore,
            _ => Self::Continue,
        }
    }
}

/// Resolved policies for a parallel execution (§4.8).
struct ParallelPolicies {
    join: JoinPolicy,
    error: ErrorPolicy,
    max_parallel: usize,
}

/// Result of a single parallel branch execution.
#[derive(Debug, Clone)]
pub struct BranchResult {
    /// Target node ID of this branch.
    pub target: String,
    /// The outcome of executing this branch.
    pub outcome: Outcome,
}

/// Handler for parallel fan-out nodes.
///
/// Executes all outgoing branches concurrently with an isolated context
/// clone per branch. The `join_policy` and `error_policy` node attributes
/// control completion semantics. Each branch performs subgraph traversal
/// (following edges forward) until reaching a fan-in node or dead end.
pub struct ParallelHandler {
    registry: Arc<HandlerRegistry>,
    emitter: Arc<dyn EventEmitter>,
}

impl std::fmt::Debug for ParallelHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParallelHandler").finish_non_exhaustive()
    }
}

impl ParallelHandler {
    /// Create a new parallel handler.
    #[must_use]
    pub fn new(registry: Arc<HandlerRegistry>, emitter: Arc<dyn EventEmitter>) -> Self {
        Self { registry, emitter }
    }
}

#[async_trait]
impl Handler for ParallelHandler {
    #[allow(clippy::too_many_lines)]
    async fn execute(
        &self,
        node: &Node,
        context: &Context,
        graph: &Graph,
        logs_root: &Path,
    ) -> AttractorResult<Outcome> {
        let edges = graph.outgoing_edges(&node.id);
        if edges.is_empty() {
            return Ok(Outcome::fail("parallel node has no outgoing edges"));
        }

        let policies = ParallelPolicies {
            join: JoinPolicy::from_str_or_default(node.get_str_attr("join_policy")),
            error: ErrorPolicy::from_str_or_default(node.get_str_attr("error_policy")),
            max_parallel: node
                .get_str_attr("max_parallel")
                .and_then(|s| s.parse::<usize>().ok())
                .unwrap_or(DEFAULT_MAX_PARALLEL)
                .max(1),
        };

        self.emitter.emit(PipelineEvent::ParallelStarted {
            node_id: node.id.clone(),
        });

        let branch_results = self
            .run_branches(&node.id, &edges, context, graph, logs_root, &policies)
            .await;

        self.emitter.emit(PipelineEvent::ParallelCompleted {
            node_id: node.id.clone(),
        });

        // Store results in context for downstream fan-in.
        // When error_policy is Ignore, filter to successful results only
        // so downstream consumers don't see suppressed failures.
        let results_json: Vec<serde_json::Value> = branch_results
            .iter()
            .filter(|br| policies.error != ErrorPolicy::Ignore || br.outcome.status.is_success())
            .map(|br| {
                serde_json::json!({
                    "target": br.target,
                    "outcome": br.outcome.status.as_str(),
                    "notes": br.outcome.notes,
                })
            })
            .collect();
        context.set("parallel.results", serde_json::Value::Array(results_json));

        let outcome = evaluate_join(&branch_results, policies.join, policies.error);

        Ok(outcome)
    }
}

impl ParallelHandler {
    /// Execute all branches concurrently with policy-driven completion.
    ///
    /// Uses `FuturesUnordered` so results are processed as they arrive.
    /// A `Semaphore` limits concurrency to `max_parallel` (§4.8).
    /// `FirstSuccess` stops polling remaining branches once one succeeds.
    /// `FailFast` stops on the first failure. Remaining futures are
    /// dropped (cancelled) when the drain loop exits early.
    async fn run_branches(
        &self,
        parallel_node_id: &str,
        edges: &[&crate::graph::Edge],
        context: &Context,
        graph: &Graph,
        logs_root: &Path,
        policies: &ParallelPolicies,
    ) -> Vec<BranchResult> {
        let semaphore = Arc::new(Semaphore::new(policies.max_parallel));

        let futs: FuturesUnordered<_> = edges
            .iter()
            .enumerate()
            .map(|(idx, edge)| {
                let target_id = edge.to.clone();
                let parent_id = parallel_node_id.to_string();
                let branch_context = context.deep_clone();
                let registry = Arc::clone(&self.registry);
                let emitter = Arc::clone(&self.emitter);
                let graph = graph.clone();
                let logs_root = logs_root.to_path_buf();
                let sem = Arc::clone(&semaphore);

                async move {
                    // Acquire a semaphore permit to bound concurrency.
                    // OwnedSemaphoreError only occurs if the Semaphore is
                    // closed, which we never do; treat it as a branch failure.
                    let Ok(_permit) = sem.acquire_owned().await else {
                        return BranchResult {
                            target: target_id,
                            outcome: Outcome::fail("concurrency semaphore closed"),
                        };
                    };

                    emitter.emit(PipelineEvent::ParallelBranchStarted {
                        node_id: parent_id.clone(),
                        branch_index: idx,
                    });

                    let result = execute_branch_subgraph(
                        &target_id,
                        &branch_context,
                        &graph,
                        &logs_root,
                        &registry,
                    )
                    .await;

                    let branch_result = match result {
                        Ok(outcome) => BranchResult {
                            target: target_id,
                            outcome,
                        },
                        Err(e) => BranchResult {
                            target: target_id,
                            outcome: Outcome::fail(e.to_string()),
                        },
                    };

                    if branch_result.outcome.status == StageStatus::Fail {
                        emitter.emit(PipelineEvent::ParallelBranchFailed {
                            node_id: parent_id.clone(),
                            branch_index: idx,
                            reason: branch_result.outcome.failure_reason.clone(),
                        });
                    } else {
                        emitter.emit(PipelineEvent::ParallelBranchCompleted {
                            node_id: parent_id,
                            branch_index: idx,
                        });
                    }

                    branch_result
                }
            })
            .collect();

        drain_with_policy(futs, policies.join, policies.error).await
    }
}

/// Drain a `FuturesUnordered` stream of branch results, applying
/// early-exit policies. Dropping the stream cancels remaining futures.
async fn drain_with_policy(
    mut futs: FuturesUnordered<impl std::future::Future<Output = BranchResult>>,
    join_policy: JoinPolicy,
    error_policy: ErrorPolicy,
) -> Vec<BranchResult> {
    let mut results = Vec::new();

    while let Some(br) = futs.next().await {
        let should_stop = match (join_policy, error_policy) {
            (JoinPolicy::FirstSuccess, _) if br.outcome.status.is_success() => true,
            (_, ErrorPolicy::FailFast) if br.outcome.status == StageStatus::Fail => true,
            _ => false,
        };
        results.push(br);
        if should_stop {
            break;
        }
    }
    // Dropping `futs` here cancels any remaining branch futures.
    results
}

/// Evaluate the join policy and return the final outcome.
fn evaluate_join(
    results: &[BranchResult],
    join_policy: JoinPolicy,
    error_policy: ErrorPolicy,
) -> Outcome {
    let success_count = results
        .iter()
        .filter(|br| br.outcome.status.is_success())
        .count();
    let fail_count = results
        .iter()
        .filter(|br| br.outcome.status == StageStatus::Fail)
        .count();

    let mut outcome = match join_policy {
        JoinPolicy::WaitAll => {
            if fail_count > 0 && error_policy != ErrorPolicy::Ignore {
                // Any failures → PARTIAL_SUCCESS per §4.8 pseudocode.
                // Never hard-FAIL here: the downstream fan-in handler is
                // designed to handle the all-fail case and will return FAIL
                // itself when appropriate.  Returning FAIL from parallel
                // would trigger failure-routing in the engine, bypassing
                // the normal edge to fan-in entirely.
                let mut o = Outcome::success();
                o.status = StageStatus::PartialSuccess;
                o.notes = format!("{success_count} succeeded, {fail_count} failed");
                o
            } else {
                Outcome::success()
            }
        }
        JoinPolicy::FirstSuccess => {
            if success_count > 0 {
                Outcome::success()
            } else {
                Outcome::fail("no branch succeeded")
            }
        }
    };

    let mut updates = IndexMap::new();
    updates.insert(
        "parallel.success_count".into(),
        serde_json::Value::Number(serde_json::Number::from(success_count)),
    );
    updates.insert(
        "parallel.fail_count".into(),
        serde_json::Value::Number(serde_json::Number::from(fail_count)),
    );
    outcome.context_updates = updates;
    outcome
}

/// Execute a branch as a subgraph traversal (§4.8).
///
/// Walks forward from `start_id`, executing each node's handler and
/// applying context updates, until reaching a fan-in node, an exit
/// node, a node with no handler, or a dead end (no outgoing edges).
async fn execute_branch_subgraph(
    start_id: &str,
    context: &Context,
    graph: &Graph,
    logs_root: &Path,
    registry: &HandlerRegistry,
) -> AttractorResult<Outcome> {
    let mut current_id = start_id.to_string();
    let mut last_outcome = Outcome::success();

    loop {
        let node = graph.get_node(&current_id).ok_or_else(|| {
            crate::error::AttractorError::NodeNotFound {
                node_id: current_id.clone(),
            }
        })?;

        // Stop at fan-in nodes — they are handled by the parent engine
        if node.handler_type() == "parallel.fan_in" {
            break;
        }

        // Stop at exit nodes
        if Graph::is_exit_node(node) {
            break;
        }

        let Some(handler) = registry.resolve(node) else {
            last_outcome = Outcome::fail(format!(
                "no handler for type '{}' in branch",
                node.handler_type()
            ));
            break;
        };

        let policy = build_retry_policy(node, graph);
        let emitter = NoOpEmitter;
        last_outcome = execute_with_retry(
            &handler, node, context, graph, logs_root, &policy, &emitter, 0,
        )
        .await;

        // Apply context updates within the branch
        if !last_outcome.context_updates.is_empty() {
            context.apply_updates(&last_outcome.context_updates);
        }

        // If failed, stop the branch
        if last_outcome.status == StageStatus::Fail {
            break;
        }

        // Use the engine's edge-selection algorithm (§3.3) so that
        // conditions, preferred labels, and weights all apply within
        // branch subgraph traversal, matching the main loop semantics.
        if let Some(edge) = select_edge(&current_id, &last_outcome, context, graph) {
            current_id.clone_from(&edge.to);
        } else {
            break; // Dead end — no viable outgoing edge
        }
    }

    Ok(last_outcome)
}
