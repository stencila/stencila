//! Spec 2.9: Event system — EventEmitter / EventReceiver channel.
//!
//! Tests cover channel construction, all 13 event kinds, ordering
//! guarantees, and drop semantics.

#![allow(clippy::result_large_err)]

use stencila_agents::error::AgentError;
use stencila_agents::events::{channel, channel_with_id};
use stencila_agents::types::{EventKind, SessionEvent, SessionState};

// ---------------------------------------------------------------------------
// Test helpers
// ---------------------------------------------------------------------------

/// Receive one event or fail with a descriptive error.
async fn recv_event(
    rx: &mut stencila_agents::events::EventReceiver,
) -> Result<SessionEvent, AgentError> {
    rx.recv().await.ok_or(AgentError::Io {
        message: "channel closed unexpectedly".into(),
    })
}

/// Extract a string field from event data or fail.
fn str_field<'a>(event: &'a SessionEvent, key: &str) -> Result<&'a str, AgentError> {
    event
        .data
        .get(key)
        .and_then(|v| v.as_str())
        .ok_or(AgentError::Io {
            message: format!("missing or non-string field: {key}"),
        })
}

/// Assert that a timestamp parses as RFC 3339.
fn assert_rfc3339(ts: &str) {
    assert!(
        chrono::DateTime::parse_from_rfc3339(ts).is_ok(),
        "expected RFC 3339 timestamp, got: {ts}"
    );
}

// =========================================================================
// Channel construction
// =========================================================================

#[tokio::test]
async fn channel_returns_emitter_and_receiver() -> Result<(), AgentError> {
    let (emitter, receiver) = channel();

    // Both sides share the same session ID.
    assert_eq!(emitter.session_id(), receiver.session_id());

    // Session ID is a valid UUID v4.
    let id = emitter.session_id();
    let parsed = uuid::Uuid::parse_str(id).map_err(|e| AgentError::Io {
        message: format!("invalid UUID: {e}"),
    })?;
    assert_eq!(
        parsed.get_version(),
        Some(uuid::Version::Random),
        "expected UUID v4 (random)"
    );

    Ok(())
}

#[tokio::test]
async fn channel_with_id_uses_provided_id() -> Result<(), AgentError> {
    let (emitter, receiver) = channel_with_id("test-session-42".into());
    assert_eq!(emitter.session_id(), "test-session-42");
    assert_eq!(receiver.session_id(), "test-session-42");
    Ok(())
}

// =========================================================================
// Individual event kinds
// =========================================================================

#[tokio::test]
async fn emit_session_start() -> Result<(), AgentError> {
    let (emitter, mut rx) = channel_with_id("s1".into());
    emitter.emit_session_start();

    let event = recv_event(&mut rx).await?;
    assert_eq!(event.kind, EventKind::SessionStart);
    assert_eq!(event.session_id, "s1");
    assert_rfc3339(&event.timestamp);
    assert!(event.data.is_empty());
    Ok(())
}

#[tokio::test]
async fn emit_session_end() -> Result<(), AgentError> {
    let (emitter, mut rx) = channel_with_id("s1".into());
    emitter.emit_session_end(SessionState::Closed);

    let event = recv_event(&mut rx).await?;
    assert_eq!(event.kind, EventKind::SessionEnd);
    assert_eq!(
        event.data.get("final_state").and_then(|v| v.as_str()),
        Some("CLOSED"),
        "SESSION_END should include SCREAMING_SNAKE_CASE final_state"
    );
    Ok(())
}

#[tokio::test]
async fn emit_user_input() -> Result<(), AgentError> {
    let (emitter, mut rx) = channel_with_id("s1".into());
    emitter.emit_user_input("hello world");

    let event = recv_event(&mut rx).await?;
    assert_eq!(event.kind, EventKind::UserInput);
    assert_eq!(str_field(&event, "content")?, "hello world");
    Ok(())
}

