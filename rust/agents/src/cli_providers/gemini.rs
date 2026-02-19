//! Gemini CLI provider — delegates to the `gemini` CLI tool.
//!
//! Spawns the `gemini` CLI as a subprocess and collects its output.
//! The Gemini CLI handles its own tool execution loop; this adapter
//! observes and streams the response text.
//!
//! Note: The Gemini CLI's streaming format and session management
//! capabilities may vary by version. This adapter handles both
//! simple text output and structured JSON output if available.

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
// GeminiCliProvider
// ---------------------------------------------------------------------------

/// Provider that delegates to the `gemini` CLI tool.
#[derive(Debug)]
pub struct GeminiCliProvider {
    config: CliProviderConfig,
    /// Handle to the running subprocess.
    child: Option<Child>,
}

impl GeminiCliProvider {
    /// Create a new Gemini CLI provider with the given configuration.
    pub fn new(config: CliProviderConfig) -> Self {
        Self {
            config,
            child: None,
        }
    }

    /// Build the `gemini` command with appropriate flags.
    ///
    /// The prompt is piped via stdin (not passed as a positional argument).
    /// The caller must write the input to the child's stdin after spawning.
    fn build_command(&self) -> Command {
        let mut cmd = Command::new("gemini");

        // Model selection
        if let Some(ref model) = self.config.model {
            cmd.arg("--model").arg(model);
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

/// Process a line of output from the Gemini CLI.
///
/// Attempts to parse as JSON first (for structured output modes),
/// falls back to treating as plain text.
fn process_output_line(
    line: &str,
    events: &EventEmitter,
    emitted_text_start: &mut bool,
    accumulated_text: &mut String,
) {
    // Try parsing as JSON for structured output
    if let Ok(json) = serde_json::from_str::<Value>(line) {
        let msg_type = json.get("type").and_then(Value::as_str).unwrap_or("");

        match msg_type {
            "model-output" => {
                if let Some(delta) = json.get("textDelta").and_then(Value::as_str) {
                    if !*emitted_text_start {
                        events.emit_assistant_text_start();
                        *emitted_text_start = true;
                    }
                    events.emit_assistant_text_delta(delta);
                    accumulated_text.push_str(delta);
                }
            }

            "tool-call" => {
                let tool_name = json
                    .get("toolName")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown");
                let call_id = json.get("callId").and_then(Value::as_str).unwrap_or("");
                let args = json.get("args").cloned().unwrap_or(Value::Null);
                events.emit_tool_call_start(tool_name, call_id, &args);
            }

            "tool-result" => {
                let call_id = json.get("callId").and_then(Value::as_str).unwrap_or("");
                let result = json
                    .get("result")
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                events.emit_tool_call_end(call_id, &result);
            }

            "event" => {
                if json.get("name").and_then(Value::as_str) == Some("thinking")
                    && let Some(text) = json.pointer("/payload/text").and_then(Value::as_str)
                {
                    events.emit_assistant_reasoning_delta(text);
                }
            }

            "status" => {
                let status = json.get("status").and_then(Value::as_str).unwrap_or("");
                if status == "error" {
                    let detail = json
                        .get("detail")
                        .map(|v| v.to_string())
                        .unwrap_or_else(|| "Unknown error".to_string());
                    events.emit_error("GEMINI_CLI_ERROR", detail);
                }
            }

            _ => {
                // Unknown structured message — treat as text if it has content
                if let Some(text) = json.get("text").and_then(Value::as_str) {
                    if !*emitted_text_start {
                        events.emit_assistant_text_start();
                        *emitted_text_start = true;
                    }
                    events.emit_assistant_text_delta(text);
                    accumulated_text.push_str(text);
                }
            }
        }
    } else {
        // Plain text output — emit as text delta
        if !line.trim().is_empty() {
            if !*emitted_text_start {
                events.emit_assistant_text_start();
                *emitted_text_start = true;
            }
            events.emit_assistant_text_delta(line);
            events.emit_assistant_text_delta("\n");
            accumulated_text.push_str(line);
            accumulated_text.push('\n');
        }
    }
}

#[async_trait]
impl super::CliProvider for GeminiCliProvider {
    fn id(&self) -> &str {
        "gemini-cli"
    }

    async fn submit(
        &mut self,
        input: &str,
        events: &EventEmitter,
        abort: Option<&AbortSignal>,
    ) -> AgentResult<()> {
        require_cli("gemini")?;

        let mut cmd = self.build_command();
        let mut child = cmd.spawn().map_err(|e| AgentError::CliProcessFailed {
            code: -1,
            stderr: format!("Failed to spawn gemini: {e}"),
        })?;

        // Build the full prompt: prepend system instructions if present
        // (the Gemini CLI has no --system-instruction flag).
        let full_prompt = if let Some(ref instructions) = self.config.instructions {
            format!("{instructions}\n\n{input}")
        } else {
            input.to_string()
        };

        // Pipe the prompt via stdin, then close to signal EOF
        if let Some(mut stdin) = child.stdin.take() {
            use tokio::io::AsyncWriteExt;
            let _ = stdin.write_all(full_prompt.as_bytes()).await;
            let _ = stdin.flush().await;
            // stdin is dropped here, closing the pipe
        }

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AgentError::CliParseError {
                message: "Failed to capture gemini stdout".to_string(),
            })?;

        // Collect stderr in a background task so the OS pipe buffer never
        // fills and blocks the subprocess.  The collected output is used as
        // error detail when the process exits with a non-zero status.
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

        let aborted = super::read_lines_until_eof_or_abort(
            &mut lines,
            &mut self.child,
            abort,
            "gemini",
            |line| {
                process_output_line(
                    &line,
                    events,
                    &mut emitted_text_start,
                    &mut accumulated_text,
                );
            },
        )
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

        // Wait for process to finish
        let stderr_collected = stderr_task.await.unwrap_or_default();
        super::wait_for_child(&mut self.child, "gemini", stderr_collected.trim()).await
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
        false
    }

    fn session_id(&self) -> Option<&str> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_command_basic() {
        let config = CliProviderConfig {
            model: Some("gemini-2.5-pro".to_string()),
            instructions: None,
            max_turns: None,
            working_dir: None,
        };
        let provider = GeminiCliProvider::new(config);
        let cmd = provider.build_command();
        let program = cmd.as_std().get_program();
        assert_eq!(program, "gemini");

        let args: Vec<_> = cmd.as_std().get_args().collect();
        assert!(args.contains(&std::ffi::OsStr::new("--model")));
        assert!(args.contains(&std::ffi::OsStr::new("gemini-2.5-pro")));
        // Prompt is piped via stdin, not as a positional arg
        assert!(!args.contains(&std::ffi::OsStr::new("hello")));
    }

    #[test]
    fn test_process_plain_text_output() {
        let (emitter, mut receiver) = crate::events::channel_with_id("test".to_string());

        let mut emitted_text_start = false;
        let mut accumulated_text = String::new();
        process_output_line(
            "Hello from Gemini",
            &emitter,
            &mut emitted_text_start,
            &mut accumulated_text,
        );

        assert!(emitted_text_start);

        let evt1 = receiver.try_recv().expect("should have text_start");
        assert_eq!(evt1.kind, crate::types::EventKind::AssistantTextStart);

        let evt2 = receiver.try_recv().expect("should have text_delta");
        assert_eq!(evt2.kind, crate::types::EventKind::AssistantTextDelta);
        assert_eq!(
            evt2.data.get("delta").and_then(Value::as_str),
            Some("Hello from Gemini")
        );
    }

    #[test]
    fn test_process_json_model_output() {
        let (emitter, mut receiver) = crate::events::channel_with_id("test".to_string());

        let json_line = r#"{"type":"model-output","textDelta":"Structured response"}"#;
        let mut emitted_text_start = false;
        let mut accumulated_text = String::new();
        process_output_line(
            json_line,
            &emitter,
            &mut emitted_text_start,
            &mut accumulated_text,
        );

        assert!(emitted_text_start);

        let evt1 = receiver.try_recv().expect("should have text_start");
        assert_eq!(evt1.kind, crate::types::EventKind::AssistantTextStart);

        let evt2 = receiver.try_recv().expect("should have text_delta");
        assert_eq!(evt2.kind, crate::types::EventKind::AssistantTextDelta);
        assert_eq!(
            evt2.data.get("delta").and_then(Value::as_str),
            Some("Structured response")
        );
    }

    #[test]
    fn test_process_tool_call_event() {
        let (emitter, mut receiver) = crate::events::channel_with_id("test".to_string());

        let json_line = r#"{"type":"tool-call","toolName":"read_file","callId":"tc-1","args":{"path":"test.txt"}}"#;
        let mut emitted_text_start = false;
        let mut accumulated_text = String::new();
        process_output_line(
            json_line,
            &emitter,
            &mut emitted_text_start,
            &mut accumulated_text,
        );

        let evt = receiver.try_recv().expect("should have tool_call_start");
        assert_eq!(evt.kind, crate::types::EventKind::ToolCallStart);
        assert_eq!(
            evt.data.get("tool_name").and_then(Value::as_str),
            Some("read_file")
        );
    }

    #[test]
    fn test_process_empty_line_ignored() {
        let (emitter, mut receiver) = crate::events::channel_with_id("test".to_string());

        let mut emitted_text_start = false;
        let mut accumulated_text = String::new();
        process_output_line("", &emitter, &mut emitted_text_start, &mut accumulated_text);
        process_output_line(
            "   ",
            &emitter,
            &mut emitted_text_start,
            &mut accumulated_text,
        );

        assert!(!emitted_text_start);
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn test_should_retry_submit_error_for_parse_error() {
        let err = AgentError::CliParseError {
            message: "invalid json".to_string(),
        };
        assert!(GeminiCliProvider::is_retryable_submit_error(&err));
    }

    #[test]
    fn test_should_not_retry_submit_error_for_cli_not_found() {
        let err = AgentError::CliNotFound {
            binary: "gemini".to_string(),
        };
        assert!(!GeminiCliProvider::is_retryable_submit_error(&err));
    }
}
