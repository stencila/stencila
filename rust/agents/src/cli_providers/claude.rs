//! Claude CLI provider — delegates to the `claude` CLI tool.
//!
//! Spawns `claude --print --output-format stream-json` as a subprocess and
//! parses newline-delimited JSON events from stdout. The Claude CLI handles
//! its own tool execution loop; this adapter observes and streams events.
//!
//! Session persistence is managed by passing `--resume <session_id>` on
//! subsequent calls after a session ID has been established.

use std::collections::HashMap;
use std::process::Stdio;

use async_trait::async_trait;
use serde_json::Value;
use tokio::io::AsyncBufReadExt;
use tokio::process::{Child, Command};

use super::{CliProviderConfig, require_cli};
use crate::error::{AgentError, AgentResult};
use crate::events::EventEmitter;
use crate::types::AbortSignal;

// ---------------------------------------------------------------------------
// ClaudeCliProvider
// ---------------------------------------------------------------------------

/// Provider that delegates to the `claude` CLI tool.
#[derive(Debug)]
pub struct ClaudeCliProvider {
    config: CliProviderConfig,
    /// Session ID obtained from Claude's output, used for `--resume`.
    cli_session_id: Option<String>,
    /// Handle to the running subprocess.
    child: Option<Child>,
}

impl ClaudeCliProvider {
    /// Create a new Claude CLI provider with the given configuration.
    pub fn new(config: CliProviderConfig) -> Self {
        Self {
            config,
            cli_session_id: None,
            child: None,
        }
    }

    /// Build the `claude` command with appropriate flags.
    ///
    /// The prompt is piped via stdin (not passed as a positional argument).
    /// The caller must write the input to the child's stdin after spawning.
    fn build_command(&self) -> Command {
        let mut cmd = Command::new("claude");

        // --print enables non-interactive mode: read prompt from stdin,
        // write response to stdout. --verbose is required when using
        // stream-json output format.
        cmd.args(["--print", "--verbose", "--output-format", "stream-json"]);

        // Model selection
        if let Some(ref model) = self.config.model {
            cmd.arg("--model").arg(model);
        }

        // System prompt / instructions
        if let Some(ref instructions) = self.config.instructions {
            cmd.arg("--append-system-prompt").arg(instructions);
        }

        // Max turns
        if let Some(max_turns) = self.config.max_turns {
            cmd.arg("--max-turns").arg(max_turns.to_string());
        }

        // Session resumption
        if let Some(ref session_id) = self.cli_session_id {
            cmd.arg("--resume").arg(session_id);
        }

        // Subprocess I/O — stdin piped so we can write the prompt
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdin(Stdio::piped());

        cmd
    }

    /// Whether a submit error is likely transient and worth one retry.
    fn is_retryable_submit_error(error: &AgentError) -> bool {
        match error {
            AgentError::CliParseError { .. } => true,
            AgentError::CliProcessFailed { code, .. } => *code == -1,
            AgentError::CliNotFound { .. }
            | AgentError::CliUnsupported { .. }
            | AgentError::FileNotFound { .. }
            | AgentError::EditConflict { .. }
            | AgentError::ShellTimeout { .. }
            | AgentError::ShellExitError { .. }
            | AgentError::PermissionDenied { .. }
            | AgentError::ValidationError { .. }
            | AgentError::UnknownTool { .. }
            | AgentError::Io { .. }
            | AgentError::Mcp { .. }
            | AgentError::SessionClosed
            | AgentError::InvalidState { .. }
            | AgentError::TurnLimitExceeded { .. }
            | AgentError::ContextLengthExceeded { .. }
            | AgentError::Sdk(_) => false,
        }
    }
}

#[async_trait]
impl super::CliProvider for ClaudeCliProvider {
    fn id(&self) -> &str {
        "claude-cli"
    }

    async fn submit(
        &mut self,
        input: &str,
        events: &EventEmitter,
        abort: Option<&AbortSignal>,
    ) -> AgentResult<()> {
        require_cli("claude")?;

        let mut cmd = self.build_command();
        let mut child = cmd.spawn().map_err(|e| AgentError::CliProcessFailed {
            code: -1,
            stderr: format!("Failed to spawn claude: {e}"),
        })?;

        // Write prompt to stdin, then close it so claude reads EOF
        {
            let stdin = child
                .stdin
                .take()
                .ok_or_else(|| AgentError::CliParseError {
                    message: "Failed to access claude stdin".to_string(),
                })?;
            let mut stdin = stdin;
            tokio::io::AsyncWriteExt::write_all(&mut stdin, input.as_bytes())
                .await
                .map_err(|e| AgentError::CliProcessFailed {
                    code: -1,
                    stderr: format!("Failed to write to claude stdin: {e}"),
                })?;
            // stdin is dropped here, closing the pipe
        }

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AgentError::CliParseError {
                message: "Failed to capture claude stdout".to_string(),
            })?;

