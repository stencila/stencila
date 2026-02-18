//! Tests for Session and core agentic loop (spec 2.1, 2.5-2.8, 2.10, Appendix B).
//!
//! Uses mock Client and execution environment for deterministic testing.
//! Test file maps to spec sections 2 and 9 (core loop, steering, reasoning,
//! error handling, parity).

#![allow(clippy::result_large_err)]

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use serde_json::json;
use stencila_models3::error::{ProviderDetails, SdkError};
use stencila_models3::types::content::ContentPart;
use stencila_models3::types::finish_reason::{FinishReason, Reason};
use stencila_models3::types::message::Message;
use stencila_models3::types::request::Request;
use stencila_models3::types::response::Response;
use stencila_models3::types::role::Role;
use stencila_models3::types::tool::{ToolCall, ToolDefinition};
use stencila_models3::types::usage::Usage;

use stencila_agents::error::{AgentError, AgentResult};
use stencila_agents::execution::{ExecutionEnvironment, FileContent};
use stencila_agents::profile::ProviderProfile;
use stencila_agents::profiles::{AnthropicProfile, GeminiProfile, OpenAiProfile};
use stencila_agents::registry::{RegisteredTool, ToolOutput, ToolRegistry};
use stencila_agents::session::{AbortController, LlmClient, Session};
use stencila_agents::types::{
    DirEntry, EventKind, ExecResult, GrepOptions, ReasoningEffort, SessionConfig, SessionState,
};

// ===========================================================================
// Mock types
// ===========================================================================

/// Mock LLM client that returns predetermined responses.
struct MockClient {
    responses: Mutex<VecDeque<Result<Response, SdkError>>>,
    requests: Mutex<Vec<Request>>,
}

impl MockClient {
    fn new(responses: Vec<Result<Response, SdkError>>) -> Self {
        Self {
            responses: Mutex::new(VecDeque::from(responses)),
            requests: Mutex::new(Vec::new()),
        }
    }

    fn take_requests(&self) -> AgentResult<Vec<Request>> {
        Ok(self
            .requests
            .lock()
            .map_err(|e| AgentError::Io {
                message: format!("mock lock: {e}"),
            })?
            .drain(..)
            .collect())
    }
}

#[async_trait]
impl LlmClient for MockClient {
    async fn complete(&self, request: Request) -> Result<Response, SdkError> {
        self.requests
            .lock()
            .map_err(|e| SdkError::Configuration {
                message: format!("mock lock: {e}"),
            })?
            .push(request);
        self.responses
            .lock()
            .map_err(|e| SdkError::Configuration {
                message: format!("mock lock: {e}"),
            })?
            .pop_front()
            .unwrap_or_else(|| {
                Err(SdkError::Configuration {
                    message: "no more mock responses".into(),
                })
            })
    }
}

/// Minimal mock execution environment for tests.
struct MockExecEnv {
    working_dir: String,
    /// When set, `read_file` returns image content instead of text.
    image_mode: bool,
}

impl MockExecEnv {
    fn new() -> Self {
        Self {
            working_dir: "/tmp/test".into(),
            image_mode: false,
        }
    }

    fn with_image_mode(mut self) -> Self {
        self.image_mode = true;
        self
    }
}

#[async_trait]
impl ExecutionEnvironment for MockExecEnv {
    async fn read_file(
        &self,
        path: &str,
        _offset: Option<usize>,
        _limit: Option<usize>,
    ) -> AgentResult<FileContent> {
        if self.image_mode {
            Ok(FileContent::Image {
                data: vec![0x89, 0x50, 0x4E, 0x47],
                media_type: "image/png".into(),
            })
        } else {
            Ok(FileContent::Text(format!("     1\t| content of {path}")))
        }
    }
    async fn write_file(&self, _path: &str, _content: &str) -> AgentResult<()> {
        Ok(())
    }
    async fn file_exists(&self, _path: &str) -> bool {
        false
    }
    async fn delete_file(&self, _path: &str) -> AgentResult<()> {
        Ok(())
    }
    async fn list_directory(&self, _path: &str, _depth: usize) -> AgentResult<Vec<DirEntry>> {
        Ok(vec![])
    }
    async fn exec_command(
        &self,
        command: &str,
        _timeout_ms: u64,
        _working_dir: Option<&str>,
        _env_vars: Option<&HashMap<String, String>>,
    ) -> AgentResult<ExecResult> {
        Ok(ExecResult {
            stdout: format!("executed: {command}"),
            stderr: String::new(),
            exit_code: 0,
            timed_out: false,
            duration_ms: 10,
        })
    }
    async fn grep(
        &self,
        _pattern: &str,
        _path: &str,
        _options: &GrepOptions,
    ) -> AgentResult<String> {
        Ok(String::new())
    }
    async fn glob_files(&self, _pattern: &str, _path: &str) -> AgentResult<Vec<String>> {
        Ok(vec![])
    }
    fn working_directory(&self) -> &str {
        &self.working_dir
    }
    fn platform(&self) -> &str {
        "linux"
    }
    fn os_version(&self) -> String {
        "test-os".into()
    }
}

/// Mock execution environment that captures exec_command calls for verification.
struct CapturingExecEnv {
    inner: MockExecEnv,
    /// Captured (command, timeout_ms) pairs from exec_command calls.
    calls: Mutex<Vec<(String, u64)>>,
}

impl CapturingExecEnv {
    fn new() -> Self {
        Self {
            inner: MockExecEnv::new(),
            calls: Mutex::new(Vec::new()),
        }
    }

    fn take_calls(&self) -> AgentResult<Vec<(String, u64)>> {
        Ok(self
            .calls
            .lock()
            .map_err(|e| AgentError::Io {
                message: e.to_string(),
            })?
            .drain(..)
            .collect())
    }
}

#[async_trait]
impl ExecutionEnvironment for CapturingExecEnv {
    async fn read_file(
        &self,
        path: &str,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> AgentResult<FileContent> {
        self.inner.read_file(path, offset, limit).await
    }
    async fn write_file(&self, path: &str, content: &str) -> AgentResult<()> {
        self.inner.write_file(path, content).await
    }
    async fn file_exists(&self, path: &str) -> bool {
        self.inner.file_exists(path).await
    }
    async fn delete_file(&self, path: &str) -> AgentResult<()> {
        self.inner.delete_file(path).await
    }
    async fn list_directory(&self, path: &str, depth: usize) -> AgentResult<Vec<DirEntry>> {
        self.inner.list_directory(path, depth).await
    }
    async fn exec_command(
        &self,
        command: &str,
        timeout_ms: u64,
        working_dir: Option<&str>,
        env_vars: Option<&HashMap<String, String>>,
    ) -> AgentResult<ExecResult> {
        if let Ok(mut calls) = self.calls.lock() {
            calls.push((command.to_string(), timeout_ms));
        }
        self.inner
            .exec_command(command, timeout_ms, working_dir, env_vars)
            .await
    }
    async fn grep(&self, pattern: &str, path: &str, options: &GrepOptions) -> AgentResult<String> {
        self.inner.grep(pattern, path, options).await
    }
    async fn glob_files(&self, pattern: &str, path: &str) -> AgentResult<Vec<String>> {
        self.inner.glob_files(pattern, path).await
    }
    fn working_directory(&self) -> &str {
        self.inner.working_directory()
    }
    fn platform(&self) -> &str {
        self.inner.platform()
    }
    fn os_version(&self) -> String {
        self.inner.os_version()
    }
}

/// Test profile with configurable tools, parallel, and streaming support.
#[derive(Debug)]
struct TestProfile {
    registry: ToolRegistry,
    model: String,
    supports_parallel: bool,
    supports_streaming: bool,
}

impl TestProfile {
    fn new() -> AgentResult<Self> {
        let mut registry = ToolRegistry::new();
        registry.register(echo_tool())?;
        Ok(Self {
            registry,
            model: "test-model".into(),
            supports_parallel: false,
            supports_streaming: true,
        })
    }

    fn with_parallel(mut self) -> Self {
        self.supports_parallel = true;
        self
    }

    fn without_streaming(mut self) -> Self {
        self.supports_streaming = false;
        self
    }

    fn with_tool(mut self, tool: RegisteredTool) -> AgentResult<Self> {
        self.registry.register(tool)?;
        Ok(self)
    }
}

impl ProviderProfile for TestProfile {
    fn id(&self) -> &str {
        "test"
    }
    fn model(&self) -> &str {
        &self.model
    }
    fn tool_registry_mut(&mut self) -> &mut ToolRegistry {
        &mut self.registry
    }
    fn tool_registry(&self) -> &ToolRegistry {
        &self.registry
    }
    fn base_instructions(&self) -> &str {
        "You are a test assistant."
    }
    fn supports_reasoning(&self) -> bool {
        true
    }
    fn supports_streaming(&self) -> bool {
        self.supports_streaming
    }
    fn supports_parallel_tool_calls(&self) -> bool {
        self.supports_parallel
    }
    fn context_window_size(&self) -> u64 {
        128_000
    }
}

// ===========================================================================
// Helpers
// ===========================================================================

/// Create a tool that echoes back its "text" argument.
fn echo_tool() -> RegisteredTool {
    RegisteredTool::new(
        ToolDefinition {
            name: "echo".into(),
            description: "Returns the text argument".into(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "text": { "type": "string" }
                },
                "required": ["text"]
            }),
            strict: false,
        },
        Box::new(|args, _env| {
            Box::pin(async move {
                let text = args
                    .get("text")
                    .and_then(|v| v.as_str())
                    .unwrap_or("no text");
                Ok(ToolOutput::Text(text.to_string()))
            })
        }),
    )
}

/// Create a tool that sleeps for the given duration before returning.
fn slow_tool(sleep_ms: u64) -> RegisteredTool {
    RegisteredTool::new(
        ToolDefinition {
            name: "slow".into(),
            description: "Sleeps then returns".into(),
            parameters: json!({"type": "object", "properties": {}}),
            strict: false,
        },
        Box::new(move |_args, _env| {
            Box::pin(async move {
                tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
                Ok(ToolOutput::Text("done".to_string()))
            })
        }),
    )
}

/// Build a ToolCall for the slow tool.
fn slow_call() -> ToolCall {
    ToolCall {
        id: format!("call-{}", uuid::Uuid::new_v4()),
        name: "slow".into(),
        arguments: json!({}),
        raw_arguments: None,
        parse_error: None,
    }
}

/// Create a tool that always returns an error.
fn failing_tool() -> RegisteredTool {
    RegisteredTool::new(
        ToolDefinition {
            name: "fail".into(),
            description: "Always fails".into(),
            parameters: json!({"type": "object", "properties": {}}),
            strict: false,
        },
        Box::new(|_args, _env| {
            Box::pin(async move {
                Err(AgentError::Io {
                    message: "intentional failure".into(),
                })
            })
        }),
    )
}

/// Build a text-only Response.
fn text_response(text: &str) -> Result<Response, SdkError> {
    Ok(Response {
        id: format!("resp-{}", uuid::Uuid::new_v4()),
        model: "test-model".into(),
        provider: "test".into(),
        message: Message::assistant(text),
        finish_reason: FinishReason::new(Reason::Stop, None),
        usage: Usage::default(),
        raw: None,
        warnings: None,
        rate_limit: None,
    })
}

/// Build a Response that contains tool calls (with optional text).
fn tool_call_response(text: &str, calls: Vec<ToolCall>) -> Result<Response, SdkError> {
    let mut parts = Vec::new();
    if !text.is_empty() {
        parts.push(ContentPart::text(text));
    }
    for tc in &calls {
        parts.push(ContentPart::tool_call(
            &tc.id,
            &tc.name,
            tc.arguments.clone(),
        ));
    }
    Ok(Response {
        id: format!("resp-{}", uuid::Uuid::new_v4()),
        model: "test-model".into(),
        provider: "test".into(),
        message: Message::new(Role::Assistant, parts),
        finish_reason: FinishReason::new(Reason::ToolCalls, None),
        usage: Usage::default(),
        raw: None,
        warnings: None,
        rate_limit: None,
    })
}

/// Build a ToolCall for the echo tool.
fn echo_call(text: &str) -> ToolCall {
    ToolCall {
        id: format!("call-{}", uuid::Uuid::new_v4()),
        name: "echo".into(),
        arguments: json!({"text": text}),
        raw_arguments: None,
        parse_error: None,
    }
}

/// Build a ToolCall for the failing tool.
fn fail_call() -> ToolCall {
    ToolCall {
        id: format!("call-{}", uuid::Uuid::new_v4()),
        name: "fail".into(),
        arguments: json!({}),
        raw_arguments: None,
        parse_error: None,
    }
}

/// Build a ToolCall for an unknown tool.
fn unknown_call() -> ToolCall {
    ToolCall {
        id: format!("call-{}", uuid::Uuid::new_v4()),
        name: "nonexistent_tool".into(),
        arguments: json!({}),
        raw_arguments: None,
        parse_error: None,
    }
}

/// Create a test session with a mock client returning the given responses.
fn test_session(
    responses: Vec<Result<Response, SdkError>>,
) -> AgentResult<(
    Session,
    stencila_agents::events::EventReceiver,
    Arc<MockClient>,
)> {
    test_session_with_config(responses, SessionConfig::default())
}

/// Create a test session with a custom config.
fn test_session_with_config(
    responses: Vec<Result<Response, SdkError>>,
    config: SessionConfig,
) -> AgentResult<(
    Session,
    stencila_agents::events::EventReceiver,
    Arc<MockClient>,
)> {
    let profile = TestProfile::new()?;
    test_session_with_profile(responses, config, profile)
}

/// Create a test session with a specific profile.
fn test_session_with_profile(
    responses: Vec<Result<Response, SdkError>>,
    config: SessionConfig,
    profile: impl ProviderProfile + 'static,
) -> AgentResult<(
    Session,
    stencila_agents::events::EventReceiver,
    Arc<MockClient>,
)> {
    let client = Arc::new(MockClient::new(responses));
    let env: Arc<dyn ExecutionEnvironment> = Arc::new(MockExecEnv::new());
    let (session, receiver) = Session::new(
        Box::new(profile),
        env,
        client.clone(),
        config,
        "test system prompt".into(),
        0,
        None,
    );
    Ok((session, receiver, client))
}

