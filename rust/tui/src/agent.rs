use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use inflector::Inflector;
use serde_json::Value;
use stencila_agents::convenience::create_session;
use stencila_agents::types::EventKind;
use tokio::sync::mpsc;

// ─── Structured response segments ───────────────────────────────────
//
// Response data is stored as a sequence of typed segments, keeping
// model text cleanly separated from tool-call and warning annotations.
// The renderer in `ui.rs` pattern-matches on these variants to apply
// appropriate styling; consumers that need plain text (response refs,
// previews, export) use `plain_text_from_segments()`.

/// Status of a tool call annotation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolCallStatus {
    /// Tool is currently executing (shown with spinner).
    Running,
    /// Tool completed successfully.
    Done,
    /// Tool completed with an error.
    Error { detail: String },
}

/// A segment of a response, interleaving assistant text with annotations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResponseSegment {
    /// Plain text from the assistant.
    Text(String),
    /// A tool call annotation.
    ToolCall {
        call_id: String,
        label: String,
        status: ToolCallStatus,
    },
    /// A warning annotation (e.g. turn limit, loop detection).
    Warning(String),
}

/// Extract only the plain assistant text from a segment list.
pub fn plain_text_from_segments(segments: &[ResponseSegment]) -> String {
    let mut text = String::new();
    for seg in segments {
        if let ResponseSegment::Text(s) = seg {
            text.push_str(s);
        }
    }
    text
}

/// Append a text delta to the segment list.
///
/// Extends the last `Text` segment if present, otherwise pushes a new one.
fn append_text(segments: &mut Vec<ResponseSegment>, delta: &str) {
    if let Some(&mut ResponseSegment::Text(ref mut s)) = segments.last_mut() {
        s.push_str(delta);
    } else {
        segments.push(ResponseSegment::Text(delta.to_string()));
    }
}

/// Find a tool call segment by `call_id` and update its status.
fn complete_tool_call(segments: &mut [ResponseSegment], call_id: &str, error: Option<&str>) {
    for seg in segments.iter_mut() {
        if let &mut ResponseSegment::ToolCall {
            call_id: ref id,
            ref mut status,
            ..
        } = seg
            && id == call_id
        {
            *status = match error {
                Some(detail) => ToolCallStatus::Error {
                    detail: detail.to_string(),
                },
                None => ToolCallStatus::Done,
            };
            return;
        }
    }
}

/// Shared progress state for a running agent exchange, updated by the
/// background event-draining task.
#[derive(Debug, Default)]
struct AgentProgress {
    /// Structured response segments (text interleaved with annotations).
    segments: Vec<ResponseSegment>,
    /// Whether any deltas were received for the current text segment.
    received_deltas: bool,
    /// Whether any tool calls have been seen (multi-turn mode).
    has_tool_calls: bool,
    /// Map of `call_id` -> `tool_name` for associating `ToolCallEnd` errors.
    pending_tools: HashMap<String, String>,
    /// Whether the exchange has completed (success or failure).
    is_complete: bool,
    /// An error message, if the exchange failed.
    error: Option<String>,
}

/// A running agent exchange, analogous to [`crate::shell::RunningCommand`].
///
/// The TUI polls this on each tick to stream incremental text updates
/// and detect completion.
pub struct RunningAgentExchange {
    progress: Arc<Mutex<AgentProgress>>,
    cancelled: Arc<AtomicBool>,
}

impl RunningAgentExchange {
    /// Return the current response segments for rendering.
    pub fn current_segments(&self) -> Vec<ResponseSegment> {
        self.progress
            .lock()
            .map(|g| g.segments.clone())
            .unwrap_or_default()
    }

    /// If the exchange is complete, return the final result.
    ///
    /// The result contains both plain text (for response refs and previews)
    /// and structured segments (for rendering with annotations).
    ///
    /// Returns `None` if still running.
    pub fn try_take_result(&self) -> Option<AgentExchangeResult> {
        let guard = self.progress.lock().ok()?;
        if guard.is_complete {
            Some(AgentExchangeResult {
                text: plain_text_from_segments(&guard.segments),
                segments: guard.segments.clone(),
                error: guard.error.clone(),
            })
        } else {
            None
        }
    }

