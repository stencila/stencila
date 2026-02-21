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
/// - Lowercase
/// - Trim whitespace
/// - Strip accelerator prefixes: `[Y] `, `Y) `, `Y - `
#[must_use]
pub fn normalize_label(label: &str) -> String {
    let trimmed = label.trim().to_lowercase();

    // Strip accelerator prefix patterns:
    // [X] prefix  (e.g., "[Y] Yes")
    // X) prefix   (e.g., "Y) Yes")
    // X - prefix  (e.g., "Y - Yes")
    if let Some(rest) = trimmed
        .strip_prefix('[')
        .and_then(|s| s.get(1..))
        .and_then(|s| s.strip_prefix("] "))
    {
        return rest.to_string();
    }

    if trimmed.len() >= 3 {
        let bytes = trimmed.as_bytes();
        if bytes.get(1) == Some(&b')') && bytes.get(2) == Some(&b' ') {
            return trimmed[3..].to_string();
        }
        if trimmed.len() >= 4 && bytes.get(1..4) == Some(b" - ".as_slice()) {
            return trimmed[4..].to_string();
        }
    }

    trimmed
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