/// Drain all available events from the receiver (non-blocking).
async fn drain_events(
    receiver: &mut stencila_agents::events::EventReceiver,
) -> Vec<stencila_agents::types::SessionEvent> {
    let mut events = Vec::new();
    while let Ok(Some(event)) =
        tokio::time::timeout(std::time::Duration::from_millis(10), receiver.recv()).await
    {
        events.push(event);
    }
    events
}

// ===========================================================================
// Core loop tests (spec 2.5, 9.1)
// ===========================================================================

#[tokio::test]
async fn natural_completion_text_only() -> AgentResult<()> {
    let (mut session, _rx, _) = test_session(vec![text_response("Hello, world!")])?;

    assert_eq!(session.state(), SessionState::Idle);
    session.submit("Hi").await?;
    assert_eq!(session.state(), SessionState::Idle);

    // History: User + Assistant
    assert_eq!(session.history().len(), 2);
    assert!(matches!(
        session.history()[0],
        stencila_agents::types::Turn::User { .. }
    ));
    assert!(matches!(
        session.history()[1],
        stencila_agents::types::Turn::Assistant { .. }
    ));

    Ok(())
}

#[tokio::test]
async fn single_tool_round() -> AgentResult<()> {
    let (mut session, _rx, _) = test_session(vec![
        tool_call_response("", vec![echo_call("hello")]),
        text_response("Done!"),
    ])?;

    session.submit("Use echo").await?;
    assert_eq!(session.state(), SessionState::Idle);

    // History: User, Assistant(tool_call), ToolResults, Assistant(text)
    assert_eq!(session.history().len(), 4);
    assert!(matches!(
        session.history()[2],
        stencila_agents::types::Turn::ToolResults { .. }
    ));

    Ok(())
}

#[tokio::test]
async fn multi_tool_rounds() -> AgentResult<()> {
    let (mut session, _rx, _) = test_session(vec![
        tool_call_response("", vec![echo_call("step1")]),
        tool_call_response("", vec![echo_call("step2")]),
        text_response("All done"),
    ])?;

    session.submit("Multi-step task").await?;
    assert_eq!(session.state(), SessionState::Idle);

    // User, Asst+TC, Results, Asst+TC, Results, Asst(text)
    assert_eq!(session.history().len(), 6);

    Ok(())
}

#[tokio::test]
async fn round_limit_reached() -> AgentResult<()> {
    // Config: max 2 tool rounds per input, client returns tool calls forever
    let config = SessionConfig {
        max_tool_rounds_per_input: 2,
        ..SessionConfig::default()
    };

    // 3 tool call responses (only 2 should be used)
    let (mut session, mut rx, _) = test_session_with_config(
        vec![
            tool_call_response("", vec![echo_call("r1")]),
            tool_call_response("", vec![echo_call("r2")]),
            tool_call_response("", vec![echo_call("r3")]),
        ],
        config,
    )?;

    session.submit("Keep going").await?;
    assert_eq!(session.state(), SessionState::Idle);

    // Should have stopped after 2 rounds: User, Asst, Results, Asst, Results
    assert_eq!(session.history().len(), 5);

    // Check TURN_LIMIT event was emitted
    let events = drain_events(&mut rx).await;
    let has_turn_limit = events.iter().any(|e| e.kind == EventKind::TurnLimit);
    assert!(has_turn_limit, "Expected TURN_LIMIT event");

    Ok(())
}

#[tokio::test]
async fn session_turn_limit() -> AgentResult<()> {
    let config = SessionConfig {
        max_turns: 1,
        ..SessionConfig::default()
    };

    let (mut session, mut rx, _) = test_session_with_config(
        vec![
            tool_call_response("", vec![echo_call("r1")]),
            text_response("done"),
        ],
        config,
    )?;

    session.submit("Go").await?;
    // After first LLM call, total_turns=1 which equals max_turns
    // The tool call executes (round_count was 0 < max_tool_rounds),
    // then on next iteration total_turns(1) >= max_turns(1) → TURN_LIMIT
    assert_eq!(session.state(), SessionState::Idle);

    let events = drain_events(&mut rx).await;
    let has_turn_limit = events.iter().any(|e| e.kind == EventKind::TurnLimit);
    assert!(has_turn_limit, "Expected TURN_LIMIT event for max_turns");

    Ok(())
}

#[tokio::test]
async fn abort_stops_loop() -> AgentResult<()> {
    let controller = AbortController::new();
    let signal = controller.signal();

    // Pre-abort before submit
    controller.abort();

    let (mut session, _rx, _) = test_session(vec![text_response("won't reach")])?;
    session.set_abort_signal(signal);

    session.submit("Go").await?;
    assert_eq!(session.state(), SessionState::Closed);

    // Only User turn recorded, no LLM call
    assert_eq!(session.history().len(), 1);

    Ok(())
}

#[tokio::test]
async fn submit_on_closed_session_errors() -> AgentResult<()> {
    let (mut session, _rx, _) = test_session(vec![])?;
    session.close();
    assert_eq!(session.state(), SessionState::Closed);

    let result = session.submit("Hello").await;
    assert!(matches!(result, Err(AgentError::SessionClosed)));

    Ok(())
}

#[tokio::test]
async fn sequential_inputs() -> AgentResult<()> {
    let (mut session, _rx, _) = test_session(vec![
        text_response("Response 1"),
        text_response("Response 2"),
    ])?;

    session.submit("First").await?;
    assert_eq!(session.state(), SessionState::Idle);
    assert_eq!(session.history().len(), 2);

    session.submit("Second").await?;
    assert_eq!(session.state(), SessionState::Idle);
    assert_eq!(session.history().len(), 4);

    Ok(())
}

// ===========================================================================
// Steering tests (spec 2.6, 9.6)
// ===========================================================================

#[tokio::test]
async fn steer_between_tool_rounds() -> AgentResult<()> {
    let (mut session, mut rx, _) = test_session(vec![
        tool_call_response("", vec![echo_call("step1")]),
        text_response("Done after steering"),
    ])?;

    // Queue steering before submit — will be drained before first LLM call
    session.steer("Focus on performance");
    session.submit("Start task").await?;

    // History should contain the steering turn
    let has_steering = session
        .history()
        .iter()
        .any(|t| matches!(t, stencila_agents::types::Turn::Steering { .. }));
    assert!(has_steering, "Expected steering turn in history");

    // Check STEERING_INJECTED event
    let events = drain_events(&mut rx).await;
    let has_event = events.iter().any(|e| e.kind == EventKind::SteeringInjected);
    assert!(has_event, "Expected STEERING_INJECTED event");

    Ok(())
}

#[tokio::test]
async fn follow_up_after_completion() -> AgentResult<()> {
    let (mut session, _rx, _) = test_session(vec![
        text_response("First answer"),
        text_response("Follow-up answer"),
    ])?;

    session.follow_up("And then do this");
    session.submit("First question").await?;
    assert_eq!(session.state(), SessionState::Idle);

    // Both inputs processed: User, Asst, User(followup), Asst
    assert_eq!(session.history().len(), 4);

    Ok(())
}

// ===========================================================================
// Event tests (spec 2.9, 9.10)
// ===========================================================================

#[tokio::test]
async fn events_natural_completion() -> AgentResult<()> {
    let (mut session, mut rx, _) = test_session(vec![text_response("Hello")])?;

    session.submit("Hi").await?;
    let events = drain_events(&mut rx).await;

    let kinds: Vec<EventKind> = events.iter().map(|e| e.kind).collect();

    // SESSION_START (from constructor), USER_INPUT, ASSISTANT_TEXT_START, ASSISTANT_TEXT_END
    assert!(kinds.contains(&EventKind::SessionStart));
    assert!(kinds.contains(&EventKind::UserInput));
    assert!(kinds.contains(&EventKind::AssistantTextStart));
    assert!(kinds.contains(&EventKind::AssistantTextEnd));

    Ok(())
}

#[tokio::test]
async fn events_tool_loop() -> AgentResult<()> {
    let (mut session, mut rx, _) = test_session(vec![
        tool_call_response("", vec![echo_call("test")]),
        text_response("Done"),
    ])?;

    session.submit("Use tool").await?;
    let events = drain_events(&mut rx).await;

    let kinds: Vec<EventKind> = events.iter().map(|e| e.kind).collect();

    assert!(kinds.contains(&EventKind::ToolCallStart));
    assert!(kinds.contains(&EventKind::ToolCallEnd));

    Ok(())
}

#[tokio::test]
async fn events_session_close() -> AgentResult<()> {
    let (mut session, mut rx, _) = test_session(vec![])?;
    session.close();

    let events = drain_events(&mut rx).await;
    let kinds: Vec<EventKind> = events.iter().map(|e| e.kind).collect();

    assert!(kinds.contains(&EventKind::SessionStart));
    assert!(kinds.contains(&EventKind::SessionEnd));

    // SESSION_END should include the final state (spec 2.9)
    let end_event = events
        .iter()
        .find(|e| e.kind == EventKind::SessionEnd)
        .expect("SESSION_END event present");
    assert_eq!(
        end_event.data.get("final_state").and_then(|v| v.as_str()),
        Some("CLOSED"),
        "SESSION_END should include final_state: CLOSED"
    );

    Ok(())
}

// ===========================================================================
// Tool execution tests (spec 3.8, 9.3)
// ===========================================================================

#[tokio::test]
async fn unknown_tool_returns_error_result() -> AgentResult<()> {
    // Model calls a tool that doesn't exist; should get error result back
    let (mut session, _rx, _) = test_session(vec![
        tool_call_response("", vec![unknown_call()]),
        text_response("I see, that tool doesn't exist"),
    ])?;

    session.submit("Call unknown tool").await?;
    assert_eq!(session.state(), SessionState::Idle);

    // Check the ToolResults turn has is_error=true
    let tool_results_turn = session
        .history()
        .iter()
        .find(|t| matches!(t, stencila_agents::types::Turn::ToolResults { .. }));
    assert!(tool_results_turn.is_some());

    if let Some(stencila_agents::types::Turn::ToolResults { results, .. }) = tool_results_turn {
        assert_eq!(results.len(), 1);
        assert!(results[0].is_error);
        let content = results[0].content.as_str().unwrap_or("");
        assert!(
            content.contains("unknown tool"),
            "Error should mention unknown tool: {content}"
        );
    }

    Ok(())
}

#[tokio::test]
async fn tool_error_sent_as_error_result() -> AgentResult<()> {
    let mut profile = TestProfile::new()?;
    profile.registry.register(failing_tool())?;

    let (mut session, _rx, _) = test_session_with_profile(
        vec![
            tool_call_response("", vec![fail_call()]),
            text_response("Recovered from error"),
        ],
        SessionConfig::default(),
        profile,
    )?;

    session.submit("Use failing tool").await?;
    assert_eq!(session.state(), SessionState::Idle);

    // Model gets error result and can recover
    let tool_results_turn = session
        .history()
        .iter()
        .find(|t| matches!(t, stencila_agents::types::Turn::ToolResults { .. }));
    if let Some(stencila_agents::types::Turn::ToolResults { results, .. }) = tool_results_turn {
        assert!(results[0].is_error);
    }

    Ok(())
}

#[tokio::test]
async fn tool_output_full_in_event_truncated_for_llm() -> AgentResult<()> {
    // Create a tool that returns a very long output
    let long_tool = RegisteredTool::new(
        ToolDefinition {
            name: "long_output".into(),
            description: "Returns long output".into(),
            parameters: json!({"type": "object", "properties": {}}),
            strict: false,
        },
        Box::new(|_args, _env| {
            Box::pin(async move {
                // 50K characters — well above default 30K fallback limit
                Ok(ToolOutput::Text("x".repeat(50_000)))
            })
        }),
    );

    let mut profile = TestProfile::new()?;
    profile.registry.register(long_tool)?;

    let call = ToolCall {
        id: "call-long".into(),
        name: "long_output".into(),
        arguments: json!({}),
        raw_arguments: None,
        parse_error: None,
    };

    let (mut session, mut rx, _) = test_session_with_profile(
        vec![tool_call_response("", vec![call]), text_response("Done")],
        SessionConfig::default(),
        profile,
    )?;

    session.submit("Get long output").await?;

    // TOOL_CALL_END event should have full 50K output
    let events = drain_events(&mut rx).await;
    let tool_end = events.iter().find(|e| e.kind == EventKind::ToolCallEnd);
    assert!(tool_end.is_some());
    let output_len = tool_end
        .and_then(|e| e.data.get("output"))
        .and_then(|v| v.as_str())
        .map(|s| s.len())
        .unwrap_or(0);
    assert_eq!(
        output_len, 50_000,
        "Event should have full untruncated output"
    );

    // ToolResult content sent to LLM should be truncated
    if let Some(stencila_agents::types::Turn::ToolResults { results, .. }) = session
        .history()
        .iter()
        .find(|t| matches!(t, stencila_agents::types::Turn::ToolResults { .. }))
    {
        let content_len = results[0].content.as_str().map(|s| s.len()).unwrap_or(0);
        assert!(
            content_len < 50_000,
            "LLM should receive truncated output, got {content_len}"
        );
    }

    Ok(())
}

#[tokio::test]
async fn parallel_tool_execution() -> AgentResult<()> {
    let profile = TestProfile::new()?.with_parallel();

    let calls = vec![echo_call("a"), echo_call("b"), echo_call("c")];

    let (mut session, _rx, _) = test_session_with_profile(
        vec![
            tool_call_response("", calls),
            text_response("All three done"),
        ],
        SessionConfig::default(),
        profile,
    )?;

    session.submit("Run three tools").await?;
    assert_eq!(session.state(), SessionState::Idle);

    // Check all three results are present
    if let Some(stencila_agents::types::Turn::ToolResults { results, .. }) = session
        .history()
        .iter()
        .find(|t| matches!(t, stencila_agents::types::Turn::ToolResults { .. }))
    {
        assert_eq!(results.len(), 3);
        for r in results {
            assert!(!r.is_error);
        }
    }

    Ok(())
}

#[tokio::test]
async fn sequential_tool_execution() -> AgentResult<()> {
    // Default TestProfile has supports_parallel = false
    let calls = vec![echo_call("a"), echo_call("b")];

    let (mut session, _rx, _) = test_session(vec![
        tool_call_response("", calls),
        text_response("Both done"),
    ])?;

    session.submit("Run two tools").await?;
    assert_eq!(session.state(), SessionState::Idle);

    if let Some(stencila_agents::types::Turn::ToolResults { results, .. }) = session
        .history()
        .iter()
        .find(|t| matches!(t, stencila_agents::types::Turn::ToolResults { .. }))
    {
        assert_eq!(results.len(), 2);
    }

    Ok(())
}

