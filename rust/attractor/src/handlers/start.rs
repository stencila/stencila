//! Start handler (§4.3).

use async_trait::async_trait;

use crate::context::Context;
use crate::error::AttractorResult;
use crate::graph::{Graph, Node};
use crate::handler::Handler;
use crate::types::Outcome;

/// Handler for the pipeline start node.
///
/// Always returns [`Outcome::success()`]. The start node serves as
/// the entry point; no actual work is performed.
#[derive(Debug, Clone, Copy)]
pub struct StartHandler;

#[async_trait]
impl Handler for StartHandler {
    async fn execute(
        &self,
        _node: &Node,
        _context: &Context,
        _graph: &Graph,
    ) -> AttractorResult<Outcome> {
        Ok(Outcome::success())
    }
}
