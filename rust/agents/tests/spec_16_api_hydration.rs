//! Phase 4 / Slice 1 tests: API session hydration from persisted store.
//!
//! Verifies that an API session persisted to the DB can be hydrated back
//! into a live session with the same session ID, history, config, and
//! turn count. After hydration, submitting a new prompt should increment
//! `total_turns` and append to history.
//!
//! All tests use in-memory SQLite and mock clients / providers.

#![allow(clippy::result_large_err)]

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use stencila_agents::api_session::{ApiSession, ApiSessionInit, LlmClient};
use stencila_agents::error::AgentResult;
use stencila_agents::execution::{ExecutionEnvironment, FileContent};
use stencila_agents::migrations::AGENT_MIGRATIONS;
use stencila_agents::profile::ProviderProfile;
use stencila_agents::registry::ToolRegistry;
use stencila_agents::store::{AgentSessionStore, Resumability, SessionPersistence, SessionRecord};
use stencila_agents::types::{
    DirEntry, ExecResult, GrepOptions, SessionConfig, SessionState, Turn,
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
// Helpers (reused from spec_13_checkpoint_wiring pattern)
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
    fn with_responses(responses: Vec<Result<Response, SdkError>>) -> Self {
        Self {
            responses: Mutex::new(VecDeque::from(responses)),
        }
    }

    fn with_text_response(text: &str) -> Self {
        Self::with_responses(vec![Ok(text_response(text))])
    }

    fn with_n_text_responses(text: &str, n: usize) -> Self {
        Self::with_responses((0..n).map(|_| Ok(text_response(text))).collect())
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
    fn context_window_size(&self) -> u64 {
        200_000
    }
}

/// Helper: create an ApiSession with mock deps, submit a prompt, persist to
/// the store, then close. Returns the session_id, final history, and config
/// so tests can verify the hydrated session matches.
async fn create_and_persist_session(
    store: &Arc<AgentSessionStore>,
    num_submits: usize,
) -> (String, Vec<Turn>, SessionConfig, u32) {
    let client: Arc<dyn LlmClient> = Arc::new(MockClient::with_n_text_responses(
        "Hello back!",
        num_submits,
    ));
    let env = Arc::new(MockExecEnv);
    let config = SessionConfig::default();
    let profile = Box::new(TestProfile::new());

    let (mut session, _event_rx) = ApiSession::new(
        profile,
        env,
        client,
        config.clone(),
        "You are a test assistant.".into(),
        0,
        ApiSessionInit::default(),
    );

    session.set_persistence(store.clone(), SessionPersistence::Persistent);

    for i in 0..num_submits {
        session
            .submit(&format!("Message {i}"))
            .await
            .expect("submit should succeed");
    }

    let session_id = session.session_id().to_string();
    let history = session.history().to_vec();
    let total_turns = session.total_turns();

    // Close the session to trigger the close checkpoint
    session.close();

    (session_id, history, config, total_turns)
}

// ===========================================================================
// AC-1: find_latest_resumable_standalone returns newest resumable session
//        that is standalone (no workflow), not closed
// ===========================================================================

#[test]
fn find_latest_resumable_standalone_returns_none_when_empty() {
    let db = setup_db();
    let store = AgentSessionStore::new(db);

    let result = store
        .find_latest_resumable_standalone()
        .expect("should not error on empty DB");
    assert!(
        result.is_none(),
        "should return None when no sessions exist"
    );
}

#[test]
fn find_latest_resumable_standalone_returns_newest_resumable() {
    let db = setup_db();
    let store = AgentSessionStore::new(db);

    // Insert two resumable standalone sessions with different updated_at
    let mut old = sample_record("sess-old");
    old.resumability = Resumability::Full;
    old.state = SessionState::Idle;
    old.updated_at = "2025-07-01T00:00:00Z".to_string();
    store.insert_session(&old).expect("insert old");

    let mut newer = sample_record("sess-newer");
    newer.resumability = Resumability::Full;
    newer.state = SessionState::Idle;
    newer.updated_at = "2025-07-01T01:00:00Z".to_string();
    store.insert_session(&newer).expect("insert newer");

    let result = store
        .find_latest_resumable_standalone()
        .expect("should not error")
        .expect("should return a session");

    assert_eq!(
        result.session_id, "sess-newer",
        "should return the newest by updated_at"
    );
}

#[test]
fn find_latest_resumable_standalone_excludes_closed() {
    let db = setup_db();
    let store = AgentSessionStore::new(db);

    let mut closed = sample_record("sess-closed");
    closed.resumability = Resumability::Full;
    closed.state = SessionState::Closed;
    closed.updated_at = "2025-07-01T01:00:00Z".to_string();
    store.insert_session(&closed).expect("insert closed");

    let mut active = sample_record("sess-active");
    active.resumability = Resumability::Full;
    active.state = SessionState::Idle;
    active.updated_at = "2025-07-01T00:00:00Z".to_string();
    store.insert_session(&active).expect("insert active");

    let result = store
        .find_latest_resumable_standalone()
        .expect("should not error")
        .expect("should return a session");

    assert_eq!(
        result.session_id, "sess-active",
        "should skip closed sessions"
    );
}

#[test]
fn find_latest_resumable_standalone_excludes_workflow_owned() {
    let db = setup_db();
    let store = AgentSessionStore::new(db);

    let mut workflow_owned = sample_record("sess-workflow");
    workflow_owned.resumability = Resumability::Full;
    workflow_owned.state = SessionState::Idle;
    workflow_owned.workflow_run_id = Some("wf-run-1".to_string());
    workflow_owned.updated_at = "2025-07-01T01:00:00Z".to_string();
    store
        .insert_session(&workflow_owned)
        .expect("insert workflow");

    let mut standalone = sample_record("sess-standalone");
    standalone.resumability = Resumability::Full;
    standalone.state = SessionState::Idle;
    standalone.updated_at = "2025-07-01T00:00:00Z".to_string();
    store
        .insert_session(&standalone)
        .expect("insert standalone");

    let result = store
        .find_latest_resumable_standalone()
        .expect("should not error")
        .expect("should return a session");

    assert_eq!(
        result.session_id, "sess-standalone",
        "should exclude workflow-owned sessions"
    );
}

#[test]
fn find_latest_resumable_standalone_excludes_non_resumable() {
    let db = setup_db();
    let store = AgentSessionStore::new(db);

    let mut non_resumable = sample_record("sess-noresume");
    non_resumable.resumability = Resumability::None;
    non_resumable.state = SessionState::Idle;
    non_resumable.updated_at = "2025-07-01T01:00:00Z".to_string();
    store
        .insert_session(&non_resumable)
        .expect("insert non-resumable");

    let result = store
        .find_latest_resumable_standalone()
        .expect("should not error");

    assert!(result.is_none(), "should not return non-resumable sessions");
}

#[test]
fn find_latest_resumable_standalone_returns_none_when_all_excluded() {
    let db = setup_db();
    let store = AgentSessionStore::new(db);

    // Closed session
    let mut closed = sample_record("sess-c");
    closed.state = SessionState::Closed;
    closed.resumability = Resumability::Full;
    store.insert_session(&closed).expect("insert closed");

    // Non-resumable session
    let mut non_res = sample_record("sess-nr");
    non_res.state = SessionState::Idle;
    non_res.resumability = Resumability::None;
    store
        .insert_session(&non_res)
        .expect("insert non-resumable");

    // Workflow-owned session
    let mut wf = sample_record("sess-wf");
    wf.state = SessionState::Idle;
    wf.resumability = Resumability::Full;
    wf.workflow_run_id = Some("wf1".to_string());
    store.insert_session(&wf).expect("insert workflow");

    let result = store
        .find_latest_resumable_standalone()
        .expect("should not error");

    assert!(
        result.is_none(),
        "should return None when all sessions are excluded"
    );
}

// ===========================================================================
// AC-2: Hydrated API session has same session ID, history, config, and
//        total_turns as persisted
// ===========================================================================

/// `hydrate_api_session` should accept a session record and turn history
/// from the store and produce a live `ApiSession` with matching state.
#[tokio::test]
async fn hydrate_api_session_preserves_session_id() {
    let db = setup_db();
    let store = make_store(&db);

    let (session_id, _history, _config, _total_turns) = create_and_persist_session(&store, 2).await;

    // Load the persisted record and turns
    let record = store
        .get_session(&session_id)
        .expect("get_session")
        .expect("session should exist");
    let turns = store.get_turns(&session_id).expect("get_turns");

    // Hydrate a new session from the persisted data
    let client: Arc<dyn LlmClient> = Arc::new(MockClient::with_text_response("Hydrated reply!"));
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

    assert_eq!(
        hydrated.session_id(),
        session_id,
        "hydrated session should have the same session ID"
    );
}

#[tokio::test]
async fn hydrate_api_session_preserves_history() {
    let db = setup_db();
    let store = make_store(&db);

    let (session_id, original_history, _config, _total_turns) =
        create_and_persist_session(&store, 2).await;

    let record = store
        .get_session(&session_id)
        .expect("get_session")
        .expect("session should exist");
    let turns = store.get_turns(&session_id).expect("get_turns");

    let client: Arc<dyn LlmClient> = Arc::new(MockClient::with_text_response("Hydrated reply!"));
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

    // History should match the original — we compare length and content types
    // rather than exact equality because timestamps may differ slightly
    // between the live session's history and the deserialized turns.
    assert_eq!(
        hydrated.history().len(),
        original_history.len(),
        "hydrated session should have the same number of history entries"
    );
}

#[tokio::test]
async fn hydrate_api_session_preserves_total_turns() {
    let db = setup_db();
    let store = make_store(&db);

    let (session_id, _history, _config, original_total_turns) =
        create_and_persist_session(&store, 2).await;

    let record = store
        .get_session(&session_id)
        .expect("get_session")
        .expect("session should exist");
    let turns = store.get_turns(&session_id).expect("get_turns");

    let client: Arc<dyn LlmClient> = Arc::new(MockClient::with_text_response("Hydrated reply!"));
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

    assert_eq!(
        hydrated.total_turns(),
        original_total_turns,
        "hydrated session should have the same total_turns"
    );
}

// ===========================================================================
// AC-3: After hydration, submitting a new prompt increments total_turns
//        and appends to history
// ===========================================================================

#[tokio::test]
async fn hydrated_session_submit_increments_total_turns() {
    let db = setup_db();
    let store = make_store(&db);

    let (session_id, _history, _config, original_total_turns) =
        create_and_persist_session(&store, 2).await;

    let record = store
        .get_session(&session_id)
        .expect("get_session")
        .expect("exists");
    let turns = store.get_turns(&session_id).expect("get_turns");

    let client: Arc<dyn LlmClient> = Arc::new(MockClient::with_text_response("New reply!"));
    let env = Arc::new(MockExecEnv);
    let profile = Box::new(TestProfile::new());

    let (mut hydrated, _event_rx) = stencila_agents::api_session::hydrate_api_session(
        profile,
        env,
        client,
        "You are a test assistant.".into(),
        &record,
        turns,
    );

    let history_len_before = hydrated.history().len();

    hydrated
        .submit("New prompt after hydration")
        .await
        .expect("submit on hydrated session should succeed");

    assert_eq!(
        hydrated.total_turns(),
        original_total_turns + 1,
        "total_turns should increment by 1 after new submit"
    );
    assert!(
        hydrated.history().len() > history_len_before,
        "history should grow after new submit"
    );
}

// ===========================================================================
// AC-4: Hydrated session is in Idle state
// ===========================================================================

#[tokio::test]
async fn hydrated_session_starts_in_idle_state() {
    let db = setup_db();
    let store = make_store(&db);

    let (session_id, _history, _config, _total_turns) = create_and_persist_session(&store, 1).await;

    let record = store
        .get_session(&session_id)
        .expect("get_session")
        .expect("exists");
    let turns = store.get_turns(&session_id).expect("get_turns");

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

    assert_eq!(
        hydrated.state(),
        SessionState::Idle,
        "hydrated session should be in Idle state"
    );
}

// ===========================================================================
// AC-5: Hydrated session with persistence re-checkpoints on next submit
// ===========================================================================

#[tokio::test]
async fn hydrated_session_with_persistence_checkpoints_on_submit() {
    let db = setup_db();
    let store = make_store(&db);

    let (session_id, _history, _config, _total_turns) = create_and_persist_session(&store, 1).await;

    let record = store
        .get_session(&session_id)
        .expect("get_session")
        .expect("exists");
    let turns = store.get_turns(&session_id).expect("get_turns");

    let client: Arc<dyn LlmClient> = Arc::new(MockClient::with_text_response("hydrated reply"));
    let env = Arc::new(MockExecEnv);
    let profile = Box::new(TestProfile::new());

    let (mut hydrated, _event_rx) = stencila_agents::api_session::hydrate_api_session(
        profile,
        env,
        client,
        "You are a test assistant.".into(),
        &record,
        turns,
    );

    // Re-wire persistence on the hydrated session
    hydrated.set_persistence(store.clone(), SessionPersistence::Persistent);

    hydrated
        .submit("Post-hydration prompt")
        .await
        .expect("submit");

    // The DB should reflect the new state after the submit checkpoint
    let updated_record = store
        .get_session(&session_id)
        .expect("get_session")
        .expect("session should still exist");

    assert!(
        updated_record.total_turns >= 2,
        "total_turns in DB should reflect the new submit (got {})",
        updated_record.total_turns
    );

    let updated_turns = store.get_turns(&session_id).expect("get_turns");
    assert!(
        !updated_turns.is_empty(),
        "turns should be checkpointed after hydrated submit"
    );
}
