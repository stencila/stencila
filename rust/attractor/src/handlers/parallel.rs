//! Parallel handler (§4.8).
//!
//! Fans out execution to multiple branches concurrently. Each branch
//! receives an isolated clone of the parent context and runs independently.
//! The handler uses `FuturesUnordered` for policy-driven completion:
//! `FirstSuccess` returns as soon as one branch succeeds; `FailFast`
//! cancels remaining branches on the first failure.

use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

use async_trait::async_trait;
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use indexmap::IndexMap;
use inflector::Inflector;

use tokio::sync::Semaphore;

use crate::context::{Context, ctx};
use crate::edge_selection::select_edge;
use crate::error::AttractorResult;
use crate::events::{EventEmitter, NoOpEmitter, PipelineEvent};
use crate::graph::{AttrValue, Graph, Node, attr};
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

/// Resolve join, error, and max-parallel policies from node attributes.
fn resolve_policies(node: &Node) -> ParallelPolicies {
    ParallelPolicies {
        join: JoinPolicy::from_str_or_default(node.get_str_attr("join_policy")),
        error: ErrorPolicy::from_str_or_default(node.get_str_attr("error_policy")),
        max_parallel: node
            .get_attr("max_parallel")
            .and_then(|v| match v {
                AttrValue::Integer(n) => usize::try_from(*n).ok(),
                AttrValue::String(s) => s.parse::<usize>().ok(),
                _ => None,
            })
            .unwrap_or(DEFAULT_MAX_PARALLEL)
            .max(1),
    }
}

/// Result of a single parallel branch execution.
#[derive(Debug, Clone)]
pub struct BranchResult {
    /// Target node ID of this branch.
    pub target: String,
    /// The outcome of executing this branch.
    pub outcome: Outcome,
    /// The `last_output_full` value from the branch context after execution,
    /// if available. Used to populate `parallel.outputs`.
    pub output: Option<String>,
    /// The original submission index of this branch. For dynamic fan-out,
    /// this is the zero-based index into the source list. For static
    /// fan-out, this is the edge index. Used to restore submission order
    /// after `FuturesUnordered` yields results in completion order.
    pub index: usize,
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
    ) -> AttractorResult<Outcome> {
        // Check for dynamic fan-out attribute
        if node.attrs.contains_key(attr::FAN_OUT) {
            return self.execute_dynamic(node, context, graph).await;
        }

        let edges = graph.outgoing_edges(&node.id);
        if edges.is_empty() {
            return Ok(Outcome::fail("parallel node has no outgoing edges"));
        }

        let policies = resolve_policies(node);

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
            dynamic_item_count: None,
        });

        let mut branch_results = self
            .run_branches(
                &node.id,
                &edges,
                context,
                graph,
                &policies,
                fan_in_id.as_deref(),
            )
            .await;

        // Sort by submission index for deterministic ordering
        branch_results.sort_by_key(|br| br.index);

        self.emitter.emit(PipelineEvent::ParallelCompleted {
            node_id: node.id.clone(),
        });

        // Build parallel.outputs from all branch results (before error filtering)
        let outputs_json: Vec<serde_json::Value> = branch_results
            .iter()
            .map(|br| {
                br.output.as_ref().map_or(serde_json::Value::Null, |s| {
                    serde_json::Value::String(s.clone())
                })
            })
            .collect();
        context.set(
            ctx::PARALLEL_OUTPUTS,
            serde_json::Value::Array(outputs_json),
        );

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
            ctx::PARALLEL_RESULTS,
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

/// Find the fan-in node for a dynamic fan-out using forward BFS from the
/// template entry node.
///
/// Unlike [`find_fan_in_node`] which requires multi-branch convergence,
/// this function searches for the first explicit `parallel.fan_in`
/// (`tripleoctagon`) node reachable from `template_entry_id`. Dynamic
/// fan-out has only one outgoing edge, so structural convergence detection
/// (reachable from ≥2 branches) does not apply.
fn find_dynamic_fan_in_node(graph: &Graph, template_entry_id: &str) -> Option<String> {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back(template_entry_id.to_string());

    while let Some(current) = queue.pop_front() {
        if !visited.insert(current.clone()) {
            continue;
        }

        if let Some(node) = graph.get_node(&current)
            && node.handler_type() == HandlerType::ParallelFanIn
        {
            return Some(current);
        }

        for edge in graph.outgoing_edges(&current) {
            if !visited.contains(&edge.to) {
                queue.push_back(edge.to.clone());
            }
        }
    }

    None
}

/// Find the first successor of a fan-in node by examining its outgoing edges.
///
/// Used by the empty-list bypass to skip the fan-in node and jump directly
/// to its successor.
fn find_fan_in_successor(graph: &Graph, fan_in_id: &str) -> Option<String> {
    graph
        .outgoing_edges(fan_in_id)
        .first()
        .map(|e| e.to.clone())
}