#[tokio::test]
async fn emit_assistant_text_lifecycle() -> Result<(), AgentError> {
    let (emitter, mut rx) = channel_with_id("s1".into());

    emitter.emit_assistant_text_start();
    emitter.emit_assistant_text_delta("Hello");
    emitter.emit_assistant_text_delta(", world!");
    emitter.emit_assistant_text_end("Hello, world!", Some("I greeted the user.".into()));
    emitter.emit_assistant_text_end("No reasoning here.", None);

    let e1 = recv_event(&mut rx).await?;
    assert_eq!(e1.kind, EventKind::AssistantTextStart);

    let e2 = recv_event(&mut rx).await?;
    assert_eq!(e2.kind, EventKind::AssistantTextDelta);
    assert_eq!(str_field(&e2, "delta")?, "Hello");

    let e3 = recv_event(&mut rx).await?;
    assert_eq!(e3.kind, EventKind::AssistantTextDelta);
    assert_eq!(str_field(&e3, "delta")?, ", world!");

    let e4 = recv_event(&mut rx).await?;
    assert_eq!(e4.kind, EventKind::AssistantTextEnd);
    assert_eq!(str_field(&e4, "text")?, "Hello, world!");
    assert_eq!(str_field(&e4, "reasoning")?, "I greeted the user.");

    // None reasoning path: key must be absent.
    let e5 = recv_event(&mut rx).await?;
    assert_eq!(e5.kind, EventKind::AssistantTextEnd);
    assert_eq!(str_field(&e5, "text")?, "No reasoning here.");
    assert!(
        e5.data.get("reasoning").is_none(),
        "reasoning key must be absent when None"
    );

    Ok(())
}

#[tokio::test]
async fn emit_tool_call_lifecycle() -> Result<(), AgentError> {
    let (emitter, mut rx) = channel_with_id("s1".into());

    emitter.emit_tool_call_start("read_file", "call-1", &serde_json::Value::Null);
    emitter.emit_tool_call_output_delta("call-1", "line 1\n");
    emitter.emit_tool_call_end("call-1", "line 1\nline 2\n");

    let e1 = recv_event(&mut rx).await?;
    assert_eq!(e1.kind, EventKind::ToolCallStart);
    assert_eq!(str_field(&e1, "tool_name")?, "read_file");
    assert_eq!(str_field(&e1, "call_id")?, "call-1");

    let e2 = recv_event(&mut rx).await?;
    assert_eq!(e2.kind, EventKind::ToolCallOutputDelta);
    assert_eq!(str_field(&e2, "call_id")?, "call-1");
    assert_eq!(str_field(&e2, "delta")?, "line 1\n");

    let e3 = recv_event(&mut rx).await?;
    assert_eq!(e3.kind, EventKind::ToolCallEnd);
    assert_eq!(str_field(&e3, "call_id")?, "call-1");
    assert_eq!(str_field(&e3, "output")?, "line 1\nline 2\n");
    // Success path must not carry an error field.
    assert!(
        e3.data.get("error").is_none(),
        "success TOOL_CALL_END must not contain 'error'"
    );

    Ok(())
}

#[tokio::test]
async fn emit_tool_call_end_carries_full_untruncated_output() -> Result<(), AgentError> {
    let (emitter, mut rx) = channel_with_id("s1".into());

    // Spec 9.10: TOOL_CALL_END must carry the FULL untruncated output.
    let big_output: String = "x".repeat(100_000);
    emitter.emit_tool_call_end("call-big", &big_output);

    let event = recv_event(&mut rx).await?;
    assert_eq!(str_field(&event, "output")?.len(), 100_000);

    Ok(())
}

#[tokio::test]
async fn emit_tool_call_end_error() -> Result<(), AgentError> {
    let (emitter, mut rx) = channel_with_id("s1".into());
    emitter.emit_tool_call_end_error("call-err", "permission denied");

    let event = recv_event(&mut rx).await?;
    assert_eq!(event.kind, EventKind::ToolCallEnd);
    assert_eq!(str_field(&event, "call_id")?, "call-err");
    assert_eq!(str_field(&event, "error")?, "permission denied");
    // Error path must not carry an output field.
    assert!(
        event.data.get("output").is_none(),
        "error TOOL_CALL_END must not contain 'output'"
    );
    Ok(())
}

#[tokio::test]
async fn emit_steering_injected() -> Result<(), AgentError> {
    let (emitter, mut rx) = channel_with_id("s1".into());
    emitter.emit_steering_injected("Focus on security.");

    let event = recv_event(&mut rx).await?;
    assert_eq!(event.kind, EventKind::SteeringInjected);
    assert_eq!(str_field(&event, "content")?, "Focus on security.");
    Ok(())
}

#[tokio::test]
async fn emit_turn_limit() -> Result<(), AgentError> {
    let (emitter, mut rx) = channel_with_id("s1".into());

    // Spec says data is Map<String, Any> — use numeric values as the spec
    // pseudocode does (round = round_count, total_turns = count_turns).
    let mut data = serde_json::Map::new();
    data.insert("max_turns".into(), serde_json::Value::Number(50.into()));
    data.insert("current_turn".into(), serde_json::Value::Number(50.into()));
    emitter.emit_turn_limit(data);

    let event = recv_event(&mut rx).await?;
    assert_eq!(event.kind, EventKind::TurnLimit);
    assert_eq!(
        event.data.get("max_turns").and_then(|v| v.as_u64()),
        Some(50)
    );
    assert_eq!(
        event.data.get("current_turn").and_then(|v| v.as_u64()),
        Some(50)
    );
    Ok(())
}

