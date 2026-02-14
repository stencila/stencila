//! Spec Section 4 conformance tests.
//!
//! Target areas:
//! - High-level `generate()` behavior
//! - High-level `stream_generate()` behavior
//! - Stream accumulator correctness
//! - `generate_object()` / `stream_object()` success and failure paths
//! - Prompt standardization (prompt vs messages, system)
//! - Usage aggregation across steps
//! - Cancellation via abort signal
#![allow(clippy::result_large_err)]

mod common;

use futures::StreamExt;

use stencila_models3::api::accumulator::StreamAccumulator;
use stencila_models3::api::cancel::AbortController;
use stencila_models3::api::generate::{GenerateOptions, generate};
use stencila_models3::api::generate_object::{GenerateObjectOptions, generate_object};
use stencila_models3::api::stream::{StreamOptions, stream_generate};
use stencila_models3::api::stream_object::{StreamObjectOptions, stream_object};
use stencila_models3::api::types::Tool;
use stencila_models3::client::Client;
use stencila_models3::error::{SdkError, SdkResult};
use stencila_models3::types::finish_reason::{FinishReason, Reason};
use stencila_models3::types::message::Message;
use stencila_models3::types::stream_event::{StreamEvent, StreamEventType};
use stencila_models3::types::tool::ToolDefinition;
use stencila_models3::types::usage::Usage;

use stencila_models3::types::timeout::Timeout;

use common::{
    MidStreamErrorAdapter, MockAdapter, SlowAdapter, ToolCallAdapter, make_response,
    make_tool_call_response,
};

fn test_client(text: &str) -> SdkResult<Client> {
    Client::builder()
        .add_provider(MockAdapter::with_text("mock", text))
        .build()
}

fn tool_call_client() -> SdkResult<(Client, std::sync::Arc<std::sync::atomic::AtomicU32>)> {
    let tc_resp = make_tool_call_response(
        "mock",
        vec![(
            "call-1",
            "get_weather",
            serde_json::json!({"location": "NYC"}),
        )],
    );
    let final_resp = make_response("mock", "The weather in NYC is sunny.");
    let adapter = ToolCallAdapter::new("mock", tc_resp, final_resp);
    let counter = adapter.call_counter();
    let client = Client::builder().add_provider(adapter).build()?;
    Ok((client, counter))
}

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

// ── generate() basic ────────────────────────────────────────────

#[tokio::test]
async fn generate_simple_prompt() -> SdkResult<()> {
    let client = test_client("Hello!")?;
    let opts = GenerateOptions::new("test-model")
        .prompt("Say hello")
        .client(&client);
    let result = generate(opts).await?;
    assert_eq!(result.text, "Hello!");
    assert_eq!(result.steps.len(), 1);
    assert_eq!(result.finish_reason.reason, Reason::Stop);
    Ok(())
}

#[tokio::test]
async fn generate_with_messages() -> SdkResult<()> {
    let client = test_client("World")?;
    let msgs = vec![
        Message::user("Hello"),
        Message::assistant("Hi"),
        Message::user("World?"),
    ];
    let opts = GenerateOptions::new("test-model")
        .messages(msgs)
        .client(&client);
    let result = generate(opts).await?;
    assert_eq!(result.text, "World");
    Ok(())
}

#[tokio::test]
async fn generate_with_system_message() -> SdkResult<()> {
    let client = test_client("I am helpful.")?;
    let opts = GenerateOptions::new("test-model")
        .prompt("Who are you?")
        .system("You are a helpful assistant.")
        .client(&client);
    let result = generate(opts).await?;
    assert_eq!(result.text, "I am helpful.");
    Ok(())
}

#[tokio::test]
async fn generate_rejects_both_prompt_and_messages() {
    let client = test_client("nope").expect("client should build");
    let opts = GenerateOptions::new("test-model")
        .prompt("hello")
        .messages(vec![Message::user("world")])
        .client(&client);
    let err = generate(opts).await.expect_err("should error");
    assert!(matches!(err, SdkError::Configuration { .. }));
}

// ── generate() usage aggregation ─────────────────────────────────