    /// Soft-cancel: stop updating the UI with further events, but let the
    /// agent session finish in the background so it remains usable for
    /// future messages.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }
}

/// The final result of a completed agent exchange.
pub struct AgentExchangeResult {
    /// Plain assistant text (no annotations).
    pub text: String,
    /// Structured segments for rendering (text + annotations).
    pub segments: Vec<ResponseSegment>,
    /// Error message, if any.
    pub error: Option<String>,
}

/// Commands sent to the background agent task.
enum AgentCommand {
    Submit {
        text: String,
        progress: Arc<Mutex<AgentProgress>>,
        cancelled: Arc<AtomicBool>,
    },
}

/// Handle for submitting messages to the background agent task.
///
/// Owns the sending half of the command channel. Dropping this handle
/// signals the background task to shut down.
pub struct AgentHandle {
    tx: mpsc::UnboundedSender<AgentCommand>,
}

impl AgentHandle {
    /// Spawn the background agent task and return a handle.
    ///
    /// If `model` is `Some((provider, model_id))`, the agent session will
    /// use that specific model instead of the default. The session is created
    /// lazily on the first submit. Returns `None` if no Tokio runtime is
    /// available (e.g. in synchronous tests).
    pub fn spawn(model: Option<(String, String)>) -> Option<Self> {
        // Check that a tokio runtime is available before spawning
        let _handle = tokio::runtime::Handle::try_current().ok()?;
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(agent_task(rx, model));
        Some(Self { tx })
    }

    /// Submit a chat message to the agent. Returns a `RunningAgentExchange`
    /// for polling, or `None` if the background task has shut down.
    pub fn submit(&self, text: String) -> Option<RunningAgentExchange> {
        let progress = Arc::new(Mutex::new(AgentProgress::default()));
        let cancelled = Arc::new(AtomicBool::new(false));

        let exchange = RunningAgentExchange {
            progress: Arc::clone(&progress),
            cancelled: Arc::clone(&cancelled),
        };

        self.tx
            .send(AgentCommand::Submit {
                text,
                progress,
                cancelled,
            })
            .ok()?;

        Some(exchange)
    }
}

/// Background task that owns the agent session.
///
/// Waits for `Submit` commands, runs `session.submit()`, and drains
/// events into the shared `AgentProgress`. The session is created lazily
/// on the first submit so that startup errors are surfaced to the user.
async fn agent_task(
    mut rx: mpsc::UnboundedReceiver<AgentCommand>,
    model: Option<(String, String)>,
) {
    // Session and event receiver are created lazily on first submit.
    let mut session = None;
    let mut event_rx = None;

    while let Some(AgentCommand::Submit {
        text,
        progress,
        cancelled,
    }) = rx.recv().await
    {
        // Lazy session init
        if session.is_none() {
            let (provider, model_id) = match &model {
                Some((p, m)) => (Some(p.as_str()), Some(m.as_str())),
                None => (None, None),
            };
            match create_session(provider, model_id).await {
                Ok((s, er)) => {
                    session = Some(s);
                    event_rx = Some(er);
                }
                Err(e) => {
                    if let Ok(mut g) = progress.lock() {
                        g.error = Some(e.to_string());
                        g.is_complete = true;
                    }
                    continue;
                }
            }
        }

        let sess = session.as_mut().expect("session initialized above");
        let ev_rx = event_rx.as_mut().expect("event_rx initialized above");

        // Pin the submit future so we can poll it in tokio::select!
        let mut submit_fut = Box::pin(sess.submit(&text));
        let mut submit_done = false;
        let mut submit_result: Option<Result<(), stencila_agents::error::AgentError>> = None;

        loop {
            tokio::select! {
                biased;

                event = ev_rx.recv() => {
                    let Some(event) = event else {
                        // Event channel closed — session dropped
                        if let Ok(mut g) = progress.lock() {
                            g.is_complete = true;
                        }
                        break;
                    };

                    if cancelled.load(Ordering::Acquire) {
                        // Soft cancel: keep draining events so the channel
                        // doesn't back up, but don't update progress.
                        continue;
                    }

                    process_event(&event, &progress);
                }

                result = &mut submit_fut, if !submit_done => {
                    submit_done = true;
                    submit_result = Some(result);
                }
            }

            // Once submit is done, drain any remaining buffered events
            // using non-blocking try_recv to avoid stalling if the channel
            // is already empty.
            if submit_done {
                while let Ok(event) = ev_rx.try_recv() {
                    if cancelled.load(Ordering::Acquire) {
                        continue;
                    }
                    process_event(&event, &progress);
                }

                // Record any submit error
                if let Some(Err(e)) = &submit_result
                    && let Ok(mut g) = progress.lock()
                {
                    g.error = Some(e.to_string());
                }

                // Mark as complete (cancelled or not)
                if let Ok(mut g) = progress.lock() {
                    g.is_complete = true;
                }

                break;
            }
        }
    }
}

