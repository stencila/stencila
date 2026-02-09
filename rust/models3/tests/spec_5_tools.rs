//! Spec Section 5 conformance tests.
//!
//! Target areas:
//! - Active vs passive tool behavior
//! - Parallel execution batching and deterministic result ordering
//! - Tool failure propagation as tool results (not hard aborts)
//! - Unknown tool handling
//! - max_tool_rounds enforcement

mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

use stencila_models3::api::cancel::AbortController;
use stencila_models3::api::generate::{GenerateOptions, generate};
use stencila_models3::api::types::Tool;
use stencila_models3::client::Client;
use stencila_models3::error::{SdkError, SdkResult};
use stencila_models3::types::finish_reason::{FinishReason, Reason};
use stencila_models3::types::tool::ToolDefinition;

use common::{ToolCallAdapter, make_response, make_tool_call_response};

fn weather_tool() -> Tool {
    Tool::active(
        ToolDefinition {
            name: "get_weather".into(),
            description: "Get weather for a location".into(),
            parameters: serde_json::json!({"type": "object", "properties": {"location": {"type": "string"}}}),
            strict: false,
        },
        |_args| async { Ok(serde_json::json!("72F and sunny")) },
    )
}

fn failing_tool() -> Tool {
    Tool::active(
        ToolDefinition {
            name: "fail_tool".into(),
            description: "A tool that always fails".into(),
            parameters: serde_json::json!({"type": "object"}),
            strict: false,
        },
        |_args| async {
            Err(SdkError::InvalidToolCall {
                message: "tool execution failed".into(),
            })
        },
    )
}

// ── Active tool execution ────────────────────────────────────────

#[tokio::test]
async fn active_tool_triggers_execution_loop() -> SdkResult<()> {
    let tc_resp = make_tool_call_response(
        "mock",
        vec![(
            "call-1",
            "get_weather",
            serde_json::json!({"location": "NYC"}),
        )],
    );
    let final_resp = make_response("mock", "It's sunny.");
    let adapter = ToolCallAdapter::new("mock", tc_resp, final_resp);
    let counter = adapter.call_counter();
    let client = Client::builder().add_provider(adapter).build()?;

    let opts = GenerateOptions::new("test-model")
        .prompt("Weather?")
        .tools(vec![weather_tool()])
        .max_tool_rounds(1)
        .client(&client);

    let result = generate(opts).await?;

    // 2 LLM calls: tool call + continuation
    assert_eq!(counter.load(Ordering::SeqCst), 2);
    assert_eq!(result.steps.len(), 2);

    // First step should have tool calls and results
    assert_eq!(result.steps[0].tool_calls.len(), 1);
    assert_eq!(result.steps[0].tool_results.len(), 1);
    assert!(!result.steps[0].tool_results[0].is_error);

    // Final step should be text
    assert_eq!(result.text, "It's sunny.");
    assert!(result.steps[1].tool_calls.is_empty());
    Ok(())
}

// ── Passive tools ────────────────────────────────────────────────

#[tokio::test]
async fn passive_tool_returns_tool_calls_without_execution() -> SdkResult<()> {
    let tc_resp = make_tool_call_response(
        "mock",
        vec![(
            "call-1",
            "get_weather",
            serde_json::json!({"location": "NYC"}),
        )],
    );
    let final_resp = make_response("mock", "Shouldn't reach this.");
    let adapter = ToolCallAdapter::new("mock", tc_resp, final_resp);
    let counter = adapter.call_counter();
    let client = Client::builder().add_provider(adapter).build()?;

    let passive = Tool::passive(ToolDefinition {
        name: "get_weather".into(),
        description: "Get weather".into(),
        parameters: serde_json::json!({"type": "object"}),
        strict: false,
    });

    let opts = GenerateOptions::new("test-model")
        .prompt("Weather?")
        .tools(vec![passive])
        .max_tool_rounds(5)
        .client(&client);

    let result = generate(opts).await?;

    // Only 1 LLM call — passive tools don't trigger loop
    assert_eq!(counter.load(Ordering::SeqCst), 1);
    assert_eq!(result.tool_calls.len(), 1);
    assert!(result.tool_results.is_empty());
    Ok(())
}

