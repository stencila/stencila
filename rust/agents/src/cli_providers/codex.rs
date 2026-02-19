//! Codex CLI provider — delegates to the `codex` CLI tool via MCP.
//!
//! Spawns `codex mcp-server` (or `codex mcp` for older versions) as a
//! long-lived MCP server and communicates via JSON-RPC over stdin/stdout.
//! Sessions are started by calling the `codex` tool and continued with
//! `codex-reply`.
//!
//! The Codex CLI handles its own tool execution loop; this adapter observes
//! events via MCP notifications (`codex/event` method).

use std::process::Stdio;
use std::sync::atomic::{AtomicI64, Ordering};

use async_trait::async_trait;
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};

use super::{CliProviderConfig, cli_version, require_cli};
use crate::error::{AgentError, AgentResult};
use crate::events::EventEmitter;
use crate::types::AbortSignal;

// ---------------------------------------------------------------------------
// CodexCliProvider
// ---------------------------------------------------------------------------

/// Provider that delegates to the `codex` CLI via MCP server mode.
#[derive(Debug)]
pub struct CodexCliProvider {
    config: CliProviderConfig,
    /// The long-lived MCP server subprocess.
    child: Option<Child>,
    /// Codex session ID for continuing conversations.
    codex_session_id: Option<String>,
    /// Codex conversation ID for continuing conversations.
    codex_conversation_id: Option<String>,
    /// Whether the MCP server has been initialized.
    initialized: bool,
    /// JSON-RPC request ID counter.
    #[allow(dead_code)]
    next_id: AtomicI64,
    /// Which subcommand to use: "mcp-server" (>= 0.43.0) or "mcp" (older).
    mcp_subcommand: String,
}

impl CodexCliProvider {
    /// Create a new Codex CLI provider with the given configuration.
    pub fn new(config: CliProviderConfig) -> Self {
        Self {
            config,
            child: None,
            codex_session_id: None,
            codex_conversation_id: None,
            initialized: false,
            next_id: AtomicI64::new(1),
            mcp_subcommand: "mcp-server".to_string(),
        }
    }

    /// Detect the correct MCP subcommand based on codex version.
    async fn detect_mcp_subcommand(&mut self) -> AgentResult<()> {
        if let Some(version_str) = cli_version("codex").await {
            let threshold = semver::Version::new(0, 43, 0);
            if version_less_than(&version_str, &threshold) {
                self.mcp_subcommand = "mcp".to_string();
            }
        }
        Ok(())
    }

    /// Spawn the MCP server process if not already running.
    async fn ensure_server(&mut self) -> AgentResult<()> {
        if self.child.is_some() {
            return Ok(());
        }

        self.detect_mcp_subcommand().await?;

        let mut cmd = Command::new("codex");

        // --model must be passed before the subcommand
        if let Some(ref model) = self.config.model {
            cmd.arg("--model").arg(model);
        }

        cmd.arg(&self.mcp_subcommand);
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| AgentError::CliProcessFailed {
            code: -1,
            stderr: format!("Failed to spawn codex {}: {e}", self.mcp_subcommand),
        })?;

        // Drain stderr in a background task so the OS pipe buffer never
        // fills and blocks the long-lived MCP server process.
        if let Some(stderr) = child.stderr.take() {
            tokio::spawn(async move {
                let mut buf = String::new();
                let _ = tokio::io::AsyncReadExt::read_to_string(&mut { stderr }, &mut buf).await;
            });
        }

