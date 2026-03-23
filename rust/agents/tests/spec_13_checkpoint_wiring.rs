//! Phase 3 / Slices 1-2 tests: Checkpoint persistence wiring for ApiSession
//! and CliSession.
//!
//! Verifies that both session types accept an `Arc<AgentSessionStore>` and
//! `SessionPersistence` via `CreateSessionOptions`, that `checkpoint()` writes
//! the correct state/turns/resumability to the store, and that checkpoints
//! fire automatically at session creation, after submit, on AwaitingInput
//! transition, and on close.
//!
//! All tests use in-memory SQLite and mock clients / providers.

#![allow(clippy::result_large_err)]

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use stencila_agents::api_session::{ApiSession, ApiSessionInit, LlmClient};
use stencila_agents::cli_providers::{CliProvider, CliSession};
use stencila_agents::error::AgentResult;
use stencila_agents::events::EventEmitter;
use stencila_agents::execution::{ExecutionEnvironment, FileContent};
use stencila_agents::migrations::AGENT_MIGRATIONS;
use stencila_agents::profile::ProviderProfile;
use stencila_agents::registry::ToolRegistry;
use stencila_agents::store::{AgentSessionStore, Resumability, SessionPersistence};
use stencila_agents::types::{
    AbortSignal, DirEntry, ExecResult, GrepOptions, SessionConfig, SessionState,
};
use stencila_models3::error::SdkError;
use stencila_models3::types::content::ContentPart;
use stencila_models3::types::finish_reason::{FinishReason, Reason};
use stencila_models3::types::message::Message;
use stencila_models3::types::request::Request;
use stencila_models3::types::response::Response;
use stencila_models3::types::role::Role;
use stencila_models3::types::usage::Usage;

// ===========================================================================
// Helpers
// ===========================================================================

/// Create an in-memory SQLite database with all agent migrations applied.
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

/// Build a mock `Response` with the given text content.
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

// -- Mock CLI Provider -------------------------------------------------------

#[derive(Debug)]
struct MockCliProvider {
    resume_support: bool,
    cli_session_id: Option<String>,
}

impl MockCliProvider {
    fn new(resume_support: bool, cli_session_id: Option<String>) -> Self {
        Self {
            resume_support,
            cli_session_id,
        }
    }
}

#[async_trait]
impl CliProvider for MockCliProvider {
    fn id(&self) -> &str {
        "mock-cli"
    }

    async fn submit(
        &mut self,
        _input: &str,
        _events: &EventEmitter,
        _abort: Option<&AbortSignal>,
    ) -> AgentResult<()> {
        Ok(())
    }

    fn close(&mut self) {}

    fn supports_resume(&self) -> bool {
        self.resume_support
    }

    fn session_id(&self) -> Option<&str> {
        self.cli_session_id.as_deref()
    }
}

// ===========================================================================
// AC-1: ApiSession accepts Arc<AgentSessionStore> and SessionPersistence
//        via CreateSessionOptions
// ===========================================================================

/// The `CreateSessionOptions` struct must have a `store` field that accepts
/// `Option<Arc<AgentSessionStore>>`.
#[test]
fn create_session_options_has_store_field() {
    use stencila_agents::convenience::CreateSessionOptions;

    let db = setup_db();
    let store = make_store(&db);

    let opts = CreateSessionOptions {
        store: Some(store),
        ..Default::default()
    };
    assert!(
        opts.store.is_some(),
        "store field should accept Arc<AgentSessionStore>"
    );
}

/// When store is not provided it should default to None.
#[test]
fn create_session_options_store_defaults_to_none() {
    use stencila_agents::convenience::CreateSessionOptions;

    let opts = CreateSessionOptions::default();
    assert!(opts.store.is_none(), "store should default to None");
}

// ===========================================================================
// AC-2: After ApiSession with persistence + submit, DB has session record
//        with total_turns=1 and correct turn history
// ===========================================================================

#[tokio::test]
async fn api_session_submit_checkpoints_session_and_turns() {
    let db = setup_db();
    let store = make_store(&db);

    let client: Arc<dyn LlmClient> = Arc::new(MockClient::with_text_response("Hello back!"));
    let env = Arc::new(MockExecEnv);
    let config = SessionConfig::default();
    let profile = Box::new(TestProfile::new());

    let (mut session, _event_rx) = ApiSession::new(
        profile,
        env,
        client,
        config,
        "You are a test assistant.".into(),
        0,
        ApiSessionInit::default(),
    );

    // Wire persistence into the session
    session.set_persistence(store.clone(), SessionPersistence::Persistent);

    session
        .submit("Hello!")
        .await
        .expect("submit should succeed");

    // Read the session record from the DB
    let record = store
        .get_session(session.session_id())
        .expect("get_session should not error")
        .expect("session record should exist after submit");

    assert_eq!(
        record.total_turns, 1,
        "total_turns should be 1 after one submit"
    );

    // Verify turn history was checkpointed
    let turns = store.get_turns(session.session_id()).expect("get_turns");
    assert!(
        !turns.is_empty(),
        "turn history should be non-empty after submit"
    );
}