        // Spawn a task to collect stderr in the background
        let stderr_handle = child.stderr.take();
        let stderr_task = tokio::spawn(async move {
            let Some(stderr) = stderr_handle else {
                return String::new();
            };
            let mut buf = String::new();
            let _ = tokio::io::AsyncReadExt::read_to_string(&mut { stderr }, &mut buf).await;
            buf
        });

        self.child = Some(child);

        let reader = tokio::io::BufReader::new(stdout);
        let mut lines = reader.lines();
        let mut emitted_text_start = false;
        let mut accumulated_text = String::new();
        // Capture non-JSON stdout lines as potential error messages
        let mut non_json_output = Vec::new();

        // Destructure to satisfy the borrow checker: `child` and
        // `cli_session_id` are borrowed independently by the read
        // loop and the per-line closure.
        let child = &mut self.child;
        let session_id = &mut self.cli_session_id;
        // Maps content block index → tool call ID so that
        // `input_json_delta` events can be correlated with their
        // `content_block_start` (which carries the actual call ID).
        let mut block_call_ids = HashMap::new();

        // Process streaming JSON events line by line
        let aborted =
            super::read_lines_until_eof_or_abort(&mut lines, child, abort, "claude", |line| {
                match serde_json::from_str::<Value>(&line) {
                    Ok(event) => {
                        process_stream_event(
                            &event,
                            events,
                            &mut emitted_text_start,
                            &mut accumulated_text,
                            session_id,
                            &mut block_call_ids,
                        );
                    }
                    Err(_) => {
                        non_json_output.push(line);
                    }
                }
            })
            .await?;

        if emitted_text_start {
            events.emit_assistant_text_end(&accumulated_text, None);
        }

        // On abort the child was already killed — don't check its exit
        // status since a non-zero code is expected and should not be
        // treated as an error (soft abort keeps the session alive).
        if aborted {
            return Ok(());
        }

        // Wait for process to finish and check exit status
        let stderr_collected = stderr_task.await.unwrap_or_default();
        let error_detail = if !stderr_collected.trim().is_empty() {
            stderr_collected.trim().to_string()
        } else if !non_json_output.is_empty() {
            non_json_output.join("\n")
        } else {
            String::new()
        };
        super::wait_for_child(&mut self.child, "claude", &error_detail).await
    }

    fn should_retry_submit_error(&self, error: &AgentError) -> bool {
        Self::is_retryable_submit_error(error)
    }

    fn reset_after_submit_error(&mut self) {
        super::close_child(&mut self.child);
    }

    fn close(&mut self) {
        super::close_child(&mut self.child);
    }

    fn supports_resume(&self) -> bool {
        true
    }

    fn session_id(&self) -> Option<&str> {
        self.cli_session_id.as_deref()
    }
}