#[tokio::test]
async fn generate_aggregates_usage_across_steps() -> SdkResult<()> {
    let (client, counter) = tool_call_client()?;
    let opts = GenerateOptions::new("test-model")
        .prompt("What's the weather in NYC?")
        .tools(vec![weather_tool()])
        .max_tool_rounds(1)
        .client(&client);

    let result = generate(opts).await?;

    // Two LLM calls: first returns tool call, second returns text
    assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 2);
    assert_eq!(result.steps.len(), 2);

    // total_usage should be sum of both steps
    let step1_usage = &result.steps[0].usage;
    let step2_usage = &result.steps[1].usage;
    assert_eq!(
        result.total_usage.input_tokens,
        step1_usage.input_tokens + step2_usage.input_tokens
    );
    assert_eq!(
        result.total_usage.total_tokens,
        step1_usage.total_tokens + step2_usage.total_tokens
    );

    // Final text should be from the second step
    assert_eq!(result.text, "The weather in NYC is sunny.");
    Ok(())
}

// ── generate() with max_tool_rounds=0 (passive) ──────────────────

#[tokio::test]
async fn generate_max_tool_rounds_zero_returns_tool_calls() -> SdkResult<()> {
    let (client, counter) = tool_call_client()?;
    let opts = GenerateOptions::new("test-model")
        .prompt("What's the weather?")
        .tools(vec![weather_tool()])
        .max_tool_rounds(0)
        .client(&client);

    let result = generate(opts).await?;

    // Only one LLM call — no tool execution
    assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(result.steps.len(), 1);
    assert_eq!(result.tool_calls.len(), 1);
    assert_eq!(result.tool_calls[0].name, "get_weather");
    assert!(result.tool_results.is_empty());
    Ok(())
}

// ── generate() with passive tools ────────────────────────────────

#[tokio::test]
async fn generate_passive_tools_no_execution() -> SdkResult<()> {
    let (client, counter) = tool_call_client()?;
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

    // Only one LLM call — passive tools don't trigger execution loop
    assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(result.tool_calls.len(), 1);
    Ok(())
}

// ── generate() with abort signal ─────────────────────────────────

#[tokio::test]
async fn generate_respects_abort_signal() {
    let client = test_client("hello").expect("client");
    let controller = AbortController::new();
    controller.abort(); // pre-abort

    let opts = GenerateOptions::new("test-model")
        .prompt("hi")
        .abort_signal(controller.signal())
        .client(&client);

    let err = generate(opts).await.expect_err("should be aborted");
    assert!(matches!(err, SdkError::Abort { .. }));
}

// ── StreamAccumulator ────────────────────────────────────────────

#[test]
fn accumulator_text_deltas() {
    let mut acc = StreamAccumulator::new();
    acc.process(&StreamEvent::stream_start());
    acc.process(&StreamEvent::text_delta("Hello, "));
    acc.process(&StreamEvent::text_delta("world!"));
    acc.process(&StreamEvent::finish(
        FinishReason::new(Reason::Stop, None),
        Usage {
            input_tokens: 5,
            output_tokens: 3,
            total_tokens: 8,
            ..Usage::default()
        },
    ));

    assert_eq!(acc.text(), "Hello, world!");

    let response = acc.response();
    assert_eq!(response.text(), "Hello, world!");
    assert_eq!(response.finish_reason.reason, Reason::Stop);
    assert_eq!(response.usage.input_tokens, 5);
    assert_eq!(response.usage.total_tokens, 8);
}

#[test]
fn accumulator_tool_calls() {
    use stencila_models3::types::tool::ToolCall;

    let mut acc = StreamAccumulator::new();
    acc.process(&StreamEvent::stream_start());

    // Tool call start
    let tc = ToolCall {
        id: "call-1".into(),
        name: "get_weather".into(),
        arguments: serde_json::Value::Null,
        raw_arguments: None,
        parse_error: None,
    };
    acc.process(&StreamEvent::tool_call_event(
        StreamEventType::ToolCallStart,
        tc,
        serde_json::Value::Null,
    ));

    // Tool call delta with arguments
    let delta_tc = ToolCall {
        id: "call-1".into(),
        name: "get_weather".into(),
        arguments: serde_json::Value::Null,
        raw_arguments: Some(r#"{"location":"NYC"}"#.into()),
        parse_error: None,
    };
    acc.process(&StreamEvent::tool_call_event(
        StreamEventType::ToolCallDelta,
        delta_tc,
        serde_json::Value::Null,
    ));

    // Tool call end
    let end_tc = ToolCall {
        id: "call-1".into(),
        name: "get_weather".into(),
        arguments: serde_json::Value::Null,
        raw_arguments: None,
        parse_error: None,
    };
    acc.process(&StreamEvent::tool_call_event(
        StreamEventType::ToolCallEnd,
        end_tc,
        serde_json::Value::Null,
    ));

    acc.process(&StreamEvent::finish(
        FinishReason::new(Reason::ToolCalls, None),
        Usage::default(),
    ));

    let calls = acc.tool_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "get_weather");
    assert_eq!(calls[0].arguments, serde_json::json!({"location": "NYC"}));
    assert!(calls[0].parse_error.is_none());
}

