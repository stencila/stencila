//! Codergen handler (§4.5).
//!
//! Executes LLM-backed code generation tasks. Supports a pluggable
//! backend trait for the actual LLM call, with a built-in simulation
//! mode for testing.

use std::path::Path;
use std::sync::Arc;

use async_trait::async_trait;
use indexmap::IndexMap;

use crate::context::Context;
use crate::error::AttractorResult;
use crate::graph::{Graph, Node};
use crate::handler::Handler;
use crate::run_directory::RunDirectory;
use crate::types::Outcome;

/// The response type returned by a codergen backend.
pub enum CodergenResponse {
    /// A plain text response from the LLM.
    Text(String),
    /// A fully constructed outcome (bypasses default outcome building).
    FullOutcome(Outcome),
}

/// Trait for LLM backends that power the codergen handler.
#[async_trait]
pub trait CodergenBackend: Send + Sync {
    /// Run the LLM call for the given node and prompt.
    async fn run(
        &self,
        node: &Node,
        prompt: &str,
        context: &Context,
    ) -> AttractorResult<CodergenResponse>;
}

/// Handler for codergen (LLM code generation) nodes.
///
/// When no backend is configured, operates in simulation mode and
/// returns a placeholder response. When a backend is provided, it
/// delegates the LLM call and writes logs to the run directory.
///
/// **Note:** Variable expansion (e.g., `$goal` in prompts) is handled
/// by the [`TransformRegistry`](crate::transform::TransformRegistry)
/// as a graph-level preprocessing step before the engine loop begins
/// (§9.1). When using this handler directly outside of
/// [`engine::run`](crate::engine::run), callers must apply transforms
/// to the graph first.
pub struct CodergenHandler {
    backend: Option<Arc<dyn CodergenBackend>>,
}

impl CodergenHandler {
    /// Create a handler in simulation mode (no LLM backend).
    #[must_use]
    pub fn simulation() -> Self {
        Self { backend: None }
    }

    /// Create a handler with the given LLM backend.
    #[must_use]
    pub fn with_backend(backend: Arc<dyn CodergenBackend>) -> Self {
        Self {
            backend: Some(backend),
        }
    }
}

/// Maximum length for the truncated response stored in context updates.
const RESPONSE_TRUNCATION_LIMIT: usize = 200;

/// Truncate a string to the limit, appending `...` if truncated.
///
/// Finds the last char boundary at or before the limit to avoid
/// panicking on multi-byte UTF-8.
fn truncate_response(s: &str) -> String {
    if s.len() <= RESPONSE_TRUNCATION_LIMIT {
        s.to_string()
    } else {
        // Find the last char boundary at or before the byte limit.
        let boundary = s
            .char_indices()
            .map(|(i, _)| i)
            .take_while(|&i| i <= RESPONSE_TRUNCATION_LIMIT)
            .last()
            .unwrap_or(0);
        let mut truncated = s[..boundary].to_string();
        truncated.push_str("...");
        truncated
    }
}

#[async_trait]
impl Handler for CodergenHandler {
    async fn execute(
        &self,
        node: &Node,
        context: &Context,
        _graph: &Graph,
        logs_root: &Path,
    ) -> AttractorResult<Outcome> {
        // Build prompt: prefer explicit "prompt" attr, fall back to node label
        let prompt = node.get_str_attr("prompt").unwrap_or_else(|| node.label());

        // Use RunDirectory for consistent path layout and status writing.
        let run_dir = RunDirectory::open(logs_root);
        let stage_dir = run_dir.node_dir(&node.id);
        std::fs::create_dir_all(&stage_dir)?;

        // Write the prompt
        std::fs::write(stage_dir.join("prompt.md"), prompt)?;

        // Run the backend (or simulate)
        let response = match &self.backend {
            None => {
                // Simulation mode
                CodergenResponse::Text(format!("[Simulated] Response for stage: {}", node.id))
            }
            Some(backend) => match backend.run(node, prompt, context).await {
                Ok(resp) => resp,
                Err(e) => {
                    let outcome = Outcome::fail(format!("Backend error: {e}"));
                    run_dir.write_status(&node.id, &outcome)?;
                    return Ok(outcome);
                }
            },
        };

        // Handle the response
        match response {
            CodergenResponse::FullOutcome(outcome) => {
                run_dir.write_status(&node.id, &outcome)?;
                Ok(outcome)
            }
            CodergenResponse::Text(text) => {
                std::fs::write(stage_dir.join("response.md"), &text)?;
                let outcome = build_text_outcome(&node.id, &text);
                run_dir.write_status(&node.id, &outcome)?;
                Ok(outcome)
            }
        }
    }
}

/// Build a success outcome from a text response, with context updates.
fn build_text_outcome(node_id: &str, text: &str) -> Outcome {
    let mut outcome = Outcome::success();
    outcome.notes = format!("Codergen completed for node '{node_id}'");
    outcome.context_updates = IndexMap::new();
    outcome.context_updates.insert(
        "last_stage".to_string(),
        serde_json::Value::String(node_id.to_string()),
    );
    outcome.context_updates.insert(
        "last_response".to_string(),
        serde_json::Value::String(truncate_response(text)),
    );
    outcome
}
