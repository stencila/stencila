//! Edge selection algorithm (§3.3).
//!
//! Implements the 5-step priority algorithm for choosing the next edge
//! during pipeline traversal. Steps are evaluated in order; the first
//! step that yields a candidate wins:
//!
//! 1. Condition-matching edges → best by weight, then lexical target ID
//! 2. Preferred label match (normalized)
//! 3. Suggested next IDs (first match in order)
//! 4. Highest weight among unconditional edges
//! 5. Lexical tiebreak among remaining candidates

use crate::condition::evaluate_condition;
use crate::context::Context;
use crate::graph::{Edge, Graph};
use crate::handlers::parse_accelerator_label;
use crate::types::Outcome;

/// Select the best outgoing edge from `node_id` given the current
/// outcome and context, using the 5-step priority algorithm (§3.3).
///
/// Returns `None` if no outgoing edges exist or none match.
#[must_use]
pub fn select_edge<'a>(
    node_id: &str,
    outcome: &Outcome,
    context: &Context,
    graph: &'a Graph,
) -> Option<&'a Edge> {
    let edges = graph.outgoing_edges(node_id);
    if edges.is_empty() {
        return None;
    }

    // Step 1: Condition-matching edges
    let condition_matches: Vec<&Edge> = edges
        .iter()
        .filter(|e| {
            let cond = e.condition();
            !cond.is_empty() && evaluate_condition(cond, outcome, context)
        })
        .copied()
        .collect();
    if let Some(edge) = best_by_weight_then_lexical(&condition_matches) {
        return Some(edge);
    }

    // Step 2: Preferred label match (searches all edges per spec)
    if !outcome.preferred_label.is_empty() {
        let normalized_pref = normalize_label(&outcome.preferred_label);
        let label_match = edges
            .iter()
            .find(|e| normalize_label(e.label()) == normalized_pref);
        if let Some(edge) = label_match {
            return Some(edge);
        }
    }

    // Step 3: Suggested next IDs (searches all edges per spec)
    for suggested in &outcome.suggested_next_ids {
        if let Some(edge) = edges.iter().find(|e| e.to == *suggested) {
            return Some(edge);
        }
    }

    // Step 4 & 5: Highest weight, then lexical tiebreak (unconditional only)
    let unconditional: Vec<&Edge> = edges
        .iter()
        .filter(|e| e.condition().is_empty())
        .copied()
        .collect();
    if let Some(edge) = best_by_weight_then_lexical(&unconditional) {
        return Some(edge);
    }

    // Fallback: any edge (including conditional) by weight then lexical
    best_by_weight_then_lexical(&edges)
}

/// Normalize an edge label for comparison:
/// - Strip accelerator prefixes via [`parse_accelerator_label`]
/// - Lowercase
/// - Trim whitespace
#[must_use]
pub fn normalize_label(label: &str) -> String {
    let (_key, display) = parse_accelerator_label(label);
    display.to_lowercase()
}

/// Pick the best edge from a set by descending weight, then ascending
/// target ID (lexicographic) as tiebreak.
///
/// Returns `None` if the slice is empty.
#[must_use]
pub fn best_by_weight_then_lexical<'a>(edges: &[&'a Edge]) -> Option<&'a Edge> {
    if edges.is_empty() {
        return None;
    }

    let mut best = edges[0];
    for &edge in &edges[1..] {
        if edge.weight() > best.weight() || (edge.weight() == best.weight() && edge.to < best.to) {
            best = edge;
        }
    }
    Some(best)
}
