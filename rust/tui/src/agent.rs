use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use inflector::Inflector;
use serde_json::Value;
use stencila_agents::convenience::create_session;
use stencila_agents::error::AgentError;
use stencila_agents::session::AbortController;
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
    /// Model thinking/reasoning content (e.g. extended thinking, chain-of-thought).
    Thinking { text: String, complete: bool },
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

/// Insert a completed `Thinking` segment before the trailing run of `Text` segments.
///
/// In multi-turn streaming, accumulated text deltas for the current turn sit
/// at the tail of the segment list. This inserts the thinking block just
/// before that trailing text so it renders above the response.
fn insert_thinking_before_text_tail(segments: &mut Vec<ResponseSegment>, reasoning: &str) {
    let insert_pos = segments
        .iter()
        .rposition(|s| !matches!(s, ResponseSegment::Text(_)))
        .map_or(0, |i| i + 1);
    segments.insert(
        insert_pos,
        ResponseSegment::Thinking {
            text: reasoning.to_string(),
            complete: true,
        },
    );
}

/// Append a reasoning delta to the last `Thinking` segment.
///
/// Extends the last `Thinking` segment if present, otherwise pushes a new one.
fn append_thinking(segments: &mut Vec<ResponseSegment>, delta: &str) {
    if let Some(ResponseSegment::Thinking { text, .. }) = segments.last_mut() {
        text.push_str(delta);
    } else {
        segments.push(ResponseSegment::Thinking {
            text: delta.to_string(),
            complete: false,
        });
    }
}