// ─── Annotation formatting ──────────────────────────────────────────
//
// These functions control the *text content* of inline annotations.
// To change how a specific tool or warning appears, edit here.
// To change colors/symbols, see `render_response_segments()` in ui.rs.

/// Format a tool-start event into a human-readable display label.
///
/// Examples: `"Read file src/main.rs"`, `"Grep TODO"`, `"Shell cargo build"`
fn format_tool_start(tool_name: &str, arguments: &Value) -> String {
    let label = tool_name.to_sentence_case();
    let key_arg = extract_key_argument(arguments);
    if key_arg.is_empty() {
        label
    } else {
        format!("{label} {key_arg}")
    }
}

/// Extract a compact display string from tool call arguments.
/// Tries well-known keys in priority order, then falls back to first string value.
fn extract_key_argument(arguments: &Value) -> String {
    let obj = match arguments.as_object() {
        Some(o) if !o.is_empty() => o,
        _ => return String::new(),
    };
    // Priority order matching common tool argument names
    for key in &["file_path", "path", "command", "pattern", "query", "name"] {
        if let Some(Value::String(v)) = obj.get(*key) {
            return truncate_for_display(v, 40);
        }
    }
    // Fallback: first string value
    for v in obj.values() {
        if let Some(s) = v.as_str() {
            return truncate_for_display(s, 40);
        }
    }
    String::new()
}

/// Truncate a string for display, keeping the head.
/// Uses char boundaries to avoid panics on multi-byte UTF-8.
fn truncate_for_display(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        return s.to_string();
    }
    let keep = max_chars - 1; // room for ellipsis
    let byte_offset = s.char_indices().nth(keep).map_or(s.len(), |(i, _)| i);
    format!("{}\u{2026}", &s[..byte_offset])
}

