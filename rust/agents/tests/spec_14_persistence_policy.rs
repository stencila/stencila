//! Phase 3 / Slice 3 tests: BestEffort vs Required persistence policy.
//!
//! Verifies that when a DB write failure occurs during checkpoint:
//! - `BestEffort` policy logs a warning but does not fail the session
//!   (submit returns Ok, session stays operational).
//! - `Required` policy propagates the checkpoint error and the session
//!   operation fails (submit returns Err).
//!
//! Uses a "broken" SQLite connection (opened, then the underlying table
//! is dropped) to simulate DB write failures.

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
use stencila_agents::store::{AgentSessionStore, SessionPersistence};
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
// Helpers (shared with spec_13)
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

/// Create a store backed by a "broken" connection.
///
/// After migrations are applied the `agent_sessions` table is dropped,
/// so any subsequent `INSERT` or `UPDATE` on `agent_sessions` will fail
/// with a SQLite error.
fn make_broken_store() -> Arc<AgentSessionStore> {
    let conn = stencila_db::rusqlite::Connection::open_in_memory().expect("open in-memory SQLite");
    conn.execute_batch("PRAGMA foreign_keys = ON;")
        .expect("enable FK");
    for m in AGENT_MIGRATIONS {
        conn.execute_batch(m.sql).expect("apply migration");
    }
    // Drop the sessions table so writes fail
    conn.execute_batch(
        "DROP TABLE IF EXISTS agent_session_turns; DROP TABLE IF EXISTS agent_sessions;",
    )
    .expect("drop tables");
    let conn = Arc::new(Mutex::new(conn));
    Arc::new(AgentSessionStore::new(conn))
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

// -- Mock LLM Client --------------------------------------------------------

struct MockClient {
    responses: Mutex<VecDeque<Result<Response, SdkError>>>,
}

impl MockClient {
    fn with_text_response(text: &str) -> Self {
        let mut q = VecDeque::new();
        // Provide multiple responses so repeated submits work
        q.push_back(Ok(text_response(text)));
        q.push_back(Ok(text_response(text)));
        q.push_back(Ok(text_response(text)));
        Self {
            responses: Mutex::new(q),
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
// AC-1: BestEffort policy — DB write failure does NOT fail the session
// ===========================================================================

/// With `SessionPersistence::BestEffort`, a DB write failure during
/// `set_persistence` (creation checkpoint) should be silently tolerated
/// — the session should remain operational.
#[test]
fn api_session_best_effort_set_persistence_tolerates_db_failure() {
    let store = make_broken_store();

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

    // BestEffort: creation checkpoint fails but session stays alive
    session.set_persistence(store, SessionPersistence::BestEffort);

    assert_eq!(
        session.state(),
        SessionState::Idle,
        "session should remain Idle after BestEffort creation checkpoint failure"
    );
}

/// With `SessionPersistence::BestEffort`, a DB write failure during
/// `submit` (post-submit checkpoint) should not cause submit to return Err.
#[tokio::test]
async fn api_session_best_effort_submit_succeeds_despite_db_failure() {
    let store = make_broken_store();

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

    session.set_persistence(store, SessionPersistence::BestEffort);

    // Submit should succeed even though the DB is broken
    let result = session.submit("Hello!").await;
    assert!(
        result.is_ok(),
        "submit should succeed with BestEffort when DB is broken, got: {result:?}"
    );

    assert_eq!(
        session.state(),
        SessionState::Idle,
        "session should be Idle after successful submit with BestEffort"
    );
}

/// With `SessionPersistence::BestEffort` on a CLI session, submit should
/// succeed despite a broken DB.
#[tokio::test]
async fn cli_session_best_effort_submit_succeeds_despite_db_failure() {
    let store = make_broken_store();

    let provider = Box::new(MockCliProvider::new(true, Some("test-123".into())));
    let config = SessionConfig::default();

    let (mut session, _event_rx) = CliSession::new(provider, config);
    session.set_persistence(store, SessionPersistence::BestEffort);

    let result = session.submit("Hello CLI!").await;
    assert!(
        result.is_ok(),
        "CLI submit should succeed with BestEffort when DB is broken, got: {result:?}"
    );
}

/// With `SessionPersistence::BestEffort` on an API session, close should
/// not panic even when the DB is broken.
#[tokio::test]
async fn api_session_best_effort_close_tolerates_db_failure() {
    let store = make_broken_store();

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

    session.set_persistence(store, SessionPersistence::BestEffort);

    // Close should not panic
    session.close();
    assert_eq!(session.state(), SessionState::Closed);
}

// ===========================================================================
// AC-2: Required policy — DB write failure FAILS the session operation
// ===========================================================================

/// With `SessionPersistence::Required`, a DB write failure during
/// `set_persistence` (creation checkpoint) should propagate the error.
#[test]
fn api_session_required_set_persistence_propagates_db_failure() {
    let store = make_broken_store();

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

    // Required: creation checkpoint fails, and the error should be surfaced.
    // set_persistence should return a Result when using Required policy.
    let result = session.set_persistence_checked(store, SessionPersistence::Required);
    assert!(
        result.is_err(),
        "set_persistence_checked with Required should return Err when DB is broken"
    );
}

/// With `SessionPersistence::Required`, a DB write failure during submit
/// should cause submit to return Err.
#[tokio::test]
async fn api_session_required_submit_fails_on_db_write_error() {
    // Use a working DB initially so set_persistence creation checkpoint succeeds,
    // then break it before submit.
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

    session.set_persistence(store, SessionPersistence::Required);

    // Now break the DB by dropping tables
    {
        let conn = db.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        conn.execute_batch(
            "DROP TABLE IF EXISTS agent_session_turns; DROP TABLE IF EXISTS agent_sessions;",
        )
        .expect("drop tables");
    }

    // Submit should fail because the Required checkpoint cannot persist
    let result = session.submit("Hello!").await;
    assert!(
        result.is_err(),
        "submit should fail with Required when DB is broken"
    );
}

/// With `SessionPersistence::Required` on a CLI session, submit should
/// fail when the DB write fails.
#[tokio::test]
async fn cli_session_required_submit_fails_on_db_write_error() {
    let db = setup_db();
    let store = make_store(&db);

    let provider = Box::new(MockCliProvider::new(true, Some("test-123".into())));
    let config = SessionConfig::default();

    let (mut session, _event_rx) = CliSession::new(provider, config);
    session.set_persistence(store, SessionPersistence::Required);

    // Break the DB
    {
        let conn = db.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        conn.execute_batch(
            "DROP TABLE IF EXISTS agent_session_turns; DROP TABLE IF EXISTS agent_sessions;",
        )
        .expect("drop tables");
    }

    let result = session.submit("Hello CLI!").await;
    assert!(
        result.is_err(),
        "CLI submit should fail with Required when DB is broken"
    );
}

// ===========================================================================
// AC-1 (continued): BestEffort with a working DB still persists normally
// ===========================================================================

/// With `SessionPersistence::BestEffort` and a working DB, checkpoints
/// should still be written successfully (BestEffort is best-effort, not
/// no-effort).
#[tokio::test]
async fn api_session_best_effort_persists_when_db_is_healthy() {
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

    session.set_persistence(store.clone(), SessionPersistence::BestEffort);
    session
        .submit("Hello!")
        .await
        .expect("submit should succeed");

    // Even with BestEffort, the session should be persisted when the DB is healthy
    let record = store
        .get_session(session.session_id())
        .expect("get_session should not error")
        .expect("session record should exist with BestEffort when DB is healthy");

    assert_eq!(
        record.total_turns, 1,
        "total_turns should be 1 after one submit"
    );
}