// ===========================================================================
// AC-3: ApiSession records resumability=Resumable at checkpoint time
// ===========================================================================

#[tokio::test]
async fn api_session_checkpoint_records_resumable() {
    let db = setup_db();
    let store = make_store(&db);

    let client: Arc<dyn LlmClient> = Arc::new(MockClient::with_text_response("Hi!"));
    let env = Arc::new(MockExecEnv);
    let config = SessionConfig::default();
    let profile = Box::new(TestProfile::new());

    let (mut session, _event_rx) = ApiSession::new(
        profile,
        env,
        client,
        config,
        "You are a test assistant.".into(),
        0,
        ApiSessionInit::default(),
    );

    session.set_persistence(store.clone(), SessionPersistence::Persistent);
    session.submit("test").await.expect("submit");

    let record = store
        .get_session(session.session_id())
        .expect("get_session")
        .expect("record exists");

    assert_eq!(
        record.resumability,
        Resumability::Full,
        "ApiSession should record resumability as Full (Resumable)"
    );
}

// ===========================================================================
// AC-4: CliSession accepts Arc<AgentSessionStore> and SessionPersistence
// ===========================================================================

#[test]
fn cli_session_accepts_store_and_persistence() {
    let db = setup_db();
    let store = make_store(&db);

    let provider = Box::new(MockCliProvider::new(true, Some("test-123".into())));
    let config = SessionConfig::default();

    let (mut session, _event_rx) = CliSession::new(provider, config);

    // Wire persistence
    session.set_persistence(store.clone(), SessionPersistence::Persistent);

    // Should not panic — the session now holds the store
    assert_eq!(session.state(), SessionState::Idle);
}

// ===========================================================================
// AC-5: CliSession with MockCliProvider persists provider_resume_state
// ===========================================================================

#[tokio::test]
async fn cli_session_persists_provider_resume_state() {
    let db = setup_db();
    let store = make_store(&db);

    let provider = Box::new(MockCliProvider::new(true, Some("test-123".into())));
    let config = SessionConfig::default();

    let (mut session, _event_rx) = CliSession::new(provider, config);
    session.set_persistence(store.clone(), SessionPersistence::Persistent);

    session.submit("Hello CLI!").await.expect("submit");

    let record = store
        .get_session(session.session_id())
        .expect("get_session")
        .expect("record exists");

    assert!(
        record.provider_resume_state.is_some(),
        "provider_resume_state should be set after CLI submit"
    );
    let resume_state = record.provider_resume_state.as_ref().expect("has value");
    assert!(
        resume_state.contains("test-123"),
        "provider_resume_state should contain the CLI session ID 'test-123', got: {resume_state}"
    );
}

// ===========================================================================
// AC-6: CliSession resumability depends on supports_resume + session_id
// ===========================================================================

#[tokio::test]
async fn cli_session_resumable_when_supports_resume_and_session_id() {
    let db = setup_db();
    let store = make_store(&db);

    let provider = Box::new(MockCliProvider::new(true, Some("sess-abc".into())));
    let config = SessionConfig::default();

    let (mut session, _event_rx) = CliSession::new(provider, config);
    session.set_persistence(store.clone(), SessionPersistence::Persistent);

    session.submit("hello").await.expect("submit");

    let record = store
        .get_session(session.session_id())
        .expect("get_session")
        .expect("record");

    assert_eq!(
        record.resumability,
        Resumability::Full,
        "CliSession should be Resumable (Full) when supports_resume()=true and session_id()=Some"
    );
}

#[tokio::test]
async fn cli_session_non_resumable_when_no_resume_support() {
    let db = setup_db();
    let store = make_store(&db);

    let provider = Box::new(MockCliProvider::new(false, None));
    let config = SessionConfig::default();

    let (mut session, _event_rx) = CliSession::new(provider, config);
    session.set_persistence(store.clone(), SessionPersistence::Persistent);

    session.submit("hello").await.expect("submit");

    let record = store
        .get_session(session.session_id())
        .expect("get_session")
        .expect("record");

    assert_eq!(
        record.resumability,
        Resumability::None,
        "CliSession should be NonResumable when supports_resume()=false"
    );
}

