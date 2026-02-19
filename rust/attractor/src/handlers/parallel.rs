//! Parallel handler (§4.8).
//!
//! Fans out execution to multiple branches concurrently. Each branch
//! receives an isolated clone of the parent context and runs independently.
//! The handler uses `FuturesUnordered` for policy-driven completion:
//! `FirstSuccess` returns as soon as one branch succeeds; `FailFast`
//! cancels remaining branches on the first failure.

use std::collections::{HashSet, VecDeque};
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
use crate::types::{HandlerType, Outcome, StageStatus};

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
                .get_attr("max_parallel")
                .and_then(|v| match v {
                    crate::graph::AttrValue::Integer(n) => usize::try_from(*n).ok(),
                    crate::graph::AttrValue::String(s) => s.parse::<usize>().ok(),
                    _ => None,
                })
                .unwrap_or(DEFAULT_MAX_PARALLEL)
                .max(1),
        };

        // Compute the structural fan-in node before launching branches so
        // that (a) branches can stop when they reach it, and (b) the engine
        // can jump directly to it after the parallel handler returns.
        //
        // Use a Vec (preserving edge order from the graph) for deterministic
        // BFS traversal — HashSet iteration order varies across runs.
        let branch_entry_ids: Vec<String> = edges.iter().map(|e| e.to.clone()).collect();
        let fan_in_id = find_fan_in_node(graph, &node.id, &branch_entry_ids);

        self.emitter.emit(PipelineEvent::ParallelStarted {
            node_id: node.id.clone(),
        });

        let branch_results = self
            .run_branches(
                &node.id,
                &edges,
                context,
                graph,
                logs_root,
                &policies,
                fan_in_id.as_deref(),
            )
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
        context.set(
            format!("{}.results", HandlerType::Parallel),
            serde_json::Value::Array(results_json),
        );

        let mut outcome = evaluate_join(&branch_results, policies.join, policies.error);

        // Set the jump target so the engine advances directly to the
        // fan-in node, bypassing normal edge selection (which would
        // re-enter one of the already-executed branches).
        outcome.jump_target = fan_in_id;

        Ok(outcome)
    }
}

/// Find the structural fan-in (convergence) node for a set of parallel branches.
///
/// A fan-in node is the first node where parallel branches reconverge —
/// i.e. a node that is reachable from multiple distinct branch entry
/// nodes. This covers both:
///
/// - **Explicit fan-in nodes** (handler type `parallel.fan_in`) that may be
///   multiple hops downstream from branch entries.
/// - **Implicit convergence points** — ordinary nodes where branches happen
///   to merge (the "diamond" topology).
///
/// ## Algorithm
///
/// 1. For each branch entry node (iterated in the caller-provided slice
///    order), perform a BFS forward through the graph to collect the set
///    of all reachable node IDs.
///
/// 2. Among discovered nodes (excluding branch entries), select the
///    candidate reachable from the **most** branches. For ≥3 branches
///    this prevents selecting an early pairwise merge when a later
///    all-branch convergence exists (e.g. in a staggered merge topology
///    where A+B converge at `merge_ab` and then `merge_ab`+C converge
///    at `merge_abc`, the function selects `merge_abc`).
///
/// 3. If multiple candidates tie on branch count, BFS discovery order
///    (earliest encounter first) breaks the tie, preferring the
///    topologically closest convergence point.
///
/// A candidate must be reachable from at least 2 branches; single-branch
/// parallel nodes (or fully divergent branches) yield `None`.
///
/// The `parallel_node_id` is excluded from candidates to prevent cyclic
/// graphs (where branches have back-edges to the parallel node) from
/// selecting the parallel node itself as the fan-in, which would cause
/// an infinite re-execution loop.
///
/// ## Determinism
///
/// The caller must provide `branch_entry_ids` as an ordered slice (not a
/// `HashSet`) so that BFS traversal order — and therefore the global
/// discovery order — is determined by graph topology (edge ordering), not
/// by hash randomization. This ensures the same fan-in node is selected
/// across runs for the same graph.
///
/// ## Complexity
///
/// O(B × (N + E)) where B = number of branches, N = nodes, E = edges.
/// For typical pipeline graphs (small B, moderate N) this is negligible.
fn find_fan_in_node(
    graph: &Graph,
    parallel_node_id: &str,
    branch_entry_ids: &[String],
) -> Option<String> {
    let entry_set: HashSet<&str> = branch_entry_ids.iter().map(String::as_str).collect();
    let num_branches = branch_entry_ids.len();

    // Phase 1: BFS forward from each branch entry to collect reachable sets.
    // Also record a global discovery order so we can use it as a tiebreaker
    // (earlier in discovery = topologically closer to the parallel node).
    let mut reachable_per_branch: Vec<HashSet<String>> = Vec::new();
    let mut discovery_order: Vec<String> = Vec::new();
    let mut discovered: HashSet<String> = HashSet::new();

    for entry_id in branch_entry_ids {
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();
        queue.push_back(entry_id.clone());

        while let Some(current) = queue.pop_front() {
            if !visited.insert(current.clone()) {
                continue;
            }

            if discovered.insert(current.clone()) {
                discovery_order.push(current.clone());
            }

            for edge in graph.outgoing_edges(&current) {
                if !visited.contains(&edge.to) {
                    queue.push_back(edge.to.clone());
                }
            }
        }

        reachable_per_branch.push(visited);
    }

    // Phase 2: Select the candidate reachable from the most branches.
    //
    // Among nodes reachable from ≥2 branches, prefer the one with the
    // highest branch count. This ensures that in a staggered merge
    // topology (A+B → merge_ab → merge_abc ← C), we select merge_abc
    // (count=3) over merge_ab (count=2).
    //
    // On a tie (same branch count), BFS discovery order breaks it:
    // earlier in discovery = closer to the parallel node.
    let mut best: Option<(String, usize)> = None;

    for node_id in &discovery_order {
        if entry_set.contains(node_id.as_str()) || node_id == parallel_node_id {
            continue;
        }
        let branch_count = reachable_per_branch
            .iter()
            .filter(|set| set.contains(node_id))
            .count();
        if branch_count < 2 {
            continue;
        }

        // If this candidate is reachable from all branches, it's the
        // ideal convergence point — return immediately.
        if branch_count == num_branches {
            return Some(node_id.clone());
        }

        // Otherwise, track the best-so-far (highest branch count wins;
        // discovery order is the implicit tiebreak since we iterate in
        // that order and only upgrade on strictly greater count).
        if best
            .as_ref()
            .is_none_or(|(_, best_count)| branch_count > *best_count)
        {
            best = Some((node_id.clone(), branch_count));
        }
    }

    best.map(|(id, _)| id)
}

