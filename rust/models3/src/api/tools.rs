use crate::error::{SdkError, SdkResult};
use crate::types::finish_reason::{FinishReason, Reason};
use crate::types::message::Message;
use crate::types::response::Response;
use crate::types::tool::{ToolCall, ToolDefinition, ToolResult};

use super::cancel::AbortSignal;
use super::types::Tool;

/// Execute active tool calls concurrently, preserving order.
///
/// Per spec Section 5.7:
/// 1. Launch all tool executions concurrently.
/// 2. Wait for ALL results before returning.
/// 3. Preserve ordering — results in same order as calls.
/// 4. Handle partial failures — failed tools produce `is_error = true` results.
///
/// Passive tool calls (tools with no execute handler) are **skipped** —
/// they are not executed and produce no `ToolResult`. This supports mixed
/// active+passive tool sets: active tools execute automatically while
/// passive tool calls are returned to the caller.
///
/// Unknown tools produce an error result rather than a hard abort, giving
/// the model a chance to self-correct.
pub(crate) async fn execute_all_tools(
    tools: &[Tool],
    tool_calls: &[ToolCall],
    abort_signal: Option<&AbortSignal>,
) -> Vec<ToolResult> {
    let futures: Vec<_> = tool_calls
        .iter()
        .filter(|call| !is_passive_call(tools, call))
        .map(|call| execute_single_tool(tools, call, abort_signal))
        .collect();

    futures::future::join_all(futures).await
}

/// Whether a tool call targets a passive tool (one with no execute handler).
fn is_passive_call(tools: &[Tool], call: &ToolCall) -> bool {
    tools
        .iter()
        .find(|t| t.definition.name == call.name)
        .is_some_and(|t| !t.is_active())
}

/// Execute a single tool call, catching errors.
async fn execute_single_tool(
    tools: &[Tool],
    call: &ToolCall,
    abort_signal: Option<&AbortSignal>,
) -> ToolResult {
    // Check abort before starting
    if let Some(signal) = abort_signal
        && signal.is_aborted()
    {
        return ToolResult {
            tool_call_id: call.id.clone(),
            content: serde_json::Value::String("operation was aborted".into()),
            is_error: true,
        };
    }

    // Check for parse errors in tool call arguments
    if let Some(ref err) = call.parse_error {
        return ToolResult {
            tool_call_id: call.id.clone(),
            content: serde_json::Value::String(format!(
                "invalid arguments for tool '{}': {err}",
                call.name
            )),
            is_error: true,
        };
    }

    // Find the tool
    let tool = tools.iter().find(|t| t.definition.name == call.name);

    let Some(tool) = tool else {
        return ToolResult {
            tool_call_id: call.id.clone(),
            content: serde_json::Value::String(format!("unknown tool: {}", call.name)),
            is_error: true,
        };
    };

    // Passive tools should have been filtered out by execute_all_tools.
    // Defensive fallback in case this is called directly.
    let Some(ref execute) = tool.execute else {
        return ToolResult {
            tool_call_id: call.id.clone(),
            content: serde_json::Value::String(format!(
                "tool '{}' has no execute handler",
                call.name
            )),
            is_error: true,
        };
    };

    // Validate arguments against the tool's parameter schema (spec §5.8).
    // This gives the model a chance to self-correct with a descriptive error.
    if let Some(errors) = validate_tool_arguments(&call.arguments, &tool.definition.parameters) {
        return ToolResult {
            tool_call_id: call.id.clone(),
            content: serde_json::Value::String(format!(
                "invalid arguments for tool '{}': {errors}",
                call.name
            )),
            is_error: true,
        };
    }

    // Execute the handler
    match execute(call.arguments.clone()).await {
        Ok(content) => ToolResult {
            tool_call_id: call.id.clone(),
            content,
            is_error: false,
        },
        Err(e) => ToolResult {
            tool_call_id: call.id.clone(),
            content: serde_json::Value::String(e.to_string()),
            is_error: true,
        },
    }
}

/// Check whether any tool in the list has an execute handler (is active).
pub(crate) fn has_active_tools(tools: &[Tool]) -> bool {
    tools.iter().any(Tool::is_active)
}