#[test]
fn accumulator_empty_produces_default_response() {
    let acc = StreamAccumulator::new();
    let response = acc.response();
    assert!(response.text().is_empty());
    assert_eq!(response.finish_reason.reason, Reason::Other);
}

/// Gemini emits ToolCallStart (empty args) + ToolCallEnd (full args), no deltas.
/// The accumulator must take arguments from the end event.
#[test]
fn accumulator_gemini_tool_calls_no_delta() {
    use stencila_models3::types::tool::ToolCall;

    let mut acc = StreamAccumulator::new();
    acc.process(&StreamEvent::stream_start());

    // Gemini: ToolCallStart with empty args
    let start_tc = ToolCall {
        id: "call-g1".into(),
        name: "search".into(),
        arguments: serde_json::Value::Null,
        raw_arguments: None,
        parse_error: None,
    };
    acc.process(&StreamEvent::tool_call_event(
        StreamEventType::ToolCallStart,
        start_tc,
        serde_json::Value::Null,
    ));

    // Gemini: ToolCallEnd with full args (no deltas in between)
    let end_tc = ToolCall {
        id: "call-g1".into(),
        name: "search".into(),
        arguments: serde_json::json!({"query": "rust programming"}),
        raw_arguments: None,
        parse_error: None,
    };
    acc.process(&StreamEvent::tool_call_event(
        StreamEventType::ToolCallEnd,
        end_tc,
        serde_json::Value::Null,
    ));

    acc.process(&StreamEvent::finish(
        FinishReason::new(Reason::ToolCalls, None),
        Usage::default(),
    ));

    let calls = acc.tool_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].id, "call-g1");
    assert_eq!(calls[0].name, "search");
    assert_eq!(
        calls[0].arguments,
        serde_json::json!({"query": "rust programming"})
    );
    assert!(calls[0].parse_error.is_none());
}