// ── Parallel tool execution ──────────────────────────────────────

#[tokio::test]
async fn parallel_tool_calls_all_executed() -> SdkResult<()> {
    let tc_resp = make_tool_call_response(
        "mock",
        vec![
            (
                "call-1",
                "get_weather",
                serde_json::json!({"location": "NYC"}),
            ),
            (
                "call-2",
                "get_weather",
                serde_json::json!({"location": "LA"}),
            ),
            (
                "call-3",
                "get_weather",
                serde_json::json!({"location": "Chicago"}),
            ),
        ],
    );
    let final_resp = make_response("mock", "Weather report complete.");
    let adapter = ToolCallAdapter::new("mock", tc_resp, final_resp);
    let client = Client::builder().add_provider(adapter).build()?;

    let execution_count = Arc::new(AtomicU32::new(0));
    let count_clone = execution_count.clone();

    let counting_tool = Tool::active(
        ToolDefinition {
            name: "get_weather".into(),
            description: "Get weather".into(),
            parameters: serde_json::json!({"type": "object", "properties": {"location": {"type": "string"}}}),
            strict: false,
        },
        move |_args| {
            let c = count_clone.clone();
            async move {
                c.fetch_add(1, Ordering::SeqCst);
                Ok(serde_json::json!("sunny"))
            }
        },
    );

    let opts = GenerateOptions::new("test-model")
        .prompt("Weather for all cities?")
        .tools(vec![counting_tool])
        .max_tool_rounds(1)
        .client(&client);

    let result = generate(opts).await?;

    // All 3 tool calls executed
    assert_eq!(execution_count.load(Ordering::SeqCst), 3);

    // Results in order
    assert_eq!(result.steps[0].tool_results.len(), 3);
    assert_eq!(result.steps[0].tool_results[0].tool_call_id, "call-1");
    assert_eq!(result.steps[0].tool_results[1].tool_call_id, "call-2");
    assert_eq!(result.steps[0].tool_results[2].tool_call_id, "call-3");

    // None should be errors
    for r in &result.steps[0].tool_results {
        assert!(!r.is_error);
    }
    Ok(())
}

// ── Partial failure ──────────────────────────────────────────────

#[tokio::test]
async fn partial_failure_does_not_abort_batch() -> SdkResult<()> {
    let tc_resp = make_tool_call_response(
        "mock",
        vec![
            (
                "call-1",
                "get_weather",
                serde_json::json!({"location": "NYC"}),
            ),
            ("call-2", "fail_tool", serde_json::json!({})),
        ],
    );
    let final_resp = make_response("mock", "Partial results available.");
    let adapter = ToolCallAdapter::new("mock", tc_resp, final_resp);
    let client = Client::builder().add_provider(adapter).build()?;

    let opts = GenerateOptions::new("test-model")
        .prompt("Do things")
        .tools(vec![weather_tool(), failing_tool()])
        .max_tool_rounds(1)
        .client(&client);

    let result = generate(opts).await?;

    // Both results present
    assert_eq!(result.steps[0].tool_results.len(), 2);

    // First succeeded
    assert!(!result.steps[0].tool_results[0].is_error);
    assert_eq!(result.steps[0].tool_results[0].tool_call_id, "call-1");

    // Second failed but didn't abort
    assert!(result.steps[0].tool_results[1].is_error);
    assert_eq!(result.steps[0].tool_results[1].tool_call_id, "call-2");
    Ok(())
}

// ── Unknown tool call ────────────────────────────────────────────

