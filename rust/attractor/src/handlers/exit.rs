//! Exit handler (§4.4).

use std::path::Path;

use async_trait::async_trait;

use crate::context::Context;
use crate::error::AttractorResult;
use crate::graph::{Graph, Node};
use crate::handler::Handler;
use crate::types::Outcome;

/// Handler for the pipeline exit node.
///
/// Always returns [`Outcome::success()`]. The exit node marks pipeline
/// completion; no actual work is performed.
#[derive(Debug, Clone, Copy)]
pub struct ExitHandler;

#[async_trait]
impl Handler for ExitHandler {
    async fn execute(
        &self,
        _node: &Node,
        _context: &Context,
        _graph: &Graph,
        _logs_root: &Path,
    ) -> AttractorResult<Outcome> {
        Ok(Outcome::success())
    }
}
