//! Goal gate enforcement and failure routing (§3.4, §3.7).

use indexmap::IndexMap;

use crate::graph::Graph;
use crate::types::{Outcome, StageStatus};

/// Result of checking goal gates before pipeline exit.
#[derive(Debug, Clone, PartialEq)]
pub struct GoalGateResult {
    /// Whether all goal gates are satisfied.
    pub satisfied: bool,
    /// The node ID of the first unsatisfied goal gate (if any).
    pub failed_node_id: Option<String>,
}

/// Check whether all visited goal-gate nodes have been satisfied.
///
/// Per §3.4, only visited nodes (those in `node_outcomes`) are checked.
/// A node is a goal gate if it has `goal_gate=true` in its attributes.
/// A goal gate is satisfied if its outcome status is a success
/// (i.e., `Success` or `PartialSuccess`).
///
/// Returns a [`GoalGateResult`] indicating whether all gates passed
/// and, if not, which node failed first (by execution order).
#[must_use]
pub fn check_goal_gates(
    graph: &Graph,
    node_outcomes: &IndexMap<String, Outcome>,
) -> GoalGateResult {
    for (node_id, outcome) in node_outcomes {
        let Some(node) = graph.get_node(node_id) else {
            continue;
        };

        let is_gate = node
            .get_attr("goal_gate")
            .and_then(super::super::graph::AttrValue::as_bool)
            .unwrap_or(false);

        if !is_gate {
            continue;
        }

        if !outcome.status.is_success() {
            return GoalGateResult {
                satisfied: false,
                failed_node_id: Some(node_id.clone()),
            };
        }
    }

    GoalGateResult {
        satisfied: true,
        failed_node_id: None,
    }
}

/// Get the retry target for a failed node using the 4-level chain (§3.7).
///
/// Resolution order:
/// 1. Node attribute `retry_target`
/// 2. Node attribute `fallback_retry_target`
/// 3. Graph attribute `retry_target`
/// 4. Graph attribute `fallback_retry_target`
///
/// Each candidate is validated against the graph — if a target does not
/// exist as a node, the chain continues to the next level.
#[must_use]
pub fn get_retry_target(node: &crate::graph::Node, graph: &Graph) -> Option<String> {
    let candidates: [Option<&str>; 4] = [
        node.get_str_attr("retry_target"),
        node.get_str_attr("fallback_retry_target"),
        graph
            .get_graph_attr("retry_target")
            .and_then(|v| v.as_str()),
        graph
            .get_graph_attr("fallback_retry_target")
            .and_then(|v| v.as_str()),
    ];

    candidates
        .into_iter()
        .flatten()
        .find(|target| graph.get_node(target).is_some())
        .map(String::from)
}

/// Find a "fail" edge from the given node — an outgoing edge whose
/// condition evaluates to `true` given a fail outcome (§3.7).
///
/// Only edges with a non-empty condition are considered. The condition
/// is evaluated with the outcome's status forced to `Fail` and the
/// actual pipeline context.
#[must_use]
pub fn find_fail_edge<'a>(
    node_id: &str,
    graph: &'a Graph,
    outcome: &Outcome,
    context: &crate::context::Context,
) -> Option<&'a crate::graph::Edge> {
    let fail_outcome = Outcome {
        status: StageStatus::Fail,
        ..outcome.clone()
    };

    let edges = graph.outgoing_edges(node_id);
    edges.into_iter().find(|e| {
        let cond = e.condition();
        if cond.is_empty() {
            return false;
        }
        crate::condition::evaluate_condition(cond, &fail_outcome, context)
    })
}