// ===========================================================================
// Loop detection tests (spec 2.10)
// ===========================================================================

#[tokio::test]
async fn loop_detection_injects_steering() -> AgentResult<()> {
    let config = SessionConfig {
        enable_loop_detection: true,
        loop_detection_window: 4,
        max_tool_rounds_per_input: 10,
        ..SessionConfig::default()
    };

    // Same tool call repeated many times
    let call = ToolCall {
        id: "call-loop".into(),
        name: "echo".into(),
        arguments: json!({"text": "same"}),
        raw_arguments: None,
        parse_error: None,
    };

    let mut responses: Vec<Result<Response, SdkError>> = Vec::new();
    for _ in 0..5 {
        responses.push(tool_call_response("", vec![call.clone()]));
    }
    responses.push(text_response("Finally done"));

    let (mut session, mut rx, _) = test_session_with_config(responses, config)?;

    session.submit("Loop forever").await?;

    let events = drain_events(&mut rx).await;
    let has_loop = events.iter().any(|e| e.kind == EventKind::LoopDetection);
    assert!(has_loop, "Expected LOOP_DETECTION event");

    // Steering turn should be in history
    let steering_count = session
        .history()
        .iter()
        .filter(|t| matches!(t, stencila_agents::types::Turn::Steering { content, .. } if content.contains("Loop detected")))
        .count();
    assert!(steering_count > 0, "Expected loop detection steering turn");

    Ok(())
}

#[tokio::test]
async fn loop_detection_disabled() -> AgentResult<()> {
    let config = SessionConfig {
        enable_loop_detection: false,
        max_tool_rounds_per_input: 3,
        ..SessionConfig::default()
    };

    let call = ToolCall {
        id: "call-loop".into(),
        name: "echo".into(),
        arguments: json!({"text": "same"}),
        raw_arguments: None,
        parse_error: None,
    };

    let mut responses: Vec<Result<Response, SdkError>> = Vec::new();
    for _ in 0..3 {
        responses.push(tool_call_response("", vec![call.clone()]));
    }
    responses.push(text_response("Hit limit"));

    let (mut session, mut rx, _) = test_session_with_config(responses, config)?;
    session.submit("Loop").await?;

    let events = drain_events(&mut rx).await;
    let has_loop = events.iter().any(|e| e.kind == EventKind::LoopDetection);
    assert!(!has_loop, "Should not emit LOOP_DETECTION when disabled");

    Ok(())
}

// ===========================================================================
// Reasoning effort tests (spec 2.7, 9.7)
// ===========================================================================

#[tokio::test]
async fn reasoning_effort_passthrough() -> AgentResult<()> {
    let config = SessionConfig {
        reasoning_effort: Some(ReasoningEffort::High),
        ..SessionConfig::default()
    };

    let (mut session, _rx, client) =
        test_session_with_config(vec![text_response("thought deeply")], config)?;

    session.submit("Think hard").await?;

    let requests = client.take_requests()?;
    assert_eq!(requests.len(), 1);
    assert_eq!(requests[0].reasoning_effort.as_deref(), Some("high"));

    Ok(())
}

#[tokio::test]
async fn reasoning_effort_mid_session_change() -> AgentResult<()> {
    let (mut session, _rx, client) = test_session(vec![
        text_response("low effort"),
        text_response("high effort"),
    ])?;

    // First call: no reasoning effort
    session.submit("Quick question").await?;

    // Change effort mid-session
    session.config_mut().reasoning_effort = Some(ReasoningEffort::High);
    session.submit("Deep question").await?;

    let requests = client.take_requests()?;
    assert_eq!(requests.len(), 2);
    assert!(requests[0].reasoning_effort.is_none());
    assert_eq!(requests[1].reasoning_effort.as_deref(), Some("high"));

    Ok(())
}

/// When the response contains thinking/reasoning content, it should be
/// preserved in the conversation history and included in subsequent
/// API requests. This is required by providers like Anthropic that
/// mandate thinking blocks be replayed in multi-turn conversations.
#[tokio::test]
async fn thinking_parts_preserved_in_history() -> AgentResult<()> {
    use stencila_models3::types::content::ThinkingData;

    // Response 1: tool call with thinking
    let thinking_response = {
        let mut parts = vec![
            ContentPart::Thinking {
                thinking: ThinkingData {
                    text: "Let me think about this...".into(),
                    signature: Some("sig_abc".into()),
                    redacted: false,
                },
            },
            ContentPart::text("I'll read the file."),
        ];
        parts.push(ContentPart::tool_call(
            "call-1",
            "read_file",
            json!({"file_path": "test.rs"}),
        ));
        Ok(Response {
            id: "resp-1".into(),
            model: "test-model".into(),
            provider: "test".into(),
            message: Message::new(Role::Assistant, parts),
            finish_reason: FinishReason::new(Reason::ToolCalls, None),
            usage: Usage::default(),
            raw: None,
            warnings: None,
            rate_limit: None,
        })
    };

    // Response 2: final text
    let final_response = text_response("Here are the contents.")?;

    let (mut session, _rx, client) = test_session(vec![thinking_response, Ok(final_response)])?;
    session.submit("Read test.rs").await?;

    let requests = client.take_requests()?;
    // Should have 2 requests: initial + after tool result
    assert_eq!(requests.len(), 2);

    // The second request should contain the assistant message with thinking
    let second_req = &requests[1];
    let assistant_msgs: Vec<_> = second_req
        .messages
        .iter()
        .filter(|m| m.role == Role::Assistant)
        .collect();
    assert!(!assistant_msgs.is_empty(), "should have assistant messages");

    // Find thinking parts in the assistant messages
    let has_thinking = assistant_msgs.iter().any(|m| {
        m.content
            .iter()
            .any(|p| matches!(p, ContentPart::Thinking { .. }))
    });
    assert!(
        has_thinking,
        "assistant message should include thinking content parts"
    );

    // Verify the signature is preserved
    let has_signature = assistant_msgs.iter().any(|m| {
        m.content.iter().any(|p| {
            matches!(p, ContentPart::Thinking { thinking } if thinking.signature.as_deref() == Some("sig_abc"))
        })
    });
    assert!(has_signature, "thinking content should preserve signature");

    Ok(())
}

// ===========================================================================
// Error handling tests (spec Appendix B, 9.11)
// ===========================================================================

#[tokio::test]
async fn authentication_error_closes_session() -> AgentResult<()> {
    let (mut session, mut rx, _) = test_session(vec![Err(SdkError::Authentication {
        message: "invalid key".into(),
        details: ProviderDetails::default(),
    })])?;

    let result = session.submit("Hi").await;
    assert!(result.is_err());
    assert_eq!(session.state(), SessionState::Closed);

    let events = drain_events(&mut rx).await;
    let has_error = events.iter().any(|e| e.kind == EventKind::Error);
    let has_end = events.iter().any(|e| e.kind == EventKind::SessionEnd);
    assert!(has_error, "Expected ERROR event");
    assert!(has_end, "Expected SESSION_END event");

    Ok(())
}

#[tokio::test]
async fn context_length_error_closes_session_with_warning_severity() -> AgentResult<()> {
    let (mut session, mut rx, _) = test_session(vec![Err(SdkError::ContextLength {
        message: "too long".into(),
        details: ProviderDetails::default(),
    })])?;

    let result = session.submit("Very long context").await;
    assert!(result.is_err());
    assert_eq!(session.state(), SessionState::Closed);

    // Context-length errors should emit ERROR with severity:warning
    let events = drain_events(&mut rx).await;
    let error_event = events.iter().find(|e| e.kind == EventKind::Error);
    assert!(
        error_event.is_some(),
        "Expected ERROR event for context length"
    );

    let severity = error_event
        .and_then(|e| e.data.get("severity"))
        .and_then(|v| v.as_str());
    assert_eq!(
        severity,
        Some("warning"),
        "Context-length error should have severity:warning"
    );

    let has_end = events.iter().any(|e| e.kind == EventKind::SessionEnd);
    assert!(has_end, "Expected SESSION_END after context length error");

    Ok(())
}

#[tokio::test]
async fn turn_limit_is_idle_not_closed() -> AgentResult<()> {
    let config = SessionConfig {
        max_tool_rounds_per_input: 1,
        ..SessionConfig::default()
    };

    let (mut session, _rx, _) = test_session_with_config(
        vec![
            tool_call_response("", vec![echo_call("round1")]),
            text_response("unreachable"),
        ],
        config,
    )?;

    session.submit("Go").await?;
    // After 1 tool round, round limit hit → IDLE, not CLOSED
    assert_eq!(session.state(), SessionState::Idle);

    // Can still submit again — session is IDLE, not CLOSED
    // The remaining mock response ("unreachable") gets consumed here
    let result = session.submit("Again").await;
    assert!(
        result.is_ok(),
        "Second submit should succeed because session is IDLE"
    );
    assert_eq!(session.state(), SessionState::Idle);

    Ok(())
}

#[tokio::test]
async fn server_error_closes_session() -> AgentResult<()> {
    // Retryable error that exhausted SDK retries
    let (mut session, _rx, _) = test_session(vec![Err(SdkError::Server {
        message: "500 internal server error".into(),
        details: ProviderDetails::default(),
    })])?;

    let result = session.submit("Hi").await;
    assert!(result.is_err());
    assert_eq!(session.state(), SessionState::Closed);

    Ok(())
}

#[tokio::test]
async fn rate_limit_error_closes_session() -> AgentResult<()> {
    // RateLimit errors are retried by the SDK layer; if still failing after
    // retries, the session should transition to CLOSED (spec Appendix B).
    let (mut session, mut rx, _) = test_session(vec![Err(SdkError::RateLimit {
        message: "429 too many requests".into(),
        details: ProviderDetails::default(),
    })])?;

    let result = session.submit("Hi").await;
    assert!(result.is_err());
    assert_eq!(session.state(), SessionState::Closed);

    let events = drain_events(&mut rx).await;
    let has_error = events.iter().any(|e| e.kind == EventKind::Error);
    let has_end = events.iter().any(|e| e.kind == EventKind::SessionEnd);
    assert!(has_error, "Expected ERROR event for rate limit");
    assert!(has_end, "Expected SESSION_END after rate limit");

    Ok(())
}

#[tokio::test]
async fn network_error_closes_session() -> AgentResult<()> {
    // Network errors are retried by the SDK layer; if still failing after
    // retries, the session should transition to CLOSED (spec Appendix B).
    let (mut session, mut rx, _) = test_session(vec![Err(SdkError::Network {
        message: "connection refused".into(),
    })])?;

    let result = session.submit("Hi").await;
    assert!(result.is_err());
    assert_eq!(session.state(), SessionState::Closed);

    let events = drain_events(&mut rx).await;
    let has_error = events.iter().any(|e| e.kind == EventKind::Error);
    let has_end = events.iter().any(|e| e.kind == EventKind::SessionEnd);
    assert!(has_error, "Expected ERROR event for network error");
    assert!(has_end, "Expected SESSION_END after network error");

    Ok(())
}

// ===========================================================================
// Prompt integration tests (spec 6.1, 9.8)
// ===========================================================================

#[tokio::test]
async fn system_prompt_in_request() -> AgentResult<()> {
    let (mut session, _rx, client) = test_session(vec![text_response("ok")])?;

    session.submit("Hello").await?;

    let requests = client.take_requests()?;
    assert_eq!(requests.len(), 1);

    // First message should be system prompt
    let first_msg = &requests[0].messages[0];
    assert_eq!(first_msg.role, Role::System);
    let text = first_msg.text();
    assert!(
        text.contains("test system prompt"),
        "System message should contain prompt text: {text}"
    );

    Ok(())
}

#[tokio::test]
async fn history_to_messages_user_turn() -> AgentResult<()> {
    let (mut session, _rx, client) = test_session(vec![
        text_response("First response"),
        text_response("Second response"),
    ])?;

    session.submit("First").await?;
    session.submit("Second").await?;

    let requests = client.take_requests()?;
    // Second request should contain: system, user("First"), assistant("First response"), user("Second")
    let msgs = &requests[1].messages;
    assert!(msgs.len() >= 4);
    assert_eq!(msgs[0].role, Role::System); // system prompt
    assert_eq!(msgs[1].role, Role::User); // first input
    assert_eq!(msgs[2].role, Role::Assistant); // first response
    assert_eq!(msgs[3].role, Role::User); // second input

    Ok(())
}

#[tokio::test]
async fn history_to_messages_assistant_with_tools() -> AgentResult<()> {
    let (mut session, _rx, client) = test_session(vec![
        tool_call_response("thinking...", vec![echo_call("test")]),
        text_response("Done"),
    ])?;

    session.submit("Use tool").await?;

    let requests = client.take_requests()?;
    // Second LLM call should include assistant message with tool calls
    assert!(requests.len() >= 2);
    let msgs = &requests[1].messages;

    // Find assistant message with tool calls
    let asst_msg = msgs
        .iter()
        .find(|m| m.role == Role::Assistant && m.content.len() > 1);
    assert!(
        asst_msg.is_some(),
        "Expected assistant message with multiple content parts"
    );

    // Find tool result message
    let tool_msg = msgs.iter().find(|m| m.role == Role::Tool);
    assert!(tool_msg.is_some(), "Expected tool result message");

    Ok(())
}

#[tokio::test]
async fn history_to_messages_steering_as_user() -> AgentResult<()> {
    let (mut session, _rx, client) = test_session(vec![
        tool_call_response("", vec![echo_call("test")]),
        text_response("Done"),
    ])?;

    session.steer("Be concise");
    session.submit("Start").await?;

    let requests = client.take_requests()?;
    // First request should have: system, steering("Be concise"), user("Start")
    let msgs = &requests[0].messages;
    // The steering message appears as a user message before the user input
    let user_msgs: Vec<_> = msgs.iter().filter(|m| m.role == Role::User).collect();
    assert!(
        user_msgs.len() >= 2,
        "Expected at least 2 user messages (steering + input)"
    );

    Ok(())
}