/// Interleaved tool-call deltas from multiple concurrent calls must be
/// correctly attributed to their respective calls by ID.
/// This pattern occurs with OpenAI Chat Completions when multiple tool calls
/// arrive in the same SSE chunk.
#[test]
fn accumulator_interleaved_tool_call_deltas() {
    use stencila_models3::types::tool::ToolCall;

    let mut acc = StreamAccumulator::new();
    acc.process(&StreamEvent::stream_start());

    // Start call A
    acc.process(&StreamEvent::tool_call_event(
        StreamEventType::ToolCallStart,
        ToolCall {
            id: "call-a".into(),
            name: "get_weather".into(),
            arguments: serde_json::Value::Null,
            raw_arguments: None,
            parse_error: None,
        },
        serde_json::Value::Null,
    ));

    // Start call B (before A ends)
    acc.process(&StreamEvent::tool_call_event(
        StreamEventType::ToolCallStart,
        ToolCall {
            id: "call-b".into(),
            name: "get_time".into(),
            arguments: serde_json::Value::Null,
            raw_arguments: None,
            parse_error: None,
        },
        serde_json::Value::Null,
    ));

    // Delta for A
    acc.process(&StreamEvent::tool_call_event(
        StreamEventType::ToolCallDelta,
        ToolCall {
            id: "call-a".into(),
            name: String::new(),
            arguments: serde_json::Value::Null,
            raw_arguments: Some(r#"{"city":"#.into()),
            parse_error: None,
        },
        serde_json::Value::Null,
    ));

    // Delta for B (interleaved)
    acc.process(&StreamEvent::tool_call_event(
        StreamEventType::ToolCallDelta,
        ToolCall {
            id: "call-b".into(),
            name: String::new(),
            arguments: serde_json::Value::Null,
            raw_arguments: Some(r#"{"tz":"#.into()),
            parse_error: None,
        },
        serde_json::Value::Null,
    ));

    // Another delta for A
    acc.process(&StreamEvent::tool_call_event(
        StreamEventType::ToolCallDelta,
        ToolCall {
            id: "call-a".into(),
            name: String::new(),
            arguments: serde_json::Value::Null,
            raw_arguments: Some(r#""NYC"}"#.into()),
            parse_error: None,
        },
        serde_json::Value::Null,
    ));

    // Another delta for B
    acc.process(&StreamEvent::tool_call_event(
        StreamEventType::ToolCallDelta,
        ToolCall {
            id: "call-b".into(),
            name: String::new(),
            arguments: serde_json::Value::Null,
            raw_arguments: Some(r#""UTC"}"#.into()),
            parse_error: None,
        },
        serde_json::Value::Null,
    ));

    // End A
    acc.process(&StreamEvent::tool_call_event(
        StreamEventType::ToolCallEnd,
        ToolCall {
            id: "call-a".into(),
            name: "get_weather".into(),
            arguments: serde_json::Value::Null,
            raw_arguments: None,
            parse_error: None,
        },
        serde_json::Value::Null,
    ));

    // End B
    acc.process(&StreamEvent::tool_call_event(
        StreamEventType::ToolCallEnd,
        ToolCall {
            id: "call-b".into(),
            name: "get_time".into(),
            arguments: serde_json::Value::Null,
            raw_arguments: None,
            parse_error: None,
        },
        serde_json::Value::Null,
    ));

    acc.process(&StreamEvent::finish(
        FinishReason::new(Reason::ToolCalls, None),
        Usage::default(),
    ));

    let calls = acc.tool_calls();
    assert_eq!(calls.len(), 2);

    assert_eq!(calls[0].id, "call-a");
    assert_eq!(calls[0].name, "get_weather");
    assert_eq!(calls[0].arguments, serde_json::json!({"city": "NYC"}));
    assert!(calls[0].parse_error.is_none());

    assert_eq!(calls[1].id, "call-b");
    assert_eq!(calls[1].name, "get_time");
    assert_eq!(calls[1].arguments, serde_json::json!({"tz": "UTC"}));
    assert!(calls[1].parse_error.is_none());
}

/// ToolCallEnd with an empty ID should fall back to the sole pending call.
#[test]
fn accumulator_tool_call_end_empty_id_fallback() {
    use stencila_models3::types::tool::ToolCall;

    let mut acc = StreamAccumulator::new();
    acc.process(&StreamEvent::stream_start());

    // Start with a real ID
    acc.process(&StreamEvent::tool_call_event(
        StreamEventType::ToolCallStart,
        ToolCall {
            id: "call-x".into(),
            name: "lookup".into(),
            arguments: serde_json::Value::Null,
            raw_arguments: None,
            parse_error: None,
        },
        serde_json::Value::Null,
    ));

    // Delta
    acc.process(&StreamEvent::tool_call_event(
        StreamEventType::ToolCallDelta,
        ToolCall {
            id: "call-x".into(),
            name: String::new(),
            arguments: serde_json::Value::Null,
            raw_arguments: Some(r#"{"q":"hi"}"#.into()),
            parse_error: None,
        },
        serde_json::Value::Null,
    ));

    // End with empty ID — should still resolve the sole pending call
    acc.process(&StreamEvent::tool_call_event(
        StreamEventType::ToolCallEnd,
        ToolCall {
            id: String::new(),
            name: String::new(),
            arguments: serde_json::Value::Null,
            raw_arguments: None,
            parse_error: None,
        },
        serde_json::Value::Null,
    ));

    acc.process(&StreamEvent::finish(
        FinishReason::new(Reason::ToolCalls, None),
        Usage::default(),
    ));

    let calls = acc.tool_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].id, "call-x");
    assert_eq!(calls[0].name, "lookup");
    assert_eq!(calls[0].arguments, serde_json::json!({"q": "hi"}));
}

/// Gemini-style ToolCallEnd (no deltas, full args on end event) with empty ID
/// should still work via the sole-pending-call fallback.
#[test]
fn accumulator_gemini_end_empty_id_fallback() {
    use stencila_models3::types::tool::ToolCall;

    let mut acc = StreamAccumulator::new();
    acc.process(&StreamEvent::stream_start());

    acc.process(&StreamEvent::tool_call_event(
        StreamEventType::ToolCallStart,
        ToolCall {
            id: "call-g2".into(),
            name: "search".into(),
            arguments: serde_json::Value::Null,
            raw_arguments: None,
            parse_error: None,
        },
        serde_json::Value::Null,
    ));

    // End event has empty ID but carries full arguments
    acc.process(&StreamEvent::tool_call_event(
        StreamEventType::ToolCallEnd,
        ToolCall {
            id: String::new(),
            name: "search".into(),
            arguments: serde_json::json!({"query": "test"}),
            raw_arguments: None,
            parse_error: None,
        },
        serde_json::Value::Null,
    ));

    acc.process(&StreamEvent::finish(
        FinishReason::new(Reason::ToolCalls, None),
        Usage::default(),
    ));

    let calls = acc.tool_calls();
    assert_eq!(calls.len(), 1);
    // ID comes from the start event via the pending state
    assert_eq!(calls[0].id, "call-g2");
    assert_eq!(calls[0].name, "search");
    assert_eq!(calls[0].arguments, serde_json::json!({"query": "test"}));
}

// ── stream_generate() ────────────────────────────────────────────

#[tokio::test]
async fn stream_generate_simple() -> SdkResult<()> {
    let client = test_client("Hello, stream!")?;
    let opts = StreamOptions::new("test-model")
        .prompt("hello")
        .client(&client);

    let result = stream_generate(opts).await?.collect().await?;
    assert_eq!(result.response.text(), "Hello, stream!");
    assert_eq!(result.steps.len(), 1);

    // Should have stream_start, text_delta, finish events
    let text_events: Vec<_> = result
        .events
        .iter()
        .filter(|e| e.event_type == StreamEventType::TextDelta)
        .collect();
    assert_eq!(text_events.len(), 1);
    assert_eq!(text_events[0].delta.as_deref(), Some("Hello, stream!"));
    Ok(())
}

#[tokio::test]
async fn stream_generate_with_tools() -> SdkResult<()> {
    let (client, counter) = tool_call_client()?;
    let opts = StreamOptions::new("test-model")
        .prompt("Weather in NYC?")
        .tools(vec![weather_tool()])
        .max_tool_rounds(1)
        .client(&client);

    let result = stream_generate(opts).await?.collect().await?;

    assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 2);
    assert_eq!(result.steps.len(), 2);
    assert_eq!(result.response.text(), "The weather in NYC is sunny.");

    // A StepFinish event should have been emitted between the two steps
    let step_finishes: Vec<_> = result
        .events
        .iter()
        .filter(|e| e.event_type == StreamEventType::StepFinish)
        .collect();
    assert_eq!(step_finishes.len(), 1);
    Ok(())
}

#[tokio::test]
async fn stream_generate_rejects_both_prompt_and_messages() {
    let client = test_client("nope").expect("client");
    let opts = StreamOptions::new("test-model")
        .prompt("hi")
        .messages(vec![Message::user("bye")])
        .client(&client);

    let result = stream_generate(opts).await;
    assert!(result.is_err());
    let err = result.err().expect("should have error");
    assert!(matches!(err, SdkError::Configuration { .. }));
}

/// Verify events arrive one at a time via `next_event()`.
#[tokio::test]
async fn stream_generate_next_event_incremental() -> SdkResult<()> {
    let client = test_client("incremental")?;
    let opts = StreamOptions::new("test-model")
        .prompt("hello")
        .client(&client);

    let mut stream = stream_generate(opts).await?;

    let mut event_types = Vec::new();
    while let Some(result) = stream.next_event().await {
        let event = result?;
        event_types.push(event.event_type.clone());
    }

    // MockAdapter emits: stream_start, text_delta, finish
    assert_eq!(event_types.len(), 3);
    assert_eq!(event_types[0], StreamEventType::StreamStart);
    assert_eq!(event_types[1], StreamEventType::TextDelta);
    assert_eq!(event_types[2], StreamEventType::Finish);
    Ok(())
}

/// Verify `partial_response()` reflects accumulated state mid-stream.
#[tokio::test]
async fn stream_generate_partial_response() -> SdkResult<()> {
    let client = test_client("partial-test")?;
    let opts = StreamOptions::new("test-model")
        .prompt("hello")
        .client(&client);

    let mut stream = stream_generate(opts).await?;

    // Before any events, partial response should be empty
    assert!(stream.partial_response().text().is_empty());

    // Consume stream_start
    let event = stream.next_event().await;
    assert!(event.is_some());

    // Consume text_delta
    let event = stream.next_event().await;
    assert!(event.is_some());

    // After text_delta, partial response should have text
    assert_eq!(stream.partial_response().text(), "partial-test");
    Ok(())
}

/// Verify `text_stream()` yields only text deltas.
#[tokio::test]
async fn stream_generate_text_stream() -> SdkResult<()> {
    let client = test_client("text-only")?;
    let opts = StreamOptions::new("test-model")
        .prompt("hello")
        .client(&client);

    let stream = stream_generate(opts).await?;
    let mut text_stream = stream.text_stream();

    let mut texts = Vec::new();
    while let Some(result) = text_stream.next().await {
        texts.push(result?);
    }

    assert_eq!(texts.len(), 1);
    assert_eq!(texts[0], "text-only");
    Ok(())
}

/// Verify `text_stream()` surfaces mid-stream errors instead of silently ending.
///
/// Uses a `HangingStreamAdapter` that yields one text delta then pends forever.
/// The abort signal fires after the first delta, and `text_stream()` must
/// surface the resulting `Err(Abort)` rather than returning `None`.
#[tokio::test]
async fn stream_generate_text_stream_surfaces_abort_error() -> SdkResult<()> {
    use common::HangingStreamAdapter;
    use stencila_models3::api::cancel::AbortController;

    let adapter = HangingStreamAdapter::new("mock");
    let client = Client::builder().add_provider(adapter).build()?;

    let controller = AbortController::new();
    let signal = controller.signal();

    let opts = StreamOptions::new("test-model")
        .prompt("hi")
        .abort_signal(signal)
        .client(&client);

    let stream = stream_generate(opts).await?;
    let mut text_stream = stream.text_stream();

    // First item should be the text delta
    let first = text_stream.next().await;
    assert!(
        matches!(&first, Some(Ok(text)) if text == "hello"),
        "expected first text delta, got {first:?}"
    );

    // Abort while the stream is hanging
    controller.abort();

    // Next item must be an error, not silent termination
    let second = text_stream.next().await;
    assert!(
        matches!(&second, Some(Err(SdkError::Abort { .. }))),
        "expected Abort error from text_stream, got {second:?}"
    );
    Ok(())
}

/// Verify that a pre-aborted signal causes stream_generate to error.
#[tokio::test]
async fn stream_generate_respects_abort_signal() {
    let client = test_client("hello").expect("client");
    let controller = AbortController::new();
    controller.abort(); // pre-abort

    let opts = StreamOptions::new("test-model")
        .prompt("hi")
        .abort_signal(controller.signal())
        .client(&client);

    let result = stream_generate(opts).await;
    assert!(result.is_err());
    let err = result.err().expect("should have error");
    assert!(matches!(err, SdkError::Abort { .. }));
}

/// Mid-stream provider error becomes an error event, not a propagated Err.
/// Per spec §6.6, no retry after partial delivery; emit error event and end.
#[tokio::test]
async fn stream_generate_mid_stream_error_becomes_error_event() -> SdkResult<()> {
    let adapter = MidStreamErrorAdapter::new(
        "mock",
        SdkError::Server {
            message: "internal server error".into(),
            details: stencila_models3::error::ProviderDetails {
                provider: Some("mock".into()),
                status_code: Some(500),
                retryable: false,
                ..Default::default()
            },
        },
    );
    let client = Client::builder().add_provider(adapter).build()?;
    let opts = StreamOptions::new("test-model")
        .prompt("hello")
        .max_retries(0)
        .client(&client);

    let mut stream = stream_generate(opts).await?;

    let mut event_types = Vec::new();
    let mut had_error_event = false;
    while let Some(result) = stream.next_event().await {
        let event = result?;
        if event.event_type == StreamEventType::Error {
            had_error_event = true;
        }
        event_types.push(event.event_type.clone());
    }

    // Should have: stream_start, text_delta, error — no Err propagated
    assert!(had_error_event, "expected an error event in the stream");
    assert!(
        event_types.contains(&StreamEventType::StreamStart),
        "expected stream_start before error"
    );
    assert!(
        event_types.contains(&StreamEventType::TextDelta),
        "expected text_delta before error"
    );
    Ok(())
}

// ── generate_object() ────────────────────────────────────────────

#[tokio::test]
async fn generate_object_parses_json() -> SdkResult<()> {
    let json_text = r#"{"name": "Alice", "age": 30}"#;
    let client = Client::builder()
        .add_provider(MockAdapter::new("mock", make_response("mock", json_text)))
        .build()?;

    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "name": {"type": "string"},
            "age": {"type": "integer"}
        },
        "required": ["name", "age"]
    });

    let opts = GenerateObjectOptions::new("test-model", schema)
        .prompt("Extract name and age")
        .client(&client);

    let result = generate_object(opts).await?;
    let output = result.output.expect("should have output");
    assert_eq!(output["name"], "Alice");
    assert_eq!(output["age"], 30);
    Ok(())
}

