//! Phase 4 / Slices 2-3 tests: Processing-state normalization during API
//! session hydration, and CLI provider `resume_state`/`set_resume_state`.
//!
//! Acceptance criteria tested:
//!
//! - AC-S2-1: Hydrating a session persisted with `Processing` state and an
//!   incomplete final assistant turn normalizes state to `Idle` and
//!   drops the incomplete turn.
//! - AC-S2-2: Hydrating a session with `Idle` state and complete turns
//!   preserves all turns (regression guard).
//! - AC-S3-1: `ClaudeCliProvider::resume_state()` returns the session
//!   continuation ID; `set_resume_state()` re-imports it.
//! - AC-S3-2: `CodexCliProvider::resume_state()` returns the conversation ID;
//!   `set_resume_state()` re-imports it.
//! - AC-S3-3: `GeminiCliProvider::resume_state()` returns `None`.
//! - AC-S3-4: Default `CliProvider` trait `resume_state()` returns `None` and
//!   `set_resume_state()` returns `Ok(())`.
//!
//! All tests use in-memory SQLite and mock clients / providers.

#![allow(clippy::result_large_err)]

use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde_json::json;
use stencila_agents::api_session::LlmClient;
use stencila_agents::cli_providers::claude::ClaudeCliProvider;
use stencila_agents::cli_providers::codex::CodexCliProvider;
use stencila_agents::cli_providers::gemini::GeminiCliProvider;
use stencila_agents::cli_providers::{CliProvider, CliProviderConfig};
use stencila_agents::error::AgentResult;
use stencila_agents::execution::{ExecutionEnvironment, FileContent};
use stencila_agents::migrations::AGENT_MIGRATIONS;
use stencila_agents::profile::ProviderProfile;
use stencila_agents::registry::ToolRegistry;
use stencila_agents::store::{AgentSessionStore, Resumability, SessionRecord};
use stencila_agents::types::{DirEntry, ExecResult, GrepOptions, SessionState, Turn};
use stencila_models3::error::SdkError;
use stencila_models3::types::content::ContentPart;
use stencila_models3::types::finish_reason::{FinishReason, Reason};
use stencila_models3::types::message::Message;
use stencila_models3::types::request::Request;
use stencila_models3::types::response::Response;
use stencila_models3::types::role::Role;
use stencila_models3::types::usage::Usage;

// ===========================================================================
// Test infrastructure (mirrors spec_16 conventions)
// ===========================================================================

fn setup_db() -> Arc<Mutex<stencila_db::rusqlite::Connection>> {
    let conn = stencila_db::rusqlite::Connection::open_in_memory().expect("open in-memory SQLite");
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .expect("enable FK");
    for m in AGENT_MIGRATIONS {
        conn.execute_batch(m.sql).expect("apply migration");
    }
    Arc::new(Mutex::new(conn))
}

fn make_store(conn: &Arc<Mutex<stencila_db::rusqlite::Connection>>) -> Arc<AgentSessionStore> {
    Arc::new(AgentSessionStore::new(conn.clone()))
}

fn text_response(text: &str) -> Response {
    Response {
        id: "resp-1".into(),
        model: "test-model".into(),
        provider: "test".into(),
        message: Message {
            role: Role::Assistant,
            content: vec![ContentPart::Text {
                text: text.to_string(),
            }],
            name: None,
            tool_call_id: None,
        },
        finish_reason: FinishReason::new(Reason::Stop, None),
        usage: Usage::default(),
        raw: None,
        warnings: None,
        rate_limit: None,
    }
}

fn sample_record(session_id: &str) -> SessionRecord {
    SessionRecord {
        session_id: session_id.to_string(),
        backend_kind: "api".to_string(),
        agent_name: "general".to_string(),
        provider_name: "test".to_string(),
        model_name: "test-model".to_string(),
        state: SessionState::Idle,
        total_turns: 0,
        resumability: Resumability::Full,
        created_at: "2025-07-01T00:00:00Z".to_string(),
        updated_at: "2025-07-01T00:00:00Z".to_string(),
        workflow_run_id: None,
        workflow_thread_id: None,
        workflow_node_id: None,
        provider_resume_state: None,
        config_snapshot: None,
        system_prompt: None,
        lease_holder: None,
        lease_expires_at: None,
    }
}

// -- Mock LLM Client --------------------------------------------------------

struct MockClient {
    responses: Mutex<VecDeque<Result<Response, SdkError>>>,
}

impl MockClient {
    fn with_text_response(text: &str) -> Self {
        Self {
            responses: Mutex::new(VecDeque::from(vec![Ok(text_response(text))])),
        }
    }
}

