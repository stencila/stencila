//! Tests for subagent spawning and lifecycle (spec 7.1-7.4, 9.9).
//!
//! Uses mock Client and execution environment for deterministic testing.
//! Validates spawn, send_input, wait, close_agent tools, depth limiting,
//! independent history, shared execution environment, and profile tool lists.

#![allow(clippy::result_large_err)]

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::json;
use stencila_models3::error::SdkError;
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
use stencila_agents::session::{LlmClient, Session};
use stencila_agents::subagents;
use stencila_agents::types::{DirEntry, EventKind, ExecResult, GrepOptions, SessionConfig};

// ===========================================================================
// Mock types (shared with spec_2_loop.rs)
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
    /// Tracks write_file calls for verification.
    writes: Mutex<Vec<(String, String)>>,
}

impl MockExecEnv {
    fn new() -> Self {
        Self {
            working_dir: "/tmp/test".into(),
            writes: Mutex::new(Vec::new()),
        }
    }

    #[allow(dead_code)]
    fn take_writes(&self) -> AgentResult<Vec<(String, String)>> {
        Ok(self
            .writes
            .lock()
            .map_err(|e| AgentError::Io {
                message: e.to_string(),
            })?
            .drain(..)
            .collect())
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
        Ok(FileContent::Text(format!("     1\t| content of {path}")))
    }
    async fn write_file(&self, path: &str, content: &str) -> AgentResult<()> {
        if let Ok(mut writes) = self.writes.lock() {
            writes.push((path.to_string(), content.to_string()));
        }
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

// ===========================================================================
// Helpers
// ===========================================================================

/// Test profile with an echo tool and subagent tools registered.
#[derive(Debug)]
struct TestProfile {
    registry: ToolRegistry,
    model: String,
}

impl TestProfile {
    fn new() -> AgentResult<Self> {
        let mut registry = ToolRegistry::new();
        registry.register(echo_tool())?;
        subagents::register_subagent_tools(&mut registry)?;
        Ok(Self {
            registry,
            model: "test-model".into(),
        })
    }
}

impl ProviderProfile for TestProfile {
    fn id(&self) -> &str {
        "anthropic" // Use anthropic so child profile creation works
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
        true
    }
    fn supports_parallel_tool_calls(&self) -> bool {
        false
    }
    fn context_window_size(&self) -> u64 {
        128_000
    }
    fn register_subagent_tools(&mut self) -> AgentResult<()> {
        subagents::register_subagent_tools(&mut self.registry)
    }
}

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

/// Build a Response with tool calls.
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

/// Build a ToolCall for spawn_agent.
fn spawn_call(task: &str) -> ToolCall {
    ToolCall {
        id: format!("call-{}", uuid::Uuid::new_v4()),
        name: "spawn_agent".into(),
        arguments: json!({"task": task}),
        raw_arguments: None,
        parse_error: None,
    }
}

/// Build a ToolCall for spawn_agent with max_turns.
fn spawn_call_with_limit(task: &str, max_turns: u32) -> ToolCall {
    ToolCall {
        id: format!("call-{}", uuid::Uuid::new_v4()),
        name: "spawn_agent".into(),
        arguments: json!({"task": task, "max_turns": max_turns}),
        raw_arguments: None,
        parse_error: None,
    }
}

/// Build a ToolCall for send_input.
fn send_input_call(agent_id: &str, message: &str) -> ToolCall {
    ToolCall {
        id: format!("call-{}", uuid::Uuid::new_v4()),
        name: "send_input".into(),
        arguments: json!({"agent_id": agent_id, "message": message}),
        raw_arguments: None,
        parse_error: None,
    }
}

/// Build a ToolCall for wait.
fn wait_call(agent_id: &str) -> ToolCall {
    ToolCall {
        id: format!("call-{}", uuid::Uuid::new_v4()),
        name: "wait".into(),
        arguments: json!({"agent_id": agent_id}),
        raw_arguments: None,
        parse_error: None,
    }
}

/// Build a ToolCall for close_agent.
fn close_call(agent_id: &str) -> ToolCall {
    ToolCall {
        id: format!("call-{}", uuid::Uuid::new_v4()),
        name: "close_agent".into(),
        arguments: json!({"agent_id": agent_id}),
        raw_arguments: None,
        parse_error: None,
    }
}

/// Create a test session with subagent tools.
///
/// The mock client responses are shared between parent and child sessions.
/// Responses are consumed in order: parent's first LLM call, then child's
/// calls (during spawn_agent execution), then parent's remaining calls.
fn test_session(
    responses: Vec<Result<Response, SdkError>>,
) -> AgentResult<(
    Session,
    stencila_agents::events::EventReceiver,
    Arc<MockClient>,
    Arc<MockExecEnv>,
)> {
    test_session_with_config(responses, SessionConfig::default())
}

fn test_session_with_config(
    responses: Vec<Result<Response, SdkError>>,
    config: SessionConfig,
) -> AgentResult<(
    Session,
    stencila_agents::events::EventReceiver,
    Arc<MockClient>,
    Arc<MockExecEnv>,
)> {
    let profile = TestProfile::new()?;
    let client = Arc::new(MockClient::new(responses));
    let env = Arc::new(MockExecEnv::new());
    let (session, receiver) = Session::new(
        Box::new(profile),
        env.clone() as Arc<dyn ExecutionEnvironment>,
        client.clone(),
        config,
        "test system prompt".into(),
        0,
    );
    Ok((session, receiver, client, env))
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
// Tool definition tests (spec 7.2)
// ===========================================================================

#[test]
fn spawn_agent_definition_valid() -> AgentResult<()> {
    let def = subagents::spawn_agent_definition();
    assert_eq!(def.name, "spawn_agent");
    assert!(!def.description.is_empty());
    // Has required "task" parameter
    let required = def.parameters.get("required").and_then(|v| v.as_array());
    assert!(
        required.is_some_and(|arr| arr.iter().any(|v| v.as_str() == Some("task"))),
        "spawn_agent should require 'task' parameter"
    );
    def.validate()?;
    Ok(())
}

#[test]
fn send_input_definition_valid() -> AgentResult<()> {
    let def = subagents::send_input_definition();
    assert_eq!(def.name, "send_input");
    def.validate()?;
    let required = def.parameters.get("required").and_then(|v| v.as_array());
    assert!(required.is_some_and(|arr| arr.len() == 2));
    Ok(())
}

#[test]
fn wait_definition_valid() -> AgentResult<()> {
    let def = subagents::wait_definition();
    assert_eq!(def.name, "wait");
    def.validate()?;
    Ok(())
}

#[test]
fn close_agent_definition_valid() -> AgentResult<()> {
    let def = subagents::close_agent_definition();
    assert_eq!(def.name, "close_agent");
    def.validate()?;
    Ok(())
}

#[test]
fn subagent_definitions_returns_four() {
    let defs = subagents::subagent_definitions();
    assert_eq!(defs.len(), 4);
    let names: Vec<&str> = defs.iter().map(|d| d.name.as_str()).collect();
    assert!(names.contains(&"spawn_agent"));
    assert!(names.contains(&"send_input"));
    assert!(names.contains(&"wait"));
    assert!(names.contains(&"close_agent"));
}

// ===========================================================================
// Tool registration tests (spec 7.2, 9.9)
// ===========================================================================

#[test]
fn register_subagent_tools_adds_four_tools() -> AgentResult<()> {
    let mut registry = ToolRegistry::new();
    subagents::register_subagent_tools(&mut registry)?;
    assert_eq!(registry.len(), 4);
    assert!(registry.get("spawn_agent").is_some());
    assert!(registry.get("send_input").is_some());
    assert!(registry.get("wait").is_some());
    assert!(registry.get("close_agent").is_some());
    Ok(())
}

// ===========================================================================
// Profile tool list tests (spec 9.9)
// ===========================================================================

#[test]
fn openai_profile_includes_subagent_tools_after_registration() -> AgentResult<()> {
    let mut profile = OpenAiProfile::new("gpt-5.2-codex", 600_000)?;
    // Before registration: no subagent tools
    assert!(
        !profile.tool_registry().names().contains(&"spawn_agent"),
        "spawn_agent should not be present before registration"
    );

    profile.register_subagent_tools()?;

    let names = profile.tool_registry().names();
    assert!(names.contains(&"spawn_agent"), "should have spawn_agent");
    assert!(names.contains(&"send_input"), "should have send_input");
    assert!(names.contains(&"wait"), "should have wait");
    assert!(names.contains(&"close_agent"), "should have close_agent");
    // Core tools still present
    assert!(names.contains(&"read_file"), "should still have read_file");
    assert!(
        names.contains(&"apply_patch"),
        "should still have apply_patch"
    );
    Ok(())
}

#[test]
fn anthropic_profile_includes_subagent_tools_after_registration() -> AgentResult<()> {
    let mut profile = AnthropicProfile::new("claude-opus-4-6", 600_000)?;
    profile.register_subagent_tools()?;

    let names = profile.tool_registry().names();
    assert!(names.contains(&"spawn_agent"), "should have spawn_agent");
    assert!(names.contains(&"send_input"), "should have send_input");
    assert!(names.contains(&"wait"), "should have wait");
    assert!(names.contains(&"close_agent"), "should have close_agent");
    // Core tools still present
    assert!(names.contains(&"edit_file"), "should still have edit_file");
    Ok(())
}

#[test]
fn gemini_profile_includes_subagent_tools_after_registration() -> AgentResult<()> {
    let mut profile = GeminiProfile::new("gemini-2.5-pro", 600_000)?;
    profile.register_subagent_tools()?;

    let names = profile.tool_registry().names();
    assert!(names.contains(&"spawn_agent"), "should have spawn_agent");
    assert!(names.contains(&"close_agent"), "should have close_agent");
    // Gemini-specific tools still present
    assert!(
        names.contains(&"read_many_files"),
        "should still have read_many_files"
    );
    assert!(names.contains(&"list_dir"), "should still have list_dir");
    Ok(())
}

// ===========================================================================
// Spawn tests (spec 7.1, 7.3, 9.9)
// ===========================================================================

#[tokio::test]
async fn spawn_creates_independent_session() -> AgentResult<()> {
    // Parent session: LLM returns spawn_agent call, then text.
    // Child session: LLM returns text response for the child's task.
    // Mock client serves responses in order: parent call 1, child call 1, parent call 2.
    let (mut session, mut rx, client, _env) = test_session(vec![
        // Parent's first LLM call: spawn a subagent
        tool_call_response("", vec![spawn_call("Write a test")]),
        // Child's LLM call: subagent completes its task
        text_response("Test written successfully"),
        // Parent's second LLM call: natural completion
        text_response("Subagent finished the task"),
    ])?;

    session.submit("Spawn an agent to write tests").await?;
    assert_eq!(session.state(), stencila_agents::types::SessionState::Idle);

    // Parent should have history entries
    assert!(session.history().len() >= 2, "Parent should have history");

    // Check spawn_agent tool result was recorded
    let tool_results = session
        .history()
        .iter()
        .find(|t| matches!(t, stencila_agents::types::Turn::ToolResults { .. }));
    assert!(
        tool_results.is_some(),
        "Should have tool results from spawn_agent"
    );

    // The tool result should contain agent_id and status
    if let Some(stencila_agents::types::Turn::ToolResults { results, .. }) = tool_results {
        assert_eq!(results.len(), 1);
        assert!(!results[0].is_error, "spawn_agent should not return error");
        let content = results[0].content.as_str().unwrap_or("");
        assert!(
            content.contains("agent-1"),
            "Result should contain agent_id: {content}"
        );
        assert!(
            content.contains("completed"),
            "Result should contain status: {content}"
        );
    }

    // Check events include tool call start/end for spawn_agent
    let events = drain_events(&mut rx).await;
    let tool_starts: Vec<_> = events
        .iter()
        .filter(|e| e.kind == EventKind::ToolCallStart)
        .collect();
    assert!(
        tool_starts
            .iter()
            .any(|e| e.data.get("tool_name").and_then(|v| v.as_str()) == Some("spawn_agent")),
        "Should have TOOL_CALL_START for spawn_agent"
    );

    // Verify child session used shared client (it consumed a response)
    let requests = client.take_requests()?;
    // Parent call 1 + child call 1 + parent call 2 = 3 requests
    assert_eq!(
        requests.len(),
        3,
        "Should have 3 LLM requests (parent + child + parent)"
    );

    Ok(())
}

#[tokio::test]
async fn spawn_shares_execution_environment() -> AgentResult<()> {
    // The child session should share the parent's execution environment.
    // We verify this by checking the child's requests reference the same
    // working directory.
    let (mut session, _rx, client, _env) = test_session(vec![
        tool_call_response("", vec![spawn_call("Check the environment")]),
        text_response("Environment checked"),
        text_response("All good"),
    ])?;

    session.submit("Verify env sharing").await?;

    // The child's system prompt should reference the same working directory
    let requests = client.take_requests()?;
    // Request 1 = parent's first, Request 2 = child's
    assert!(requests.len() >= 2);
    let child_system = requests[1].messages[0].text();
    assert!(
        child_system.contains("/tmp/test"),
        "Child should reference parent's working dir in system prompt: {child_system}"
    );

    Ok(())
}

#[tokio::test]
async fn spawn_with_custom_max_turns() -> AgentResult<()> {
    // Subagent with max_turns=1 should be limited.
    // The child gets exactly 1 LLM call (text response), so turns_used=1.
    let (mut session, _rx, _client, _env) = test_session(vec![
        tool_call_response("", vec![spawn_call_with_limit("Quick task", 1)]),
        // Child's first (and only) LLM call
        text_response("Done quickly"),
        // Parent's second call
        text_response("Agent finished"),
    ])?;

    session.submit("Quick spawn").await?;
    assert_eq!(session.state(), stencila_agents::types::SessionState::Idle);

    // Verify the spawn result includes turns_used
    let tool_results = session
        .history()
        .iter()
        .find(|t| matches!(t, stencila_agents::types::Turn::ToolResults { .. }));
    if let Some(stencila_agents::types::Turn::ToolResults { results, .. }) = tool_results {
        assert!(!results[0].is_error, "spawn should succeed");
        let content = results[0].content.as_str().unwrap_or("");
        let parsed: serde_json::Value =
            serde_json::from_str(content).map_err(|e| AgentError::Io {
                message: e.to_string(),
            })?;
        assert_eq!(
            parsed.get("turns_used").and_then(|v| v.as_u64()),
            Some(1),
            "Child should have used exactly 1 turn"
        );
        assert_eq!(
            parsed.get("success").and_then(|v| v.as_bool()),
            Some(true),
            "Child should have succeeded"
        );
    }
    Ok(())
}

// ===========================================================================
// Depth limiting tests (spec 7.3, 9.9)
// ===========================================================================

#[tokio::test]
async fn depth_limiting_blocks_sub_sub_agents() -> AgentResult<()> {
    // Create a session at depth 0 with max_subagent_depth=1.
    // The parent spawns a child (depth 1). The child tries to spawn a
    // grandchild — this should fail because depth 1 >= max_depth 1.
    //
    // Mock responses:
    // 1. Parent LLM: spawn_agent("Level 1 task")
    // 2. Child LLM: spawn_agent("Level 2 task") — this should fail
    // 3. Child LLM: natural completion after depth error
    // 4. Parent LLM: natural completion
    let config = SessionConfig {
        max_subagent_depth: 1,
        ..SessionConfig::default()
    };

    let (mut session, _rx, _client, _env) = test_session_with_config(
        vec![
            // Parent: spawn a child
            tool_call_response("", vec![spawn_call("Level 1 task")]),
            // Child: tries to spawn a grandchild
            tool_call_response("", vec![spawn_call("Level 2 task")]),
            // Child: gets error result, produces text
            text_response("Could not spawn sub-agent, completing directly"),
            // Parent: natural completion
            text_response("Child finished"),
        ],
        config,
    )?;

    session.submit("Deep task").await?;
    assert_eq!(session.state(), stencila_agents::types::SessionState::Idle);

    // The child's spawn_agent call should have returned an error
    // (depth exceeded), but the parent should still complete.
    Ok(())
}

#[tokio::test]
async fn depth_zero_allows_no_subagents() -> AgentResult<()> {
    // With max_subagent_depth=0, even the top-level session cannot spawn.
    let config = SessionConfig {
        max_subagent_depth: 0,
        ..SessionConfig::default()
    };

    let (mut session, _rx, _client, _env) = test_session_with_config(
        vec![
            tool_call_response("", vec![spawn_call("Should fail")]),
            text_response("Spawn failed, continuing without subagent"),
        ],
        config,
    )?;

    session.submit("Try to spawn").await?;
    assert_eq!(session.state(), stencila_agents::types::SessionState::Idle);

    // Check the tool result has is_error=true
    let tool_results = session
        .history()
        .iter()
        .find(|t| matches!(t, stencila_agents::types::Turn::ToolResults { .. }));
    if let Some(stencila_agents::types::Turn::ToolResults { results, .. }) = tool_results {
        assert!(
            results[0].is_error,
            "spawn_agent with depth=0 should return error"
        );
        let content = results[0].content.as_str().unwrap_or("");
        assert!(
            content.contains("depth") || content.contains("exceeded"),
            "Error should mention depth: {content}"
        );
    }

    Ok(())
}

// ===========================================================================
// send_input / wait / close_agent tests (spec 7.2, 9.9)
// ===========================================================================

#[tokio::test]
async fn unknown_agent_id_returns_error() -> AgentResult<()> {
    // Send input to an agent that doesn't exist
    let (mut session, _rx, _client, _env) = test_session(vec![
        tool_call_response("", vec![send_input_call("nonexistent-agent", "hello")]),
        text_response("Got error for unknown agent"),
    ])?;

    session.submit("Send to unknown").await?;
    assert_eq!(session.state(), stencila_agents::types::SessionState::Idle);

    // Tool result should be an error
    let tool_results = session
        .history()
        .iter()
        .find(|t| matches!(t, stencila_agents::types::Turn::ToolResults { .. }));
    if let Some(stencila_agents::types::Turn::ToolResults { results, .. }) = tool_results {
        assert!(
            results[0].is_error,
            "send_input with unknown agent_id should error"
        );
        let content = results[0].content.as_str().unwrap_or("");
        assert!(
            content.contains("unknown") || content.contains("nonexistent"),
            "Error should mention unknown agent: {content}"
        );
    }

    Ok(())
}

#[tokio::test]
async fn wait_unknown_agent_returns_error() -> AgentResult<()> {
    let (mut session, _rx, _client, _env) = test_session(vec![
        tool_call_response("", vec![wait_call("no-such-agent")]),
        text_response("Agent not found"),
    ])?;

    session.submit("Wait for unknown").await?;
    assert_eq!(session.state(), stencila_agents::types::SessionState::Idle);

    let tool_results = session
        .history()
        .iter()
        .find(|t| matches!(t, stencila_agents::types::Turn::ToolResults { .. }));
    if let Some(stencila_agents::types::Turn::ToolResults { results, .. }) = tool_results {
        assert!(
            results[0].is_error,
            "wait with unknown agent_id should error"
        );
    }

    Ok(())
}

#[tokio::test]
async fn close_unknown_agent_returns_error() -> AgentResult<()> {
    let (mut session, _rx, _client, _env) = test_session(vec![
        tool_call_response("", vec![close_call("no-such-agent")]),
        text_response("Agent not found"),
    ])?;

    session.submit("Close unknown").await?;

    let tool_results = session
        .history()
        .iter()
        .find(|t| matches!(t, stencila_agents::types::Turn::ToolResults { .. }));
    if let Some(stencila_agents::types::Turn::ToolResults { results, .. }) = tool_results {
        assert!(
            results[0].is_error,
            "close_agent with unknown agent_id should error"
        );
    }

    Ok(())
}

// ===========================================================================
// Success-path tests for send_input, wait, close_agent (spec 7.2)
// ===========================================================================

#[tokio::test]
async fn send_input_success_after_spawn() -> AgentResult<()> {
    // Spawn an agent, then send additional input to it.
    // Mock responses (consumed in order):
    // 1. Parent: spawn_agent call
    // 2. Child (spawn): completes task
    // 3. Parent: send_input call
    // 4. Child (send_input): processes follow-up
    // 5. Parent: natural completion
    let (mut session, _rx, _client, _env) = test_session(vec![
        // Parent's first LLM call: spawn
        tool_call_response("", vec![spawn_call("Initial task")]),
        // Child completes spawn
        text_response("Initial task done"),
        // Parent's second LLM call: send_input to agent-1
        tool_call_response("", vec![send_input_call("agent-1", "Do more work")]),
        // Child processes send_input
        text_response("Follow-up work done"),
        // Parent's third LLM call: natural completion
        text_response("All done"),
    ])?;

    session.submit("Spawn and then interact").await?;
    assert_eq!(session.state(), stencila_agents::types::SessionState::Idle);

    // Find all tool results
    let tool_results: Vec<_> = session
        .history()
        .iter()
        .filter_map(|t| {
            if let stencila_agents::types::Turn::ToolResults { results, .. } = t {
                Some(results)
            } else {
                None
            }
        })
        .collect();

    // Should have 2 tool result turns: spawn_agent and send_input
    assert_eq!(tool_results.len(), 2, "Should have 2 tool result turns");

    // First: spawn_agent result
    assert!(!tool_results[0][0].is_error, "spawn should succeed");
    let spawn_content = tool_results[0][0].content.as_str().unwrap_or("");
    assert!(
        spawn_content.contains("agent-1"),
        "Spawn should return agent-1: {spawn_content}"
    );

    // Second: send_input result
    assert!(
        !tool_results[1][0].is_error,
        "send_input should succeed: {:?}",
        tool_results[1][0].content
    );
    let input_content = tool_results[1][0].content.as_str().unwrap_or("");
    assert!(
        input_content.contains("completed"),
        "send_input result should show completed: {input_content}"
    );

    Ok(())
}

#[tokio::test]
async fn wait_success_after_spawn() -> AgentResult<()> {
    // Spawn an agent, then wait for its result.
    let (mut session, _rx, _client, _env) = test_session(vec![
        // Parent: spawn
        tool_call_response("", vec![spawn_call("Do something")]),
        // Child completes
        text_response("Task completed successfully"),
        // Parent: wait
        tool_call_response("", vec![wait_call("agent-1")]),
        // Parent: natural completion
        text_response("Got the result"),
    ])?;

    session.submit("Spawn and wait").await?;
    assert_eq!(session.state(), stencila_agents::types::SessionState::Idle);

    let tool_results: Vec<_> = session
        .history()
        .iter()
        .filter_map(|t| {
            if let stencila_agents::types::Turn::ToolResults { results, .. } = t {
                Some(results)
            } else {
                None
            }
        })
        .collect();

    assert_eq!(tool_results.len(), 2, "Should have 2 tool result turns");

    // Wait result should contain the subagent output
    assert!(!tool_results[1][0].is_error, "wait should succeed");
    let wait_content = tool_results[1][0].content.as_str().unwrap_or("");
    let parsed: serde_json::Value =
        serde_json::from_str(wait_content).map_err(|e| AgentError::Io {
            message: e.to_string(),
        })?;
    assert_eq!(
        parsed.get("status").and_then(|v| v.as_str()),
        Some("completed"),
        "Wait should report completed status"
    );
    assert_eq!(
        parsed.get("success").and_then(|v| v.as_bool()),
        Some(true),
        "Wait should report success"
    );
    assert!(
        parsed
            .get("turns_used")
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            > 0,
        "Wait should report turns_used > 0"
    );

    Ok(())
}

#[tokio::test]
async fn close_agent_success_after_spawn() -> AgentResult<()> {
    // Spawn an agent, then close it.
    let (mut session, _rx, _client, _env) = test_session(vec![
        // Parent: spawn
        tool_call_response("", vec![spawn_call("Temporary task")]),
        // Child completes
        text_response("Done"),
        // Parent: close
        tool_call_response("", vec![close_call("agent-1")]),
        // Parent: natural completion
        text_response("Agent closed"),
    ])?;

    session.submit("Spawn and close").await?;
    assert_eq!(session.state(), stencila_agents::types::SessionState::Idle);

    let tool_results: Vec<_> = session
        .history()
        .iter()
        .filter_map(|t| {
            if let stencila_agents::types::Turn::ToolResults { results, .. } = t {
                Some(results)
            } else {
                None
            }
        })
        .collect();

    assert_eq!(tool_results.len(), 2, "Should have 2 tool result turns");

    // Close result
    assert!(!tool_results[1][0].is_error, "close should succeed");
    let close_content = tool_results[1][0].content.as_str().unwrap_or("");
    let parsed: serde_json::Value =
        serde_json::from_str(close_content).map_err(|e| AgentError::Io {
            message: e.to_string(),
        })?;
    assert_eq!(
        parsed.get("closed").and_then(|v| v.as_bool()),
        Some(true),
        "Close should report closed=true"
    );

    Ok(())
}

#[tokio::test]
async fn close_then_wait_returns_error() -> AgentResult<()> {
    // After close_agent, wait should return an error (agent removed from map).
    let (mut session, _rx, _client, _env) = test_session(vec![
        // Parent: spawn
        tool_call_response("", vec![spawn_call("Task")]),
        // Child completes
        text_response("Done"),
        // Parent: close then wait in sequence
        tool_call_response("", vec![close_call("agent-1"), wait_call("agent-1")]),
        // Parent: natural completion
        text_response("Handled errors"),
    ])?;

    session.submit("Close then wait").await?;
    assert_eq!(session.state(), stencila_agents::types::SessionState::Idle);

    // Find the tool results for the close+wait turn
    let tool_results: Vec<_> = session
        .history()
        .iter()
        .filter_map(|t| {
            if let stencila_agents::types::Turn::ToolResults { results, .. } = t {
                Some(results)
            } else {
                None
            }
        })
        .collect();

    // Second turn has close + wait results
    assert!(tool_results.len() >= 2);
    let close_wait_results = &tool_results[1];
    assert_eq!(
        close_wait_results.len(),
        2,
        "Should have close and wait results"
    );
    assert!(!close_wait_results[0].is_error, "close should succeed");
    assert!(
        close_wait_results[1].is_error,
        "wait after close should error (agent removed)"
    );

    Ok(())
}

#[tokio::test]
async fn send_input_to_failed_agent_returns_error() -> AgentResult<()> {
    // Spawn an agent that fails, then try to send input to it.
    // We make the child fail by having its LLM call return an error.
    use stencila_models3::error::ProviderDetails;
    let (mut session, _rx, _client, _env) = test_session(vec![
        // Parent: spawn
        tool_call_response("", vec![spawn_call("Doomed task")]),
        // Child's LLM call fails
        Err(SdkError::Server {
            message: "internal server error".into(),
            details: ProviderDetails {
                provider: None,
                status_code: Some(500),
                error_code: None,
                retryable: true,
                retry_after: None,
                raw: None,
            },
        }),
        // Parent: send_input to the failed agent
        tool_call_response("", vec![send_input_call("agent-1", "hello")]),
        // Parent: natural completion
        text_response("Got error for failed agent"),
    ])?;

    session.submit("Spawn and fail").await?;
    assert_eq!(session.state(), stencila_agents::types::SessionState::Idle);

    let tool_results: Vec<_> = session
        .history()
        .iter()
        .filter_map(|t| {
            if let stencila_agents::types::Turn::ToolResults { results, .. } = t {
                Some(results)
            } else {
                None
            }
        })
        .collect();

    assert!(tool_results.len() >= 2);
    // send_input to failed agent should have is_error=true
    assert!(
        tool_results[1][0].is_error,
        "send_input to failed agent should return error"
    );
    let content = tool_results[1][0].content.as_str().unwrap_or("");
    assert!(
        content.contains("failed"),
        "Error should mention 'failed': {content}"
    );
    Ok(())
}

// ===========================================================================
// working_dir and model override tests (spec 7.2)
// ===========================================================================

#[tokio::test]
async fn spawn_with_working_dir_scopes_system_prompt() -> AgentResult<()> {
    // When working_dir is specified, the child's system prompt should include
    // a scoping directive for that directory.
    let spawn_with_dir = ToolCall {
        id: format!("call-{}", uuid::Uuid::new_v4()),
        name: "spawn_agent".into(),
        arguments: json!({"task": "Work in subdir", "working_dir": "src/components"}),
        raw_arguments: None,
        parse_error: None,
    };

    let (mut session, _rx, client, _env) = test_session(vec![
        tool_call_response("", vec![spawn_with_dir]),
        // Child completes
        text_response("Worked in components"),
        // Parent completes
        text_response("Done"),
    ])?;

    session.submit("Scope to subdir").await?;

    // Check the child's request (request index 1) has working_dir in system prompt
    let requests = client.take_requests()?;
    assert!(requests.len() >= 2, "Should have parent + child requests");
    let child_system = requests[1].messages[0].text();
    assert!(
        child_system.contains("src/components"),
        "Child system prompt should mention working_dir: {child_system}"
    );
    assert!(
        child_system.contains("scoped to the subdirectory"),
        "Child system prompt should have scoping directive: {child_system}"
    );

    Ok(())
}

#[tokio::test]
async fn spawn_with_model_override_uses_custom_model() -> AgentResult<()> {
    // When model is specified, the child's request should use that model.
    let spawn_with_model = ToolCall {
        id: format!("call-{}", uuid::Uuid::new_v4()),
        name: "spawn_agent".into(),
        arguments: json!({"task": "Use custom model", "model": "claude-sonnet-4-5-20250929"}),
        raw_arguments: None,
        parse_error: None,
    };

    let (mut session, _rx, client, _env) = test_session(vec![
        tool_call_response("", vec![spawn_with_model]),
        // Child completes
        text_response("Used custom model"),
        // Parent completes
        text_response("Done"),
    ])?;

    session.submit("Custom model spawn").await?;

    // Check the child's request uses the overridden model
    let requests = client.take_requests()?;
    assert!(requests.len() >= 2, "Should have parent + child requests");
    assert_eq!(
        requests[1].model, "claude-sonnet-4-5-20250929",
        "Child should use the overridden model"
    );

    Ok(())
}

// ===========================================================================
// Auto-registration test (finding 3)
// ===========================================================================

#[tokio::test]
async fn session_auto_registers_subagent_tools_when_depth_allows() -> AgentResult<()> {
    // When max_subagent_depth > 0 and current_depth < max_depth,
    // Session::new should auto-register subagent tools.
    // We verify by submitting and checking the LLM request includes them.
    let profile = AnthropicProfile::new("claude-opus-4-6", 600_000)?;
    assert!(
        !profile.tool_registry().names().contains(&"spawn_agent"),
        "Profile should not have spawn_agent before session creation"
    );

    let client = Arc::new(MockClient::new(vec![text_response("hi")]));
    let env = Arc::new(MockExecEnv::new());
    let config = SessionConfig {
        max_subagent_depth: 1,
        ..SessionConfig::default()
    };

    let (mut session, _rx) = Session::new(
        Box::new(profile),
        env as Arc<dyn ExecutionEnvironment>,
        client.clone(),
        config,
        "test".into(),
        0,
    );

    session.submit("hello").await?;

    // Check the LLM request includes subagent tools
    let requests = client.take_requests()?;
    assert!(!requests.is_empty());
    let tools = requests[0]
        .tools
        .as_ref()
        .expect("request should have tools");
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    assert!(
        tool_names.contains(&"spawn_agent"),
        "Request should include spawn_agent after auto-registration: {tool_names:?}"
    );
    assert!(
        tool_names.contains(&"send_input"),
        "Request should include send_input after auto-registration: {tool_names:?}"
    );
    assert!(
        tool_names.contains(&"wait"),
        "Request should include wait after auto-registration: {tool_names:?}"
    );
    assert!(
        tool_names.contains(&"close_agent"),
        "Request should include close_agent after auto-registration: {tool_names:?}"
    );
    Ok(())
}

#[tokio::test]
async fn session_does_not_register_subagent_tools_at_max_depth() -> AgentResult<()> {
    // When current_depth >= max_depth, subagent tools should NOT be registered.
    // We verify this by spawning at depth=1 with max_depth=1: the child
    // should not get subagent tools (depth 1 >= max 1).
    let config = SessionConfig {
        max_subagent_depth: 1,
        ..SessionConfig::default()
    };

    let (mut session, _rx, client, _env) = test_session_with_config(
        vec![
            // Parent: spawn child
            tool_call_response("", vec![spawn_call("Check child tools")]),
            // Child: natural completion (no subagent tools available)
            text_response("No subagent tools here"),
            // Parent: natural completion
            text_response("Done"),
        ],
        config,
    )?;

    session.submit("Check child tools").await?;

    // The child (depth 1, max_depth 1) should NOT have subagent tools in its request
    let requests = client.take_requests()?;
    assert!(requests.len() >= 2);
    let child_tools = &requests[1].tools;
    if let Some(tools) = child_tools {
        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(
            !tool_names.contains(&"spawn_agent"),
            "Child at max depth should not have spawn_agent tool: {tool_names:?}"
        );
    }

    Ok(())
}

// ===========================================================================
// is_subagent_tool classification
// ===========================================================================

#[test]
fn is_subagent_tool_recognizes_all_four() {
    use stencila_agents::subagents::SubAgentManager;
    assert!(SubAgentManager::is_subagent_tool("spawn_agent"));
    assert!(SubAgentManager::is_subagent_tool("send_input"));
    assert!(SubAgentManager::is_subagent_tool("wait"));
    assert!(SubAgentManager::is_subagent_tool("close_agent"));
}

#[test]
fn is_subagent_tool_rejects_regular_tools() {
    use stencila_agents::subagents::SubAgentManager;
    assert!(!SubAgentManager::is_subagent_tool("echo"));
    assert!(!SubAgentManager::is_subagent_tool("read_file"));
    assert!(!SubAgentManager::is_subagent_tool("shell"));
    assert!(!SubAgentManager::is_subagent_tool(""));
}

// ===========================================================================
// SubAgentResult type tests
// ===========================================================================

#[test]
fn subagent_result_serde_roundtrip() -> AgentResult<()> {
    let result = subagents::SubAgentResult {
        output: "task completed".into(),
        success: true,
        turns_used: 3,
    };
    let json = serde_json::to_string(&result).map_err(|e| AgentError::Io {
        message: e.to_string(),
    })?;
    let parsed: subagents::SubAgentResult =
        serde_json::from_str(&json).map_err(|e| AgentError::Io {
            message: e.to_string(),
        })?;
    assert_eq!(parsed.output, "task completed");
    assert!(parsed.success);
    assert_eq!(parsed.turns_used, 3);
    Ok(())
}

#[test]
fn subagent_status_serde_roundtrip() -> AgentResult<()> {
    let statuses = vec![
        subagents::SubAgentStatus::Running,
        subagents::SubAgentStatus::Completed,
        subagents::SubAgentStatus::Failed,
    ];
    for status in statuses {
        let json = serde_json::to_string(&status).map_err(|e| AgentError::Io {
            message: e.to_string(),
        })?;
        let parsed: subagents::SubAgentStatus =
            serde_json::from_str(&json).map_err(|e| AgentError::Io {
                message: e.to_string(),
            })?;
        assert_eq!(parsed, status);
    }
    Ok(())
}

// ===========================================================================
// Missing parameter validation
// ===========================================================================

#[tokio::test]
async fn spawn_without_task_returns_error() -> AgentResult<()> {
    // spawn_agent without the required "task" parameter
    let bad_spawn = ToolCall {
        id: format!("call-{}", uuid::Uuid::new_v4()),
        name: "spawn_agent".into(),
        arguments: json!({}), // missing "task"
        raw_arguments: None,
        parse_error: None,
    };

    let (mut session, _rx, _client, _env) = test_session(vec![
        tool_call_response("", vec![bad_spawn]),
        text_response("Got validation error"),
    ])?;

    session.submit("Bad spawn").await?;

    let tool_results = session
        .history()
        .iter()
        .find(|t| matches!(t, stencila_agents::types::Turn::ToolResults { .. }));
    if let Some(stencila_agents::types::Turn::ToolResults { results, .. }) = tool_results {
        assert!(results[0].is_error, "Missing task should produce error");
        let content = results[0].content.as_str().unwrap_or("");
        assert!(
            content.contains("task"),
            "Error should mention 'task': {content}"
        );
    }

    Ok(())
}