/// Mark the last `Thinking` segment as complete.
fn complete_thinking(segments: &mut [ResponseSegment]) {
    for seg in segments.iter_mut().rev() {
        if let ResponseSegment::Thinking { complete, .. } = seg {
            *complete = true;
            return;
        }
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
#[allow(clippy::struct_excessive_bools)]
struct AgentProgress {
    /// Structured response segments (text interleaved with annotations).
    segments: Vec<ResponseSegment>,
    /// Whether any deltas were received for the current text segment.
    received_deltas: bool,
    /// Whether reasoning deltas were received for this turn.
    received_reasoning_deltas: bool,
    /// Whether any tool calls have been seen (multi-turn mode).
    has_tool_calls: bool,
    /// Map of `call_id` -> `tool_name` for associating `ToolCallEnd` errors.
    pending_tools: HashMap<String, String>,
    /// Whether the exchange has completed (success or failure).
    is_complete: bool,
    /// An error message, if the exchange failed.
    error: Option<String>,
    /// Approximate context usage percentage (0–100+).
    context_usage_percent: u32,
}

/// A running agent exchange, analogous to [`crate::shell::RunningCommand`].
///
/// The TUI polls this on each tick to stream incremental text updates
/// and detect completion.
pub struct RunningAgentExchange {
    progress: Arc<Mutex<AgentProgress>>,
    cancelled: Arc<AtomicBool>,
    abort_controller: AbortController,
}

impl RunningAgentExchange {
    /// Return the current response segments for rendering.
    pub fn current_segments(&self) -> Vec<ResponseSegment> {
        self.progress
            .lock()
            .map(|g| g.segments.clone())
            .unwrap_or_default()
    }

    /// Return the latest context usage percentage (0–100+).
    pub fn context_usage_percent(&self) -> u32 {
        self.progress
            .lock()
            .map(|g| g.context_usage_percent)
            .unwrap_or(0)
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

    /// Cancel the current exchange: signal the agent session to abort the
    /// in-flight LLM call / tool execution (soft abort) so the session
    /// returns to Idle and can accept the next `submit()`.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
        self.abort_controller.soft_abort();
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
        abort_controller: AbortController,
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
    /// The session is created lazily on the first
    /// submit. Returns `None` if no Tokio runtime is available (e.g. in
    /// synchronous tests).
    pub fn spawn(name: &str) -> Option<Self> {
        // Check that a tokio runtime is available before spawning
        let _handle = tokio::runtime::Handle::try_current().ok()?;
        let (tx, rx) = mpsc::unbounded_channel();
        tokio::spawn(agent_task(rx, name.to_string()));
        Some(Self { tx })
    }

    /// Submit a chat message to the agent. Returns a `RunningAgentExchange`
    /// for polling, or `None` if the background task has shut down.
    pub fn submit(&self, text: String) -> Option<RunningAgentExchange> {
        let progress = Arc::new(Mutex::new(AgentProgress::default()));
        let cancelled = Arc::new(AtomicBool::new(false));
        let abort_controller = AbortController::new();

        let exchange = RunningAgentExchange {
            progress: Arc::clone(&progress),
            cancelled: Arc::clone(&cancelled),
            abort_controller: abort_controller.clone(),
        };

        self.tx
            .send(AgentCommand::Submit {
                text,
                progress,
                cancelled,
                abort_controller,
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
async fn agent_task(mut rx: mpsc::UnboundedReceiver<AgentCommand>, name: String) {
    // Session and event receiver are created lazily on first submit.
    let mut session = None;
    let mut event_rx = None;

    while let Some(AgentCommand::Submit {
        text,
        progress,
        cancelled,
        abort_controller,
    }) = rx.recv().await
    {
        // Lazy session init
        if session.is_none() {
            match create_session(&name).await {
                Ok((.., s, er)) => {
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

        // Attach the abort signal so the session can be soft-aborted.
        sess.set_abort_signal(abort_controller.signal());

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

        // Drop the submit future (which borrows the session) before
        // potentially resetting the session below.
        drop(submit_fut);

        // Session closed — drop it so a fresh one is created automatically
        // on the next submit to this agent task.
        if matches!(submit_result, Some(Err(AgentError::SessionClosed))) {
            session = None;
            event_rx = None;
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
/// Dispatches to per-tool summary functions for known tools, producing
/// compact labels like `"Read src/main.rs"` or `"Shell cargo build"`.
/// Unknown tools (e.g. MCP tools) fall back to a generic summary.
fn format_tool_start(tool_name: &str, arguments: &Value) -> String {
    let obj = arguments.as_object();
    let str_arg = |key: &str| -> Option<String> {
        obj.and_then(|o| o.get(key))
            .and_then(Value::as_str)
            .map(|s| strip_cwd(s))
    };
    let int_arg = |key: &str| -> Option<i64> {
        obj.and_then(|o| o.get(key)).and_then(Value::as_i64)
    };

    match tool_name {
        "read_file" => {
            let path = str_arg("file_path").unwrap_or_default();
            let mut label = format!("Read {path}");
            if let Some(offset) = int_arg("offset") {
                label.push_str(&format!(":{offset}"));
            }
            label
        }
        "write_file" => {
            let path = str_arg("file_path").unwrap_or_default();
            format!("Write {path}")
        }
        "edit_file" => {
            let path = str_arg("file_path").unwrap_or_default();
            format!("Edit {path}")
        }
        "apply_patch" => {
            let summary = obj
                .and_then(|o| o.get("patch"))
                .and_then(Value::as_str)
                .map(extract_patch_summary)
                .unwrap_or_default();
            if summary.is_empty() {
                "Apply patch".to_string()
            } else {
                format!("Patch {summary}")
            }
        }
        "shell" => {
            if let Some(desc) = str_arg("description") {
                format!("Shell: {desc}")
            } else {
                let cmd = str_arg("command").unwrap_or_default();
                format!("Shell {cmd}")
            }
        }
        "grep" => {
            let pattern = str_arg("pattern").unwrap_or_default();
            let mut label = format!("Grep \"{pattern}\"");
            if let Some(path) = str_arg("path") {
                label.push_str(&format!(" in {path}"));
            }
            if let Some(glob) = str_arg("glob_filter") {
                label.push_str(&format!(" ({glob})"));
            }
            label
        }
        "glob" => {
            let pattern = str_arg("pattern").unwrap_or_default();
            let mut label = format!("Glob {pattern}");
            if let Some(path) = str_arg("path") {
                label.push_str(&format!(" in {path}"));
            }
            label
        }
        "spawn_agent" => {
            let task = str_arg("task").unwrap_or_default();
            let short = truncate_for_display(&task, 60);
            format!("Spawn agent: {short}")
        }
        "send_input" => {
            let id = str_arg("agent_id").unwrap_or_default();
            format!("Send input to {id}")
        }
        "wait" => {
            let id = str_arg("agent_id").unwrap_or_default();
            format!("Wait for {id}")
        }
        "close_agent" => {
            let id = str_arg("agent_id").unwrap_or_default();
            format!("Close agent {id}")
        }
        _ => {
            let label = tool_name.to_sentence_case();
            let summary = generic_summary(arguments);
            if summary.is_empty() {
                label
            } else {
                format!("{label} {summary}")
            }
        }
    }
}

/// Strip occurrences of the current working directory from a display string.
/// Replaces `<cwd>/` with empty (making paths relative) and standalone `<cwd>` with `.`.
fn strip_cwd(s: &str) -> String {
    let Some(cwd) = std::env::current_dir().ok() else {
        return s.to_owned();
    };
    let cwd_str = cwd.display().to_string();
    let with_slash = format!("{cwd_str}/");
    // First replace "<cwd>/" → "" (turns absolute into relative)
    let result = s.replace(&with_slash, "");
    // Then replace any remaining standalone "<cwd>" → "."
    result.replace(&cwd_str, ".")
}

/// Generic fallback summary for unknown tools (e.g. MCP tools).
/// Joins all scalar argument values with spaces, stripping CWD prefixes.
fn generic_summary(arguments: &Value) -> String {
    let obj = match arguments.as_object() {
        Some(o) if !o.is_empty() => o,
        _ => return String::new(),
    };
    let parts: Vec<String> = obj
        .iter()
        .filter_map(|(_k, v)| {
            let s = match v {
                Value::String(s) => strip_cwd(s),
                Value::Bool(b) => b.to_string(),
                Value::Number(n) => n.to_string(),
                _ => return None,
            };
            Some(s)
        })
        .collect();
    parts.join(" ")
}

/// Extract a compact summary from a v4a patch string.
///
/// Scans for `*** Add File:`, `*** Delete File:`, and `*** Update File:` lines
/// and returns the file paths joined by `, `. Falls back to empty string.
fn extract_patch_summary(patch: &str) -> String {
    let paths: Vec<&str> = patch
        .lines()
        .filter_map(|line| {
            line.strip_prefix("*** Add File: ")
                .or_else(|| line.strip_prefix("*** Delete File: "))
                .or_else(|| line.strip_prefix("*** Update File: "))
        })
        .collect();
    let summary = paths.join(", ");
    strip_cwd(&summary)
}

/// Truncate a string for display, keeping the head.
/// Uses char boundaries to avoid panics on multi-byte UTF-8.
pub(crate) fn truncate_for_display(s: &str, max_chars: usize) -> String {
    let char_count = s.chars().count();
    if char_count <= max_chars {
        return s.to_string();
    }
    let keep = max_chars - 1; // room for ellipsis
    let byte_offset = s.char_indices().nth(keep).map_or(s.len(), |(i, _)| i);
    format!("{}\u{2026}", &s[..byte_offset])
}

/// Handle an `AssistantTextEnd` event, reconciling streamed deltas with the
/// final text and inserting any reasoning/thinking content.
fn handle_assistant_text_end(
    event: &stencila_agents::types::SessionEvent,
    progress: &Arc<Mutex<AgentProgress>>,
) {
    if let Some(Value::String(text)) = event.data.get("text")
        && let Ok(mut g) = progress.lock()
    {
        // Only insert reasoning from the event if it wasn't already streamed
        // via ReasoningStart/Delta/End events.
        let reasoning = if g.received_reasoning_deltas {
            None
        } else {
            event
                .data
                .get("reasoning")
                .and_then(Value::as_str)
                .filter(|r| !r.is_empty())
        };

        if !g.received_deltas {
            // Non-streaming provider: insert thinking, then text
            if let Some(r) = reasoning {
                g.segments.push(ResponseSegment::Thinking {
                    text: r.to_string(),
                    complete: true,
                });
            }
            append_text(&mut g.segments, text);
        } else if !g.has_tool_calls {
            // Streaming, single-turn: reconcile text (keep Thinking segments)
            let thinking: Vec<ResponseSegment> = g
                .segments
                .drain(..)
                .filter(|s| matches!(s, ResponseSegment::Thinking { .. }))
                .collect();
            g.segments = thinking;
            if let Some(r) = reasoning {
                g.segments.push(ResponseSegment::Thinking {
                    text: r.to_string(),
                    complete: true,
                });
            }
            g.segments.push(ResponseSegment::Text(text.clone()));
        } else {
            // Streaming + multi-turn: insert thinking before trailing text
            if let Some(r) = reasoning {
                insert_thinking_before_text_tail(&mut g.segments, r);
            }
        }
        g.received_deltas = false;
        g.received_reasoning_deltas = false;
    }
}

/// Handle a `ToolCallStart` event.
fn handle_tool_call_start(
    event: &stencila_agents::types::SessionEvent,
    progress: &Arc<Mutex<AgentProgress>>,
) {
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
        EventKind::AssistantTextEnd => handle_assistant_text_end(event, progress),
        EventKind::AssistantReasoningStart => {
            if let Ok(mut g) = progress.lock() {
                g.segments.push(ResponseSegment::Thinking {
                    text: String::new(),
                    complete: false,
                });
                g.received_reasoning_deltas = true;
            }
        }
        EventKind::AssistantReasoningDelta => {
            if let Some(Value::String(delta)) = event.data.get("delta")
                && let Ok(mut g) = progress.lock()
            {
                append_thinking(&mut g.segments, delta);
                g.received_reasoning_deltas = true;
            }
        }
        EventKind::AssistantReasoningEnd => {
            if let Ok(mut g) = progress.lock() {
                complete_thinking(&mut g.segments);
            }
        }
        EventKind::ToolCallStart => handle_tool_call_start(event, progress),
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
        EventKind::ContextUsage => {
            if let Some(pct) = event.data.get("percent").and_then(Value::as_u64)
                && let Ok(mut g) = progress.lock()
            {
                #[allow(clippy::cast_possible_truncation)]
                {
                    g.context_usage_percent = pct as u32;
                }
            }
        }
        EventKind::Error => {
            // Context-usage warnings have severity "warning" — show inline
            // but do not mark the exchange as failed.
            let is_warning = event
                .data
                .get("severity")
                .and_then(Value::as_str)
                == Some("warning");
            if let Ok(mut g) = progress.lock() {
                if is_warning {
                    let message = event
                        .data
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or("warning");
                    g.segments
                        .push(ResponseSegment::Warning(message.to_string()));
                } else {
                    g.error = Some(
                        event
                            .data
                            .get("message")
                            .and_then(Value::as_str)
                            .unwrap_or("unknown error")
                            .to_string(),
                    );
                }
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

    // ─── format_tool_start per-tool summaries ──────────────────────

    #[test]
    fn format_read_file() {
        let args = serde_json::json!({"file_path": "src/main.rs"});
        assert_eq!(format_tool_start("read_file", &args), "Read src/main.rs");
    }

    #[test]
    fn format_read_file_with_offset() {
        let args = serde_json::json!({"file_path": "src/main.rs", "offset": 42});
        assert_eq!(
            format_tool_start("read_file", &args),
            "Read src/main.rs:42"
        );
    }

    #[test]
    fn format_write_file() {
        let args = serde_json::json!({"file_path": "out.txt", "content": "hello"});
        assert_eq!(format_tool_start("write_file", &args), "Write out.txt");
    }

    #[test]
    fn format_edit_file() {
        let args =
            serde_json::json!({"file_path": "foo.rs", "old_string": "a", "new_string": "b"});
        assert_eq!(format_tool_start("edit_file", &args), "Edit foo.rs");
    }

    #[test]
    fn format_shell_with_description() {
        let args =
            serde_json::json!({"command": "cargo build --release", "description": "Build project"});
        assert_eq!(
            format_tool_start("shell", &args),
            "Shell: Build project"
        );
    }

    #[test]
    fn format_shell_without_description() {
        let args = serde_json::json!({"command": "cargo build"});
        assert_eq!(format_tool_start("shell", &args), "Shell cargo build");
    }

    #[test]
    fn format_grep_with_path_and_glob() {
        let args =
            serde_json::json!({"pattern": "TODO", "path": "src/", "glob_filter": "*.rs"});
        assert_eq!(
            format_tool_start("grep", &args),
            "Grep \"TODO\" in src/ (*.rs)"
        );
    }

    #[test]
    fn format_grep_pattern_only() {
        let args = serde_json::json!({"pattern": "TODO"});
        assert_eq!(format_tool_start("grep", &args), "Grep \"TODO\"");
    }

    #[test]
    fn format_glob_with_path() {
        let args = serde_json::json!({"pattern": "**/*.rs", "path": "src/"});
        assert_eq!(
            format_tool_start("glob", &args),
            "Glob **/*.rs in src/"
        );
    }

    #[test]
    fn format_spawn_agent() {
        let args = serde_json::json!({"task": "Fix the broken tests"});
        assert_eq!(
            format_tool_start("spawn_agent", &args),
            "Spawn agent: Fix the broken tests"
        );
    }

    #[test]
    fn format_wait() {
        let args = serde_json::json!({"agent_id": "agent-1"});
        assert_eq!(format_tool_start("wait", &args), "Wait for agent-1");
    }

    #[test]
    fn format_unknown_tool_fallback() {
        let args = serde_json::json!({"custom_key": "some_value"});
        assert_eq!(
            format_tool_start("my_tool", &args),
            "My tool some_value"
        );
    }

    #[test]
    fn format_unknown_tool_no_args() {
        assert_eq!(format_tool_start("list_tools", &Value::Null), "List tools");
    }

    #[test]
    fn format_unknown_tool_empty_object() {
        let args = serde_json::json!({});
        assert_eq!(format_tool_start("tool", &args), "Tool");
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

    // ─── strip_cwd tests ────────────────────────────────────────────

    #[test]
    fn strip_cwd_removes_prefix_from_path() {
        let cwd = std::env::current_dir().expect("cwd");
        let cwd_str = cwd.display().to_string();
        let input = format!("{cwd_str}/rust/tui/src/app.rs");
        assert_eq!(strip_cwd(&input), "rust/tui/src/app.rs");
    }

    #[test]
    fn strip_cwd_replaces_standalone_with_dot() {
        let cwd = std::env::current_dir().expect("cwd");
        let cwd_str = cwd.display().to_string();
        let input = format!("cd {cwd_str} && cargo build");
        assert_eq!(strip_cwd(&input), "cd . && cargo build");
    }

    #[test]
    fn strip_cwd_leaves_other_paths_unchanged() {
        assert_eq!(strip_cwd("/tmp/foo/bar.rs"), "/tmp/foo/bar.rs");
    }

    #[test]
    fn strip_cwd_handles_relative_paths() {
        assert_eq!(strip_cwd("src/main.rs"), "src/main.rs");
    }

    #[test]
    fn strip_cwd_in_format_tool_start() {
        let cwd = std::env::current_dir().expect("cwd");
        let cwd_str = cwd.display().to_string();
        let args = serde_json::json!({"file_path": format!("{cwd_str}/src/main.rs")});
        assert_eq!(format_tool_start("read_file", &args), "Read src/main.rs");
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
            ResponseSegment::Thinking {
                text: "some reasoning".to_string(),
                complete: true,
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

    // ─── insert_thinking_before_text_tail ─────────────────────────────

    #[test]
    fn insert_thinking_empty_segments() {
        let mut segments: Vec<ResponseSegment> = Vec::new();
        insert_thinking_before_text_tail(&mut segments, "thought");
        assert_eq!(
            segments,
            vec![ResponseSegment::Thinking {
                text: "thought".to_string(),
                complete: true
            }]
        );
    }

    #[test]
    fn insert_thinking_all_text_segments() {
        let mut segments = vec![
            ResponseSegment::Text("hello ".to_string()),
            ResponseSegment::Text("world".to_string()),
        ];
        insert_thinking_before_text_tail(&mut segments, "reason");
        assert_eq!(segments.len(), 3);
        assert_eq!(
            segments[0],
            ResponseSegment::Thinking {
                text: "reason".to_string(),
                complete: true
            }
        );
        assert_eq!(segments[1], ResponseSegment::Text("hello ".to_string()));
        assert_eq!(segments[2], ResponseSegment::Text("world".to_string()));
    }

    #[test]
    fn insert_thinking_after_toolcall() {
        let mut segments = vec![
            ResponseSegment::Text("before".to_string()),
            ResponseSegment::ToolCall {
                call_id: "c1".to_string(),
                label: "Read file".to_string(),
                status: ToolCallStatus::Done,
            },
            ResponseSegment::Text("after".to_string()),
        ];
        insert_thinking_before_text_tail(&mut segments, "thought");
        assert_eq!(segments.len(), 4);
        // Thinking inserted after the ToolCall, before trailing Text
        assert_eq!(segments[0], ResponseSegment::Text("before".to_string()));
        assert!(matches!(segments[1], ResponseSegment::ToolCall { .. }));
        assert_eq!(
            segments[2],
            ResponseSegment::Thinking {
                text: "thought".to_string(),
                complete: true
            }
        );
        assert_eq!(segments[3], ResponseSegment::Text("after".to_string()));
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
            abort_controller: AbortController::new(),
        };
        let result = exchange.try_take_result().expect("should be complete");
        // Plain text has no annotations
        assert_eq!(result.text, "Hello\nWorld");
        // Segments preserve annotations
        assert_eq!(result.segments, segments);
    }

    // ─── process_event with reasoning ────────────────────────────────

    fn make_event(
        kind: EventKind,
        data: serde_json::Map<String, Value>,
    ) -> stencila_agents::types::SessionEvent {
        stencila_agents::types::SessionEvent {
            kind,
            timestamp: String::new(),
            session_id: String::new(),
            data,
        }
    }

    #[test]
    fn process_event_assistant_text_end_with_reasoning_non_streaming() {
        let progress = Arc::new(Mutex::new(AgentProgress::default()));
        // Non-streaming: received_deltas is false
        let mut data = serde_json::Map::new();
        data.insert("text".into(), Value::String("response".into()));
        data.insert(
            "reasoning".into(),
            Value::String("I think therefore".into()),
        );
        let event = make_event(EventKind::AssistantTextEnd, data);
        process_event(&event, &progress);

        let g = progress.lock().expect("lock");
        assert_eq!(g.segments.len(), 2);
        assert_eq!(
            g.segments[0],
            ResponseSegment::Thinking {
                text: "I think therefore".to_string(),
                complete: true
            }
        );
        assert_eq!(g.segments[1], ResponseSegment::Text("response".to_string()));
    }

    #[test]
    fn process_event_assistant_text_end_without_reasoning() {
        let progress = Arc::new(Mutex::new(AgentProgress::default()));
        let mut data = serde_json::Map::new();
        data.insert("text".into(), Value::String("response".into()));
        let event = make_event(EventKind::AssistantTextEnd, data);
        process_event(&event, &progress);

        let g = progress.lock().expect("lock");
        assert_eq!(g.segments.len(), 1);
        assert_eq!(g.segments[0], ResponseSegment::Text("response".to_string()));
    }

    #[test]
    fn process_event_assistant_text_end_with_reasoning_streaming_single_turn() {
        let progress = Arc::new(Mutex::new(AgentProgress {
            segments: vec![ResponseSegment::Text("streamed text".to_string())],
            received_deltas: true,
            ..AgentProgress::default()
        }));
        let mut data = serde_json::Map::new();
        data.insert("text".into(), Value::String("full text".into()));
        data.insert("reasoning".into(), Value::String("my reasoning".into()));
        let event = make_event(EventKind::AssistantTextEnd, data);
        process_event(&event, &progress);

        let g = progress.lock().expect("lock");
        assert_eq!(g.segments.len(), 2);
        assert_eq!(
            g.segments[0],
            ResponseSegment::Thinking {
                text: "my reasoning".to_string(),
                complete: true
            }
        );
        assert_eq!(
            g.segments[1],
            ResponseSegment::Text("full text".to_string())
        );
    }

    #[test]
    fn process_event_assistant_text_end_with_reasoning_streaming_multi_turn() {
        let progress = Arc::new(Mutex::new(AgentProgress {
            segments: vec![
                ResponseSegment::Text("turn 1".to_string()),
                ResponseSegment::ToolCall {
                    call_id: "c1".to_string(),
                    label: "Read file".to_string(),
                    status: ToolCallStatus::Done,
                },
                ResponseSegment::Text("turn 2".to_string()),
            ],
            received_deltas: true,
            has_tool_calls: true,
            ..AgentProgress::default()
        }));
        let mut data = serde_json::Map::new();
        data.insert("text".into(), Value::String("turn 2".into()));
        data.insert("reasoning".into(), Value::String("deep thought".into()));
        let event = make_event(EventKind::AssistantTextEnd, data);
        process_event(&event, &progress);

        let g = progress.lock().expect("lock");
        assert_eq!(g.segments.len(), 4);
        assert_eq!(g.segments[0], ResponseSegment::Text("turn 1".to_string()));
        assert!(matches!(g.segments[1], ResponseSegment::ToolCall { .. }));
        assert_eq!(
            g.segments[2],
            ResponseSegment::Thinking {
                text: "deep thought".to_string(),
                complete: true
            }
        );
        assert_eq!(g.segments[3], ResponseSegment::Text("turn 2".to_string()));
    }

    #[test]
    fn process_event_empty_reasoning_is_ignored() {
        let progress = Arc::new(Mutex::new(AgentProgress::default()));
        let mut data = serde_json::Map::new();
        data.insert("text".into(), Value::String("response".into()));
        data.insert("reasoning".into(), Value::String(String::new()));
        let event = make_event(EventKind::AssistantTextEnd, data);
        process_event(&event, &progress);

        let g = progress.lock().expect("lock");
        assert_eq!(g.segments.len(), 1);
        assert_eq!(g.segments[0], ResponseSegment::Text("response".to_string()));
    }

    // ─── Streaming reasoning events ──────────────────────────────────

    #[test]
    fn streaming_reasoning_start_delta_end() {
        let progress = Arc::new(Mutex::new(AgentProgress::default()));

        // ReasoningStart
        process_event(
            &make_event(EventKind::AssistantReasoningStart, serde_json::Map::new()),
            &progress,
        );
        {
            let g = progress.lock().expect("lock");
            assert_eq!(g.segments.len(), 1);
            assert!(matches!(
                g.segments[0],
                ResponseSegment::Thinking {
                    complete: false,
                    ..
                }
            ));
        }

        // ReasoningDelta
        let mut data = serde_json::Map::new();
        data.insert("delta".into(), Value::String("let me think".into()));
        process_event(
            &make_event(EventKind::AssistantReasoningDelta, data),
            &progress,
        );
        {
            let g = progress.lock().expect("lock");
            assert_eq!(
                g.segments[0],
                ResponseSegment::Thinking {
                    text: "let me think".to_string(),
                    complete: false,
                }
            );
        }

        // ReasoningEnd
        process_event(
            &make_event(EventKind::AssistantReasoningEnd, serde_json::Map::new()),
            &progress,
        );
        {
            let g = progress.lock().expect("lock");
            assert_eq!(
                g.segments[0],
                ResponseSegment::Thinking {
                    text: "let me think".to_string(),
                    complete: true,
                }
            );
        }
    }

    #[test]
    fn streamed_reasoning_not_duplicated_by_text_end() {
        let progress = Arc::new(Mutex::new(AgentProgress {
            received_deltas: true,
            received_reasoning_deltas: true,
            segments: vec![
                ResponseSegment::Thinking {
                    text: "streamed thought".to_string(),
                    complete: true,
                },
                ResponseSegment::Text("hello".to_string()),
            ],
            ..AgentProgress::default()
        }));

        // AssistantTextEnd with reasoning — should NOT duplicate thinking
        let mut data = serde_json::Map::new();
        data.insert("text".into(), Value::String("full text".into()));
        data.insert("reasoning".into(), Value::String("streamed thought".into()));
        process_event(&make_event(EventKind::AssistantTextEnd, data), &progress);

        let g = progress.lock().expect("lock");
        // Should have Thinking + Text (no duplicate Thinking)
        assert_eq!(g.segments.len(), 2);
        assert!(matches!(g.segments[0], ResponseSegment::Thinking { .. }));
        assert_eq!(
            g.segments[1],
            ResponseSegment::Text("full text".to_string())
        );
    }

    #[test]
    fn append_thinking_extends_last() {
        let mut segments = vec![ResponseSegment::Thinking {
            text: "hello".to_string(),
            complete: false,
        }];
        append_thinking(&mut segments, " world");
        assert_eq!(
            segments[0],
            ResponseSegment::Thinking {
                text: "hello world".to_string(),
                complete: false,
            }
        );
    }

    #[test]
    fn append_thinking_creates_new_when_no_thinking() {
        let mut segments = vec![ResponseSegment::Text("text".to_string())];
        append_thinking(&mut segments, "thought");
        assert_eq!(segments.len(), 2);
        assert_eq!(
            segments[1],
            ResponseSegment::Thinking {
                text: "thought".to_string(),
                complete: false,
            }
        );
    }

    #[test]
    fn complete_thinking_marks_last() {
        let mut segments = vec![
            ResponseSegment::Thinking {
                text: "first".to_string(),
                complete: true,
            },
            ResponseSegment::Text("middle".to_string()),
            ResponseSegment::Thinking {
                text: "second".to_string(),
                complete: false,
            },
        ];
        complete_thinking(&mut segments);
        // Only the last Thinking should be marked complete
        assert!(matches!(
            segments[2],
            ResponseSegment::Thinking { complete: true, .. }
        ));
    }
}