#[tokio::test]
async fn emit_loop_detection() -> Result<(), AgentError> {
    let (emitter, mut rx) = channel_with_id("s1".into());
    emitter.emit_loop_detection("Detected 10 identical read_file calls");

    let event = recv_event(&mut rx).await?;
    assert_eq!(event.kind, EventKind::LoopDetection);
    assert_eq!(
        str_field(&event, "message")?,
        "Detected 10 identical read_file calls"
    );
    Ok(())
}

#[tokio::test]
async fn emit_error() -> Result<(), AgentError> {
    let (emitter, mut rx) = channel_with_id("s1".into());
    emitter.emit_error("RATE_LIMIT", "Too many requests");

    let event = recv_event(&mut rx).await?;
    assert_eq!(event.kind, EventKind::Error);
    assert_eq!(str_field(&event, "code")?, "RATE_LIMIT");
    assert_eq!(str_field(&event, "message")?, "Too many requests");
    Ok(())
}

// =========================================================================
// Ordering and cross-cutting
// =========================================================================

#[tokio::test]
async fn events_received_in_emission_order() -> Result<(), AgentError> {
    let (emitter, mut rx) = channel_with_id("order".into());

    // Emit 10 mixed events.
    emitter.emit_session_start();
    emitter.emit_user_input("hi");
    emitter.emit_assistant_text_start();
    emitter.emit_assistant_text_delta("tok");
    emitter.emit_assistant_text_end("tok", None);
    emitter.emit_tool_call_start("shell", "c1", &serde_json::Value::Null);
    emitter.emit_tool_call_output_delta("c1", "out");
    emitter.emit_tool_call_end("c1", "out");
    emitter.emit_error("E1", "oops");
    emitter.emit_session_end(SessionState::Closed);

    let expected_kinds = [
        EventKind::SessionStart,
        EventKind::UserInput,
        EventKind::AssistantTextStart,
        EventKind::AssistantTextDelta,
        EventKind::AssistantTextEnd,
        EventKind::ToolCallStart,
        EventKind::ToolCallOutputDelta,
        EventKind::ToolCallEnd,
        EventKind::Error,
        EventKind::SessionEnd,
    ];

    for (i, expected) in expected_kinds.iter().enumerate() {
        let event = recv_event(&mut rx).await?;
        assert_eq!(&event.kind, expected, "mismatch at position {i}");
    }

    Ok(())
}

#[tokio::test]
async fn all_events_carry_session_id_and_timestamp() -> Result<(), AgentError> {
    let (emitter, mut rx) = channel_with_id("all-kinds".into());

    // Emit one event per kind (13 total).
    emitter.emit_session_start();
    emitter.emit_session_end(SessionState::Closed);
    emitter.emit_user_input("x");
    emitter.emit_assistant_text_start();
    emitter.emit_assistant_text_delta("d");
    emitter.emit_assistant_text_end("d", None);
    emitter.emit_tool_call_start("t", "c", &serde_json::Value::Null);
    emitter.emit_tool_call_output_delta("c", "o");
    emitter.emit_tool_call_end("c", "o");
    emitter.emit_steering_injected("s");
    emitter.emit_turn_limit(serde_json::Map::new());
    emitter.emit_loop_detection("l");
    emitter.emit_error("E", "m");

    for _ in 0..13 {
        let event = recv_event(&mut rx).await?;
        assert_eq!(event.session_id, "all-kinds");
        assert_rfc3339(&event.timestamp);
    }

    Ok(())
}

// =========================================================================
// Drop semantics
// =========================================================================

#[tokio::test]
async fn receiver_returns_none_when_emitter_dropped() -> Result<(), AgentError> {
    let (emitter, mut rx) = channel_with_id("drop-test".into());

    // Emit two events then drop the emitter.
    emitter.emit_session_start();
    emitter.emit_session_end(SessionState::Closed);
    drop(emitter);

    // Drain the two buffered events.
    assert!(rx.recv().await.is_some());
    assert!(rx.recv().await.is_some());

    // Now returns None (channel closed, buffer empty).
    assert!(rx.recv().await.is_none());

    Ok(())
}

#[tokio::test]
async fn emit_after_receiver_dropped_does_not_panic() -> Result<(), AgentError> {
    let (emitter, receiver) = channel_with_id("silent-discard".into());
    drop(receiver);

    // These must not panic or return an error.
    emitter.emit_session_start();
    emitter.emit_user_input("ignored");
    emitter.emit_assistant_text_delta("also ignored");
    emitter.emit_error("E", "nobody listening");

    Ok(())
}