impl ParallelHandler {
    /// Execute all branches concurrently with policy-driven completion.
    ///
    /// Uses `FuturesUnordered` so results are processed as they arrive.
    /// A `Semaphore` limits concurrency to `max_parallel` (§4.8).
    /// `FirstSuccess` stops polling remaining branches once one succeeds.
    /// `FailFast` stops on the first failure. Remaining futures are
    /// dropped (cancelled) when the drain loop exits early.
    ///
    /// `fan_in_id` is the pre-computed structural fan-in node (if any).
    /// It is passed to each branch executor so that branches stop
    /// traversal when they reach the convergence point.
    #[allow(clippy::too_many_arguments)]
    async fn run_branches(
        &self,
        parallel_node_id: &str,
        edges: &[&crate::graph::Edge],
        context: &Context,
        graph: &Graph,
        logs_root: &Path,
        policies: &ParallelPolicies,
        fan_in_id: Option<&str>,
    ) -> Vec<BranchResult> {
        let semaphore = Arc::new(Semaphore::new(policies.max_parallel));
        let fan_in_id: Arc<Option<String>> = Arc::new(fan_in_id.map(String::from));

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
                let fan_in_id = Arc::clone(&fan_in_id);

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
                        fan_in_id.as_deref(),
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
        format!("{}.success_count", HandlerType::Parallel),
        serde_json::Value::Number(serde_json::Number::from(success_count)),
    );
    updates.insert(
        format!("{}.fail_count", HandlerType::Parallel),
        serde_json::Value::Number(serde_json::Number::from(fail_count)),
    );
    outcome.context_updates = updates;
    outcome
}

/// Execute a branch as a subgraph traversal (§4.8).
///
/// Walks forward from `start_id`, executing each node's handler and
/// applying context updates, until reaching a termination point:
///
/// - The **fan-in node** (convergence point computed by [`find_fan_in_node`]).
///   This is the single source of truth for fan-in detection — both explicit
///   `parallel.fan_in` handler nodes and implicit structural convergence
///   points are identified by the same BFS reachability analysis, so the
///   branch executor only needs a simple ID comparison.
/// - An **exit node** (graph exit/end).
/// - A **dead end** (no viable outgoing edge).
/// - A **handler failure** (node has no registered handler, or execution fails).
///
/// `fan_in_id` is `None` when no convergence point exists (e.g. branches
/// that terminate independently without reconverging).
async fn execute_branch_subgraph(
    start_id: &str,
    context: &Context,
    graph: &Graph,
    logs_root: &Path,
    registry: &HandlerRegistry,
    fan_in_id: Option<&str>,
) -> AttractorResult<Outcome> {
    let mut current_id = start_id.to_string();
    let mut last_outcome = Outcome::success();

    loop {
        let node = graph.get_node(&current_id).ok_or_else(|| {
            crate::error::AttractorError::NodeNotFound {
                node_id: current_id.clone(),
            }
        })?;

        // Stop at the pre-computed fan-in node. This covers both explicit
        // `parallel.fan_in` handler nodes and structural convergence points
        // — the BFS in `find_fan_in_node` discovers both.
        if fan_in_id == Some(current_id.as_str()) {
            break;
        }

        // Fallback: always stop at explicit fan-in nodes even when
        // `find_fan_in_node` returned None (e.g. single-branch parallel).
        // Without this, the branch would execute the FanInHandler before
        // `parallel.results` exists in context, causing a spurious failure.
        if node.handler_type() == HandlerType::ParallelFanIn {
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

        // Advance to the next node within the branch.
        //
        // This mirrors the engine's advance() logic so that nested
        // parallel nodes, jump targets, and edge selection all behave
        // consistently with the top-level engine loop.
        if let Some(target) = &last_outcome.jump_target {
            // The handler (e.g. a nested parallel) set an explicit jump
            // target. Honor it instead of following outgoing edges —
            // those edges are the nested handler's internal branches.
            current_id.clone_from(target);
        } else if node.handler_type() == HandlerType::Parallel {
            // A nested parallel handler with no jump target means its
            // branches were all terminal. Don't follow its outgoing
            // edges (which are already-executed branch entries).
            break;
        } else if let Some(edge) = select_edge(&current_id, &last_outcome, context, graph) {
            // Normal edge selection (§3.3) — conditions, preferred
            // labels, and weights all apply within branch traversal.
            current_id.clone_from(&edge.to);
        } else {
            break; // Dead end — no viable outgoing edge
        }
    }

    Ok(last_outcome)
}