// ===========================================================================
// Request structure tests
// ===========================================================================

#[tokio::test]
async fn request_includes_tools_from_profile() -> AgentResult<()> {
    let (mut session, _rx, client) = test_session(vec![text_response("ok")])?;

    session.submit("Hello").await?;

    let requests = client.take_requests()?;
    let tools = requests[0].tools.as_ref();
    assert!(tools.is_some(), "Request should include tools");
    let tool_names: Vec<&str> = tools
        .iter()
        .flat_map(|t| t.iter())
        .map(|td| td.name.as_str())
        .collect();
    assert!(
        tool_names.contains(&"echo"),
        "Tools should include echo: {tool_names:?}"
    );

    Ok(())
}

#[tokio::test]
async fn request_has_tool_choice_auto() -> AgentResult<()> {
    let (mut session, _rx, client) = test_session(vec![text_response("ok")])?;

    session.submit("Hello").await?;

    let requests = client.take_requests()?;
    assert!(
        matches!(
            requests[0].tool_choice,
            Some(stencila_models3::types::tool::ToolChoice::Auto)
        ),
        "tool_choice should be Auto"
    );

    Ok(())
}

#[tokio::test]
async fn request_has_provider_id() -> AgentResult<()> {
    let (mut session, _rx, client) = test_session(vec![text_response("ok")])?;

    session.submit("Hello").await?;

    let requests = client.take_requests()?;
    assert_eq!(requests[0].provider.as_deref(), Some("test"));

    Ok(())
}

// ===========================================================================
// Parity-shape tests (spec 9.12, deterministic)
// ===========================================================================

#[tokio::test]
async fn openai_profile_has_apply_patch_tool() -> AgentResult<()> {
    let profile = OpenAiProfile::new("gpt-5.2-codex", 600_000)?;
    let names = profile.tool_registry().names();
    assert!(
        names.contains(&"apply_patch"),
        "OpenAI should have apply_patch: {names:?}"
    );
    assert!(names.contains(&"read_file"), "OpenAI should have read_file");
    assert!(
        names.contains(&"write_file"),
        "OpenAI should have write_file"
    );
    assert!(names.contains(&"shell"), "OpenAI should have shell");
    assert!(names.contains(&"grep"), "OpenAI should have grep");
    assert!(names.contains(&"glob"), "OpenAI should have glob");
    // Should NOT have edit_file (uses apply_patch instead)
    assert!(
        !names.contains(&"edit_file"),
        "OpenAI should NOT have edit_file"
    );

    Ok(())
}

#[tokio::test]
async fn anthropic_profile_has_edit_file_tool() -> AgentResult<()> {
    let profile = AnthropicProfile::new("claude-opus-4-6", 600_000)?;
    let names = profile.tool_registry().names();
    assert!(
        names.contains(&"edit_file"),
        "Anthropic should have edit_file: {names:?}"
    );
    assert!(
        names.contains(&"read_file"),
        "Anthropic should have read_file"
    );
    // Should NOT have apply_patch
    assert!(
        !names.contains(&"apply_patch"),
        "Anthropic should NOT have apply_patch"
    );

    Ok(())
}

#[tokio::test]
async fn gemini_profile_has_extended_tools() -> AgentResult<()> {
    let profile = GeminiProfile::new("gemini-2.5-pro", 600_000)?;
    let names = profile.tool_registry().names();
    assert!(
        names.contains(&"read_many_files"),
        "Gemini should have read_many_files: {names:?}"
    );
    assert!(names.contains(&"list_dir"), "Gemini should have list_dir");
    assert!(names.contains(&"edit_file"), "Gemini should have edit_file");

    Ok(())
}

/// Helper: build a shell tool call with no explicit timeout.
fn shell_call(command: &str) -> ToolCall {
    ToolCall {
        id: format!("call-{}", uuid::Uuid::new_v4()),
        name: "shell".into(),
        arguments: json!({"command": command}),
        raw_arguments: None,
        parse_error: None,
    }
}

/// Shared parity test harness: creates a session with a CapturingExecEnv,
/// submits a prompt that triggers a shell tool call, and returns the tool
/// names from the request and the captured exec_command calls.
async fn run_parity_session(
    profile: impl ProviderProfile + 'static,
) -> AgentResult<(Vec<String>, Vec<(String, u64)>)> {
    let client = Arc::new(MockClient::new(vec![
        tool_call_response("", vec![shell_call("echo hi")]),
        text_response("ok"),
    ]));
    let env = Arc::new(CapturingExecEnv::new());
    let (mut session, _rx, _) = {
        let (s, r) = Session::new(
            Box::new(profile),
            env.clone() as Arc<dyn ExecutionEnvironment>,
            client.clone(),
            SessionConfig::default(),
            "parity test".into(),
            0,
            None,
        );
        (s, r, client.clone())
    };

    session.submit("parity test").await?;

    let requests = client.take_requests()?;
    let tool_names: Vec<String> = requests[0]
        .tools
        .as_ref()
        .iter()
        .flat_map(|v| v.iter())
        .map(|td| td.name.clone())
        .collect();
    let calls = env.take_calls()?;

    Ok((tool_names, calls))
}

#[tokio::test]
async fn parity_openai_session_wires_correct_tools() -> AgentResult<()> {
    let profile = OpenAiProfile::new("gpt-5.2-codex", 600_000)?;
    let (tool_names, calls) = run_parity_session(profile).await?;

    assert!(
        tool_names.contains(&"apply_patch".into()),
        "OpenAI session should have apply_patch"
    );
    assert!(
        tool_names.contains(&"write_file".into()),
        "OpenAI session should have write_file"
    );
    assert!(
        !tool_names.contains(&"edit_file".into()),
        "OpenAI session should NOT have edit_file"
    );

    assert_eq!(calls.len(), 1, "One shell call should have been executed");
    assert_eq!(
        calls[0].1, 10_000,
        "OpenAI shell default timeout should be 10s"
    );

    Ok(())
}

#[tokio::test]
async fn parity_anthropic_session_wires_correct_tools() -> AgentResult<()> {
    let profile = AnthropicProfile::new("claude-opus-4-6", 600_000)?;
    let (tool_names, calls) = run_parity_session(profile).await?;

    assert!(
        tool_names.contains(&"edit_file".into()),
        "Anthropic session should have edit_file"
    );
    assert!(
        tool_names.contains(&"shell".into()),
        "Anthropic session should have shell"
    );
    assert!(
        !tool_names.contains(&"apply_patch".into()),
        "Anthropic session should NOT have apply_patch"
    );

    assert_eq!(calls.len(), 1, "One shell call should have been executed");
    assert_eq!(
        calls[0].1, 120_000,
        "Anthropic shell default timeout should be 120s"
    );

    Ok(())
}

#[tokio::test]
async fn parity_gemini_session_wires_correct_tools() -> AgentResult<()> {
    let profile = GeminiProfile::new("gemini-2.5-pro", 600_000)?;
    let (tool_names, calls) = run_parity_session(profile).await?;

    assert!(
        tool_names.contains(&"list_dir".into()),
        "Gemini session should have list_dir"
    );
    assert!(
        tool_names.contains(&"read_many_files".into()),
        "Gemini session should have read_many_files"
    );
    assert!(
        tool_names.contains(&"edit_file".into()),
        "Gemini session should have edit_file"
    );

    assert_eq!(calls.len(), 1, "One shell call should have been executed");
    assert_eq!(
        calls[0].1, 10_000,
        "Gemini shell default timeout should be 10s"
    );

    Ok(())
}

// ===========================================================================
// Session getters and state tests
// ===========================================================================

#[tokio::test]
async fn session_id_is_present() -> AgentResult<()> {
    let (session, _rx, _) = test_session(vec![])?;
    assert!(!session.session_id().is_empty());

    Ok(())
}

#[tokio::test]
async fn config_returns_default_values() -> AgentResult<()> {
    let (session, _rx, _) = test_session(vec![])?;
    assert_eq!(session.config().max_tool_rounds_per_input, 0);
    assert_eq!(session.config().max_turns, 0);
    assert!(session.config().enable_loop_detection);

    Ok(())
}

#[tokio::test]
async fn close_emits_session_end_only_once() -> AgentResult<()> {
    let (mut session, mut rx, _) = test_session(vec![])?;
    session.close();
    session.close(); // second close is no-op

    let events = drain_events(&mut rx).await;
    let end_count = events
        .iter()
        .filter(|e| e.kind == EventKind::SessionEnd)
        .count();
    assert_eq!(end_count, 1, "SESSION_END should be emitted exactly once");

    Ok(())
}

#[tokio::test]
async fn abort_mid_tool_loop() -> AgentResult<()> {
    let controller = AbortController::new();
    let signal = controller.signal();

    // Model returns tool call, then on next iteration abort is checked
    let (mut session, _rx, _) = test_session(vec![
        tool_call_response("", vec![echo_call("step1")]),
        text_response("should not reach"),
    ])?;
    session.set_abort_signal(signal);

    // Abort after first LLM call completes (between loop iterations)
    // We can't precisely time this with mock, so we abort before submit
    // and verify the loop exits at the abort check
    controller.abort();
    session.submit("Go").await?;
    assert_eq!(session.state(), SessionState::Closed);

    Ok(())
}

#[tokio::test]
async fn abort_cancels_in_flight_tool_execution() -> AgentResult<()> {
    // Use a slow tool (500ms sleep) and abort after 50ms.
    // The session should close without waiting for the tool to finish.
    let profile = TestProfile::new()?.with_tool(slow_tool(500))?;
    let client = Arc::new(MockClient::new(vec![
        tool_call_response("", vec![slow_call()]),
        text_response("should not reach"),
    ]));
    let env: Arc<dyn ExecutionEnvironment> = Arc::new(MockExecEnv::new());
    let (mut session, _rx, _) = {
        let (s, r) = Session::new(
            Box::new(profile),
            env,
            client.clone(),
            SessionConfig::default(),
            "test system prompt".into(),
            0,
            None,
        );
        (s, r, client.clone())
    };

    let controller = AbortController::new();
    session.set_abort_signal(controller.signal());

    // Abort after a short delay — while the slow tool is still running
    let ctrl = controller.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        ctrl.abort();
    });

    let start = std::time::Instant::now();
    session.submit("Run slow tool").await?;
    let elapsed = start.elapsed();

    assert_eq!(session.state(), SessionState::Closed);
    // Should complete much faster than the 500ms tool sleep
    assert!(
        elapsed < Duration::from_millis(300),
        "Abort should cancel in-flight tool: took {elapsed:?}"
    );

    Ok(())
}

// ===========================================================================
// Follow-up with turn limit (Finding 1 regression test)
// ===========================================================================

#[tokio::test]
async fn follow_up_processed_after_turn_limit() -> AgentResult<()> {
    let config = SessionConfig {
        max_tool_rounds_per_input: 1,
        ..SessionConfig::default()
    };

    // First submit: tool call → round limit hit → break → follow-up runs
    // Follow-up: gets text response → natural completion
    let (mut session, _rx, _) = test_session_with_config(
        vec![
            tool_call_response("", vec![echo_call("round1")]),
            // Follow-up consumes this response
            text_response("Follow-up answer"),
        ],
        config,
    )?;

    session.follow_up("Follow-up question");
    session.submit("First question").await?;

    assert_eq!(session.state(), SessionState::Idle);

    // Both the original input and the follow-up should be in history
    let user_turns: Vec<_> = session
        .history()
        .iter()
        .filter(|t| matches!(t, stencila_agents::types::Turn::User { .. }))
        .collect();
    assert_eq!(
        user_turns.len(),
        2,
        "Both original and follow-up user turns should be present"
    );

    Ok(())
}

// ===========================================================================
// Context usage warning (spec 5.5)
// ===========================================================================