#[async_trait]
impl LlmClient for MockClient {
    async fn complete(&self, _request: Request) -> Result<Response, SdkError> {
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

// -- Mock Execution Environment ----------------------------------------------

struct MockExecEnv;

#[async_trait]
impl ExecutionEnvironment for MockExecEnv {
    async fn read_file(
        &self,
        path: &str,
        _offset: Option<usize>,
        _limit: Option<usize>,
    ) -> AgentResult<FileContent> {
        Ok(FileContent::Text(format!("content of {path}")))
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
        "/tmp/test"
    }
    fn platform(&self) -> &str {
        "linux"
    }
    fn os_version(&self) -> String {
        "test-os".into()
    }
}

// -- Test Profile ------------------------------------------------------------

#[derive(Debug)]
struct TestProfile {
    registry: ToolRegistry,
}

impl TestProfile {
    fn new() -> Self {
        Self {
            registry: ToolRegistry::new(),
        }
    }
}

impl ProviderProfile for TestProfile {
    fn id(&self) -> &str {
        "test"
    }
    fn model(&self) -> &str {
        "test-model"
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
        false
    }
    fn supports_streaming(&self) -> bool {
        false
    }
    fn supports_parallel_tool_calls(&self) -> bool {
        false
    }
    fn supports_vision(&self) -> bool {
        false
    }
    fn context_window_size(&self) -> u64 {
        200_000
    }
}

// ===========================================================================
// AC-S2-1: Hydrating a session persisted with Processing state and an
//          incomplete final assistant turn → state normalised to Idle,
//          incomplete turn dropped
// ===========================================================================

#[tokio::test]
async fn hydrate_normalizes_processing_state_to_idle_and_drops_incomplete_turn() {
    let db = setup_db();
    let store = make_store(&db);

    // Build a record that was persisted while the session was still Processing
    let mut record = sample_record("sess-processing");
    record.state = SessionState::Processing;
    record.total_turns = 2;
    store.insert_session(&record).expect("insert");

    // Build history with a complete user+assistant exchange, then an incomplete
    // final assistant turn (empty content — represents a partial/interrupted
    // response that hadn't finished streaming).
    let turns = vec![
        Turn::user("Hello"),
        Turn::assistant("Hi there!"),
        Turn::user("Second question"),
        Turn::assistant(""), // incomplete: empty content, no tool_calls
    ];
    store
        .checkpoint_turns("sess-processing", &turns)
        .expect("checkpoint turns");

    // Hydrate
    let client: Arc<dyn LlmClient> = Arc::new(MockClient::with_text_response("reply"));
    let env = Arc::new(MockExecEnv);
    let profile = Box::new(TestProfile::new());

    let (hydrated, _event_rx) = stencila_agents::api_session::hydrate_api_session(
        profile,
        env,
        client,
        "You are a test assistant.".into(),
        &record,
        turns,
    );

    // State should be normalised to Idle
    assert_eq!(
        hydrated.state(),
        SessionState::Idle,
        "hydrated session should normalize Processing to Idle"
    );

    // The incomplete final assistant turn should have been dropped.
    // We started with 4 turns; the last one (empty assistant) should be removed,
    // leaving 3 turns (user, assistant, user).
    assert_eq!(
        hydrated.history().len(),
        3,
        "incomplete final assistant turn should be dropped (4 → 3)"
    );

    // The last remaining turn should be the second user message
    match hydrated.history().last() {
        Some(Turn::User { content, .. }) => {
            assert_eq!(content, "Second question");
        }
        other => panic!("expected last turn to be User('Second question'), got: {other:?}"),
    }
}

#[tokio::test]
async fn hydrate_normalizes_processing_drops_incomplete_turn_with_no_content() {
    // Variant: the incomplete assistant turn has content that is only whitespace
    let db = setup_db();
    let store = make_store(&db);

    let mut record = sample_record("sess-processing-ws");
    record.state = SessionState::Processing;
    record.total_turns = 1;
    store.insert_session(&record).expect("insert");

    let turns = vec![
        Turn::user("What is 2+2?"),
        Turn::assistant("   "), // whitespace-only — considered incomplete
    ];
    store
        .checkpoint_turns("sess-processing-ws", &turns)
        .expect("checkpoint");

    let client: Arc<dyn LlmClient> = Arc::new(MockClient::with_text_response("reply"));
    let env = Arc::new(MockExecEnv);
    let profile = Box::new(TestProfile::new());

    let (hydrated, _event_rx) = stencila_agents::api_session::hydrate_api_session(
        profile,
        env,
        client,
        "You are a test assistant.".into(),
        &record,
        turns,
    );

    assert_eq!(hydrated.state(), SessionState::Idle);
    assert_eq!(
        hydrated.history().len(),
        1,
        "whitespace-only final assistant turn should be dropped"
    );
    match hydrated.history().last() {
        Some(Turn::User { content, .. }) => {
            assert_eq!(content, "What is 2+2?");
        }
        other => panic!("expected User turn, got: {other:?}"),
    }
}

// ===========================================================================
// AC-S2-2: Hydrating a session with Idle state and complete turns preserves
//          all turns (regression guard)
// ===========================================================================

#[tokio::test]
async fn hydrate_idle_session_preserves_all_complete_turns() {
    let db = setup_db();
    let store = make_store(&db);

    let mut record = sample_record("sess-idle");
    record.state = SessionState::Idle;
    record.total_turns = 2;
    store.insert_session(&record).expect("insert");

    let turns = vec![
        Turn::user("Hello"),
        Turn::assistant("Hi! How can I help?"),
        Turn::user("Do something"),
        Turn::assistant("Done!"),
    ];
    store
        .checkpoint_turns("sess-idle", &turns)
        .expect("checkpoint");

    let client: Arc<dyn LlmClient> = Arc::new(MockClient::with_text_response("reply"));
    let env = Arc::new(MockExecEnv);
    let profile = Box::new(TestProfile::new());

    let (hydrated, _event_rx) = stencila_agents::api_session::hydrate_api_session(
        profile,
        env,
        client,
        "You are a test assistant.".into(),
        &record,
        turns.clone(),
    );

    assert_eq!(hydrated.state(), SessionState::Idle);
    assert_eq!(
        hydrated.history().len(),
        4,
        "Idle session with complete turns should preserve all 4 turns"
    );
}

#[tokio::test]
async fn hydrate_idle_session_with_nonempty_final_assistant_preserves_it() {
    // Even if state was Processing but the final assistant turn has real content,
    // it's "complete" and should be preserved. This tests the edge case.
    let db = setup_db();
    let store = make_store(&db);

    let mut record = sample_record("sess-proc-complete");
    record.state = SessionState::Processing;
    record.total_turns = 1;
    store.insert_session(&record).expect("insert");

    let turns = vec![
        Turn::user("Hello"),
        Turn::assistant("I was fully written before the crash."),
    ];
    store
        .checkpoint_turns("sess-proc-complete", &turns)
        .expect("checkpoint");

    let client: Arc<dyn LlmClient> = Arc::new(MockClient::with_text_response("reply"));
    let env = Arc::new(MockExecEnv);
    let profile = Box::new(TestProfile::new());

    let (hydrated, _event_rx) = stencila_agents::api_session::hydrate_api_session(
        profile,
        env,
        client,
        "You are a test assistant.".into(),
        &record,
        turns,
    );

    assert_eq!(hydrated.state(), SessionState::Idle);
    // The final assistant turn has real content — keep it
    assert_eq!(
        hydrated.history().len(),
        2,
        "Processing session with complete final assistant turn should preserve all turns"
    );
}

// ===========================================================================
// AC-S3-1: ClaudeCliProvider resume_state / set_resume_state
// ===========================================================================

#[test]
fn claude_cli_resume_state_returns_none_initially() {
    let config = CliProviderConfig {
        model: None,
        instructions: None,
        max_turns: None,
        working_dir: None,
    };
    let provider = ClaudeCliProvider::new(config);

    // Before any session is established, resume_state should return None
    assert!(
        provider.resume_state().is_none(),
        "resume_state should be None before session established"
    );
}

#[test]
fn claude_cli_resume_state_round_trips_through_set_resume_state() {
    let config = CliProviderConfig {
        model: None,
        instructions: None,
        max_turns: None,
        working_dir: None,
    };
    let mut provider = ClaudeCliProvider::new(config);

    // Set a session continuation ID via set_resume_state
    let input_state = json!({"session_id": "claude-sess-abc123"});
    provider
        .set_resume_state(input_state)
        .expect("set_resume_state should succeed");

    // resume_state should now return the session continuation ID
    let state = provider.resume_state();
    assert!(
        state.is_some(),
        "resume_state should be Some after set_resume_state"
    );

    let state_json = state.expect("just asserted Some");
    assert_eq!(
        state_json.get("session_id").and_then(|v| v.as_str()),
        Some("claude-sess-abc123"),
        "resume_state should contain the session_id that was set"
    );
}

#[test]
fn claude_cli_set_resume_state_reimports_session_id() {
    let config = CliProviderConfig {
        model: None,
        instructions: None,
        max_turns: None,
        working_dir: None,
    };
    let mut provider = ClaudeCliProvider::new(config);

    // Initially no session ID
    assert!(provider.session_id().is_none());

    // Set resume state with a session ID
    let state = json!({"session_id": "claude-sess-restored"});
    provider
        .set_resume_state(state)
        .expect("set_resume_state should succeed");

    assert_eq!(
        provider.session_id(),
        Some("claude-sess-restored"),
        "session_id should be restored after set_resume_state"
    );
}

// ===========================================================================
// AC-S3-2: CodexCliProvider resume_state / set_resume_state
// ===========================================================================

#[test]
fn codex_cli_resume_state_returns_none_initially() {
    let config = CliProviderConfig {
        model: None,
        instructions: None,
        max_turns: None,
        working_dir: None,
    };
    let provider = CodexCliProvider::new(config);

    // Before any conversation is established
    assert!(
        provider.resume_state().is_none(),
        "resume_state should be None before conversation established"
    );
}

#[test]
fn codex_cli_set_resume_state_reimports_conversation_id() {
    let config = CliProviderConfig {
        model: None,
        instructions: None,
        max_turns: None,
        working_dir: None,
    };
    let mut provider = CodexCliProvider::new(config);

    // Initially no conversation ID
    assert!(provider.session_id().is_none());

    // Set resume state with a conversation ID
    let state = json!({"conversation_id": "conv-restored-456"});
    provider
        .set_resume_state(state)
        .expect("set_resume_state should succeed");

    // After set_resume_state, the provider should report the conversation
    // (Codex uses codex_session_id or codex_conversation_id internally)
    let restored_state = provider.resume_state();
    assert!(
        restored_state.is_some(),
        "resume_state should be Some after restore"
    );
    assert_eq!(
        restored_state
            .expect("Some")
            .get("conversation_id")
            .and_then(|v| v.as_str()),
        Some("conv-restored-456"),
        "conversation_id should round-trip through set_resume_state"
    );
}

// ===========================================================================
// AC-S3-3: GeminiCliProvider resume_state returns None
// ===========================================================================

#[test]
fn gemini_cli_resume_state_returns_none() {
    let config = CliProviderConfig {
        model: None,
        instructions: None,
        max_turns: None,
        working_dir: None,
    };
    let provider = GeminiCliProvider::new(config);

    assert!(
        provider.resume_state().is_none(),
        "GeminiCliProvider should always return None for resume_state"
    );
}

#[test]
fn gemini_cli_set_resume_state_is_noop() {
    let config = CliProviderConfig {
        model: None,
        instructions: None,
        max_turns: None,
        working_dir: None,
    };
    let mut provider = GeminiCliProvider::new(config);

    // set_resume_state on Gemini should be a no-op that succeeds
    let result = provider.set_resume_state(json!({"some": "state"}));
    assert!(
        result.is_ok(),
        "GeminiCliProvider::set_resume_state should return Ok (no-op)"
    );

    // resume_state should still be None after set
    assert!(
        provider.resume_state().is_none(),
        "resume_state should still be None after set_resume_state on Gemini"
    );
}

// ===========================================================================
// AC-S3-4: Default CliProvider trait methods
// ===========================================================================

/// A minimal test provider that only implements required trait methods,
/// relying on default implementations for `resume_state` and `set_resume_state`.
#[derive(Debug)]
struct MinimalCliProvider;

#[async_trait]
impl CliProvider for MinimalCliProvider {
    fn id(&self) -> &str {
        "minimal-cli"
    }

    async fn submit(
        &mut self,
        _input: &str,
        _events: &stencila_agents::events::EventEmitter,
        _abort: Option<&stencila_agents::types::AbortSignal>,
    ) -> AgentResult<()> {
        Ok(())
    }

    fn close(&mut self) {}

    fn supports_resume(&self) -> bool {
        false
    }

    fn session_id(&self) -> Option<&str> {
        None
    }
}

#[test]
fn default_cli_provider_resume_state_returns_none() {
    let provider = MinimalCliProvider;
    assert!(
        provider.resume_state().is_none(),
        "default CliProvider::resume_state() should return None"
    );
}

#[test]
fn default_cli_provider_set_resume_state_returns_ok() {
    let mut provider = MinimalCliProvider;
    let result = provider.set_resume_state(json!({"anything": "here"}));
    assert!(
        result.is_ok(),
        "default CliProvider::set_resume_state() should return Ok(())"
    );
}
