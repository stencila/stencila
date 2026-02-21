use std::future::Future;
use std::pin::Pin;

use crate::error::SdkResult;
use crate::types::finish_reason::FinishReason;
use crate::types::response::Response;
use crate::types::tool::{ToolCall, ToolDefinition, ToolResult};
use crate::types::usage::Usage;
use crate::types::warning::Warning;

/// An async tool execution handler.
///
/// Receives parsed JSON arguments and returns a JSON-serializable result.
/// Errors are caught and sent to the model as `ToolResult { is_error: true }`.
pub type ToolExecuteFn = Box<
    dyn Fn(serde_json::Value) -> Pin<Box<dyn Future<Output = SdkResult<serde_json::Value>> + Send>>
        + Send
        + Sync,
>;

/// A tool with an optional execute handler.
///
/// **Active tools** have an `execute` handler and participate in the automatic
/// tool execution loop in [`generate()`](super::generate::generate) and
/// [`stream()`](super::stream::stream).
///
/// **Passive tools** have no handler. Tool calls are returned to the caller
/// for manual execution.
pub struct Tool {
    /// The tool's definition (name, description, parameter schema).
    pub definition: ToolDefinition,
    /// Optional async execute handler. When present, the tool is "active".
    pub execute: Option<ToolExecuteFn>,
}

impl std::fmt::Debug for Tool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tool")
            .field("definition", &self.definition)
            .field("has_execute", &self.execute.is_some())
            .finish()
    }
}

impl Tool {
    /// Create an active tool with an execute handler.
    pub fn active<F, Fut>(definition: ToolDefinition, handler: F) -> Self
    where
        F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = SdkResult<serde_json::Value>> + Send + 'static,
    {
        Self {
            definition,
            execute: Some(Box::new(move |args| Box::pin(handler(args)))),
        }
    }

    /// Create a passive tool (no execute handler).
    #[must_use]
    pub fn passive(definition: ToolDefinition) -> Self {
        Self {
            definition,
            execute: None,
        }
    }

    /// Whether this tool has an execute handler (is "active").
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.execute.is_some()
    }
}

/// A custom stop condition for the tool execution loop.
///
/// Receives the list of completed steps so far. Return `true` to stop
/// the loop early, `false` to continue.
pub type StopCondition = Box<dyn Fn(&[StepResult]) -> bool + Send + Sync>;

/// Result of a single step in the tool execution loop.
#[derive(Debug, Clone)]
pub struct StepResult {
    /// Text from this step's response.
    pub text: String,
    /// Reasoning/thinking text, if present.
    pub reasoning: Option<String>,
    /// Tool calls from this step's response.
    pub tool_calls: Vec<ToolCall>,
    /// Tool results from executing this step's tool calls.
    pub tool_results: Vec<ToolResult>,
    /// Why the model stopped generating.
    pub finish_reason: FinishReason,
    /// Token usage for this step.
    pub usage: Usage,
    /// The full response for this step.
    pub response: Response,
    /// Non-fatal warnings.
    pub warnings: Vec<Warning>,
}

impl StepResult {
    /// Build a `StepResult` from a response and its tool execution results.
    pub(crate) fn new(
        response: &Response,
        tool_calls: Vec<ToolCall>,
        tool_results: Vec<ToolResult>,
    ) -> Self {
        Self {
            text: response.text(),
            reasoning: response.reasoning(),
            tool_calls,
            tool_results,
            finish_reason: response.finish_reason.clone(),
            usage: response.usage.clone(),
            response: response.clone(),
            warnings: response.warnings.clone().unwrap_or_default(),
        }
    }
}

/// Result of a `generate()` call, potentially spanning multiple steps.
#[derive(Debug, Clone)]
pub struct GenerateResult {
    /// Text from the final step.
    pub text: String,
    /// Reasoning from the final step.
    pub reasoning: Option<String>,
    /// Tool calls from the final step (non-empty if the loop ended
    /// due to budget exhaustion or a stop condition while the model
    /// was still calling tools).
    pub tool_calls: Vec<ToolCall>,
    /// Tool results from the final step.
    pub tool_results: Vec<ToolResult>,
    /// Finish reason from the final step.
    pub finish_reason: FinishReason,
    /// Usage from the final step.
    pub usage: Usage,
    /// Aggregated usage across ALL steps.
    pub total_usage: Usage,
    /// Detailed results for each step.
    pub steps: Vec<StepResult>,
    /// The final response object.
    pub response: Response,
    /// Parsed structured output (populated by `generate_object()`).
    pub output: Option<serde_json::Value>,
}
