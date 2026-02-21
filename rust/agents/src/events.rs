//! Async event delivery for agent sessions (spec 2.9).
//!
//! Provides [`EventEmitter`] / [`EventReceiver`] pairs built on
//! `tokio::sync::mpsc::unbounded_channel`. The emitter stamps every event
//! with a session ID and timestamp before sending. Emission silently
//! discards if the receiver has been dropped — the agent loop must never
//! fail because nobody is listening.
//!
//! # Usage
//!
//! ```
//! use stencila_agents::events::{channel, EventEmitter, EventReceiver};
//!
//! let (emitter, mut receiver) = channel();
//! emitter.emit_session_start();
//! // receiver.recv().await => Some(SessionEvent { kind: SessionStart, .. })
//! ```

use serde_json::Value;
use tokio::sync::mpsc;

use crate::types::{EventKind, SessionEvent, now_timestamp};

// ---------------------------------------------------------------------------
// Channel constructors
// ---------------------------------------------------------------------------

/// Create an `(EventEmitter, EventReceiver)` pair with an auto-generated
/// UUID v4 session ID.
#[must_use]
pub fn channel() -> (EventEmitter, EventReceiver) {
    channel_with_id(uuid::Uuid::new_v4().to_string())
}

/// Create an `(EventEmitter, EventReceiver)` pair with a caller-supplied
/// session ID. Useful for deterministic testing.
#[must_use]
pub fn channel_with_id(session_id: String) -> (EventEmitter, EventReceiver) {
    let (tx, rx) = mpsc::unbounded_channel();
    (
        EventEmitter {
            session_id: session_id.clone(),
            tx,
        },
        EventReceiver { session_id, rx },
    )
}

// ---------------------------------------------------------------------------
// EventEmitter
// ---------------------------------------------------------------------------

/// Sends typed [`SessionEvent`]s into the channel.
///
/// All events are stamped with the emitter's session ID and the current
/// UTC timestamp. If the paired [`EventReceiver`] has been dropped, `emit`
/// silently discards the event (no panic, no error).
#[derive(Debug)]
pub struct EventEmitter {
    session_id: String,
    tx: mpsc::UnboundedSender<SessionEvent>,
}

impl EventEmitter {
    /// The session ID shared with the paired [`EventReceiver`].
    #[must_use]
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Core emission: constructs a [`SessionEvent`] and sends it.
    ///
    /// Silently discards if the receiver has been dropped.
    pub fn emit(&self, kind: EventKind, data: serde_json::Map<String, Value>) {
        let event = SessionEvent {
            kind,
            timestamp: now_timestamp(),
            session_id: self.session_id.clone(),
            data,
        };
        // Intentionally ignore SendError — the agent loop must never fail
        // because nobody is listening.
        let _ = self.tx.send(event);
    }

    // -- Convenience methods (one per EventKind) --

    /// Emit a `SESSION_START` event.
    pub fn emit_session_start(&self) {
        self.emit(EventKind::SessionStart, serde_json::Map::new());
    }

    /// Emit a `SESSION_END` event with the final session state.
    pub fn emit_session_end(&self, final_state: crate::types::SessionState) {
        let mut data = serde_json::Map::new();
        // Use serde serialization for SCREAMING_SNAKE_CASE consistency
        if let Ok(val) = serde_json::to_value(final_state) {
            data.insert("final_state".into(), val);
        }
        self.emit(EventKind::SessionEnd, data);
    }

    /// Emit a `USER_INPUT` event with the user's content.
    pub fn emit_user_input(&self, content: impl Into<String>) {
        let mut data = serde_json::Map::new();
        data.insert("content".into(), Value::String(content.into()));
        self.emit(EventKind::UserInput, data);
    }

    /// Emit an `ASSISTANT_TEXT_START` event.
    pub fn emit_assistant_text_start(&self) {
        self.emit(EventKind::AssistantTextStart, serde_json::Map::new());
    }

    /// Emit an `ASSISTANT_TEXT_DELTA` event with an incremental text token.
    pub fn emit_assistant_text_delta(&self, delta: impl Into<String>) {
        let mut data = serde_json::Map::new();
        data.insert("delta".into(), Value::String(delta.into()));
        self.emit(EventKind::AssistantTextDelta, data);
    }

    /// Emit an `ASSISTANT_TEXT_END` event with the full text and optional reasoning.
    pub fn emit_assistant_text_end(&self, text: impl Into<String>, reasoning: Option<String>) {
        let mut data = serde_json::Map::new();
        data.insert("text".into(), Value::String(text.into()));
        if let Some(r) = reasoning {
            data.insert("reasoning".into(), Value::String(r));
        }
        self.emit(EventKind::AssistantTextEnd, data);
    }

    /// Emit an `ASSISTANT_REASONING_START` event.
    pub fn emit_assistant_reasoning_start(&self) {
        self.emit(EventKind::AssistantReasoningStart, serde_json::Map::new());
    }

    /// Emit an `ASSISTANT_REASONING_DELTA` event with incremental thinking text.
    pub fn emit_assistant_reasoning_delta(&self, delta: impl Into<String>) {
        let mut data = serde_json::Map::new();
        data.insert("delta".into(), Value::String(delta.into()));
        self.emit(EventKind::AssistantReasoningDelta, data);
    }

