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
use crate::events::{EventEmitter, NoOpEmitter, PipelineEvent};
use crate::graph::{Graph, Node};
use crate::handler::Handler;
use crate::run_directory::RunDirectory;
use crate::types::Outcome;

/// The output type returned by a codergen backend.
pub enum CodergenOutput {
    /// A plain text output from the LLM.
    Text(String),
    /// A fully constructed outcome (bypasses default outcome building).
    FullOutcome(Outcome),
}

/// Trait for LLM backends that power the codergen handler.
#[async_trait]
pub trait CodergenBackend: Send + Sync {
    /// Run the LLM call for the given node and prompt.
    ///
    /// The backend receives an `EventEmitter` and `stage_index` so it can
    /// emit `StageSessionEvent` events during streaming LLM calls.
    async fn run(
        &self,
        node: &Node,
        prompt: &str,
        context: &Context,
        emitter: Arc<dyn EventEmitter>,
        stage_index: usize,
    ) -> AttractorResult<CodergenOutput>;
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
    emitter: Arc<dyn EventEmitter>,
}

impl CodergenHandler {
    /// Create a handler in simulation mode (no LLM backend).
    #[must_use]
    pub fn simulation() -> Self {
        Self {
            backend: None,
            emitter: Arc::new(NoOpEmitter),
        }
    }

    /// Create a handler with the given LLM backend.
    #[must_use]
    pub fn with_backend(backend: Arc<dyn CodergenBackend>) -> Self {
        Self {
            backend: Some(backend),
            emitter: Arc::new(NoOpEmitter),
        }
    }

    /// Create a handler with the given LLM backend and event emitter.
    #[must_use]
    pub fn with_backend_and_emitter(
        backend: Arc<dyn CodergenBackend>,
        emitter: Arc<dyn EventEmitter>,
    ) -> Self {
        Self {
            backend: Some(backend),
            emitter,
        }
    }
}

/// Maximum length for the truncated response stored in context updates.
const RESPONSE_TRUNCATION_LIMIT: usize = 200;

/// Truncate a string to the limit, appending `...` if truncated.
///
/// Finds the last char boundary at or before the limit to avoid
/// panicking on multi-byte UTF-8.
fn truncate_output(s: &str) -> String {
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
        // TODO(§5.4): When a real LLM backend is wired in, resolve fidelity
        // mode here (via `resolve_fidelity(node, incoming_edge, graph)`) and
        // check `context.get("internal.resume_degrade_fidelity")` to apply
        // §5.3 degradation on the first resumed hop. Currently only the
        // simulation backend is used, which has no LLM sessions to degrade.

        // Build prompt: prefer explicit "prompt" attr, fall back to node label
        let raw_prompt = node.get_str_attr("prompt").unwrap_or_else(|| node.label());

        // Expand runtime variables ($last_output, $last_stage) from context.
        // This runs at execution time (not at parse time like VariableExpansionTransform)
        // so that each stage sees the outputs of previously completed stages.
        let prompt = expand_runtime_variables(raw_prompt, context);

        // Read agent name from node attributes.
        let agent_name = node.get_str_attr("agent").unwrap_or("default");

        // Read stage_index from context (set by the engine loop).
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let stage_index = context.get_i64("internal.stage_index").unwrap_or(0) as usize;

        // Use RunDirectory for consistent path layout and status writing.
        let run_dir = RunDirectory::open(logs_root);
        let stage_dir = run_dir.node_dir(&node.id);
        std::fs::create_dir_all(&stage_dir)?;

        // Write the input (after expansion)
        std::fs::write(stage_dir.join("input.md"), &prompt)?;

        // Emit the input event
        self.emitter.emit(PipelineEvent::StageInput {
            node_id: node.id.clone(),
            stage_index,
            input: prompt.clone(),
            agent_name: agent_name.to_string(),
        });

        // Run the backend (or simulate)
        let response = match &self.backend {
            None => {
                // Simulation mode
                CodergenOutput::Text(format!("[Simulated] Response for stage: {}", node.id))
            }
            Some(backend) => match backend
                .run(node, &prompt, context, self.emitter.clone(), stage_index)
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    let outcome = Outcome::fail(format!("Backend error: {e}"));
                    run_dir.write_status(&node.id, &outcome)?;
                    return Ok(outcome);
                }
            },
        };

        // Handle the output
        match response {
            CodergenOutput::FullOutcome(outcome) => {
                self.emitter.emit(PipelineEvent::StageOutput {
                    node_id: node.id.clone(),
                    stage_index,
                    output: outcome.notes.clone(),
                });
                run_dir.write_status(&node.id, &outcome)?;
                Ok(outcome)
            }
            CodergenOutput::Text(text) => {
                self.emitter.emit(PipelineEvent::StageOutput {
                    node_id: node.id.clone(),
                    stage_index,
                    output: text.clone(),
                });
                std::fs::write(stage_dir.join("output.md"), &text)?;
                let outcome = build_text_outcome(&node.id, &text, context);
                run_dir.write_status(&node.id, &outcome)?;
                Ok(outcome)
            }
        }
    }
}

/// Build a success outcome from a text response, with context updates.
///
/// Stores both a truncated `last_output` (for checkpoint serialization and
/// condition expressions) and the full `last_output_full` (for runtime
/// variable expansion in subsequent stages).
fn build_text_outcome(node_id: &str, text: &str, context: &Context) -> Outcome {
    let mut outcome = Outcome::success();
    outcome.notes = format!("Codergen completed for node '{node_id}'");
    outcome.context_updates = IndexMap::new();
    outcome.context_updates.insert(
        "last_stage".to_string(),
        serde_json::Value::String(node_id.to_string()),
    );
    outcome.context_updates.insert(
        "last_output".to_string(),
        serde_json::Value::String(truncate_output(text)),
    );
    outcome.context_updates.insert(
        "last_output_full".to_string(),
        serde_json::Value::String(text.to_string()),
    );

    // Accumulate completed stages as a JSON array of {id, status} objects.
    let mut stages: Vec<serde_json::Value> = context
        .get("completed_stages")
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    stages.push(serde_json::json!({"id": node_id, "status": "success"}));
    outcome.context_updates.insert(
        "completed_stages".to_string(),
        serde_json::Value::Array(stages),
    );

    outcome
}

/// Expand runtime variables in an input string from context values.
///
/// Replaces `$last_output` with the full text of the last stage's output,
/// `$last_stage` with the ID of the last completed stage, and `$last_outcome`
/// with the outcome status (e.g. "success", "fail") of the last stage. These
/// are expanded at execution time (not parse time) so each stage sees the
/// outputs of previously completed stages.
fn expand_runtime_variables(prompt: &str, context: &Context) -> String {
    let mut result = prompt.to_string();

    if result.contains("$last_stage") {
        let value = context.get_string("last_stage");
        result = result.replace("$last_stage", &value);
    }

    if result.contains("$last_outcome") {
        let value = context.get_string("outcome");
        result = result.replace("$last_outcome", &value);
    }

    if result.contains("$last_output") {
        let value = context.get_string("last_output_full");
        result = result.replace("$last_output", &value);
    }

    result
}