/// Check whether any tool call targets a passive tool.
///
/// Used by the tool loop to decide whether to stop and return
/// passive tool calls to the caller.
pub(crate) fn has_passive_tool_calls(tools: &[Tool], tool_calls: &[ToolCall]) -> bool {
    tool_calls.iter().any(|call| is_passive_call(tools, call))
}

/// Extract `ToolDefinition` values from a `Tool` slice for inclusion
/// in a [`Request`](crate::types::request::Request).
pub(crate) fn tool_definitions(tools: &[Tool]) -> Vec<ToolDefinition> {
    tools.iter().map(|t| t.definition.clone()).collect()
}

/// Check whether the model's response contains tool calls that should
/// trigger the execution loop (i.e., the model's finish reason is
/// `ToolCalls` and there are actual tool call items).
pub(crate) fn should_execute_tools(tool_calls: &[ToolCall], finish_reason: &FinishReason) -> bool {
    !tool_calls.is_empty() && finish_reason.reason == Reason::ToolCalls
}

/// Build tool-result messages to append to the conversation after
/// tool execution.
pub(crate) fn tool_result_messages(
    response: &Response,
    tool_results: &[ToolResult],
) -> (Message, Vec<Message>) {
    // The assistant's message (with tool calls) goes first.
    let assistant_msg = response.message.clone();

    // Then one tool-result message per result, in order.
    let result_msgs: Vec<_> = tool_results
        .iter()
        .map(|r| Message::tool_result(&r.tool_call_id, r.content.clone(), r.is_error))
        .collect();

    (assistant_msg, result_msgs)
}

/// Validate that `prompt` and `messages` are not both provided.
pub(crate) fn validate_prompt_messages(
    prompt: Option<&str>,
    messages: &[Message],
) -> SdkResult<()> {
    if prompt.is_some() && !messages.is_empty() {
        return Err(crate::error::SdkError::Configuration {
            message: "cannot provide both 'prompt' and 'messages'".into(),
        });
    }
    Ok(())
}

/// Build the initial message list from prompt/messages/system.
pub(crate) fn build_messages(
    prompt: Option<&str>,
    messages: &[Message],
    system: Option<&str>,
) -> Vec<Message> {
    let mut result = Vec::new();

    if let Some(sys) = system {
        result.push(Message::system(sys));
    }

    if let Some(p) = prompt {
        result.push(Message::user(p));
    } else {
        result.extend(messages.iter().cloned());
    }

    result
}

/// Validate a JSON value against a JSON schema.
///
/// Used by `generate_object()` and `stream_object()` for response validation,
/// and internally for tool argument validation.
///
/// # Errors
///
/// Returns `SdkError::NoObjectGenerated` if validation fails,
/// `SdkError::Configuration` if the schema itself is invalid.
pub(crate) fn validate_against_schema(
    value: &serde_json::Value,
    schema: &serde_json::Value,
) -> SdkResult<()> {
    let validator = jsonschema::validator_for(schema).map_err(|e| SdkError::Configuration {
        message: format!("invalid JSON schema: {e}"),
    })?;
    let errors: Vec<String> = validator
        .iter_errors(value)
        .map(|e| e.to_string())
        .collect();
    if !errors.is_empty() {
        return Err(SdkError::NoObjectGenerated {
            message: format!("schema validation failed: {}", errors.join("; ")),
        });
    }
    Ok(())
}

/// Validate tool-call arguments against the tool's parameter schema.
///
/// Returns `None` if valid, or `Some(error_description)` if validation fails.
/// Skips validation if the schema cannot be compiled (to avoid blocking
/// on invalid tool definitions — those are caught at definition time).
fn validate_tool_arguments(
    arguments: &serde_json::Value,
    schema: &serde_json::Value,
) -> Option<String> {
    let validator = jsonschema::validator_for(schema).ok()?;
    let errors: Vec<String> = validator
        .iter_errors(arguments)
        .map(|e| e.to_string())
        .collect();
    if errors.is_empty() {
        None
    } else {
        Some(errors.join("; "))
    }
}