#[tokio::test]
async fn cli_session_non_resumable_when_no_session_id() {
    let db = setup_db();
    let store = make_store(&db);

    // supports_resume = true but session_id = None
    let provider = Box::new(MockCliProvider::new(true, None));
    let config = SessionConfig::default();

    let (mut session, _event_rx) = CliSession::new(provider, config);
    session.set_persistence(store.clone(), SessionPersistence::Persistent);

    session.submit("hello").await.expect("submit");

    let record = store
        .get_session(session.session_id())
        .expect("get_session")
        .expect("record");

    assert_eq!(
        record.resumability,
        Resumability::None,
        "CliSession should be NonResumable when session_id() is None, even if supports_resume()"
    );
}

// ===========================================================================
// AC-7: Checkpoint fires at: creation, after submit, on AwaitingInput, close
// ===========================================================================

#[tokio::test]
async fn api_session_checkpoint_on_creation() {
    let db = setup_db();
    let store = make_store(&db);

    let client: Arc<dyn LlmClient> = Arc::new(MockClient::with_text_response("Hello!"));
    let env = Arc::new(MockExecEnv);
    let config = SessionConfig::default();
    let profile = Box::new(TestProfile::new());

    let (mut session, _event_rx) = ApiSession::new(
        profile,
        env,
        client,
        config,
        "You are a test assistant.".into(),
        0,
        ApiSessionInit::default(),
    );

    // Wire persistence — checkpoint should fire at set_persistence time
    // (which is the "creation" checkpoint for sessions that add persistence)
    session.set_persistence(store.clone(), SessionPersistence::Persistent);

    // DB should contain the session record even before any submit
    let record = store
        .get_session(session.session_id())
        .expect("get_session")
        .expect("session should be checkpointed on creation");

    assert_eq!(record.total_turns, 0, "no submits yet");
    assert_eq!(record.state, SessionState::Idle);
}

#[tokio::test]
async fn api_session_checkpoint_on_awaiting_input() {
    let db = setup_db();
    let store = make_store(&db);

    let client: Arc<dyn LlmClient> = Arc::new(MockClient::with_text_response("Done."));
    let env = Arc::new(MockExecEnv);
    let config = SessionConfig::default();
    let profile = Box::new(TestProfile::new());

    let (mut session, _event_rx) = ApiSession::new(
        profile,
        env,
        client,
        config,
        "You are a test assistant.".into(),
        0,
        ApiSessionInit::default(),
    );

    session.set_persistence(store.clone(), SessionPersistence::Persistent);

    // Submit so the session completes and returns to Idle
    session.submit("Do something").await.expect("submit");
    assert_eq!(
        session.state(),
        SessionState::Idle,
        "session should be Idle after successful submit"
    );

    // Deterministically transition to AwaitingInput using the public API
    session
        .set_awaiting_input()
        .expect("set_awaiting_input should succeed from Idle");
    assert_eq!(
        session.state(),
        SessionState::AwaitingInput,
        "session should be AwaitingInput after set_awaiting_input()"
    );

    // The checkpoint implementation must fire on AwaitingInput transition,
    // so the DB record should reflect the new state.
    let record = store
        .get_session(session.session_id())
        .expect("get_session")
        .expect("record should exist");
    assert_eq!(
        record.state,
        SessionState::AwaitingInput,
        "checkpoint should reflect AwaitingInput state in the DB"
    );
}

#[tokio::test]
async fn api_session_checkpoint_on_close() {
    let db = setup_db();
    let store = make_store(&db);

    let client: Arc<dyn LlmClient> = Arc::new(MockClient::with_text_response("Done!"));
    let env = Arc::new(MockExecEnv);
    let config = SessionConfig::default();
    let profile = Box::new(TestProfile::new());

    let (mut session, _event_rx) = ApiSession::new(
        profile,
        env,
        client,
        config,
        "You are a test assistant.".into(),
        0,
        ApiSessionInit::default(),
    );

    session.set_persistence(store.clone(), SessionPersistence::Persistent);
    session.submit("Do something").await.expect("submit");

    let session_id = session.session_id().to_string();

    // Explicitly close
    session.close();

    let record = store
        .get_session(&session_id)
        .expect("get_session")
        .expect("record should exist after close");

    assert_eq!(
        record.state,
        SessionState::Closed,
        "checkpoint on close should record Closed state"
    );
}

#[tokio::test]
async fn cli_session_checkpoint_on_close() {
    let db = setup_db();
    let store = make_store(&db);

    let provider = Box::new(MockCliProvider::new(true, Some("test-close".into())));
    let config = SessionConfig::default();

    let (mut session, _event_rx) = CliSession::new(provider, config);
    session.set_persistence(store.clone(), SessionPersistence::Persistent);

    session.submit("do stuff").await.expect("submit");
    let session_id = session.session_id().to_string();

    session.close();

    let record = store
        .get_session(&session_id)
        .expect("get_session")
        .expect("record exists after close");

    assert_eq!(
        record.state,
        SessionState::Closed,
        "CLI session checkpoint on close should record Closed state"
    );
}
