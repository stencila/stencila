//! Conditional handler (§4.7).

use async_trait::async_trait;

use crate::context::Context;
use crate::error::AttractorResult;
use crate::graph::{Graph, Node};
use crate::handler::Handler;
use crate::types::Outcome;

/// Handler for conditional (diamond) nodes.
///
/// Returns [`Outcome::success()`] with a note indicating that routing
/// is handled by the engine's edge selection algorithm. The conditional
/// handler itself performs no work — it exists so that diamond nodes
/// resolve to a handler without requiring a default fallback.
#[derive(Debug, Clone, Copy)]
pub struct ConditionalHandler;

#[async_trait]
impl Handler for ConditionalHandler {
    async fn execute(
        &self,
        node: &Node,
        _context: &Context,
        _graph: &Graph,
    ) -> AttractorResult<Outcome> {
        let mut outcome = Outcome::success();
        outcome.notes = format!(
            "Conditional node '{}': routing handled by edge selection",
            node.id
        );
        Ok(outcome)
    }
}