#[tokio::test]
async fn generate_object_returns_error_on_invalid_json() {
    let client = Client::builder()
        .add_provider(MockAdapter::with_text("mock", "this is not json"))
        .build()
        .expect("client");

    let schema = serde_json::json!({"type": "object"});
    let opts = GenerateObjectOptions::new("test-model", schema)
        .prompt("Extract data")
        .client(&client);

    let err = generate_object(opts).await.expect_err("should error");
    assert!(matches!(err, SdkError::NoObjectGenerated { .. }));
}

/// Valid JSON but doesn't match schema → `NoObjectGenerated`.
#[tokio::test]
async fn generate_object_schema_validation_failure() {
    let json_text = r#"{"name": "Alice"}"#; // missing required "age"
    let client = Client::builder()
        .add_provider(MockAdapter::with_text("mock", json_text))
        .build()
        .expect("client");

    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "name": {"type": "string"},
            "age": {"type": "integer"}
        },
        "required": ["name", "age"]
    });

    let opts = GenerateObjectOptions::new("test-model", schema)
        .prompt("Extract data")
        .client(&client);

    let err = generate_object(opts).await.expect_err("should error");
    match err {
        SdkError::NoObjectGenerated { ref message } => {
            assert!(
                message.contains("schema validation failed"),
                "unexpected message: {message}"
            );
        }
        other => panic!("expected NoObjectGenerated, got: {other:?}"),
    }
}