/// Process a single streaming JSON event from Claude CLI.
///
/// Claude's `stream-json` format emits newline-delimited JSON objects.
/// Key event types include:
/// - `content_block_start` / `content_block_delta` / `content_block_stop`
/// - `message_start` / `message_delta` / `message_stop`
/// - System messages with session metadata
fn process_stream_event(
    event: &Value,
    events: &EventEmitter,
    emitted_text_start: &mut bool,
    accumulated_text: &mut String,
    cli_session_id: &mut Option<String>,
    block_call_ids: &mut HashMap<u64, String>,
) {
    let event_type = event.get("type").and_then(Value::as_str).unwrap_or("");

    match event_type {
        // Extract session ID from message_start or result events
        "system" => {
            if let Some(session_id) = event
                .get("session_id")
                .or_else(|| event.pointer("/message/session_id"))
                .and_then(Value::as_str)
            {
                *cli_session_id = Some(session_id.to_string());
            }
        }

        // Content block events
        "content_block_start" => {
            let block_type = event
                .pointer("/content_block/type")
                .and_then(Value::as_str)
                .unwrap_or("");

            match block_type {
                "tool_use" => {
                    let tool_name = event
                        .pointer("/content_block/name")
                        .and_then(Value::as_str)
                        .unwrap_or("unknown");
                    let call_id = event
                        .pointer("/content_block/id")
                        .and_then(Value::as_str)
                        .unwrap_or("");
                    if let Some(index) = event.get("index").and_then(Value::as_u64) {
                        block_call_ids.insert(index, call_id.to_string());
                    }
                    events.emit_tool_call_start(tool_name, call_id, &Value::Null);
                }
                "thinking" => {
                    events.emit_assistant_reasoning_start();
                }
                _ => {}
            }
        }

        "content_block_delta" => {
            let delta_type = event
                .pointer("/delta/type")
                .and_then(Value::as_str)
                .unwrap_or("");

            match delta_type {
                "text_delta" => {
                    if let Some(text) = event.pointer("/delta/text").and_then(Value::as_str) {
                        if !*emitted_text_start {
                            events.emit_assistant_text_start();
                            *emitted_text_start = true;
                        }
                        events.emit_assistant_text_delta(text);
                        accumulated_text.push_str(text);
                    }
                }
                "thinking_delta" => {
                    if let Some(thinking) = event.pointer("/delta/thinking").and_then(Value::as_str)
                    {
                        events.emit_assistant_reasoning_delta(thinking);
                    }
                }
                "input_json_delta" => {
                    // Tool input streaming — emit as tool output delta
                    if let Some(json) = event.pointer("/delta/partial_json").and_then(Value::as_str)
                    {
                        let index = event.get("index").and_then(Value::as_u64);
                        let call_id = index
                            .and_then(|i| block_call_ids.get(&i))
                            .cloned()
                            .unwrap_or_default();
                        events.emit_tool_call_output_delta(&call_id, json);
                    }
                }
                _ => {}
            }
        }

        "content_block_stop" => {
            let index = event.get("index").and_then(Value::as_u64).unwrap_or(0);
            // We don't have reliable block type info at stop time,
            // so we emit reasoning end conservatively
            let _ = index; // Used for correlation if needed
        }

        // Message-level events
        "message_start" => {
            // Extract session ID from message metadata
            if let Some(session_id) = event.pointer("/message/session_id").and_then(Value::as_str) {
                *cli_session_id = Some(session_id.to_string());
            }
        }

        "message_stop" | "message_delta" => {
            // Message complete — assistant text end is emitted in submit()
        }

        // Result event (non-streaming final output)
        "result" => {
            // Extract session ID
            if let Some(session_id) = event.get("session_id").and_then(Value::as_str) {
                *cli_session_id = Some(session_id.to_string());
            }

            // If there's a result text, emit it
            if let Some(result) = event.get("result").and_then(Value::as_str) {
                if !*emitted_text_start {
                    events.emit_assistant_text_start();
                    *emitted_text_start = true;
                }
                events.emit_assistant_text_delta(result);
                accumulated_text.push_str(result);
            }
        }

        _ => {} // Unknown event types are silently ignored
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_command_basic() {
        let config = CliProviderConfig {
            model: Some("claude-sonnet-4-5".to_string()),
            instructions: None,
            max_turns: None,
            working_dir: None,
        };
        let provider = ClaudeCliProvider::new(config);
        let cmd = provider.build_command();
        let program = cmd.as_std().get_program();
        assert_eq!(program, "claude");

        let args: Vec<_> = cmd.as_std().get_args().collect();
        assert!(args.contains(&std::ffi::OsStr::new("--print")));
        assert!(args.contains(&std::ffi::OsStr::new("--verbose")));
        assert!(args.contains(&std::ffi::OsStr::new("stream-json")));
        assert!(args.contains(&std::ffi::OsStr::new("--model")));
        assert!(args.contains(&std::ffi::OsStr::new("claude-sonnet-4-5")));
        // Prompt is piped via stdin, not as a positional arg
        assert!(!args.contains(&std::ffi::OsStr::new("hello")));
    }

    #[test]
    fn test_build_command_with_resume() {
        let config = CliProviderConfig {
            model: None,
            instructions: Some("Be helpful".to_string()),
            max_turns: Some(10),
            working_dir: None,
        };
        let mut provider = ClaudeCliProvider::new(config);
        provider.cli_session_id = Some("test-session-123".to_string());

        let cmd = provider.build_command();
        let args: Vec<_> = cmd.as_std().get_args().collect();
        assert!(args.contains(&std::ffi::OsStr::new("--resume")));
        assert!(args.contains(&std::ffi::OsStr::new("test-session-123")));
        assert!(args.contains(&std::ffi::OsStr::new("--append-system-prompt")));
        assert!(args.contains(&std::ffi::OsStr::new("--max-turns")));
        assert!(args.contains(&std::ffi::OsStr::new("10")));
    }

    #[test]
    fn test_process_text_delta_event() {
        let config = CliProviderConfig {
            model: None,
            instructions: None,
            max_turns: None,
            working_dir: None,
        };
        let mut provider = ClaudeCliProvider::new(config);
        let (emitter, mut receiver) = crate::events::channel_with_id("test".to_string());

        let event: Value = serde_json::json!({
            "type": "content_block_delta",
            "delta": {
                "type": "text_delta",
                "text": "Hello world"
            }
        });

        let mut emitted_text_start = false;
        let mut accumulated_text = String::new();
        let mut block_call_ids = HashMap::new();
        process_stream_event(
            &event,
            &emitter,
            &mut emitted_text_start,
            &mut accumulated_text,
            &mut provider.cli_session_id,
            &mut block_call_ids,
        );

        assert!(emitted_text_start);

        // Should have emitted TEXT_START and TEXT_DELTA
        let evt1 = receiver.try_recv().expect("should have text_start event");
        assert_eq!(evt1.kind, crate::types::EventKind::AssistantTextStart);

        let evt2 = receiver.try_recv().expect("should have text_delta event");
        assert_eq!(evt2.kind, crate::types::EventKind::AssistantTextDelta);
        assert_eq!(
            evt2.data.get("delta").and_then(Value::as_str),
            Some("Hello world")
        );
    }

    #[test]
    fn test_process_session_id_extraction() {
        let config = CliProviderConfig {
            model: None,
            instructions: None,
            max_turns: None,
            working_dir: None,
        };
        let mut provider = ClaudeCliProvider::new(config);
        let (emitter, _receiver) = crate::events::channel_with_id("test".to_string());

        let event: Value = serde_json::json!({
            "type": "result",
            "session_id": "abc-123-def",
            "result": "Done"
        });

        let mut emitted_text_start = false;
        let mut accumulated_text = String::new();
        let mut block_call_ids = HashMap::new();
        process_stream_event(
            &event,
            &emitter,
            &mut emitted_text_start,
            &mut accumulated_text,
            &mut provider.cli_session_id,
            &mut block_call_ids,
        );

        assert_eq!(provider.cli_session_id.as_deref(), Some("abc-123-def"));
    }

    #[test]
    fn test_process_tool_use_event() {
        let config = CliProviderConfig {
            model: None,
            instructions: None,
            max_turns: None,
            working_dir: None,
        };
        let mut provider = ClaudeCliProvider::new(config);
        let (emitter, mut receiver) = crate::events::channel_with_id("test".to_string());

        let event: Value = serde_json::json!({
            "type": "content_block_start",
            "content_block": {
                "type": "tool_use",
                "id": "call_123",
                "name": "read_file"
            }
        });

        let mut emitted_text_start = false;
        let mut accumulated_text = String::new();
        let mut block_call_ids = HashMap::new();
        process_stream_event(
            &event,
            &emitter,
            &mut emitted_text_start,
            &mut accumulated_text,
            &mut provider.cli_session_id,
            &mut block_call_ids,
        );

        let evt = receiver
            .try_recv()
            .expect("should have tool_call_start event");
        assert_eq!(evt.kind, crate::types::EventKind::ToolCallStart);
        assert_eq!(
            evt.data.get("tool_name").and_then(Value::as_str),
            Some("read_file")
        );
    }

    #[test]
    fn test_process_thinking_event() {
        let config = CliProviderConfig {
            model: None,
            instructions: None,
            max_turns: None,
            working_dir: None,
        };
        let mut provider = ClaudeCliProvider::new(config);
        let (emitter, mut receiver) = crate::events::channel_with_id("test".to_string());

        // Start thinking
        let start_event: Value = serde_json::json!({
            "type": "content_block_start",
            "content_block": {
                "type": "thinking"
            }
        });
        let mut emitted_text_start = false;
        let mut accumulated_text = String::new();
        let mut block_call_ids = HashMap::new();
        process_stream_event(
            &start_event,
            &emitter,
            &mut emitted_text_start,
            &mut accumulated_text,
            &mut provider.cli_session_id,
            &mut block_call_ids,
        );

        let evt = receiver
            .try_recv()
            .expect("should have reasoning_start event");
        assert_eq!(evt.kind, crate::types::EventKind::AssistantReasoningStart);

        // Thinking delta
        let delta_event: Value = serde_json::json!({
            "type": "content_block_delta",
            "delta": {
                "type": "thinking_delta",
                "thinking": "Let me think..."
            }
        });
        process_stream_event(
            &delta_event,
            &emitter,
            &mut emitted_text_start,
            &mut accumulated_text,
            &mut provider.cli_session_id,
            &mut block_call_ids,
        );

        let evt = receiver
            .try_recv()
            .expect("should have reasoning_delta event");
        assert_eq!(evt.kind, crate::types::EventKind::AssistantReasoningDelta);
        assert_eq!(
            evt.data.get("delta").and_then(Value::as_str),
            Some("Let me think...")
        );
    }

    #[test]
    fn test_should_retry_submit_error_for_parse_error() {
        let err = AgentError::CliParseError {
            message: "invalid stream json".to_string(),
        };
        assert!(ClaudeCliProvider::is_retryable_submit_error(&err));
    }

    #[test]
    fn test_should_not_retry_submit_error_for_cli_not_found() {
        let err = AgentError::CliNotFound {
            binary: "claude".to_string(),
        };
        assert!(!ClaudeCliProvider::is_retryable_submit_error(&err));
    }
}