#[tokio::test]
async fn unknown_tool_returns_error_result() -> SdkResult<()> {
    let tc_resp = make_tool_call_response(
        "mock",
        vec![("call-1", "nonexistent_tool", serde_json::json!({}))],
    );
    let final_resp = make_response("mock", "Recovered.");
    let adapter = ToolCallAdapter::new("mock", tc_resp, final_resp);
    let client = Client::builder().add_provider(adapter).build()?;

    let opts = GenerateOptions::new("test-model")
        .prompt("Do something")
        .tools(vec![weather_tool()])
        .max_tool_rounds(1)
        .client(&client);

    let result = generate(opts).await?;

    // Should have error result for unknown tool
    assert_eq!(result.steps[0].tool_results.len(), 1);
    assert!(result.steps[0].tool_results[0].is_error);
    let content = result.steps[0].tool_results[0].content.as_str();
    assert!(content.is_some_and(|s| s.contains("unknown tool")));
    Ok(())
}

// ── max_tool_rounds enforcement ──────────────────────────────────

#[tokio::test]
async fn max_tool_rounds_limits_loop_iterations() -> SdkResult<()> {
    // Adapter always returns tool calls (never text)
    let tc_resp = make_tool_call_response(
        "mock",
        vec![(
            "call-1",
            "get_weather",
            serde_json::json!({"location": "NYC"}),
        )],
    );
    // Even the "final" response has tool calls
    let tc_resp2 = {
        let mut r = make_tool_call_response(
            "mock",
            vec![(
                "call-2",
                "get_weather",
                serde_json::json!({"location": "LA"}),
            )],
        );
        r.finish_reason = FinishReason::new(Reason::ToolCalls, None);
        r
    };
    let adapter = ToolCallAdapter::new("mock", tc_resp, tc_resp2);
    let counter = adapter.call_counter();
    let client = Client::builder().add_provider(adapter).build()?;

    let opts = GenerateOptions::new("test-model")
        .prompt("Weather?")
        .tools(vec![weather_tool()])
        .max_tool_rounds(1)
        .client(&client);

    let result = generate(opts).await?;

    // max_tool_rounds=1: initial call + 1 continuation = 2 LLM calls
    assert_eq!(counter.load(Ordering::SeqCst), 2);
    assert_eq!(result.steps.len(), 2);
    Ok(())
}

// ── Tool with abort signal ───────────────────────────────────────

#[tokio::test]
async fn tool_execution_respects_abort() -> SdkResult<()> {
    let tc_resp = make_tool_call_response(
        "mock",
        vec![("call-1", "get_weather", serde_json::json!({}))],
    );
    let final_resp = make_response("mock", "done");
    let adapter = ToolCallAdapter::new("mock", tc_resp, final_resp);
    let client = Client::builder().add_provider(adapter).build()?;

    let controller = AbortController::new();

    // Create a tool that aborts the controller
    let signal_clone = controller.signal();
    let aborting_tool = Tool::active(
        ToolDefinition {
            name: "get_weather".into(),
            description: "Get weather".into(),
            parameters: serde_json::json!({"type": "object"}),
            strict: false,
        },
        move |_args| {
            let _s = signal_clone.clone();
            async move {
                // This tool succeeds but the next round will see the abort
                Ok(serde_json::json!("result"))
            }
        },
    );

    // Pre-abort before the tool loop even starts
    controller.abort();

    let opts = GenerateOptions::new("test-model")
        .prompt("Weather?")
        .tools(vec![aborting_tool])
        .max_tool_rounds(1)
        .abort_signal(controller.signal())
        .client(&client);

    let err = generate(opts).await.expect_err("should be aborted");
    assert!(matches!(err, SdkError::Abort { .. }));
    Ok(())
}

// ── Tool argument schema validation (spec §5.8) ─────────────────