#[tokio::test]
async fn context_usage_warning_emitted_at_80_percent() -> AgentResult<()> {
    // TestProfile has context_window_size = 128_000 tokens.
    // 80% threshold = 102_400 tokens ~ 409_600 chars.
    // We'll set a huge system prompt to push usage over the threshold.
    let huge_prompt = "x".repeat(500_000); // 500K chars ~ 125K tokens > 80%

    let client = Arc::new(MockClient::new(vec![text_response("ok")]));
    let env: Arc<dyn ExecutionEnvironment> = Arc::new(MockExecEnv::new());
    let profile = TestProfile::new()?;
    let (mut session, mut rx, _) = {
        let (s, r) = Session::new(
            Box::new(profile),
            env,
            client.clone(),
            SessionConfig::default(),
            huge_prompt,
            0,
            None,
        );
        (s, r, client.clone())
    };

    session.submit("Short question").await?;

    let events = drain_events(&mut rx).await;
    let warning = events.iter().find(|e| {
        e.kind == EventKind::Error
            && e.data
                .get("severity")
                .and_then(|v| v.as_str())
                .is_some_and(|s| s == "warning")
    });
    assert!(warning.is_some(), "Expected context usage warning event");

    // Verify the warning contains context info
    let msg = warning
        .and_then(|e| e.data.get("message"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert!(
        msg.contains("Context usage"),
        "Warning should mention context usage: {msg}"
    );

    Ok(())
}

#[tokio::test]
async fn context_usage_no_warning_below_threshold() -> AgentResult<()> {
    // Short prompt — should not trigger warning
    let (mut session, mut rx, _) = test_session(vec![text_response("ok")])?;

    session.submit("Short question").await?;

    let events = drain_events(&mut rx).await;
    let has_context_warning = events.iter().any(|e| {
        e.kind == EventKind::Error
            && e.data
                .get("severity")
                .and_then(|v| v.as_str())
                .is_some_and(|s| s == "warning")
    });
    assert!(
        !has_context_warning,
        "Should NOT emit context warning for small context"
    );

    Ok(())
}

// ===========================================================================
// End-to-end prompt integration (Finding 4)
// ===========================================================================

#[tokio::test]
async fn end_to_end_prompt_has_base_instructions_and_env_context() -> AgentResult<()> {
    // Use build_system_prompt with a real profile to verify layers are
    // assembled in the correct order (spec 6.1).
    let mut profile = Box::new(OpenAiProfile::new("gpt-5.2-codex", 600_000)?);
    let env: Arc<dyn ExecutionEnvironment> = Arc::new(MockExecEnv::new());

    let no_skills_config = SessionConfig {
        enable_skills: false,
        ..SessionConfig::default()
    };
    let (prompt, _) = stencila_agents::session::build_system_prompt(
        &mut *profile as &mut dyn ProviderProfile,
        &*env,
        &no_skills_config,
    )
    .await?;

    // Layer 1: base instructions from profile
    assert!(
        prompt.contains("coding assistant"),
        "Prompt should contain base instructions"
    );

    // Layer 2: environment context
    assert!(
        prompt.contains("<environment>"),
        "Prompt should contain environment context block"
    );
    assert!(
        prompt.contains("linux"),
        "Prompt should contain platform from env context"
    );
    assert!(
        prompt.contains("/tmp/test"),
        "Prompt should contain working directory"
    );

    // Layer ordering: base instructions BEFORE environment context
    let base_pos = prompt
        .find("coding assistant")
        .expect("base instructions present");
    let env_pos = prompt
        .find("<environment>")
        .expect("environment context present");
    assert!(
        base_pos < env_pos,
        "Base instructions (pos {base_pos}) must appear before environment context (pos {env_pos})"
    );

    Ok(())
}

#[tokio::test]
async fn end_to_end_prompt_in_request_matches_build_system_prompt() -> AgentResult<()> {
    // Verify the prompt passed to Session::new() is the one sent in the LLM request.
    let mut profile = Box::new(OpenAiProfile::new("gpt-5.2-codex", 600_000)?);
    let env: Arc<dyn ExecutionEnvironment> = Arc::new(MockExecEnv::new());

    let no_skills_config = SessionConfig {
        enable_skills: false,
        ..SessionConfig::default()
    };
    let (prompt, _) = stencila_agents::session::build_system_prompt(
        &mut *profile as &mut dyn ProviderProfile,
        &*env,
        &no_skills_config,
    )
    .await?;

    let profile2 = OpenAiProfile::new("gpt-5.2-codex", 600_000)?;
    let client = Arc::new(MockClient::new(vec![text_response("ok")]));
    let (mut session, _rx, _) = {
        let (s, r) = Session::new(
            Box::new(profile2),
            env.clone(),
            client.clone(),
            SessionConfig::default(),
            prompt.clone(),
            0,
            None,
        );
        (s, r, client.clone())
    };

    session.submit("Hello").await?;

    let requests = client.take_requests()?;
    let system_msg = &requests[0].messages[0];
    assert_eq!(system_msg.role, Role::System);
    let sent_prompt = system_msg.text();
    assert_eq!(
        sent_prompt, prompt,
        "Request system message should match the prompt from build_system_prompt()"
    );

    Ok(())
}

/// Mock environment that serves an AGENTS.md file for project-docs testing.
struct MockExecEnvWithDocs {
    inner: MockExecEnv,
    agents_md_content: String,
}

impl MockExecEnvWithDocs {
    fn new(agents_md_content: &str) -> Self {
        Self {
            inner: MockExecEnv::new(),
            agents_md_content: agents_md_content.to_string(),
        }
    }
}

#[async_trait]
impl ExecutionEnvironment for MockExecEnvWithDocs {
    async fn read_file(
        &self,
        path: &str,
        offset: Option<usize>,
        limit: Option<usize>,
    ) -> AgentResult<FileContent> {
        if path.ends_with("AGENTS.md") {
            // Return with line numbers like real read_file does
            Ok(FileContent::Text(format!(
                "     1\t| {}",
                self.agents_md_content
            )))
        } else {
            self.inner.read_file(path, offset, limit).await
        }
    }
    async fn write_file(&self, path: &str, content: &str) -> AgentResult<()> {
        self.inner.write_file(path, content).await
    }
    async fn file_exists(&self, path: &str) -> bool {
        if path.ends_with("AGENTS.md") {
            true
        } else {
            self.inner.file_exists(path).await
        }
    }
    async fn delete_file(&self, path: &str) -> AgentResult<()> {
        self.inner.delete_file(path).await
    }
    async fn list_directory(&self, path: &str, depth: usize) -> AgentResult<Vec<DirEntry>> {
        self.inner.list_directory(path, depth).await
    }
    async fn exec_command(
        &self,
        command: &str,
        timeout_ms: u64,
        working_dir: Option<&str>,
        env_vars: Option<&HashMap<String, String>>,
    ) -> AgentResult<ExecResult> {
        self.inner
            .exec_command(command, timeout_ms, working_dir, env_vars)
            .await
    }
    async fn grep(&self, pattern: &str, path: &str, options: &GrepOptions) -> AgentResult<String> {
        self.inner.grep(pattern, path, options).await
    }
    async fn glob_files(&self, pattern: &str, path: &str) -> AgentResult<Vec<String>> {
        self.inner.glob_files(pattern, path).await
    }
    fn working_directory(&self) -> &str {
        self.inner.working_directory()
    }
    fn platform(&self) -> &str {
        self.inner.platform()
    }
    fn os_version(&self) -> String {
        self.inner.os_version()
    }
}

#[tokio::test]
async fn end_to_end_prompt_includes_project_docs_layer() -> AgentResult<()> {
    // Verify layer 4 (project docs) appears in the assembled prompt when
    // an AGENTS.md file is present (spec 6.1, 6.5).
    let project_instructions = "Always use semantic versioning for releases.";
    let env: Arc<dyn ExecutionEnvironment> =
        Arc::new(MockExecEnvWithDocs::new(project_instructions));

    let mut profile = Box::new(OpenAiProfile::new("gpt-5.2-codex", 600_000)?);
    let no_skills_config = SessionConfig {
        enable_skills: false,
        ..SessionConfig::default()
    };
    let (prompt, _) = stencila_agents::session::build_system_prompt(
        &mut *profile as &mut dyn ProviderProfile,
        &*env,
        &no_skills_config,
    )
    .await?;

    // Layer 1: base instructions
    assert!(
        prompt.contains("coding assistant"),
        "Prompt should contain base instructions (layer 1)"
    );

    // Layer 2: environment context
    assert!(
        prompt.contains("<environment>"),
        "Prompt should contain environment context (layer 2)"
    );

    // Layer 4: project docs from AGENTS.md
    assert!(
        prompt.contains(project_instructions),
        "Prompt should contain project doc content (layer 4)"
    );

    // Layer ordering: base < env < project docs
    let base_pos = prompt
        .find("coding assistant")
        .expect("base instructions present");
    let env_pos = prompt
        .find("<environment>")
        .expect("environment context present");
    let docs_pos = prompt
        .find(project_instructions)
        .expect("project docs present");
    assert!(
        base_pos < env_pos,
        "Base instructions (layer 1) must precede environment context (layer 2)"
    );
    assert!(
        env_pos < docs_pos,
        "Environment context (layer 2) must precede project docs (layer 4)"
    );

    Ok(())
}

// ===========================================================================
// Abort-aware LLM call (Finding 3, spec Graceful Shutdown)
// ===========================================================================

/// Mock client with configurable delay before returning the response.
struct DelayedClient {
    delay: Duration,
    response: Mutex<Option<Result<Response, SdkError>>>,
}

impl DelayedClient {
    fn new(delay: Duration, response: Result<Response, SdkError>) -> Self {
        Self {
            delay,
            response: Mutex::new(Some(response)),
        }
    }
}

#[async_trait]
impl LlmClient for DelayedClient {
    async fn complete(&self, _request: Request) -> Result<Response, SdkError> {
        tokio::time::sleep(self.delay).await;
        self.response
            .lock()
            .map_err(|e| SdkError::Configuration {
                message: format!("mock lock: {e}"),
            })?
            .take()
            .unwrap_or_else(|| {
                Err(SdkError::Configuration {
                    message: "no more mock responses".into(),
                })
            })
    }
}

#[tokio::test]
async fn abort_during_llm_call() -> AgentResult<()> {
    // Use a delayed client (500ms) and abort after 50ms.
    // The session should close without waiting for the LLM response.
    let client: Arc<dyn LlmClient> = Arc::new(DelayedClient::new(
        Duration::from_millis(500),
        text_response("slow response"),
    ));
    let env: Arc<dyn ExecutionEnvironment> = Arc::new(MockExecEnv::new());
    let profile = TestProfile::new()?;
    let (mut session, _rx) = Session::new(
        Box::new(profile),
        env,
        client,
        SessionConfig::default(),
        "test prompt".into(),
        0,
        None,
    );

    let controller = AbortController::new();
    session.set_abort_signal(controller.signal());

    // Abort after a short delay — while the LLM call is still in-flight
    let ctrl = controller.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        ctrl.abort();
    });

    let start = std::time::Instant::now();
    session.submit("Test abort during LLM").await?;
    let elapsed = start.elapsed();

    assert_eq!(session.state(), SessionState::Closed);
    // Should complete much faster than the 500ms LLM delay
    assert!(
        elapsed < Duration::from_millis(300),
        "Abort should cancel in-flight LLM call: took {elapsed:?}"
    );

    Ok(())
}

// ===========================================================================
// Schema validation at runtime (Finding 1, spec 3.8 step 2)
// ===========================================================================

#[tokio::test]
async fn invalid_tool_args_returns_validation_error() -> AgentResult<()> {
    // The echo tool requires "text" (a string). Sending an integer should
    // fail schema validation and return is_error: true without executing.
    let call = ToolCall {
        id: format!("call-{}", uuid::Uuid::new_v4()),
        name: "echo".into(),
        arguments: json!({"text": 42}), // wrong type: integer instead of string
        raw_arguments: None,
        parse_error: None,
    };

    let (mut session, _rx, _) = test_session(vec![
        tool_call_response("", vec![call]),
        text_response("Saw the validation error"),
    ])?;

    session.submit("Send invalid args").await?;
    assert_eq!(session.state(), SessionState::Idle);

    // Find the ToolResults turn and verify it has is_error: true
    let tool_results_turn = session
        .history()
        .iter()
        .find(|t| matches!(t, stencila_agents::types::Turn::ToolResults { .. }));
    assert!(
        tool_results_turn.is_some(),
        "Expected ToolResults turn in history"
    );

    if let Some(stencila_agents::types::Turn::ToolResults { results, .. }) = tool_results_turn {
        assert_eq!(results.len(), 1);
        assert!(
            results[0].is_error,
            "Invalid args should produce is_error: true"
        );
        let content = results[0].content.as_str().unwrap_or("");
        assert!(
            content.contains("validation") || content.contains("not of type"),
            "Error should mention validation failure: {content}"
        );
    }

    Ok(())
}

// ===========================================================================
// Shell timeout clamping (Finding 2, max_command_timeout_ms)
// ===========================================================================

#[tokio::test]
async fn shell_timeout_clamped_to_max() -> AgentResult<()> {
    // Shell tool call requests a 999 second timeout, but max is 600_000ms (10 min).
    // Verify the exec_command receives the clamped value.
    let call = ToolCall {
        id: format!("call-{}", uuid::Uuid::new_v4()),
        name: "shell".into(),
        arguments: json!({"command": "echo hi", "timeout_ms": 999_000}),
        raw_arguments: None,
        parse_error: None,
    };

    let profile = OpenAiProfile::new("gpt-5.2-codex", 600_000)?;
    let client = Arc::new(MockClient::new(vec![
        tool_call_response("", vec![call]),
        text_response("ok"),
    ]));
    let env = Arc::new(CapturingExecEnv::new());
    let (mut session, _rx, _) = {
        let (s, r) = Session::new(
            Box::new(profile),
            env.clone() as Arc<dyn ExecutionEnvironment>,
            client.clone(),
            SessionConfig::default(),
            "test".into(),
            0,
            None,
        );
        (s, r, client.clone())
    };

    session.submit("clamp test").await?;

    let calls = env.take_calls()?;
    assert_eq!(calls.len(), 1, "Expected one exec_command call");
    assert_eq!(
        calls[0].1, 600_000,
        "timeout should be clamped to max_command_timeout_ms (600_000)"
    );

    Ok(())
}

// ===========================================================================
// Streaming tests (spec 2.9)
// ===========================================================================