    /// Emit an `ASSISTANT_REASONING_END` event.
    pub fn emit_assistant_reasoning_end(&self) {
        self.emit(EventKind::AssistantReasoningEnd, serde_json::Map::new());
    }

    /// Emit a `TOOL_CALL_START` event with tool arguments for observability.
    pub fn emit_tool_call_start(
        &self,
        tool_name: impl Into<String>,
        call_id: impl Into<String>,
        arguments: &serde_json::Value,
    ) {
        let mut data = serde_json::Map::new();
        data.insert("tool_name".into(), Value::String(tool_name.into()));
        data.insert("call_id".into(), Value::String(call_id.into()));
        data.insert("arguments".into(), arguments.clone());
        self.emit(EventKind::ToolCallStart, data);
    }

    /// Emit a `TOOL_CALL_OUTPUT_DELTA` event with incremental tool output.
    pub fn emit_tool_call_output_delta(
        &self,
        call_id: impl Into<String>,
        delta: impl Into<String>,
    ) {
        let mut data = serde_json::Map::new();
        data.insert("call_id".into(), Value::String(call_id.into()));
        data.insert("delta".into(), Value::String(delta.into()));
        self.emit(EventKind::ToolCallOutputDelta, data);
    }

    /// Emit a `TOOL_CALL_END` event with the full untruncated output.
    pub fn emit_tool_call_end(&self, call_id: impl Into<String>, output: impl Into<String>) {
        let mut data = serde_json::Map::new();
        data.insert("call_id".into(), Value::String(call_id.into()));
        data.insert("output".into(), Value::String(output.into()));
        self.emit(EventKind::ToolCallEnd, data);
    }

    /// Emit a `TOOL_CALL_END` event with an error instead of output.
    pub fn emit_tool_call_end_error(&self, call_id: impl Into<String>, error: impl Into<String>) {
        let mut data = serde_json::Map::new();
        data.insert("call_id".into(), Value::String(call_id.into()));
        data.insert("error".into(), Value::String(error.into()));
        self.emit(EventKind::ToolCallEnd, data);
    }

    /// Emit a `STEERING_INJECTED` event.
    pub fn emit_steering_injected(&self, content: impl Into<String>) {
        let mut data = serde_json::Map::new();
        data.insert("content".into(), Value::String(content.into()));
        self.emit(EventKind::SteeringInjected, data);
    }

    /// Emit a `TURN_LIMIT` event with arbitrary typed data.
    ///
    /// The spec defines `data` as `Map<String, Any>`, so callers may include
    /// numeric, boolean, or nested values — not just strings.
    pub fn emit_turn_limit(&self, data: serde_json::Map<String, Value>) {
        self.emit(EventKind::TurnLimit, data);
    }

    /// Emit a `LOOP_DETECTION` event.
    pub fn emit_loop_detection(&self, message: impl Into<String>) {
        let mut data = serde_json::Map::new();
        data.insert("message".into(), Value::String(message.into()));
        self.emit(EventKind::LoopDetection, data);
    }

    /// Emit an `INFO` event with a code and message.
    pub fn emit_info(&self, code: impl Into<String>, message: impl Into<String>) {
        let mut data = serde_json::Map::new();
        data.insert("code".into(), Value::String(code.into()));
        data.insert("message".into(), Value::String(message.into()));
        self.emit(EventKind::Info, data);
    }

    /// Emit an `ERROR` event with a code and message.
    pub fn emit_error(&self, code: impl Into<String>, message: impl Into<String>) {
        let mut data = serde_json::Map::new();
        data.insert("code".into(), Value::String(code.into()));
        data.insert("message".into(), Value::String(message.into()));
        self.emit(EventKind::Error, data);
    }
}

// ---------------------------------------------------------------------------
// EventReceiver
// ---------------------------------------------------------------------------

/// Receives typed [`SessionEvent`]s from the paired [`EventEmitter`].
///
/// Call [`recv()`](Self::recv) to await the next event. Returns `None` when
/// the emitter has been dropped and the buffer is fully drained.
#[derive(Debug)]
pub struct EventReceiver {
    session_id: String,
    rx: mpsc::UnboundedReceiver<SessionEvent>,
}

impl EventReceiver {
    /// The session ID shared with the paired [`EventEmitter`].
    #[must_use]
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Receive the next event.
    ///
    /// Returns `None` when the emitter has been dropped and all buffered
    /// events have been consumed.
    pub async fn recv(&mut self) -> Option<SessionEvent> {
        self.rx.recv().await
    }

    /// Try to receive a buffered event without blocking.
    ///
    /// Returns `Ok(event)` if one is available, `Err(TryRecvError::Empty)`
    /// if the channel is empty, or `Err(TryRecvError::Disconnected)` if
    /// the emitter has been dropped and no events remain.
    pub fn try_recv(&mut self) -> Result<SessionEvent, tokio::sync::mpsc::error::TryRecvError> {
        self.rx.try_recv()
    }
}