/// When the model sends arguments that don't match the tool's parameter
/// schema, the library should send an error result (not crash).
#[tokio::test]
async fn invalid_arguments_rejected_by_schema_validation() -> SdkResult<()> {
    // Tool requires "location" (string), model sends wrong type
    let tc_resp = make_tool_call_response(
        "mock",
        vec![(
            "call-1",
            "get_weather",
            serde_json::json!({"location": 42}), // wrong type: number instead of string
        )],
    );
    let final_resp = make_response("mock", "Recovered after bad args.");
    let adapter = ToolCallAdapter::new("mock", tc_resp, final_resp);
    let client = Client::builder().add_provider(adapter).build()?;

    let strict_tool = Tool::active(
        ToolDefinition {
            name: "get_weather".into(),
            description: "Get weather".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "location": {"type": "string"}
                },
                "required": ["location"]
            }),
            strict: false,
        },
        |_args| async { Ok(serde_json::json!("should not be called")) },
    );

    let opts = GenerateOptions::new("test-model")
        .prompt("Weather?")
        .tools(vec![strict_tool])
        .max_tool_rounds(1)
        .client(&client);

    let result = generate(opts).await?;

    // First step should have a tool result with is_error=true
    assert_eq!(result.steps[0].tool_results.len(), 1);
    assert!(result.steps[0].tool_results[0].is_error);
    let content = result.steps[0].tool_results[0].content.as_str();
    assert!(content.is_some_and(|s| s.contains("invalid arguments")));
    Ok(())
}

// ── Mixed active+passive tools ────────────────────────────────────

/// When both active and passive tools are provided and the model calls both,
/// active tool calls are executed but the loop stops so the caller can
/// handle passive tool calls manually.
#[tokio::test]
async fn mixed_active_passive_executes_active_stops_for_passive() -> SdkResult<()> {
    // Model calls both tools simultaneously
    let tc_resp = make_tool_call_response(
        "mock",
        vec![
            (
                "call-1",
                "get_weather",
                serde_json::json!({"location": "NYC"}),
            ),
            (
                "call-2",
                "send_email",
                serde_json::json!({"to": "user@example.com"}),
            ),
        ],
    );
    let final_resp = make_response("mock", "Should not reach this.");
    let adapter = ToolCallAdapter::new("mock", tc_resp, final_resp);
    let counter = adapter.call_counter();
    let client = Client::builder().add_provider(adapter).build()?;

    let active_tool = weather_tool();
    let passive_tool = Tool::passive(ToolDefinition {
        name: "send_email".into(),
        description: "Send an email (requires user confirmation)".into(),
        parameters: serde_json::json!({"type": "object", "properties": {"to": {"type": "string"}}}),
        strict: false,
    });

    let opts = GenerateOptions::new("test-model")
        .prompt("Weather and email?")
        .tools(vec![active_tool, passive_tool])
        .max_tool_rounds(5)
        .client(&client);

    let result = generate(opts).await?;

    // Only 1 LLM call — loop stops because of passive tool call
    assert_eq!(counter.load(Ordering::SeqCst), 1);
    assert_eq!(result.steps.len(), 1);

    // Both tool calls present
    assert_eq!(result.steps[0].tool_calls.len(), 2);

    // Only active tool was executed (passive tool call is skipped)
    assert_eq!(result.steps[0].tool_results.len(), 1);
    assert_eq!(result.steps[0].tool_results[0].tool_call_id, "call-1");
    assert!(!result.steps[0].tool_results[0].is_error);

    // The passive tool call is available for the caller to handle
    let passive_calls: Vec<_> = result.steps[0]
        .tool_calls
        .iter()
        .filter(|tc| tc.name == "send_email")
        .collect();
    assert_eq!(passive_calls.len(), 1);
    Ok(())
}

// ── Tool::is_active ──────────────────────────────────────────────

#[test]
fn tool_is_active_reflects_handler() {
    let active = weather_tool();
    assert!(active.is_active());

    let passive = Tool::passive(ToolDefinition {
        name: "test".into(),
        description: "test".into(),
        parameters: serde_json::json!({"type": "object"}),
        strict: false,
    });
    assert!(!passive.is_active());
}