// ── stream_object() ──────────────────────────────────────────────

#[tokio::test]
async fn stream_object_parses_json() -> SdkResult<()> {
    let json_text = r#"{"color": "blue"}"#;
    let client = Client::builder()
        .add_provider(MockAdapter::with_text("mock", json_text))
        .build()?;

    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "color": {"type": "string"}
        },
        "required": ["color"]
    });

    let opts = StreamObjectOptions::new("test-model", schema)
        .prompt("What color?")
        .client(&client);

    let result = stream_object(opts).await?;
    assert_eq!(result.output["color"], "blue");
    Ok(())
}

#[tokio::test]
async fn stream_object_returns_error_on_invalid_json() {
    let client = Client::builder()
        .add_provider(MockAdapter::with_text("mock", "not json!"))
        .build()
        .expect("client");

    let schema = serde_json::json!({"type": "object"});
    let opts = StreamObjectOptions::new("test-model", schema)
        .prompt("Extract")
        .client(&client);

    let err = stream_object(opts).await.expect_err("should error");
    assert!(matches!(err, SdkError::NoObjectGenerated { .. }));
}

// ── AbortController / AbortSignal ────────────────────────────────

#[test]
fn abort_controller_signal_lifecycle() {
    let controller = AbortController::new();
    let signal = controller.signal();

    assert!(!signal.is_aborted());

    controller.abort();

    assert!(signal.is_aborted());
}