/// Process a single session event, updating the shared progress.
fn process_event(
    event: &stencila_agents::types::SessionEvent,
    progress: &Arc<Mutex<AgentProgress>>,
) {
    match event.kind {
        EventKind::AssistantTextDelta => {
            if let Some(Value::String(delta)) = event.data.get("delta")
                && let Ok(mut g) = progress.lock()
            {
                append_text(&mut g.segments, delta);
                g.received_deltas = true;
            }
        }
        EventKind::AssistantTextEnd => {
            if let Some(Value::String(text)) = event.data.get("text")
                && let Ok(mut g) = progress.lock()
            {
                if !g.received_deltas {
                    // Non-streaming provider: append this turn's text
                    append_text(&mut g.segments, text);
                } else if !g.has_tool_calls {
                    // Streaming, single-turn: reconcile (all segments are Text)
                    g.segments.clear();
                    g.segments.push(ResponseSegment::Text(text.clone()));
                }
                // Streaming + multi-turn: trust accumulated deltas, skip
                g.received_deltas = false;
            }
        }
        EventKind::ToolCallStart => {
            if let Some(Value::String(tool_name)) = event.data.get("tool_name")
                && let Ok(mut g) = progress.lock()
            {
                let call_id = event
                    .data
                    .get("call_id")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                g.pending_tools
                    .insert(call_id.to_string(), tool_name.clone());
                let arguments = event.data.get("arguments").cloned().unwrap_or(Value::Null);
                let label = format_tool_start(tool_name, &arguments);
                g.segments.push(ResponseSegment::ToolCall {
                    call_id: call_id.to_string(),
                    label,
                    status: ToolCallStatus::Running,
                });
                g.has_tool_calls = true;
                g.received_deltas = false;
            }
        }
        EventKind::ToolCallEnd => {
            if let Ok(mut g) = progress.lock() {
                let call_id = event
                    .data
                    .get("call_id")
                    .and_then(Value::as_str)
                    .unwrap_or("");
                let error = event.data.get("error").and_then(Value::as_str);
                complete_tool_call(&mut g.segments, call_id, error);
                g.pending_tools.remove(call_id);
            }
        }
        EventKind::TurnLimit => {
            if let Ok(mut g) = progress.lock() {
                g.segments
                    .push(ResponseSegment::Warning("Turn limit reached".to_string()));
            }
        }
        EventKind::LoopDetection => {
            if let Ok(mut g) = progress.lock() {
                let message = event
                    .data
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("Loop detected");
                g.segments
                    .push(ResponseSegment::Warning(message.to_string()));
            }
        }
        EventKind::Error => {
            let message = event
                .data
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("unknown error")
                .to_string();
            if let Ok(mut g) = progress.lock() {
                g.error = Some(message);
            }
        }
        // SessionStart/End, UserInput, SteeringInjected, ToolCallOutputDelta,
        // AssistantTextStart — no UI update needed
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Formatting tests (unchanged) ───────────────────────────────

    #[test]
    fn format_tool_start_with_file_path() {
        let args = serde_json::json!({"file_path": "src/main.rs"});
        assert_eq!(
            format_tool_start("read_file", &args),
            "Read file src/main.rs"
        );
    }

    #[test]
    fn format_tool_start_with_command() {
        let args = serde_json::json!({"command": "cargo build"});
        assert_eq!(format_tool_start("shell", &args), "Shell cargo build");
    }

    #[test]
    fn format_tool_start_no_args() {
        assert_eq!(format_tool_start("list_tools", &Value::Null), "List tools");
    }

    #[test]
    fn format_tool_start_empty_object() {
        let args = serde_json::json!({});
        assert_eq!(format_tool_start("tool", &args), "Tool");
    }

    #[test]
    fn format_tool_start_fallback_to_first_string() {
        let args = serde_json::json!({"custom_key": "some_value"});
        assert_eq!(format_tool_start("my_tool", &args), "My tool some_value");
    }

    #[test]
    fn extract_key_argument_priority_order() {
        // file_path takes priority over command
        let args = serde_json::json!({"command": "ls", "file_path": "foo.rs"});
        assert_eq!(extract_key_argument(&args), "foo.rs");
    }

    #[test]
    fn truncate_short_string() {
        assert_eq!(truncate_for_display("hello", 10), "hello");
    }

    #[test]
    fn truncate_long_string() {
        let long = "a".repeat(50);
        let result = truncate_for_display(&long, 10);
        assert_eq!(result.chars().count(), 10);
        assert!(result.ends_with('\u{2026}'));
        assert!(result.starts_with("aaaaaaaaa"));
    }

    #[test]
    fn truncate_preserves_utf8() {
        let s = "\u{1f600}".repeat(20); // 20 emoji chars
        let result = truncate_for_display(&s, 10);
        assert_eq!(result.chars().count(), 10);
        assert!(result.ends_with('\u{2026}'));
    }

    // ─── Segment building tests ─────────────────────────────────────

    #[test]
    fn append_text_to_empty() {
        let mut segments = Vec::new();
        append_text(&mut segments, "hello");
        assert_eq!(segments, vec![ResponseSegment::Text("hello".to_string())]);
    }

    #[test]
    fn append_text_extends_last_text() {
        let mut segments = vec![ResponseSegment::Text("hello".to_string())];
        append_text(&mut segments, " world");
        assert_eq!(
            segments,
            vec![ResponseSegment::Text("hello world".to_string())]
        );
    }

    #[test]
    fn append_text_after_tool_call_creates_new_segment() {
        let mut segments = vec![
            ResponseSegment::Text("before".to_string()),
            ResponseSegment::ToolCall {
                call_id: "c1".to_string(),
                label: "Read file".to_string(),
                status: ToolCallStatus::Running,
            },
        ];
        append_text(&mut segments, "after");
        assert_eq!(segments.len(), 3);
        assert_eq!(segments[2], ResponseSegment::Text("after".to_string()));
    }

    #[test]
    fn complete_tool_call_success() {
        let mut segments = vec![
            ResponseSegment::Text("text".to_string()),
            ResponseSegment::ToolCall {
                call_id: "c1".to_string(),
                label: "Read file foo.rs".to_string(),
                status: ToolCallStatus::Running,
            },
        ];
        complete_tool_call(&mut segments, "c1", None);
        assert_eq!(
            segments[1],
            ResponseSegment::ToolCall {
                call_id: "c1".to_string(),
                label: "Read file foo.rs".to_string(),
                status: ToolCallStatus::Done,
            }
        );
    }

    #[test]
    fn complete_tool_call_error() {
        let mut segments = vec![ResponseSegment::ToolCall {
            call_id: "c1".to_string(),
            label: "Shell cargo build".to_string(),
            status: ToolCallStatus::Running,
        }];
        complete_tool_call(&mut segments, "c1", Some("command not found"));
        assert_eq!(
            segments[0],
            ResponseSegment::ToolCall {
                call_id: "c1".to_string(),
                label: "Shell cargo build".to_string(),
                status: ToolCallStatus::Error {
                    detail: "command not found".to_string()
                },
            }
        );
    }

    #[test]
    fn complete_tool_call_missing_id_is_noop() {
        let mut segments = vec![ResponseSegment::ToolCall {
            call_id: "c1".to_string(),
            label: "shell ls".to_string(),
            status: ToolCallStatus::Running,
        }];
        let original = segments.clone();
        complete_tool_call(&mut segments, "c999", None);
        assert_eq!(segments, original);
    }

    #[test]
    fn complete_tool_call_targets_correct_id() {
        let mut segments = vec![
            ResponseSegment::ToolCall {
                call_id: "c1".to_string(),
                label: "Read file a.rs".to_string(),
                status: ToolCallStatus::Running,
            },
            ResponseSegment::ToolCall {
                call_id: "c2".to_string(),
                label: "Read file b.rs".to_string(),
                status: ToolCallStatus::Running,
            },
        ];
        complete_tool_call(&mut segments, "c2", None);
        // c1 stays Running, c2 becomes Done
        assert_eq!(
            segments[0],
            ResponseSegment::ToolCall {
                call_id: "c1".to_string(),
                label: "Read file a.rs".to_string(),
                status: ToolCallStatus::Running,
            }
        );
        assert_eq!(
            segments[1],
            ResponseSegment::ToolCall {
                call_id: "c2".to_string(),
                label: "Read file b.rs".to_string(),
                status: ToolCallStatus::Done,
            }
        );
    }

    // ─── plain_text_from_segments ───────────────────────────────────

    #[test]
    fn plain_text_extracts_only_text_segments() {
        let segments = vec![
            ResponseSegment::Text("Hello ".to_string()),
            ResponseSegment::ToolCall {
                call_id: "c1".to_string(),
                label: "Read file foo.rs".to_string(),
                status: ToolCallStatus::Done,
            },
            ResponseSegment::Warning("Turn limit".to_string()),
            ResponseSegment::Text("World".to_string()),
        ];
        assert_eq!(plain_text_from_segments(&segments), "Hello World");
    }

    #[test]
    fn plain_text_empty_segments() {
        assert_eq!(plain_text_from_segments(&[]), "");
    }

    // ─── try_take_result ────────────────────────────────────────────

    #[test]
    fn try_take_result_returns_clean_text_and_segments() {
        let segments = vec![
            ResponseSegment::Text("Hello\n".to_string()),
            ResponseSegment::ToolCall {
                call_id: "c1".to_string(),
                label: "Read file foo.rs".to_string(),
                status: ToolCallStatus::Done,
            },
            ResponseSegment::Text("World".to_string()),
        ];
        let progress = Arc::new(Mutex::new(AgentProgress {
            segments: segments.clone(),
            is_complete: true,
            ..AgentProgress::default()
        }));
        let exchange = RunningAgentExchange {
            progress,
            cancelled: Arc::new(AtomicBool::new(false)),
        };
        let result = exchange.try_take_result().expect("should be complete");
        // Plain text has no annotations
        assert_eq!(result.text, "Hello\nWorld");
        // Segments preserve annotations
        assert_eq!(result.segments, segments);
    }
}