/// Return a human-readable type name for a JSON value.
fn json_type_name(value: &serde_json::Value) -> &'static str {
    match value {
        serde_json::Value::Null => "null",
        serde_json::Value::Bool(_) => "boolean",
        serde_json::Value::Number(_) => "number",
        serde_json::Value::String(_) => "string",
        serde_json::Value::Array(_) => "array",
        serde_json::Value::Object(_) => "object",
    }
}

impl ParallelHandler {
    /// Execute a dynamic fan-out over a runtime-determined list.
    #[allow(clippy::too_many_lines)]
    async fn execute_dynamic(
        &self,
        node: &Node,
        context: &Context,
        graph: &Graph,
    ) -> AttractorResult<Outcome> {
        let edges = graph.outgoing_edges(&node.id);
        if edges.is_empty() {
            return Ok(Outcome::fail(
                "dynamic fan-out node has no outgoing edges; exactly 1 is required",
            ));
        }
        if edges.len() > 1 {
            return Ok(Outcome::fail(format!(
                "dynamic fan-out node `{}` has {} outgoing edges; exactly 1 is required",
                node.id,
                edges.len()
            )));
        }

        // Resolve the context key from the fan_out attribute
        let context_key = match node.get_attr(attr::FAN_OUT) {
            Some(AttrValue::Boolean(false)) => {
                return Ok(Outcome::fail(format!(
                    "dynamic fan-out on node '{}': fan_out=false is not valid; use fan_out=true or fan_out=\"key\"",
                    node.id
                )));
            }
            Some(AttrValue::Boolean(true)) => {
                // Derive context key from node ID in snake_case
                node.id.to_snake_case()
            }
            Some(AttrValue::String(s)) if s == "true" => {
                // fan_out="true" (quoted) treated same as fan_out=true (unquoted)
                node.id.to_snake_case()
            }
            Some(AttrValue::String(key)) => key.clone(),
            Some(other) => {
                return Ok(Outcome::fail(format!(
                    "dynamic fan-out on node '{}': fan_out attribute has unsupported type '{}'",
                    node.id,
                    other.type_name()
                )));
            }
            None => {
                return Ok(Outcome::fail(format!(
                    "dynamic fan-out on node '{}': fan_out attribute is missing",
                    node.id
                )));
            }
        };

        // Resolve the list from context
        let items = match context.get(&context_key) {
            Some(serde_json::Value::Array(arr)) => arr,
            Some(other) => {
                return Ok(Outcome::fail(format!(
                    "dynamic fan-out on node '{}': context key '{}' resolved to {}, expected Array",
                    node.id,
                    context_key,
                    json_type_name(&other)
                )));
            }
            None => {
                return Ok(Outcome::fail(format!(
                    "dynamic fan-out on node '{}': context key '{}' not found in context",
                    node.id, context_key
                )));
            }
        };

        let template_entry_id = edges[0].to.clone();
        let fan_in_id = find_dynamic_fan_in_node(graph, &template_entry_id);
        let policies = resolve_policies(node);

        // Empty-list bypass: skip the fan-in node entirely
        if items.is_empty() {
            context.set(ctx::PARALLEL_RESULTS, serde_json::Value::Array(vec![]));
            context.set(ctx::PARALLEL_OUTPUTS, serde_json::Value::Array(vec![]));

            let mut outcome = Outcome::success();
            outcome.notes = "dynamic fan-out: 0 items, nothing to execute".into();

            // Jump past the fan-in node to its successor
            outcome.jump_target = if let Some(ref fi_id) = fan_in_id {
                find_fan_in_successor(graph, fi_id).or(fan_in_id.clone())
            } else {
                None
            };

            // Write success/fail counts
            let mut updates = IndexMap::new();
            updates.insert(
                ctx::PARALLEL_SUCCESS_COUNT.into(),
                serde_json::Value::Number(0.into()),
            );
            updates.insert(
                ctx::PARALLEL_FAIL_COUNT.into(),
                serde_json::Value::Number(0.into()),
            );
            outcome.context_updates = updates;

            return Ok(outcome);
        }

        let item_count = items.len();

        self.emitter.emit(PipelineEvent::ParallelStarted {
            node_id: node.id.clone(),
            dynamic_item_count: Some(item_count),
        });

        let mut branch_results = self
            .run_dynamic_branches(
                &node.id,
                &template_entry_id,
                &items,
                &context_key,
                context,
                graph,
                &policies,
                fan_in_id.as_deref(),
            )
            .await;

        // Sort by submission index for deterministic ordering
        branch_results.sort_by_key(|br| br.index);

        self.emitter.emit(PipelineEvent::ParallelCompleted {
            node_id: node.id.clone(),
        });

        // Build parallel.outputs indexed by original fan-out position.
        // Initialize with null for every source item so that even when
        // branches are missing (e.g. fail_fast early exit) the array stays
        // aligned to the source list: outputs[i] corresponds to item i.
        let mut outputs_json: Vec<serde_json::Value> = vec![serde_json::Value::Null; item_count];
        for br in &branch_results {
            if br.index < outputs_json.len() {
                outputs_json[br.index] = br.output.as_ref().map_or(serde_json::Value::Null, |s| {
                    serde_json::Value::String(s.clone())
                });
            }
        }
        context.set(
            ctx::PARALLEL_OUTPUTS,
            serde_json::Value::Array(outputs_json),
        );

        // Store results in context with dynamic fan-out metadata
        let results_json: Vec<serde_json::Value> = branch_results
            .iter()
            .filter(|br| policies.error != ErrorPolicy::Ignore || br.outcome.status.is_success())
            .map(|br| {
                serde_json::json!({
                    "target": br.target,
                    "outcome": br.outcome.status.as_str(),
                    "notes": br.outcome.notes,
                    "fan_out_index": br.index,
                    "fan_out_item": items.get(br.index).unwrap_or(&serde_json::Value::Null),
                })
            })
            .collect();
        context.set(
            ctx::PARALLEL_RESULTS,
            serde_json::Value::Array(results_json),
        );

        let mut outcome = evaluate_join(&branch_results, policies.join, policies.error);
        outcome.jump_target = fan_in_id;

        Ok(outcome)
    }

