//! Fan-in handler (§4.9).
//!
//! Consolidates results from a preceding parallel node and selects
//! the best candidate using heuristic ranking.

use async_trait::async_trait;
use indexmap::IndexMap;

use crate::context::Context;
use crate::error::AttractorResult;
use crate::graph::{Graph, Node};
use crate::handler::Handler;
use crate::types::{HandlerType, Outcome, StageStatus};

/// Handler for fan-in (result consolidation) nodes.
///
/// Reads `parallel.results` from context and selects the best
/// candidate by heuristic ranking: SUCCESS > `PARTIAL_SUCCESS` > RETRY > FAIL.
#[derive(Debug, Clone, Copy, Default)]
pub struct FanInHandler;

/// Rank a status string for heuristic selection (lower is better).
fn status_rank(status: &str) -> u32 {
    match status {
        "success" => 0,
        "partial_success" => 1,
        "retry" => 2,
        "fail" => 3,
        "skipped" => 4,
        _ => 5,
    }
}

#[async_trait]
impl Handler for FanInHandler {
    async fn execute(
        &self,
        _node: &Node,
        context: &Context,
        _graph: &Graph,
    ) -> AttractorResult<Outcome> {
        // Read parallel results from context
        let results = context.get(&format!("{}.results", HandlerType::Parallel));
        let results_array = results.as_ref().and_then(serde_json::Value::as_array);

        let Some(candidates) = results_array else {
            return Ok(Outcome::fail("No parallel results to evaluate"));
        };

        if candidates.is_empty() {
            return Ok(Outcome::fail("No parallel results to evaluate"));
        }

        // Heuristic selection per §4.9: sort by (outcome_rank, -score, id).
        // Lower outcome_rank is better; higher score is better; lower id
        // is the deterministic tiebreak for equal rank and score.
        let best = candidates.iter().min_by(|a, b| {
            let rank_a = status_rank(
                a.get("outcome")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("fail"),
            );
            let rank_b = status_rank(
                b.get("outcome")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("fail"),
            );

            let score_a = a
                .get("score")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0);
            let score_b = b
                .get("score")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0);

            let id_a = a
                .get("target")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");
            let id_b = b
                .get("target")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("");

            rank_a
                .cmp(&rank_b)
                .then_with(|| score_b.total_cmp(&score_a)) // higher score wins
                .then_with(|| id_a.cmp(id_b)) // lower id wins
        });

        let Some(best) = best else {
            return Ok(Outcome::fail("No candidates found"));
        };

        let best_target = best
            .get("target")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown");
        let best_status = best
            .get("outcome")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("fail");

        // If all candidates failed, return FAIL
        let all_failed = candidates.iter().all(|c| {
            c.get("outcome")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("fail")
                == "fail"
        });

        if all_failed {
            return Ok(Outcome::fail("all parallel candidates failed"));
        }

        let mut updates = IndexMap::new();
        updates.insert(
            format!("{}.best_id", HandlerType::ParallelFanIn),
            serde_json::Value::String(best_target.to_string()),
        );
        updates.insert(
            format!("{}.best_outcome", HandlerType::ParallelFanIn),
            serde_json::Value::String(best_status.to_string()),
        );

        let mut outcome = Outcome::success();
        outcome.notes = format!("Selected best candidate: {best_target}");
        outcome.context_updates = updates;

        // If best is not success, return partial
        if best_status != "success" {
            outcome.status = StageStatus::PartialSuccess;
        }

        Ok(outcome)
    }
}