        self.child = Some(child);
        Ok(())
    }

    /// Send a JSON-RPC request to the MCP server and return the request ID.
    async fn send_jsonrpc(&mut self, method: &str, params: Value) -> AgentResult<i64> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params,
        });

        let mut request_str =
            serde_json::to_string(&request).map_err(|e| AgentError::CliParseError {
                message: format!("Failed to serialize JSON-RPC request: {e}"),
            })?;
        request_str.push('\n');

        let child = self
            .child
            .as_mut()
            .ok_or_else(|| AgentError::CliProcessFailed {
                code: -1,
                stderr: "Codex MCP server not running".to_string(),
            })?;

        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| AgentError::CliParseError {
                message: "Failed to access codex stdin".to_string(),
            })?;

        stdin.write_all(request_str.as_bytes()).await.map_err(|e| {
            AgentError::CliProcessFailed {
                code: -1,
                stderr: format!("Failed to write to codex stdin: {e}"),
            }
        })?;

        stdin
            .flush()
            .await
            .map_err(|e| AgentError::CliProcessFailed {
                code: -1,
                stderr: format!("Failed to flush codex stdin: {e}"),
            })?;

        Ok(id)
    }

    /// Initialize the MCP server with the `initialize` handshake.
    async fn initialize(&mut self) -> AgentResult<()> {
        if self.initialized {
            return Ok(());
        }

        let params = serde_json::json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "stencila",
                "version": env!("CARGO_PKG_VERSION")
            }
        });

        self.send_jsonrpc("initialize", params).await?;

        // Send initialized notification
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized",
            "params": {}
        });

        let mut notification_str =
            serde_json::to_string(&notification).map_err(|e| AgentError::CliParseError {
                message: format!("Failed to serialize notification: {e}"),
            })?;
        notification_str.push('\n');

        let child = self
            .child
            .as_mut()
            .ok_or_else(|| AgentError::CliProcessFailed {
                code: -1,
                stderr: "Codex MCP server not running".to_string(),
            })?;

        if let Some(stdin) = child.stdin.as_mut() {
            let _ = stdin.write_all(notification_str.as_bytes()).await;
            let _ = stdin.flush().await;
        }

        self.initialized = true;
        Ok(())
    }

    /// Build the tool call parameters for starting or continuing a session.
    fn build_session_params(&self, prompt: &str) -> (String, Value) {
        if let (Some(session_id), Some(conversation_id)) =
            (&self.codex_session_id, &self.codex_conversation_id)
        {
            // Continue existing session
            let params = serde_json::json!({
                "sessionId": session_id,
                "conversationId": conversation_id,
                "prompt": prompt,
            });
            ("codex-reply".to_string(), params)
        } else {
            // Start new session
            let mut params = serde_json::json!({
                "prompt": prompt,
            });

            if let Some(ref model) = self.config.model {
                params["model"] = Value::String(model.clone());
            }

            if let Some(ref instructions) = self.config.instructions {
                params["base-instructions"] = Value::String(instructions.clone());
            }

            ("codex".to_string(), params)
        }
    }

    /// Process a codex/event notification and emit appropriate agent events.
    fn process_codex_event(
        &mut self,
        msg: &Value,
        events: &EventEmitter,
        emitted_text_start: &mut bool,
        accumulated_text: &mut String,
    ) {
        let msg_type = msg.get("type").and_then(Value::as_str).unwrap_or("");

        match msg_type {
            "agent_message" => {
                if let Some(text) = msg.get("text").and_then(Value::as_str) {
                    if !*emitted_text_start {
                        events.emit_assistant_text_start();
                        *emitted_text_start = true;
                    }
                    events.emit_assistant_text_delta(text);
                    accumulated_text.push_str(text);
                }
            }

            "agent_reasoning" | "agent_reasoning_delta" => {
                if let Some(delta) = msg.get("delta").and_then(Value::as_str) {
                    events.emit_assistant_reasoning_delta(delta);
                }
            }

            "exec_command_begin" => {
                let command = msg
                    .get("command")
                    .and_then(Value::as_str)
                    .unwrap_or("shell");
                let call_id = msg.get("call_id").and_then(Value::as_str).unwrap_or("");
                events.emit_tool_call_start(command, call_id, &Value::Null);
            }

            "exec_command_end" => {
                let call_id = msg.get("call_id").and_then(Value::as_str).unwrap_or("");
                let output = msg.get("output").and_then(Value::as_str).unwrap_or("");
                events.emit_tool_call_end(call_id, output);
            }

            "patch_apply_begin" => {
                let call_id = msg.get("call_id").and_then(Value::as_str).unwrap_or("");
                events.emit_tool_call_start("patch_apply", call_id, &Value::Null);
            }

            "patch_apply_end" => {
                let call_id = msg.get("call_id").and_then(Value::as_str).unwrap_or("");
                events.emit_tool_call_end(call_id, "");
            }

            "task_started" => {
                // Extract session info if present
                if let Some(session_id) = msg.get("sessionId").and_then(Value::as_str) {
                    self.codex_session_id = Some(session_id.to_string());
                }
                if let Some(conversation_id) = msg.get("conversationId").and_then(Value::as_str) {
                    self.codex_conversation_id = Some(conversation_id.to_string());
                }
            }

            "task_complete" => {
                // Session info may also appear on completion
                if let Some(session_id) = msg.get("sessionId").and_then(Value::as_str) {
                    self.codex_session_id = Some(session_id.to_string());
                }
                if let Some(conversation_id) = msg.get("conversationId").and_then(Value::as_str) {
                    self.codex_conversation_id = Some(conversation_id.to_string());
                }
            }

            _ => {} // Unknown event types silently ignored
        }
    }

    /// Read and process lines from the MCP server until the tool call completes.
    async fn read_until_complete(
        &mut self,
        request_id: i64,
        events: &EventEmitter,
        abort: Option<&AbortSignal>,
    ) -> AgentResult<()> {
        let child = self
            .child
            .as_mut()
            .ok_or_else(|| AgentError::CliProcessFailed {
                code: -1,
                stderr: "Codex MCP server not running".to_string(),
            })?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AgentError::CliParseError {
                message: "Failed to capture codex stdout".to_string(),
            })?;

        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        let mut emitted_text_start = false;
        let mut accumulated_text = String::new();

        let mut aborted = false;
        loop {
            if let Some(signal) = abort
                && signal.is_aborted()
            {
                aborted = true;
                break;
            }

            let line = tokio::select! {
                line = lines.next_line() => line,
                () = async {
                    if let Some(signal) = abort {
                        signal.cancelled().await;
                    } else {
                        std::future::pending::<()>().await;
                    }
                } => {
                    aborted = true;
                    break;
                }
            };

            let line = match line {
                Ok(Some(line)) => line,
                Ok(None) => break,
                Err(e) => {
                    return Err(AgentError::CliParseError {
                        message: format!("Failed to read codex output: {e}"),
                    });
                }
            };

            if line.trim().is_empty() {
                continue;
            }

            let msg: Value = match serde_json::from_str(&line) {
                Ok(v) => v,
                Err(_) => continue,
            };

            // Check if this is a JSON-RPC response to our request
            if let Some(id) = msg.get("id").and_then(Value::as_i64)
                && id == request_id
            {
                // Check for errors
                if let Some(error) = msg.get("error") {
                    let error_msg = error
                        .get("message")
                        .and_then(Value::as_str)
                        .unwrap_or("Unknown MCP error");
                    if emitted_text_start {
                        events.emit_assistant_text_end(&accumulated_text, None);
                    }
                    return Err(AgentError::CliProcessFailed {
                        code: error.get("code").and_then(Value::as_i64).unwrap_or(-1) as i32,
                        stderr: error_msg.to_string(),
                    });
                }

                // Extract content from result
                if let Some(content) = msg.pointer("/result/content")
                    && let Some(arr) = content.as_array()
                {
                    for item in arr {
                        if let Some(text) = item.get("text").and_then(Value::as_str) {
                            if !emitted_text_start {
                                events.emit_assistant_text_start();
                                emitted_text_start = true;
                            }
                            events.emit_assistant_text_delta(text);
                            accumulated_text.push_str(text);
                        }

                        // Extract session/conversation IDs
                        if let Some(sid) = item.get("sessionId").and_then(Value::as_str) {
                            self.codex_session_id = Some(sid.to_string());
                        }
                        if let Some(cid) = item.get("conversationId").and_then(Value::as_str) {
                            self.codex_conversation_id = Some(cid.to_string());
                        }
                    }
                }

                if emitted_text_start {
                    events.emit_assistant_text_end(&accumulated_text, None);
                }

                // Put stdout back
                let child = self
                    .child
                    .as_mut()
                    .ok_or_else(|| AgentError::CliProcessFailed {
                        code: -1,
                        stderr: "Lost codex process reference".to_string(),
                    })?;
                child.stdout = Some(lines.into_inner().into_inner());
                return Ok(());
            }

            // Check for codex/event notifications
            if msg.get("method").and_then(Value::as_str) == Some("codex/event")
                && let Some(params) = msg.get("params")
                && let Some(event_msg) = params.get("msg")
            {
                self.process_codex_event(
                    event_msg,
                    events,
                    &mut emitted_text_start,
                    &mut accumulated_text,
                );
            }
        }

        if emitted_text_start {
            events.emit_assistant_text_end(&accumulated_text, None);
        }

        // If we reach here without having been aborted, stdout hit EOF
        // before we received the matching JSON-RPC response — the MCP
        // server crashed or exited unexpectedly.
        if !aborted {
            // Put stdout back before returning so Drop doesn't panic.
            if let Some(child) = self.child.as_mut() {
                child.stdout = Some(lines.into_inner().into_inner());
            }
            return Err(AgentError::CliProcessFailed {
                code: -1,
                stderr: "Codex MCP server exited before returning a response".to_string(),
            });
        }

        // On abort, drain stdout until the in-flight response arrives so
        // the pipe is clean for the next submit.  Use a short timeout to
        // avoid blocking indefinitely if the server is slow.
        {
            let drain_deadline =
                tokio::time::Instant::now() + std::time::Duration::from_millis(500);

            loop {
                let line = tokio::select! {
                    line = lines.next_line() => line,
                    () = tokio::time::sleep_until(drain_deadline) => break,
                };

                match line {
                    Ok(Some(line)) if !line.trim().is_empty() => {
                        if let Ok(msg) = serde_json::from_str::<Value>(&line)
                            && msg.get("id").and_then(Value::as_i64) == Some(request_id)
                        {
                            // Found the response — pipe is now clean.
                            break;
                        }
                    }
                    // EOF or error — nothing more to drain.
                    _ => break,
                }
            }
        }

        // Put stdout back if we still have the child
        if let Some(child) = self.child.as_mut() {
            child.stdout = Some(lines.into_inner().into_inner());
        }

        Ok(())
    }

    /// Submit a single prompt without recovery retries.
    async fn submit_once(
        &mut self,
        input: &str,
        events: &EventEmitter,
        abort: Option<&AbortSignal>,
    ) -> AgentResult<()> {
        self.ensure_server().await?;
        self.initialize().await?;

        let (tool_name, arguments) = self.build_session_params(input);

        // Call the tool via MCP tools/call
        let params = serde_json::json!({
            "name": tool_name,
            "arguments": arguments,
        });

        let request_id = self.send_jsonrpc("tools/call", params).await?;

        // Read events and response
        self.read_until_complete(request_id, events, abort).await
    }

    /// Whether a submit error is likely recoverable by restarting the MCP
    /// subprocess and opening a fresh Codex conversation.
    fn is_retryable_submit_error(error: &AgentError) -> bool {
        match error {
            AgentError::CliParseError { .. } => true,
            AgentError::CliProcessFailed { code, stderr } => {
                *code == -1
                    || stderr.contains("before returning a response")
                    || stderr.to_ascii_lowercase().contains("tool")
            }
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

    /// Reset codex subprocess + conversation state so the next submit starts
    /// from a clean server and a fresh Codex session.
    fn reset_state_after_submit_error(&mut self) {
        super::close_child(&mut self.child);
        self.initialized = false;
        self.codex_session_id = None;
        self.codex_conversation_id = None;
    }
}

#[async_trait]
impl super::CliProvider for CodexCliProvider {
    fn id(&self) -> &str {
        "codex-cli"
    }

    async fn submit(
        &mut self,
        input: &str,
        events: &EventEmitter,
        abort: Option<&AbortSignal>,
    ) -> AgentResult<()> {
        require_cli("codex")?;
        self.submit_once(input, events, abort).await
    }

    fn should_retry_submit_error(&self, error: &AgentError) -> bool {
        Self::is_retryable_submit_error(error)
    }

    fn reset_after_submit_error(&mut self) {
        self.reset_state_after_submit_error();
    }

    fn close(&mut self) {
        super::close_child(&mut self.child);
        self.initialized = false;
    }

    fn supports_resume(&self) -> bool {
        true
    }

    fn session_id(&self) -> Option<&str> {
        self.codex_session_id.as_deref()
    }
}

/// Parse a version string (possibly prefixed, e.g. "codex v0.42.1") and
/// check if it's less than `threshold`.
fn version_less_than(version_str: &str, threshold: &semver::Version) -> bool {
    // Skip any non-digit prefix (e.g. "codex v")
    let Some(start) = version_str.find(|c: char| c.is_ascii_digit()) else {
        return false;
    };
    semver::Version::parse(&version_str[start..]).is_ok_and(|ver| ver < *threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_less_than() {
        let threshold = semver::Version::new(0, 43, 0);
        assert!(version_less_than("0.42.0", &threshold));
        assert!(version_less_than("codex v0.42.1", &threshold));
        assert!(version_less_than("0.42.9-alpha.5", &threshold));
        assert!(!version_less_than("0.43.0", &threshold));
        assert!(!version_less_than("0.44.0", &threshold));
        assert!(!version_less_than("1.0.0", &threshold));
        // semver: pre-release < release, so 0.43.0-alpha.5 < 0.43.0
        assert!(version_less_than("0.43.0-alpha.5", &threshold));
    }

    #[test]
    fn test_build_session_params_new() {
        let config = CliProviderConfig {
            model: Some("o3".to_string()),
            instructions: Some("Be helpful".to_string()),
            max_turns: None,
            working_dir: None,
        };
        let provider = CodexCliProvider::new(config);

        let (tool, params) = provider.build_session_params("hello");
        assert_eq!(tool, "codex");
        assert_eq!(params["prompt"], "hello");
        assert_eq!(params["model"], "o3");
        assert_eq!(params["base-instructions"], "Be helpful");
    }

    #[test]
    fn test_build_session_params_continue() {
        let config = CliProviderConfig {
            model: None,
            instructions: None,
            max_turns: None,
            working_dir: None,
        };
        let mut provider = CodexCliProvider::new(config);
        provider.codex_session_id = Some("sess-1".to_string());
        provider.codex_conversation_id = Some("conv-1".to_string());

        let (tool, params) = provider.build_session_params("continue please");
        assert_eq!(tool, "codex-reply");
        assert_eq!(params["sessionId"], "sess-1");
        assert_eq!(params["conversationId"], "conv-1");
        assert_eq!(params["prompt"], "continue please");
    }

    #[test]
    fn test_process_agent_message_event() {
        let config = CliProviderConfig {
            model: None,
            instructions: None,
            max_turns: None,
            working_dir: None,
        };
        let mut provider = CodexCliProvider::new(config);
        let (emitter, mut receiver) = crate::events::channel_with_id("test".to_string());

        let msg = serde_json::json!({
            "type": "agent_message",
            "text": "Hello from Codex"
        });

        let mut emitted_text_start = false;
        let mut accumulated_text = String::new();
        provider.process_codex_event(
            &msg,
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
            Some("Hello from Codex")
        );
    }

    #[test]
    fn test_process_exec_command_events() {
        let config = CliProviderConfig {
            model: None,
            instructions: None,
            max_turns: None,
            working_dir: None,
        };
        let mut provider = CodexCliProvider::new(config);
        let (emitter, mut receiver) = crate::events::channel_with_id("test".to_string());

        let begin = serde_json::json!({
            "type": "exec_command_begin",
            "command": "ls -la",
            "call_id": "cmd-1"
        });
        let end = serde_json::json!({
            "type": "exec_command_end",
            "call_id": "cmd-1",
            "output": "file1.txt\nfile2.txt"
        });

        let mut emitted_text_start = false;
        let mut accumulated_text = String::new();
        provider.process_codex_event(
            &begin,
            &emitter,
            &mut emitted_text_start,
            &mut accumulated_text,
        );
        provider.process_codex_event(
            &end,
            &emitter,
            &mut emitted_text_start,
            &mut accumulated_text,
        );

        let evt1 = receiver.try_recv().expect("should have tool_call_start");
        assert_eq!(evt1.kind, crate::types::EventKind::ToolCallStart);

        let evt2 = receiver.try_recv().expect("should have tool_call_end");
        assert_eq!(evt2.kind, crate::types::EventKind::ToolCallEnd);
    }

    #[test]
    fn test_process_session_id_extraction() {
        let config = CliProviderConfig {
            model: None,
            instructions: None,
            max_turns: None,
            working_dir: None,
        };
        let mut provider = CodexCliProvider::new(config);
        let (emitter, _receiver) = crate::events::channel_with_id("test".to_string());

        let msg = serde_json::json!({
            "type": "task_started",
            "sessionId": "codex-sess-abc",
            "conversationId": "codex-conv-123"
        });

        let mut emitted_text_start = false;
        let mut accumulated_text = String::new();
        provider.process_codex_event(
            &msg,
            &emitter,
            &mut emitted_text_start,
            &mut accumulated_text,
        );

        assert_eq!(provider.codex_session_id.as_deref(), Some("codex-sess-abc"));
        assert_eq!(
            provider.codex_conversation_id.as_deref(),
            Some("codex-conv-123")
        );
    }

    #[test]
    fn test_should_retry_submit_error_for_parse_error() {
        let err = AgentError::CliParseError {
            message: "invalid json".to_string(),
        };
        assert!(CodexCliProvider::is_retryable_submit_error(&err));
    }

    #[test]
    fn test_should_retry_submit_error_for_process_error() {
        let err = AgentError::CliProcessFailed {
            code: -1,
            stderr: "Codex MCP server exited before returning a response".to_string(),
        };
        assert!(CodexCliProvider::is_retryable_submit_error(&err));
    }

    #[test]
    fn test_should_retry_submit_error_for_tool_stderr() {
        let err = AgentError::CliProcessFailed {
            code: 1,
            stderr: "Tool execution failed unexpectedly".to_string(),
        };
        assert!(CodexCliProvider::is_retryable_submit_error(&err));
    }

    #[test]
    fn test_should_retry_submit_error_for_tool_stderr_case_insensitive() {
        let err = AgentError::CliProcessFailed {
            code: 1,
            stderr: "TOOL call returned an error".to_string(),
        };
        assert!(CodexCliProvider::is_retryable_submit_error(&err));
    }

    #[test]
    fn test_should_not_retry_submit_error_for_cli_not_found() {
        let err = AgentError::CliNotFound {
            binary: "codex".to_string(),
        };
        assert!(!CodexCliProvider::is_retryable_submit_error(&err));
    }
}