    /// Spawn dynamic branches over a list of items, each starting at
    /// `template_entry_id` with per-item context injection.
    #[allow(clippy::too_many_arguments)]
    async fn run_dynamic_branches(
        &self,
        parallel_node_id: &str,
        template_entry_id: &str,
        items: &[serde_json::Value],
        context_key: &str,
        context: &Context,
        graph: &Graph,
        policies: &ParallelPolicies,
        fan_in_id: Option<&str>,
    ) -> Vec<BranchResult> {
        let semaphore = Arc::new(Semaphore::new(policies.max_parallel));
        let fan_in_id: Arc<Option<String>> = Arc::new(fan_in_id.map(String::from));
        let total = items.len();

        let futs: FuturesUnordered<_> = items
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                let target_id = template_entry_id.to_string();
                let parent_id = parallel_node_id.to_string();
                let branch_context = context.deep_clone();

                // Inject per-item context keys
                branch_context.set(ctx::FAN_OUT_ITEM, item.clone());
                branch_context.set(
                    ctx::FAN_OUT_INDEX,
                    serde_json::Value::Number(serde_json::Number::from(idx)),
                );
                branch_context.set(
                    ctx::FAN_OUT_TOTAL,
                    serde_json::Value::Number(serde_json::Number::from(total)),
                );
                branch_context.set(
                    ctx::FAN_OUT_KEY,
                    serde_json::Value::String(context_key.to_string()),
                );

                // Object property flattening: top-level properties of object items
                if let serde_json::Value::Object(map) = item {
                    for (prop, val) in map {
                        branch_context
                            .set(format!("{}{prop}", ctx::FAN_OUT_ITEM_PREFIX), val.clone());
                    }
                }

                let registry = Arc::clone(&self.registry);
                let emitter = Arc::clone(&self.emitter);
                let graph = graph.clone();
                let sem = Arc::clone(&semaphore);
                let fan_in_id = Arc::clone(&fan_in_id);

                async move {
                    let Ok(_permit) = sem.acquire_owned().await else {
                        return BranchResult {
                            target: target_id,
                            outcome: Outcome::fail("concurrency semaphore closed"),
                            output: None,
                            index: idx,
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
                        &registry,
                        fan_in_id.as_deref(),
                    )
                    .await;

                    let output = {
                        let s = branch_context.get_string(ctx::LAST_OUTPUT_FULL);
                        if s.is_empty() { None } else { Some(s) }
                    };

                    let branch_result = match result {
                        Ok(outcome) => BranchResult {
                            target: target_id,
                            outcome,
                            output,
                            index: idx,
                        },
                        Err(e) => BranchResult {
                            target: target_id,
                            outcome: Outcome::fail(e.to_string()),
                            output,
                            index: idx,
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
    async fn run_branches(
        &self,
        parallel_node_id: &str,
        edges: &[&crate::graph::Edge],
        context: &Context,
        graph: &Graph,
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
                let sem = Arc::clone(&semaphore);
                let fan_in_id = Arc::clone(&fan_in_id);

                async move {
                    let Ok(_permit) = sem.acquire_owned().await else {
                        return BranchResult {
                            target: target_id,
                            outcome: Outcome::fail("concurrency semaphore closed"),
                            output: None,
                            index: idx,
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
                        &registry,
                        fan_in_id.as_deref(),
                    )
                    .await;

                    let output = {
                        let s = branch_context.get_string(ctx::LAST_OUTPUT_FULL);
                        if s.is_empty() { None } else { Some(s) }
                    };

                    let branch_result = match result {
                        Ok(outcome) => BranchResult {
                            target: target_id,
                            outcome,
                            output,
                            index: idx,
                        },
                        Err(e) => BranchResult {
                            target: target_id,
                            outcome: Outcome::fail(e.to_string()),
                            output,
                            index: idx,
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
        ctx::PARALLEL_SUCCESS_COUNT.into(),
        serde_json::Value::Number(serde_json::Number::from(success_count)),
    );
    updates.insert(
        ctx::PARALLEL_FAIL_COUNT.into(),
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
        last_outcome =
            execute_with_retry(&handler, node, context, graph, &policy, &emitter, 0).await;

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