#[tokio::test]
async fn streaming_emits_text_deltas() -> AgentResult<()> {
    // The default stream_complete impl synthesizes a TextDelta from the
    // response text. Verify the session emits ASSISTANT_TEXT_DELTA events.
    let (mut session, mut rx, _) = test_session(vec![text_response("Hello, world!")])?;

    session.submit("Hi").await?;
    let events = drain_events(&mut rx).await;

    let kinds: Vec<EventKind> = events.iter().map(|e| e.kind).collect();

    // TEXT_START, TEXT_DELTA, TEXT_END should all be present
    assert!(
        kinds.contains(&EventKind::AssistantTextStart),
        "Expected ASSISTANT_TEXT_START"
    );
    assert!(
        kinds.contains(&EventKind::AssistantTextDelta),
        "Expected ASSISTANT_TEXT_DELTA"
    );
    assert!(
        kinds.contains(&EventKind::AssistantTextEnd),
        "Expected ASSISTANT_TEXT_END"
    );

    // Delta text should match the full response
    let delta_event = events
        .iter()
        .find(|e| e.kind == EventKind::AssistantTextDelta);
    let delta_text = delta_event
        .and_then(|e| e.data.get("delta"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(
        delta_text, "Hello, world!",
        "Delta should contain the full response text"
    );

    Ok(())
}

#[tokio::test]
async fn streaming_text_start_before_delta_before_end() -> AgentResult<()> {
    // Verify event ordering: TEXT_START < TEXT_DELTA < TEXT_END
    let (mut session, mut rx, _) = test_session(vec![text_response("order test")])?;

    session.submit("Hi").await?;
    let events = drain_events(&mut rx).await;

    let text_start_pos = events
        .iter()
        .position(|e| e.kind == EventKind::AssistantTextStart);
    let text_delta_pos = events
        .iter()
        .position(|e| e.kind == EventKind::AssistantTextDelta);
    let text_end_pos = events
        .iter()
        .position(|e| e.kind == EventKind::AssistantTextEnd);

    assert!(text_start_pos.is_some(), "TEXT_START must be present");
    assert!(text_delta_pos.is_some(), "TEXT_DELTA must be present");
    assert!(text_end_pos.is_some(), "TEXT_END must be present");

    let start = text_start_pos.expect("checked above");
    let delta = text_delta_pos.expect("checked above");
    let end = text_end_pos.expect("checked above");

    assert!(
        start < delta,
        "TEXT_START (pos {start}) should precede TEXT_DELTA (pos {delta})"
    );
    assert!(
        delta < end,
        "TEXT_DELTA (pos {delta}) should precede TEXT_END (pos {end})"
    );

    Ok(())
}

#[tokio::test]
async fn streaming_no_delta_for_empty_text() -> AgentResult<()> {
    // When the response has no text (tool-call-only), no TEXT_DELTA should
    // be emitted (though TEXT_START and TEXT_END are still emitted).
    let (mut session, mut rx, _) = test_session(vec![
        tool_call_response("", vec![echo_call("test")]),
        text_response("Done"),
    ])?;

    session.submit("Use tool").await?;
    let events = drain_events(&mut rx).await;

    // There should be two TEXT_START events (one per LLM round)
    let text_start_count = events
        .iter()
        .filter(|e| e.kind == EventKind::AssistantTextStart)
        .count();
    assert_eq!(
        text_start_count, 2,
        "Two TEXT_START events (one per LLM call)"
    );

    // The first round (tool-call response with empty text) should not have a delta
    let text_deltas: Vec<_> = events
        .iter()
        .filter(|e| e.kind == EventKind::AssistantTextDelta)
        .collect();
    // Only the second round should produce a delta (for "Done")
    assert_eq!(
        text_deltas.len(),
        1,
        "Only one TEXT_DELTA for the text-bearing response"
    );
    let delta_text = text_deltas[0]
        .data
        .get("delta")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(delta_text, "Done", "Delta should be from the text response");

    Ok(())
}

#[tokio::test]
async fn streaming_tool_loop_emits_deltas_per_round() -> AgentResult<()> {
    // Multi-round tool use: each round with text should emit a delta.
    let (mut session, mut rx, _) = test_session(vec![
        tool_call_response("thinking...", vec![echo_call("step1")]),
        text_response("All done"),
    ])?;

    session.submit("Go").await?;
    let events = drain_events(&mut rx).await;

    let text_deltas: Vec<_> = events
        .iter()
        .filter(|e| e.kind == EventKind::AssistantTextDelta)
        .collect();

    // Both rounds have text: "thinking..." and "All done"
    assert_eq!(
        text_deltas.len(),
        2,
        "Two deltas for two text-bearing rounds"
    );

    let first = text_deltas[0]
        .data
        .get("delta")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let second = text_deltas[1]
        .data
        .get("delta")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(first, "thinking...");
    assert_eq!(second, "All done");

    Ok(())
}

/// Mock client that provides real streaming via a per-response sequence of
/// StreamEvents. This tests the full stream_complete override path.
struct StreamingMockClient {
    /// Each entry is a sequence of stream events for one call.
    streams:
        Mutex<VecDeque<Vec<Result<stencila_models3::types::stream_event::StreamEvent, SdkError>>>>,
}

impl StreamingMockClient {
    fn new(
        streams: Vec<Vec<Result<stencila_models3::types::stream_event::StreamEvent, SdkError>>>,
    ) -> Self {
        Self {
            streams: Mutex::new(VecDeque::from(streams)),
        }
    }
}

#[async_trait]
impl LlmClient for StreamingMockClient {
    async fn complete(&self, _request: Request) -> Result<Response, SdkError> {
        Err(SdkError::Configuration {
            message: "StreamingMockClient does not support complete()".into(),
        })
    }

    async fn stream_complete(
        &self,
        _request: Request,
        on_event: &(dyn Fn(stencila_models3::types::stream_event::StreamEvent) + Send + Sync),
    ) -> Result<Response, SdkError> {
        let events = self
            .streams
            .lock()
            .map_err(|e| SdkError::Configuration {
                message: format!("mock lock: {e}"),
            })?
            .pop_front()
            .ok_or_else(|| SdkError::Configuration {
                message: "no more mock streams".into(),
            })?;

        let mut accumulator = stencila_models3::api::accumulator::StreamAccumulator::new();
        for result in events {
            let event = result?;
            accumulator.process(&event);
            on_event(event);
        }
        Ok(accumulator.response())
    }
}

#[tokio::test]
async fn streaming_real_incremental_deltas() -> AgentResult<()> {
    use stencila_models3::types::stream_event::{StreamEvent as SE, StreamEventType as SET};

    // Simulate a stream with multiple text deltas
    let stream_events: Vec<Result<SE, SdkError>> = vec![
        Ok(SE {
            event_type: SET::StreamStart,
            ..SE::stream_start()
        }),
        Ok(SE::text_delta("Hello")),
        Ok(SE::text_delta(", ")),
        Ok(SE::text_delta("world!")),
        Ok(SE::finish(
            FinishReason::new(Reason::Stop, None),
            Usage::default(),
        )),
    ];

    let client: Arc<dyn LlmClient> = Arc::new(StreamingMockClient::new(vec![stream_events]));
    let env: Arc<dyn ExecutionEnvironment> = Arc::new(MockExecEnv::new());
    let profile = TestProfile::new()?;
    let (mut session, mut rx) = Session::new(
        Box::new(profile),
        env,
        client,
        SessionConfig::default(),
        "test prompt".into(),
        0,
        None,
    );

    session.submit("Stream me").await?;
    assert_eq!(session.state(), SessionState::Idle);

    let events = drain_events(&mut rx).await;

    // Should have 3 incremental ASSISTANT_TEXT_DELTA events
    let deltas: Vec<_> = events
        .iter()
        .filter(|e| e.kind == EventKind::AssistantTextDelta)
        .collect();
    assert_eq!(deltas.len(), 3, "Should have 3 incremental deltas");

    let delta_texts: Vec<&str> = deltas
        .iter()
        .filter_map(|e| e.data.get("delta").and_then(|v| v.as_str()))
        .collect();
    assert_eq!(delta_texts, vec!["Hello", ", ", "world!"]);

    // TEXT_END should have the full accumulated text
    let text_end = events
        .iter()
        .find(|e| e.kind == EventKind::AssistantTextEnd);
    let full_text = text_end
        .and_then(|e| e.data.get("text"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(full_text, "Hello, world!");

    // History should contain the accumulated text
    assert_eq!(session.history().len(), 2); // User + Assistant
    if let stencila_agents::types::Turn::Assistant { content, .. } = &session.history()[1] {
        assert_eq!(content, "Hello, world!");
    }

    Ok(())
}

#[tokio::test]
async fn non_streaming_profile_falls_back_to_complete() -> AgentResult<()> {
    // When supports_streaming() is false, the session should use complete()
    // instead of stream_complete(), still synthesizing a single delta.
    let profile = TestProfile::new()?.without_streaming();

    let (mut session, mut rx, _) = test_session_with_profile(
        vec![text_response("non-streamed response")],
        SessionConfig::default(),
        profile,
    )?;

    session.submit("Hi").await?;
    assert_eq!(session.state(), SessionState::Idle);

    let events = drain_events(&mut rx).await;
    let kinds: Vec<EventKind> = events.iter().map(|e| e.kind).collect();

    // Should still emit TEXT_START, TEXT_DELTA (synthesized), TEXT_END
    assert!(kinds.contains(&EventKind::AssistantTextStart));
    assert!(kinds.contains(&EventKind::AssistantTextDelta));
    assert!(kinds.contains(&EventKind::AssistantTextEnd));

    // Delta text should be the full response
    let delta = events
        .iter()
        .find(|e| e.kind == EventKind::AssistantTextDelta)
        .and_then(|e| e.data.get("delta"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    assert_eq!(delta, "non-streamed response");

    // History should be normal
    assert_eq!(session.history().len(), 2);

    Ok(())
}

#[tokio::test]
async fn pre_abort_closes_before_llm_call() -> AgentResult<()> {
    // When abort fires before the LLM call (top-of-loop check),
    // no TEXT_START/TEXT_END should be emitted.
    let controller = AbortController::new();
    controller.abort(); // pre-abort

    let (mut session, mut rx, _) = test_session(vec![text_response("won't reach")])?;
    session.set_abort_signal(controller.signal());

    session.submit("Go").await?;
    assert_eq!(session.state(), SessionState::Closed);

    let events = drain_events(&mut rx).await;
    let kinds: Vec<EventKind> = events.iter().map(|e| e.kind).collect();

    // TEXT_START should NOT be present because abort fires before the LLM
    // call (at the top-of-loop check, before build_request).
    assert!(kinds.contains(&EventKind::SessionStart));
    assert!(kinds.contains(&EventKind::UserInput));
    assert!(kinds.contains(&EventKind::SessionEnd));
    assert!(
        !kinds.contains(&EventKind::AssistantTextStart),
        "no TEXT_START expected for pre-abort"
    );

    Ok(())
}

#[tokio::test]
async fn inflight_abort_preserves_partial_text_in_text_end() -> AgentResult<()> {
    use stencila_models3::types::stream_event::{StreamEvent as SE, StreamEventType as SET};

    // Create a streaming client that yields two deltas, then blocks long
    // enough for the abort to fire before the stream finishes.
    struct SlowStreamClient {
        controller: AbortController,
    }

    #[async_trait]
    impl LlmClient for SlowStreamClient {
        async fn complete(&self, _request: Request) -> Result<Response, SdkError> {
            Err(SdkError::Configuration {
                message: "use stream_complete".into(),
            })
        }

        async fn stream_complete(
            &self,
            _request: Request,
            on_event: &(dyn Fn(SE) + Send + Sync),
        ) -> Result<Response, SdkError> {
            // Emit two deltas, then trigger abort, then "block".
            on_event(SE {
                event_type: SET::StreamStart,
                ..SE::stream_start()
            });
            on_event(SE::text_delta("Hello"));
            on_event(SE::text_delta(", wor"));
            // Trigger abort mid-stream
            self.controller.abort();
            // Sleep long enough for tokio::select! to pick up the abort
            tokio::time::sleep(Duration::from_secs(10)).await;
            // Should never reach here
            Err(SdkError::Configuration {
                message: "unreachable: abort should fire first".into(),
            })
        }
    }

    let controller = AbortController::new();
    let client: Arc<dyn LlmClient> = Arc::new(SlowStreamClient {
        controller: controller.clone(),
    });
    let env: Arc<dyn ExecutionEnvironment> = Arc::new(MockExecEnv::new());
    let profile = TestProfile::new()?;
    let (mut session, mut rx) = Session::new(
        Box::new(profile),
        env,
        client,
        SessionConfig::default(),
        "test prompt".into(),
        0,
        None,
    );
    session.set_abort_signal(controller.signal());

    session.submit("Stream and abort").await?;
    assert_eq!(session.state(), SessionState::Closed);

    let events = drain_events(&mut rx).await;
    let kinds: Vec<EventKind> = events.iter().map(|e| e.kind).collect();

    // Should have TEXT_START, deltas, TEXT_END, SESSION_END
    assert!(kinds.contains(&EventKind::AssistantTextStart));
    assert!(kinds.contains(&EventKind::AssistantTextEnd));
    assert!(kinds.contains(&EventKind::SessionEnd));

    // TEXT_END should carry the partial text accumulated before abort
    let text_end = events
        .iter()
        .find(|e| e.kind == EventKind::AssistantTextEnd);
    let end_text = text_end
        .and_then(|e| e.data.get("text"))
        .and_then(|v| v.as_str())
        .unwrap_or("MISSING");
    assert_eq!(
        end_text, "Hello, wor",
        "TEXT_END should carry partial text on abort"
    );

    // Should have 2 delta events
    let deltas: Vec<_> = events
        .iter()
        .filter(|e| e.kind == EventKind::AssistantTextDelta)
        .collect();
    assert_eq!(deltas.len(), 2);

    Ok(())
}

#[tokio::test]
async fn error_before_streaming_emits_empty_text_end() -> AgentResult<()> {
    // When the error occurs before any streaming (complete() fails),
    // TEXT_END should have empty text and still pair with TEXT_START.
    let (mut session, mut rx, _) = test_session(vec![Err(SdkError::Server {
        message: "500 error".into(),
        details: ProviderDetails::default(),
    })])?;

    let result = session.submit("Hi").await;
    assert!(result.is_err());
    assert_eq!(session.state(), SessionState::Closed);

    let events = drain_events(&mut rx).await;
    let kinds: Vec<EventKind> = events.iter().map(|e| e.kind).collect();

    assert!(
        kinds.contains(&EventKind::AssistantTextStart),
        "TEXT_START should be present: {kinds:?}"
    );
    assert!(
        kinds.contains(&EventKind::AssistantTextEnd),
        "TEXT_END should be present for strict pairing: {kinds:?}"
    );
    assert!(kinds.contains(&EventKind::Error));
    assert!(kinds.contains(&EventKind::SessionEnd));

    // TEXT_START must come before TEXT_END
    let start_pos = events
        .iter()
        .position(|e| e.kind == EventKind::AssistantTextStart);
    let end_pos = events
        .iter()
        .position(|e| e.kind == EventKind::AssistantTextEnd);
    assert!(start_pos < end_pos, "TEXT_START must precede TEXT_END");

    // No streaming happened, so TEXT_END should carry empty text.
    let text_end = events
        .iter()
        .find(|e| e.kind == EventKind::AssistantTextEnd);
    let end_text = text_end
        .and_then(|e| e.data.get("text"))
        .and_then(|v| v.as_str())
        .unwrap_or("MISSING");
    assert_eq!(
        end_text, "",
        "TEXT_END on pre-stream error should have empty text"
    );

    Ok(())
}

#[tokio::test]
async fn midstream_error_preserves_partial_text_in_text_end() -> AgentResult<()> {
    use stencila_models3::types::stream_event::{StreamEvent as SE, StreamEventType as SET};

    // Simulate a stream that delivers two deltas then fails mid-stream.
    let stream_events: Vec<Result<SE, SdkError>> = vec![
        Ok(SE {
            event_type: SET::StreamStart,
            ..SE::stream_start()
        }),
        Ok(SE::text_delta("partial ")),
        Ok(SE::text_delta("respon")),
        Err(SdkError::Stream {
            message: "connection reset".into(),
        }),
    ];

    let client: Arc<dyn LlmClient> = Arc::new(StreamingMockClient::new(vec![stream_events]));
    let env: Arc<dyn ExecutionEnvironment> = Arc::new(MockExecEnv::new());
    let profile = TestProfile::new()?;
    let (mut session, mut rx) = Session::new(
        Box::new(profile),
        env,
        client,
        SessionConfig::default(),
        "test prompt".into(),
        0,
        None,
    );

    let result = session.submit("Stream error").await;
    assert!(result.is_err());
    assert_eq!(session.state(), SessionState::Closed);

    let events = drain_events(&mut rx).await;
    let kinds: Vec<EventKind> = events.iter().map(|e| e.kind).collect();

    assert!(kinds.contains(&EventKind::AssistantTextStart));
    assert!(kinds.contains(&EventKind::AssistantTextEnd));
    assert!(kinds.contains(&EventKind::Error));

    // TEXT_END should carry the partial text accumulated before the error.
    let text_end = events
        .iter()
        .find(|e| e.kind == EventKind::AssistantTextEnd);
    let end_text = text_end
        .and_then(|e| e.data.get("text"))
        .and_then(|v| v.as_str())
        .unwrap_or("MISSING");
    assert_eq!(
        end_text, "partial respon",
        "TEXT_END should carry partial text on mid-stream error"
    );

    Ok(())
}

// ===========================================================================
// Image tool output tests (§3.3 — provider-gated multimodal messages)
// ===========================================================================

/// Build a `read_file` ToolCall for image tests.
fn read_file_call(path: &str) -> ToolCall {
    ToolCall {
        id: format!("call-{}", uuid::Uuid::new_v4()),
        name: "read_file".into(),
        arguments: json!({"file_path": path}),
        raw_arguments: None,
        parse_error: None,
    }
}

#[tokio::test]
async fn anthropic_image_tool_result_includes_image_in_message() -> AgentResult<()> {
    // Setup: Anthropic profile with read_file tool that returns image content.
    let profile = AnthropicProfile::new("claude-opus-4-6", 600_000)?;
    let rf_call = read_file_call("/test/photo.png");

    let client = Arc::new(MockClient::new(vec![
        tool_call_response("", vec![rf_call]),
        text_response("I see a PNG image"),
    ]));
    let env = Arc::new(MockExecEnv::new().with_image_mode());
    let (mut session, _rx) = Session::new(
        Box::new(profile),
        env as Arc<dyn ExecutionEnvironment>,
        client.clone(),
        SessionConfig::default(),
        "image test".into(),
        0,
        None,
    );

    session.submit("describe photo.png").await?;

    // The second request (after tool result) should have an image content part
    // in one of the tool-role messages.
    let requests = client.take_requests()?;
    assert!(
        requests.len() >= 2,
        "expected at least 2 requests (tool call + follow-up), got {}",
        requests.len()
    );

    let second_req = &requests[1];
    let has_image = second_req.messages.iter().any(|msg| {
        msg.role == Role::Tool
            && msg
                .content
                .iter()
                .any(|part| matches!(part, ContentPart::Image { .. }))
    });
    assert!(
        has_image,
        "Anthropic tool result message should include an Image content part"
    );

    Ok(())
}

#[tokio::test]
async fn openai_image_tool_result_excludes_image_from_message() -> AgentResult<()> {
    // Setup: OpenAI profile with read_file tool that returns image content.
    let profile = OpenAiProfile::new("gpt-5.2-codex", 600_000)?;
    let rf_call = read_file_call("/test/photo.png");

    let client = Arc::new(MockClient::new(vec![
        tool_call_response("", vec![rf_call]),
        text_response("I see a PNG image"),
    ]));
    let env = Arc::new(MockExecEnv::new().with_image_mode());
    let (mut session, _rx) = Session::new(
        Box::new(profile),
        env as Arc<dyn ExecutionEnvironment>,
        client.clone(),
        SessionConfig::default(),
        "image test".into(),
        0,
        None,
    );

    session.submit("describe photo.png").await?;

    let requests = client.take_requests()?;
    assert!(
        requests.len() >= 2,
        "expected at least 2 requests, got {}",
        requests.len()
    );

    let second_req = &requests[1];
    let has_image = second_req.messages.iter().any(|msg| {
        msg.role == Role::Tool
            && msg
                .content
                .iter()
                .any(|part| matches!(part, ContentPart::Image { .. }))
    });
    assert!(
        !has_image,
        "OpenAI tool result message should NOT include an Image content part"
    );

    // Verify it still has the text placeholder in the tool result
    let has_tool_result = second_req.messages.iter().any(|msg| {
        msg.role == Role::Tool
            && msg.content.iter().any(|part| {
                matches!(part, ContentPart::ToolResult { tool_result }
                    if tool_result.content.as_str().is_some_and(|s| s.contains("[Image file:")))
            })
    });
    assert!(
        has_tool_result,
        "OpenAI tool result should contain text placeholder for image"
    );

    Ok(())
}

// ===========================================================================
// AwaitingInput auto-detection tests (spec 2.3, Phase 5)
// ===========================================================================

#[tokio::test]
async fn awaiting_input_auto_detected_question_mark() -> AgentResult<()> {
    // Model responds with a question ending in '?' → AwaitingInput.
    let (mut session, _rx, _) = test_session(vec![text_response("What file should I edit?")])?;

    session.submit("Fix the bug").await?;
    assert_eq!(
        session.state(),
        SessionState::AwaitingInput,
        "Question ending with '?' should trigger AwaitingInput"
    );

    Ok(())
}

#[tokio::test]
async fn awaiting_input_auto_detected_interrogative_phrase() -> AgentResult<()> {
    // Model responds with an interrogative phrase (no trailing '?').
    let (mut session, _rx, _) = test_session(vec![text_response(
        "I found several options.\nWould you like me to proceed with option A",
    )])?;

    session.submit("Refactor the module").await?;
    assert_eq!(
        session.state(),
        SessionState::AwaitingInput,
        "Interrogative phrase should trigger AwaitingInput"
    );

    Ok(())
}

#[tokio::test]
async fn awaiting_input_not_triggered_for_statement() -> AgentResult<()> {
    // Model responds with a statement → stays Idle.
    let (mut session, _rx, _) =
        test_session(vec![text_response("I have completed the refactoring.")])?;

    session.submit("Refactor the module").await?;
    assert_eq!(
        session.state(),
        SessionState::Idle,
        "Statement should stay Idle"
    );

    Ok(())
}

#[tokio::test]
async fn awaiting_input_not_triggered_for_declarative_interrogative_word() -> AgentResult<()> {
    // Declarative sentences starting with interrogative words (e.g. "What",
    // "How") must NOT trigger AwaitingInput without a trailing '?'.
    for text in [
        "What follows is the implementation summary.",
        "How the module works is explained below.",
        "Which files were changed is listed above.",
        "Where the config lives is documented in README.",
    ] {
        let (mut session, _rx, _) = test_session(vec![text_response(text)])?;

        session.submit("Explain").await?;
        assert_eq!(
            session.state(),
            SessionState::Idle,
            "Declarative '{text}' should stay Idle"
        );
    }

    Ok(())
}

#[tokio::test]
async fn awaiting_input_disabled_via_config() -> AgentResult<()> {
    // Even with a question, disabled config should stay Idle.
    let config = SessionConfig {
        auto_detect_awaiting_input: false,
        ..SessionConfig::default()
    };

    let (mut session, _rx, _) =
        test_session_with_config(vec![text_response("What file should I edit?")], config)?;

    session.submit("Fix the bug").await?;
    assert_eq!(
        session.state(),
        SessionState::Idle,
        "Disabled auto-detection should stay Idle"
    );

    Ok(())
}

#[tokio::test]
async fn awaiting_input_submit_resumes_to_idle() -> AgentResult<()> {
    // After auto-detecting AwaitingInput, the next submit() should
    // resume processing and transition back to Idle (or AwaitingInput again).
    let (mut session, _rx, _) = test_session(vec![
        text_response("Which approach do you prefer?"),
        text_response("Done! I applied option B."),
    ])?;

    session.submit("Fix the bug").await?;
    assert_eq!(session.state(), SessionState::AwaitingInput);

    // User answers → resumes processing → model responds with statement → Idle.
    session.submit("Option B").await?;
    assert_eq!(
        session.state(),
        SessionState::Idle,
        "After answering, non-question response should be Idle"
    );

    Ok(())
}

#[tokio::test]
async fn awaiting_input_not_triggered_after_tool_calls() -> AgentResult<()> {
    // If the model used tool calls and then asked a question with its final
    // text response, it should still trigger AwaitingInput (natural completion
    // with a question). But a tool-call response itself should NOT.
    let (mut session, _rx, _) = test_session(vec![
        tool_call_response("", vec![echo_call("step1")]),
        text_response("I found a bug. Should I fix it?"),
    ])?;

    session.submit("Look at the code").await?;
    assert_eq!(
        session.state(),
        SessionState::AwaitingInput,
        "Final text-only question after tool calls should trigger AwaitingInput"
    );

    Ok(())
}

#[tokio::test]
async fn awaiting_input_not_triggered_on_turn_limit() -> AgentResult<()> {
    // When the loop exits due to a limit (not natural completion),
    // auto-detection is skipped entirely — only natural completions
    // (text-only responses) are eligible for question detection.
    let config = SessionConfig {
        max_tool_rounds_per_input: 1,
        ..SessionConfig::default()
    };

    let (mut session, _rx, _) = test_session_with_config(
        vec![tool_call_response(
            "Should I continue?",
            vec![echo_call("step1")],
        )],
        config,
    )?;

    session.submit("Start").await?;
    // The last assistant turn has tool calls, so even though text is a question,
    // looks_like_question returns false because tool_calls is not empty.
    assert_eq!(
        session.state(),
        SessionState::Idle,
        "Turn limit with tool calls should stay Idle"
    );

    Ok(())
}

#[tokio::test]
async fn awaiting_input_empty_text_stays_idle() -> AgentResult<()> {
    // Model returns empty text with no tool calls (unusual but valid).
    let (mut session, _rx, _) = test_session(vec![text_response("")])?;

    session.submit("Hi").await?;
    assert_eq!(
        session.state(),
        SessionState::Idle,
        "Empty response should stay Idle"
    );

    Ok(())
}

#[tokio::test]
async fn awaiting_input_multiline_question() -> AgentResult<()> {
    // Multi-paragraph response where only the last line is a question.
    let response_text = "I analyzed the codebase and found three issues:\n\
                         1. Missing error handling in auth.rs\n\
                         2. Unused imports in main.rs\n\
                         3. Deprecated API usage in client.rs\n\
                         Which issue should I fix first?";

    let (mut session, _rx, _) = test_session(vec![text_response(response_text)])?;

    session.submit("Review the code").await?;
    assert_eq!(
        session.state(),
        SessionState::AwaitingInput,
        "Multi-line response with trailing question should trigger AwaitingInput"
    );

    Ok(())
}

#[tokio::test]
async fn awaiting_input_let_me_know_pattern() -> AgentResult<()> {
    // "Let me know" is a common question-like phrase.
    let (mut session, _rx, _) = test_session(vec![text_response(
        "I've prepared the changes.\nLet me know if you want me to apply them.",
    )])?;

    session.submit("Prepare changes").await?;
    assert_eq!(
        session.state(),
        SessionState::AwaitingInput,
        "'Let me know' pattern should trigger AwaitingInput"
    );

    Ok(())
}

#[tokio::test]
async fn awaiting_input_not_triggered_on_max_turns_with_prior_question() -> AgentResult<()> {
    // Regression: if a prior submit() ended with a question (AwaitingInput),
    // and the next submit() hits max_turns before getting a new model response,
    // the session must NOT re-enter AwaitingInput from the stale question.
    let config = SessionConfig {
        max_turns: 1,
        ..SessionConfig::default()
    };

    let (mut session, _rx, _) = test_session_with_config(
        vec![
            // First submit: model asks a question → AwaitingInput.
            text_response("Which file should I edit?"),
            // Second submit: model won't be reached because max_turns is already 1.
        ],
        config,
    )?;

    // First submit: question → AwaitingInput, total_turns becomes 1.
    session.submit("Fix the bug").await?;
    assert_eq!(session.state(), SessionState::AwaitingInput);

    // Second submit: max_turns (1) already reached → loop breaks immediately.
    // Must be Idle, NOT AwaitingInput from the stale question.
    session.submit("src/main.rs").await?;
    assert_eq!(
        session.state(),
        SessionState::Idle,
        "max_turns limit should not re-trigger AwaitingInput from prior question"
    );

    Ok(())
}

#[tokio::test]
async fn awaiting_input_not_triggered_on_one_round_limit_with_prior_question() -> AgentResult<()> {
    // Regression: when the round limit fires on the first iteration (before
    // a new LLM call produces fresh text), a prior AwaitingInput question must
    // not leak into the state.
    //
    // Setup: first submit with default config produces a question → AwaitingInput.
    // Then change round limit to 1 so the next submit breaks after one round.
    let (mut session, _rx, _) = test_session_with_config(
        vec![
            text_response("What approach do you prefer?"),
            text_response("OK, done."),
        ],
        SessionConfig::default(),
    )?;

    session.submit("Refactor").await?;
    assert_eq!(session.state(), SessionState::AwaitingInput);

    // Now set round limit to 1 — next submit breaks after one round.
    session.config_mut().max_tool_rounds_per_input = 1;
    session.submit("Option A").await?;
    assert_eq!(
        session.state(),
        SessionState::Idle,
        "Round limit should not re-trigger AwaitingInput from prior question"
    );

    Ok(())
}

#[tokio::test]
async fn zero_round_limit_means_unlimited() -> AgentResult<()> {
    // Spec update: max_tool_rounds_per_input = 0 now means unlimited.
    // Verify the loop runs normally (no immediate break) when set to 0.
    let (mut session, _rx, _) = test_session_with_config(
        vec![text_response("Hello!")],
        SessionConfig {
            max_tool_rounds_per_input: 0,
            ..SessionConfig::default()
        },
    )?;

    session.submit("Hi").await?;
    assert_eq!(session.state(), SessionState::Idle);

    Ok(())
}

// ===========================================================================
// Soft abort tests
// ===========================================================================

#[tokio::test]
async fn soft_abort_returns_idle() -> AgentResult<()> {
    let controller = AbortController::new();
    let signal = controller.signal();

    // Pre-soft-abort before submit
    controller.soft_abort();

    let (mut session, _rx, _) = test_session(vec![text_response("won't reach")])?;
    session.set_abort_signal(signal);

    session.submit("Go").await?;
    assert_eq!(
        session.state(),
        SessionState::Idle,
        "Soft abort should return to Idle, not Closed"
    );

    // Only user turn recorded (no LLM call made)
    assert_eq!(session.history().len(), 1);

    Ok(())
}

#[tokio::test]
async fn soft_abort_then_resubmit() -> AgentResult<()> {
    let controller = AbortController::new();
    let signal = controller.signal();

    // Two responses: first won't be reached (soft abort), second will
    let (mut session, _rx, _) = test_session(vec![
        text_response("won't reach"),
        text_response("Second response"),
    ])?;
    session.set_abort_signal(signal);

    // Soft abort, then submit — signal auto-resets
    controller.soft_abort();
    session.submit("First").await?;
    assert_eq!(session.state(), SessionState::Idle);

    // Second submit should work normally — soft abort was reset
    session.submit("Second").await?;
    assert_eq!(session.state(), SessionState::Idle);
    // History: user1, user2, assistant2
    assert_eq!(session.history().len(), 3);

    Ok(())
}

#[tokio::test]
async fn soft_abort_during_llm_streaming() -> AgentResult<()> {
    use stencila_models3::types::stream_event::{StreamEvent as SE, StreamEventType as SET};

    // Streaming client that emits two deltas, then triggers soft abort.
    struct SoftAbortStreamClient {
        controller: AbortController,
    }

    #[async_trait]
    impl LlmClient for SoftAbortStreamClient {
        async fn complete(&self, _request: Request) -> Result<Response, SdkError> {
            Err(SdkError::Configuration {
                message: "use stream_complete".into(),
            })
        }

        async fn stream_complete(
            &self,
            _request: Request,
            on_event: &(dyn Fn(SE) + Send + Sync),
        ) -> Result<Response, SdkError> {
            on_event(SE {
                event_type: SET::StreamStart,
                ..SE::stream_start()
            });
            on_event(SE::text_delta("Hello"));
            on_event(SE::text_delta(", wor"));
            // Trigger soft abort mid-stream
            self.controller.soft_abort();
            // Sleep long enough for tokio::select! to pick up the abort
            tokio::time::sleep(Duration::from_secs(10)).await;
            Err(SdkError::Configuration {
                message: "unreachable: abort should fire first".into(),
            })
        }
    }

    let controller = AbortController::new();
    let client: Arc<dyn LlmClient> = Arc::new(SoftAbortStreamClient {
        controller: controller.clone(),
    });
    let env: Arc<dyn ExecutionEnvironment> = Arc::new(MockExecEnv::new());
    let profile = TestProfile::new()?;
    let (mut session, mut rx) = Session::new(
        Box::new(profile),
        env,
        client,
        SessionConfig::default(),
        "test prompt".into(),
        0,
        None,
    );
    session.set_abort_signal(controller.signal());

    session.submit("Stream and soft abort").await?;
    assert_eq!(
        session.state(),
        SessionState::Idle,
        "Soft abort during streaming should return to Idle"
    );

    let events = drain_events(&mut rx).await;
    let kinds: Vec<EventKind> = events.iter().map(|e| e.kind).collect();

    // TEXT_END should be emitted with partial text
    assert!(kinds.contains(&EventKind::AssistantTextStart));
    assert!(kinds.contains(&EventKind::AssistantTextEnd));

    // No SESSION_END — session is still alive
    assert!(
        !kinds.contains(&EventKind::SessionEnd),
        "Soft abort should NOT emit SESSION_END"
    );

    // TEXT_END carries partial text
    let text_end = events
        .iter()
        .find(|e| e.kind == EventKind::AssistantTextEnd);
    let end_text = text_end
        .and_then(|e| e.data.get("text"))
        .and_then(|v| v.as_str())
        .unwrap_or("MISSING");
    assert_eq!(end_text, "Hello, wor", "TEXT_END should carry partial text");

    Ok(())
}

#[tokio::test]
async fn soft_abort_during_tool_execution() -> AgentResult<()> {
    // Use a slow tool (500ms sleep) and soft-abort after 50ms.
    let profile = TestProfile::new()?.with_tool(slow_tool(500))?;
    let client = Arc::new(MockClient::new(vec![
        tool_call_response("", vec![slow_call()]),
        text_response("should not reach"),
    ]));
    let env: Arc<dyn ExecutionEnvironment> = Arc::new(MockExecEnv::new());
    let (mut session, _rx, _) = {
        let (s, r) = Session::new(
            Box::new(profile),
            env,
            client.clone(),
            SessionConfig::default(),
            "test system prompt".into(),
            0,
            None,
        );
        (s, r, client.clone())
    };

    let controller = AbortController::new();
    session.set_abort_signal(controller.signal());

    // Soft abort after a short delay — while the slow tool is running
    let ctrl = controller.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        ctrl.soft_abort();
    });

    let start = std::time::Instant::now();
    session.submit("Run slow tool").await?;
    let elapsed = start.elapsed();

    assert_eq!(
        session.state(),
        SessionState::Idle,
        "Soft abort during tool execution should return to Idle"
    );
    // Should complete much faster than the 500ms tool sleep
    assert!(
        elapsed < Duration::from_millis(300),
        "Soft abort should cancel in-flight tool: took {elapsed:?}"
    );

    // History should have: user, assistant (with tool_calls), tool_results ([Aborted])
    assert_eq!(session.history().len(), 3);
    let last = &session.history()[2];
    if let stencila_agents::types::Turn::ToolResults { results, .. } = last {
        assert!(
            results[0].is_error,
            "Aborted tool result should be an error"
        );
        assert_eq!(
            results[0].content,
            serde_json::Value::String("[Aborted]".into())
        );
    } else {
        panic!("Expected ToolResults turn, got {last:?}");
    }

    Ok(())
}

#[tokio::test]
async fn soft_abort_skips_follow_ups() -> AgentResult<()> {
    let controller = AbortController::new();
    let signal = controller.signal();

    let (mut session, _rx, _) = test_session(vec![
        text_response("First answer"),
        text_response("Follow-up answer — should NOT be reached"),
    ])?;
    session.set_abort_signal(signal);

    // Queue a follow-up, then soft abort before submit
    session.follow_up("Follow-up question");
    controller.soft_abort();

    session.submit("Start").await?;
    assert_eq!(session.state(), SessionState::Idle);

    // Only user turn recorded — follow-up was NOT processed
    assert_eq!(
        session.history().len(),
        1,
        "Soft abort should skip queued follow-ups"
    );

    Ok(())
}

#[tokio::test]
async fn hard_abort_wins_over_soft() -> AgentResult<()> {
    let controller = AbortController::new();
    let signal = controller.signal();

    // Soft first, then hard — hard should win
    controller.soft_abort();
    controller.abort();

    let (mut session, _rx, _) = test_session(vec![text_response("won't reach")])?;
    session.set_abort_signal(signal);

    session.submit("Go").await?;
    assert_eq!(
        session.state(),
        SessionState::Closed,
        "Hard abort should override soft abort → Closed"
    );

    Ok(())
}

#[tokio::test]
async fn hard_abort_sticky_across_submit() -> AgentResult<()> {
    let controller = AbortController::new();
    let signal = controller.signal();

    let (mut session, _rx, _) = test_session(vec![
        text_response("won't reach"),
        text_response("also won't reach"),
    ])?;
    session.set_abort_signal(signal);

    // Hard abort
    controller.abort();
    session.submit("First").await?;
    assert_eq!(session.state(), SessionState::Closed);

    // Second submit should fail with SessionClosed — hard abort is sticky
    let result = session.submit("Second").await;
    assert!(
        matches!(result, Err(AgentError::SessionClosed)),
        "Hard abort should be sticky — second submit should fail"
    );

    Ok(())
}

// ===========================================================================
// Multi-tool abort race tests (finding 1 & 2: 1:1 tool_calls ↔ tool_results)
// ===========================================================================

#[tokio::test]
async fn sequential_multi_tool_abort_race_backfills_results() -> AgentResult<()> {
    // 3 sequential tool calls: abort_trigger (completes + sets soft abort),
    // echo, echo. The abort_trigger tool succeeds but sets the abort flag
    // as a side effect. When the sequential loop checks is_aborted() before
    // the second tool, it sees true and backfills the remaining 2 with
    // [Aborted]. The work future wins tokio::select! because it completes
    // before cancelled() polls again (10ms interval).
    //
    // This deterministically tests finding 1: partial results must be
    // padded to match tool_calls.len().
    let controller = AbortController::new();
    let abort_ctrl = controller.clone();

    // A tool that completes successfully but triggers soft abort as a side effect.
    let abort_trigger_tool = RegisteredTool::new(
        ToolDefinition {
            name: "abort_trigger".into(),
            description: "Triggers soft abort then returns".into(),
            parameters: json!({"type": "object", "properties": {}}),
            strict: false,
        },
        {
            let ctrl = abort_ctrl.clone();
            Box::new(move |_args, _env| {
                let ctrl = ctrl.clone();
                Box::pin(async move {
                    ctrl.soft_abort();
                    Ok(ToolOutput::Text("triggered".to_string()))
                })
            })
        },
    );

    let profile = TestProfile::new()?.with_tool(abort_trigger_tool)?;

    let trigger_call = ToolCall {
        id: format!("call-{}", uuid::Uuid::new_v4()),
        name: "abort_trigger".into(),
        arguments: json!({}),
        raw_arguments: None,
        parse_error: None,
    };

    let client = Arc::new(MockClient::new(vec![
        tool_call_response(
            "",
            vec![trigger_call, echo_call("second"), echo_call("third")],
        ),
        text_response("should not reach"),
    ]));
    let env: Arc<dyn ExecutionEnvironment> = Arc::new(MockExecEnv::new());
    let (mut session, _rx, _) = {
        let (s, r) = Session::new(
            Box::new(profile),
            env,
            client.clone(),
            SessionConfig::default(),
            "test system prompt".into(),
            0,
            None,
        );
        (s, r, client.clone())
    };

    session.set_abort_signal(controller.signal());

    session.submit("Run three tools").await?;
    assert_eq!(session.state(), SessionState::Idle);

    // History: user, assistant (3 tool_calls), tool_results
    assert_eq!(session.history().len(), 3);
    let tool_results_turn = &session.history()[2];
    if let stencila_agents::types::Turn::ToolResults { results, .. } = tool_results_turn {
        assert_eq!(
            results.len(),
            3,
            "Must have exactly 3 tool results for 3 tool calls, got {}",
            results.len()
        );
        // First tool (abort_trigger) should have completed successfully
        assert!(
            !results[0].is_error,
            "abort_trigger should have completed successfully"
        );
        // Remaining should be [Aborted]
        assert!(results[1].is_error, "Second tool should be aborted");
        assert_eq!(
            results[1].content,
            serde_json::Value::String("[Aborted]".into())
        );
        assert!(results[2].is_error, "Third tool should be aborted");
        assert_eq!(
            results[2].content,
            serde_json::Value::String("[Aborted]".into())
        );
    } else {
        panic!("Expected ToolResults turn, got {tool_results_turn:?}");
    }

    Ok(())
}

#[tokio::test]
async fn parallel_multi_tool_abort_race_has_correct_result_count() -> AgentResult<()> {
    // 3 parallel tool calls: echo (fast), slow (500ms), echo (fast).
    // Abort after 50ms via tokio::select! — the work future is dropped.
    // Result must have exactly 3 ToolResults (all [Aborted] since the
    // join_all future was cancelled).
    let profile = TestProfile::new()?
        .with_parallel()
        .with_tool(slow_tool(500))?;
    let client = Arc::new(MockClient::new(vec![
        tool_call_response(
            "",
            vec![echo_call("fast1"), slow_call(), echo_call("fast2")],
        ),
        text_response("should not reach"),
    ]));
    let env: Arc<dyn ExecutionEnvironment> = Arc::new(MockExecEnv::new());
    let (mut session, _rx, _) = {
        let (s, r) = Session::new(
            Box::new(profile),
            env,
            client.clone(),
            SessionConfig::default(),
            "test system prompt".into(),
            0,
            None,
        );
        (s, r, client.clone())
    };

    let controller = AbortController::new();
    session.set_abort_signal(controller.signal());

    // Soft abort after 50ms
    let ctrl = controller.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        ctrl.soft_abort();
    });

    session.submit("Run three tools in parallel").await?;
    assert_eq!(session.state(), SessionState::Idle);

    // History: user, assistant (3 tool_calls), tool_results
    assert_eq!(session.history().len(), 3);
    let tool_results_turn = &session.history()[2];
    if let stencila_agents::types::Turn::ToolResults { results, .. } = tool_results_turn {
        assert_eq!(
            results.len(),
            3,
            "Must have exactly 3 tool results for 3 tool calls, got {}",
            results.len()
        );
        // All should be marked as errors (aborted)
        for (i, result) in results.iter().enumerate() {
            assert!(
                result.is_error,
                "Tool result {i} should be an error (aborted)"
            );
        }
    } else {
        panic!("Expected ToolResults turn, got {tool_results_turn:?}");
    }

    Ok(())
}

#[tokio::test]
async fn hard_abort_sequential_multi_tool_has_no_orphan_tool_calls() -> AgentResult<()> {
    // Verify finding 1 fix: hard abort during sequential multi-tool
    // execution should close session (no partial ToolResults in history
    // because hard abort returns immediately).
    let profile = TestProfile::new()?.with_tool(slow_tool(500))?;
    let client = Arc::new(MockClient::new(vec![
        tool_call_response(
            "",
            vec![echo_call("fast1"), slow_call(), echo_call("fast2")],
        ),
        text_response("should not reach"),
    ]));
    let env: Arc<dyn ExecutionEnvironment> = Arc::new(MockExecEnv::new());
    let (mut session, _rx, _) = {
        let (s, r) = Session::new(
            Box::new(profile),
            env,
            client.clone(),
            SessionConfig::default(),
            "test system prompt".into(),
            0,
            None,
        );
        (s, r, client.clone())
    };

    let controller = AbortController::new();
    session.set_abort_signal(controller.signal());

    // Hard abort after 50ms
    let ctrl = controller.clone();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        ctrl.abort();
    });

    session.submit("Run three tools").await?;
    assert_eq!(session.state(), SessionState::Closed);

    Ok(())
}
