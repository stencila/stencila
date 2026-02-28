//! Fail handler — explicit pipeline failure node.

use async_trait::async_trait;

use crate::context::Context;
use crate::error::AttractorResult;
use crate::graph::{Graph, Node};
use crate::handler::Handler;
use crate::types::Outcome;

/// Handler for the explicit fail node.
///
/// Always returns [`Outcome::fail()`]. The fail node lets pipelines
/// declare failure paths directly in the graph without workarounds.
#[derive(Debug, Clone, Copy)]
pub struct FailHandler;

#[async_trait]
impl Handler for FailHandler {
    async fn execute(
        &self,
        _node: &Node,
        _context: &Context,
        _graph: &Graph,
    ) -> AttractorResult<Outcome> {
        Ok(Outcome::fail("Pipeline failed (explicit fail node)"))
    }
}
