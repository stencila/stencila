//! Fidelity resolution (§5.4).
//!
//! Determines the context fidelity mode for a node based on the
//! 4-level precedence chain: edge → node → graph → default (`compact`).

use crate::graph::{Edge, Graph, Node};
use crate::types::FidelityMode;

/// Resolve the fidelity mode for a node, checking (§5.4):
///
/// 1. Incoming edge `fidelity` attribute
/// 2. Target node `fidelity` attribute
/// 3. Graph-level `default_fidelity` attribute
/// 4. Default: `compact`
#[must_use]
pub fn resolve_fidelity(node: &Node, incoming_edge: Option<&Edge>, graph: &Graph) -> FidelityMode {
    // 1. Edge-level fidelity
    if let Some(edge) = incoming_edge
        && let Some(mode) = parse_fidelity_attr(edge.get_attr("fidelity").and_then(|v| v.as_str()))
    {
        return mode;
    }

    // 2. Node-level fidelity
    if let Some(mode) = parse_fidelity_attr(node.get_str_attr("fidelity")) {
        return mode;
    }

    // 3. Graph-level default_fidelity
    if let Some(mode) = parse_fidelity_attr(
        graph
            .get_graph_attr("default_fidelity")
            .and_then(|v| v.as_str()),
    ) {
        return mode;
    }

    // 4. Default
    FidelityMode::default()
}

/// Resolve the thread ID for `full` fidelity mode (§5.4).
///
/// 5-step priority chain:
/// 1. Target node `thread_id` attribute
/// 2. Incoming edge `thread_id` attribute
/// 3. Graph-level default thread
/// 4. Derived class from node's `class` attribute
/// 5. Fallback: previous node ID
#[must_use]
pub fn resolve_thread_id(
    node: &Node,
    incoming_edge: Option<&Edge>,
    graph: &Graph,
    previous_node_id: &str,
) -> String {
    // 1. Node thread_id
    if let Some(tid) = node.get_str_attr("thread_id") {
        return tid.to_string();
    }

    // 2. Edge thread_id
    if let Some(edge) = incoming_edge
        && let Some(tid) = edge.get_attr("thread_id").and_then(|v| v.as_str())
    {
        return tid.to_string();
    }

    // 3. Graph-level default thread
    if let Some(tid) = graph
        .get_graph_attr("default_thread_id")
        .and_then(|v| v.as_str())
    {
        return tid.to_string();
    }

    // 4. Node class
    if let Some(class) = node.get_str_attr("class") {
        // Use first class as thread key
        if let Some(first_class) = class.split(',').next().map(str::trim)
            && !first_class.is_empty()
        {
            return first_class.to_string();
        }
    }

    // 5. Previous node ID
    previous_node_id.to_string()
}

/// Parse a fidelity mode from an optional string attribute.
fn parse_fidelity_attr(attr: Option<&str>) -> Option<FidelityMode> {
    attr.and_then(|s| s.parse::<FidelityMode>().ok())
}