#[test]
fn abort_signal_clones_share_state() {
    let controller = AbortController::new();
    let s1 = controller.signal();
    let s2 = s1.clone();

    assert!(!s1.is_aborted());
    assert!(!s2.is_aborted());

    controller.abort();

    assert!(s1.is_aborted());
    assert!(s2.is_aborted());
}

// ── stop_when custom stop condition ──────────────────────────────

#[tokio::test]
async fn generate_stop_when_stops_early() -> SdkResult<()> {
    // Use a multi-round tool call adapter
    let tc_resp = make_tool_call_response(
        "mock",
        vec![(
            "call-1",
            "get_weather",
            serde_json::json!({"location": "NYC"}),
        )],
    );
    let final_resp = make_response("mock", "Done.");
    let adapter = ToolCallAdapter::new("mock", tc_resp, final_resp);
    let counter = adapter.call_counter();
    let client = Client::builder().add_provider(adapter).build()?;

    // stop_when: stop after the first step
    let opts = GenerateOptions::new("test-model")
        .prompt("Weather?")
        .tools(vec![weather_tool()])
        .max_tool_rounds(10) // high limit
        .stop_when(Box::new(|_steps| true)) // always stop
        .client(&client);

    let result = generate(opts).await?;

    // Should have stopped after 1 step (the tool call)
    assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert_eq!(result.steps.len(), 1);
    Ok(())
}

// ── Timeout enforcement (spec §4.7) ─────────────────────────────

/// Total timeout fires when generate() takes too long overall.
#[tokio::test]
async fn generate_total_timeout_fires() {
    let adapter = SlowAdapter::new(
        "mock",
        make_response("mock", "slow"),
        std::time::Duration::from_millis(200),
    );
    let client = Client::builder()
        .add_provider(adapter)
        .build()
        .expect("client");

    let timeout = Timeout {
        total: Some(0.05), // 50ms — will fire before the 200ms delay
        per_step: None,
        request: None,
        connect: None,
        stream_idle: None,
    };

    let opts = GenerateOptions::new("test-model")
        .prompt("hello")
        .timeout(timeout)
        .max_retries(0)
        .client(&client);

    let err = generate(opts).await.expect_err("should timeout");
    assert!(
        matches!(err, SdkError::RequestTimeout { .. }),
        "expected RequestTimeout, got: {err:?}"
    );
}

/// Per-step timeout fires when a single LLM call takes too long.
#[tokio::test]
async fn generate_per_step_timeout_fires() {
    let adapter = SlowAdapter::new(
        "mock",
        make_response("mock", "slow"),
        std::time::Duration::from_millis(200),
    );
    let client = Client::builder()
        .add_provider(adapter)
        .build()
        .expect("client");

    let timeout = Timeout {
        total: None,
        per_step: Some(0.05), // 50ms — will fire before the 200ms delay
        request: None,
        connect: None,
        stream_idle: None,
    };

    let opts = GenerateOptions::new("test-model")
        .prompt("hello")
        .timeout(timeout)
        .max_retries(0)
        .client(&client);

    let err = generate(opts).await.expect_err("should timeout");
    assert!(
        matches!(err, SdkError::RequestTimeout { .. }),
        "expected RequestTimeout, got: {err:?}"
    );
}

/// Stream per-step timeout fires when connection takes too long.
#[tokio::test]
async fn stream_per_step_timeout_fires() {
    let adapter = SlowAdapter::new(
        "mock",
        make_response("mock", "slow"),
        std::time::Duration::from_millis(200),
    );
    let client = Client::builder()
        .add_provider(adapter)
        .build()
        .expect("client");

    let timeout = Timeout {
        total: None,
        per_step: Some(0.05), // 50ms — will fire before the 200ms delay
        request: None,
        connect: None,
        stream_idle: None,
    };

    let opts = StreamOptions::new("test-model")
        .prompt("hello")
        .timeout(timeout)
        .max_retries(0)
        .client(&client);

    let stream_result = stream_generate(opts).await;
    // The stream setup itself might succeed (lazy), so check collect too
    match stream_result {
        Err(err) => {
            assert!(
                matches!(err, SdkError::RequestTimeout { .. }),
                "expected RequestTimeout, got: {err:?}"
            );
        }
        Ok(stream) => {
            let err = stream
                .collect()
                .await
                .expect_err("should timeout during collect");
            assert!(
                matches!(err, SdkError::RequestTimeout { .. }),
                "expected RequestTimeout, got: {err:?}"
            );
        }
    }
}
