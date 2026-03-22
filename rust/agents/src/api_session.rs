//! Agent session and core agentic loop (spec 2.1, 2.5-2.8, 2.10, Appendix B).
//!
//! The [`Session`] struct manages the conversation lifecycle: accepting user
//! input, building LLM requests, executing tool calls, emitting events, and
//! handling errors. It uses [`Client::stream()`] for incremental token
//! delivery (spec 2.9), falling back to [`Client::complete()`] via the
//! default [`LlmClient::stream_complete()`] implementation when the client
//! does not support streaming.
//!
//! # Testing
//!
//! The [`LlmClient`] trait abstracts the LLM call for testability. Tests
//! inject a mock that returns predetermined responses. The default
//! [`stream_complete()`](LlmClient::stream_complete) implementation
//! delegates to [`complete()`](LlmClient::complete), so mocks only need
//! to implement `complete`.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures::StreamExt;
use serde_json::Value;
use stencila_interviews::interviewer::Interviewer;
use stencila_models3::api::accumulator::StreamAccumulator;
use stencila_models3::error::SdkError;
use stencila_models3::types::content::ContentPart;
use stencila_models3::types::message::Message;
use stencila_models3::types::request::Request;
use stencila_models3::types::response::Response;
use stencila_models3::types::role::Role;
use stencila_models3::types::stream_event::{StreamEvent, StreamEventType};
use stencila_models3::types::tool::{ToolCall, ToolChoice, ToolResult};

use crate::error::{AgentError, AgentResult};
use crate::events::{self, EventEmitter, EventReceiver};
use crate::execution::ExecutionEnvironment;
use crate::loop_detection;
use crate::profile::ProviderProfile;
use crate::prompts::McpContext;
use crate::registry::ToolOutput;
use crate::subagents::{SubAgentManager, TOOL_WAIT};
use crate::tool_guard::{GuardContext, GuardVerdict, ToolGuard};
use crate::tools::ask_user::register_ask_user_tool;
use crate::truncation::{TruncationConfig, truncate_tool_output};
use crate::types::{
    AbortKind, AbortSignal, EventKind, HistoryThinkingReplay, SessionConfig, SessionState, Turn,
    now_timestamp,
};

// ---------------------------------------------------------------------------
// LlmClient trait
// ---------------------------------------------------------------------------

/// Abstraction over the LLM client for testability.
///
/// Production code uses [`Models3Client`] which wraps the real
/// [`stencila_models3::client::Client`]. Tests inject a mock.
#[async_trait]
pub trait LlmClient: Send + Sync {
    /// Send a completion request to the LLM.
    async fn complete(&self, request: Request) -> Result<Response, SdkError>;

    /// Stream a completion request, calling `on_event` for each stream event.
    ///
    /// Returns the accumulated [`Response`] after the stream completes.
    /// The default implementation falls back to [`complete()`](Self::complete)
    /// and synthesizes a single text-delta event for the full response text,
    /// so mock clients only need to implement `complete`.
    ///
    /// Events are passed by value (owned) rather than by reference to avoid
    /// lifetime issues with the `async_trait` macro and HRTB closures.
    async fn stream_complete(
        &self,
        request: Request,
        on_event: &(dyn Fn(StreamEvent) + Send + Sync),
    ) -> Result<Response, SdkError> {
        let response = self.complete(request).await?;
        let text = response.text();
        if !text.is_empty() {
            on_event(StreamEvent::text_delta(&text));
        }
        Ok(response)
    }
}

/// Real implementation wrapping the models3 [`Client`](stencila_models3::client::Client).
pub struct Models3Client {
    client: stencila_models3::client::Client,
}

impl Models3Client {
    /// Wrap an existing models3 client.
    pub fn new(client: stencila_models3::client::Client) -> Self {
        Self { client }
    }
}

#[async_trait]
impl LlmClient for Models3Client {
    async fn complete(&self, request: Request) -> Result<Response, SdkError> {
        self.client.complete(request).await
    }

    async fn stream_complete(
        &self,
        request: Request,
        on_event: &(dyn Fn(StreamEvent) + Send + Sync),
    ) -> Result<Response, SdkError> {
        // Try streaming first; fall back to complete() only if the provider
        // signals that streaming is unsupported (Configuration / InvalidRequest).
        // Auth, rate-limit, network, and other errors must propagate.
        match self.client.stream(request.clone()).await {
            Ok(mut stream) => {
                let mut accumulator = StreamAccumulator::new();
                while let Some(result) = stream.next().await {
                    let event = result?;
                    accumulator.process(&event);
                    on_event(event);
                }
                Ok(accumulator.response())
            }
            Err(
                SdkError::Configuration { .. }
                | SdkError::InvalidRequest { .. }
                | SdkError::NotFound { .. },
            ) => {
                // Runtime safety net: streaming unsupported by provider/model.
                let response = self.client.complete(request).await?;
                let text = response.text();
                if !text.is_empty() {
                    on_event(StreamEvent::text_delta(&text));
                }
                Ok(response)
            }
            Err(e) => Err(e),
        }
    }
}

// ---------------------------------------------------------------------------
// ImageAttachment
// ---------------------------------------------------------------------------

/// Image data attached to a tool result for multimodal providers.
struct ImageAttachment {
    data: Vec<u8>,
    media_type: String,
}

/// Metrics collected during a single context compaction pass.
#[derive(Debug, Clone, Copy)]
struct CompactionStats {
    before_tokens: u64,
    after_tokens: u64,
    stripped_reasoning_turns: usize,
    stripped_thinking_parts: usize,
    summarized_tool_results: usize,
    removed_tool_result_chars: usize,
    removed_turns: usize,
}

// ---------------------------------------------------------------------------
// API Session
// ---------------------------------------------------------------------------

/// An agent session managing the conversation loop (spec 2.1).
///
/// Created via [`ApiSession::new()`], driven by [`submit()`](Self::submit).
/// Events are delivered through the [`EventReceiver`] returned by the
/// constructor.
pub struct ApiSession {
    config: SessionConfig,
    state: SessionState,
    history: Vec<Turn>,
    profile: Box<dyn ProviderProfile>,
    execution_env: Arc<dyn ExecutionEnvironment>,
    client: Arc<dyn LlmClient>,
    events: EventEmitter,
    steering_queue: VecDeque<String>,
    followup_queue: VecDeque<String>,
    system_prompt: String,
    abort_signal: Option<AbortSignal>,
    /// Count of LLM request/response cycles across the entire session.
    ///
    /// Incremented once per assistant response (not per history entry).
    /// The spec's `count_turns(session)` (line 234) is ambiguous — it could
    /// mean total history entries or LLM cycles. We use LLM cycles because
    /// it provides more useful limiting behavior: a single tool-using turn
    /// counts as 1 turn regardless of how many tool results are generated.
    total_turns: u32,
    truncation_config: TruncationConfig,
    /// Bounded sliding window of tool-call signatures for loop detection.
    tool_call_signatures: VecDeque<String>,
    /// Manager for child agent sessions (spec 7.1).
    subagent_manager: SubAgentManager,
    /// Image attachments from tool results, keyed by tool_call_id.
    /// Provider-generated tool_call_ids are UUIDs, so collisions are
    /// not a practical concern.
    image_attachments: HashMap<String, ImageAttachment>,

    /// Session store for checkpoint persistence.
    persistence_store: Option<Arc<crate::store::AgentSessionStore>>,
    /// Session persistence mode.
    persistence_mode: Option<crate::store::SessionPersistence>,

    /// Tool guard policy for this session. Shared via `Arc` with child sessions.
    tool_guard: Option<Arc<ToolGuard>>,
    /// Per-session guard context for audit attribution.
    guard_context: Option<Arc<GuardContext>>,

    /// MCP connection pool for server lifecycle and subagent sharing.
    #[cfg(any(feature = "mcp", feature = "codemode"))]
    mcp_pool: Option<Arc<stencila_mcp::ConnectionPool>>,

    /// Whether this session owns the MCP pool and should shut it down on close.
    /// Only the top-level session that created the pool owns it; child sessions
    /// share the pool via `Arc` but must not call `start_shutdown()`.
    #[cfg(any(feature = "mcp", feature = "codemode"))]
    owns_mcp_pool: bool,
}

impl std::fmt::Debug for ApiSession {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Session")
            .field("state", &self.state)
            .field("history_len", &self.history.len())
            .field("total_turns", &self.total_turns)
            .finish_non_exhaustive()
    }
}

/// Optional parameters for [`ApiSession::new`] that default to `None`.
///
/// Grouping these into a struct avoids long parameter lists and `None`
/// repetition at callsites that do not use MCP, tool guards, or explicit
/// session IDs.
#[derive(Default)]
pub struct ApiSessionInit {
    pub mcp_context: Option<McpContext>,
    pub session_id: Option<String>,
    pub tool_guard: Option<Arc<ToolGuard>>,
    pub guard_context: Option<Arc<GuardContext>>,
    pub interviewer: Option<Arc<dyn Interviewer>>,
}

impl ApiSession {
    /// Create a new session.
    ///
    /// Returns the session and an [`EventReceiver`] for consuming events.
    /// The caller is responsible for building the **complete** system prompt
    /// via [`build_system_prompt()`] (which handles all layers including
    /// commit instructions and user instructions) and passing the resulting
    /// [`McpContext`] so the session can manage pool lifecycle and
    /// propagate MCP/codemode capabilities to subagents.
    ///
    /// The `current_depth` parameter controls subagent nesting: 0 for a
    /// top-level session, incremented by 1 for each child. Top-level
    /// sessions (depth 0) own the MCP pool and shut it down on close;
    /// child sessions share the pool without owning it.
    ///
    /// The optional fields in [`ApiSessionInit`] control MCP context,
    /// session ID, and tool guard configuration. When `session_id` is
    /// `Some`, that value is used as the session identifier for both
    /// events and guard context attribution. When `None`, a random UUID
    /// v4 is generated.
    ///
    /// Emits a `SESSION_START` event immediately.
    pub fn new(
        mut profile: Box<dyn ProviderProfile>,
        execution_env: Arc<dyn ExecutionEnvironment>,
        client: Arc<dyn LlmClient>,
        config: SessionConfig,
        system_prompt: String,
        current_depth: u32,
        init: ApiSessionInit,
    ) -> (Self, EventReceiver) {
        let ApiSessionInit {
            mcp_context,
            session_id,
            tool_guard,
            guard_context,
            interviewer,
        } = init;
        let (emitter, receiver) = match session_id {
            Some(id) => events::channel_with_id(id),
            None => events::channel(),
        };
        emitter.emit_session_start();

        let truncation_config = TruncationConfig {
            preset: config.truncation_preset,
            tool_output_limits: config.tool_output_limits.clone(),
            tool_line_limits: config.tool_line_limits.clone(),
        };

        let max_depth = config.max_subagent_depth;

        // Auto-register subagent tools when this session is allowed to spawn
        // subagents (depth < max_depth). Errors are logged but non-fatal.
        if current_depth < max_depth
            && let Err(e) = profile.register_subagent_tools()
        {
            tracing::warn!("failed to register subagent tools: {e}");
        }

        // Register ask_user tool when an interviewer is provided,
        // enabling explicit human-in-the-loop via tool calls.
        if let Some(iv) = interviewer
            && let Err(e) = register_ask_user_tool(profile.tool_registry_mut(), iv)
        {
            tracing::warn!("failed to register ask_user tool: {e}");
        }

        #[allow(unused_mut)]
        let mut subagent_manager = SubAgentManager::new(
            Arc::clone(&execution_env),
            Arc::clone(&client),
            current_depth,
            config.clone(),
        );

        // Propagate tool guard to subagent manager for child session sharing.
        // Use the guard context's agent name when available, falling back to
        // "unknown" so enforcement is never silently skipped in subagents.
        if let Some(guard) = &tool_guard {
            let parent_name = guard_context
                .as_ref()
                .map(|c| Arc::clone(&c.agent_name))
                .unwrap_or_else(|| Arc::from("unknown"));
            subagent_manager.set_tool_guard(Arc::clone(guard), parent_name);
        }

        // Apply MCP context: store pool for lifecycle management and
        // propagate to subagent manager for child session sharing.
        #[cfg(any(feature = "mcp", feature = "codemode"))]
        let (mcp_pool, owns_mcp_pool) = if let Some(ctx) = mcp_context {
            subagent_manager.set_mcp_pool(std::sync::Arc::clone(&ctx.pool));

            #[cfg(feature = "codemode")]
            if let Some(ref tracker) = ctx.dirty_tracker {
                subagent_manager.set_dirty_tracker(std::sync::Arc::clone(tracker));
            }

            let owns = current_depth == 0;
            (Some(ctx.pool), owns)
        } else {
            (None, false)
        };

        // Suppress unused-variable when no MCP features are active.
        #[cfg(not(any(feature = "mcp", feature = "codemode")))]
        let _ = mcp_context;

        let session = Self {
            config,
            state: SessionState::Idle,
            history: Vec::new(),
            profile,
            execution_env,
            client,
            events: emitter,
            steering_queue: VecDeque::new(),
            followup_queue: VecDeque::new(),
            system_prompt,
            abort_signal: None,
            total_turns: 0,
            truncation_config,
            tool_call_signatures: VecDeque::new(),
            subagent_manager,
            image_attachments: HashMap::new(),
            persistence_store: None,
            persistence_mode: None,
            tool_guard,
            guard_context,
            #[cfg(any(feature = "mcp", feature = "codemode"))]
            mcp_pool,
            #[cfg(any(feature = "mcp", feature = "codemode"))]
            owns_mcp_pool,
        };

        (session, receiver)
    }

    // -- Public API --

    /// Submit user input and run the agentic loop until natural completion,
    /// a limit is hit, or an error occurs.
    ///
    /// # State transitions
    ///
    /// - IDLE → PROCESSING → IDLE (natural completion or turn limit)
    /// - IDLE → PROCESSING → AWAITING_INPUT (model asked a question, auto-detected)
    /// - AWAITING_INPUT → PROCESSING → IDLE (user answered a question)
    /// - IDLE → PROCESSING → CLOSED (unrecoverable error or abort)
    ///
    /// # Errors
    ///
    /// Returns `Err(SessionClosed)` if the session is already closed.
    /// Returns `Err(Sdk(..))` for unrecoverable LLM errors.
    pub async fn submit(&mut self, input: &str) -> AgentResult<()> {
        if self.state == SessionState::Closed {
            return Err(AgentError::SessionClosed);
        }

        self.state = SessionState::Processing;
        self.process_input(input).await
    }

    /// Manually transition the session to AwaitingInput state (spec 2.3).
    ///
    /// By default, the session auto-detects questions and transitions to
    /// `AwaitingInput` automatically (see
    /// [`SessionConfig::auto_detect_awaiting_input`]). This method is a
    /// manual override for hosts that disable auto-detection or need to
    /// force the transition based on their own heuristics.
    ///
    /// # Errors
    ///
    /// Returns `Err(InvalidState)` if the session is not in the Idle state.
    pub fn set_awaiting_input(&mut self) -> AgentResult<()> {
        if self.state != SessionState::Idle {
            return Err(AgentError::InvalidState {
                expected: "Idle".into(),
                actual: format!("{:?}", self.state),
            });
        }
        self.state = SessionState::AwaitingInput;
        let cp = self.checkpoint();
        self.handle_checkpoint_result(cp)
    }

    /// Queue a steering message to be injected after the current tool round.
    ///
    /// If the session is idle, the message is delivered on the next
    /// [`submit()`](Self::submit) call.
    pub fn steer(&mut self, message: impl Into<String>) {
        self.steering_queue.push_back(message.into());
    }

    /// Queue a follow-up message to be processed after the current input
    /// fully completes (natural completion or limit).
    pub fn follow_up(&mut self, message: impl Into<String>) {
        self.followup_queue.push_back(message.into());
    }

    /// Close the session. Emits `SESSION_END` and transitions to CLOSED.
    // TODO(spec-ambiguity): The spec pseudocode (line 301) emits SESSION_END
    // on every loop completion (transitioning to IDLE), but the event
    // definition (line 413) says "session closed". We only emit on close/
    // error/abort to avoid noisy events on every IDLE transition. (spec: 2.9)
    pub fn close(&mut self) {
        if self.state != SessionState::Closed {
            // Clean up active subagents before closing (spec graceful shutdown, line 1431)
            self.subagent_manager.close_all();

            // Shut down MCP connection pool (only if this session owns it)
            #[cfg(any(feature = "mcp", feature = "codemode"))]
            if self.owns_mcp_pool
                && let Some(ref pool) = self.mcp_pool
            {
                pool.start_shutdown();
            }

            self.state = SessionState::Closed;
            if let Err(e) = self.checkpoint() {
                tracing::warn!("close checkpoint failed (swallowed): {e}");
            }
            self.events.emit_session_end(self.state);
        }
    }

    /// Set an abort signal for cancellation.
    pub fn set_abort_signal(&mut self, signal: AbortSignal) {
        self.abort_signal = Some(signal);
    }

    /// Wire checkpoint persistence into this session.
    ///
    /// Immediately inserts a session record into the store (creation checkpoint).
    /// Errors during the creation checkpoint are logged but swallowed regardless
    /// of the persistence policy. Use [`set_persistence_checked`] when the caller
    /// needs to observe creation-checkpoint failures.
    pub fn set_persistence(
        &mut self,
        store: Arc<crate::store::AgentSessionStore>,
        persistence: crate::store::SessionPersistence,
    ) {
        self.persistence_store = Some(store);
        self.persistence_mode = Some(persistence);
        if let Err(e) = self.checkpoint() {
            tracing::warn!("creation checkpoint failed (swallowed): {e}");
        }
    }

    /// Like [`set_persistence`], but returns the creation-checkpoint result.
    ///
    /// With [`SessionPersistence::Required`] the caller can surface the error;
    /// with [`SessionPersistence::BestEffort`] or [`SessionPersistence::Persistent`]
    /// the caller may choose to ignore it.
    pub fn set_persistence_checked(
        &mut self,
        store: Arc<crate::store::AgentSessionStore>,
        persistence: crate::store::SessionPersistence,
    ) -> AgentResult<()> {
        self.persistence_store = Some(store);
        self.persistence_mode = Some(persistence);
        self.checkpoint()
    }

    /// Write the current session state to the persistence store.
    ///
    /// Returns `Ok(())` when no store is configured or the mode is
    /// [`Ephemeral`](crate::store::SessionPersistence::Ephemeral).
    fn checkpoint(&self) -> AgentResult<()> {
        let Some(ref store) = self.persistence_store else {
            return Ok(());
        };
        if matches!(
            self.persistence_mode,
            None | Some(crate::store::SessionPersistence::Ephemeral)
        ) {
            return Ok(());
        }

        let now = crate::types::now_timestamp();
        let record = crate::store::SessionRecord {
            session_id: self.session_id().to_string(),
            backend_kind: "api".to_string(),
            agent_name: String::new(),
            provider_name: self.profile.id().to_string(),
            model_name: self.profile.model().to_string(),
            state: self.state,
            total_turns: i64::from(self.total_turns),
            resumability: crate::store::Resumability::Full,
            created_at: now.clone(),
            updated_at: now,
            workflow_run_id: None,
            workflow_thread_id: None,
            workflow_node_id: None,
            provider_resume_state: None,
            config_snapshot: None,
            system_prompt: None,
            lease_holder: None,
            lease_expires_at: None,
        };

        store
            .upsert_checkpoint(&record, &self.history)
            .map_err(|e| AgentError::Io {
                message: format!("checkpoint write failed: {e}"),
            })
    }

    /// Handle the result of a checkpoint call according to the persistence policy.
    ///
    /// - [`BestEffort`](crate::store::SessionPersistence::BestEffort): log a warning and return `Ok(())`
    /// - [`Required`](crate::store::SessionPersistence::Required): propagate the error
    /// - [`Persistent`](crate::store::SessionPersistence::Persistent): swallow (legacy behaviour)
    fn handle_checkpoint_result(&self, result: AgentResult<()>) -> AgentResult<()> {
        match result {
            Ok(()) => Ok(()),
            Err(e) => match self.persistence_mode {
                Some(crate::store::SessionPersistence::Required) => Err(e),
                Some(crate::store::SessionPersistence::BestEffort) => {
                    tracing::warn!("checkpoint failed (best-effort): {e}");
                    Ok(())
                }
                _ => {
                    // Persistent / Ephemeral / None — swallow (legacy)
                    Ok(())
                }
            },
        }
    }

    // -- Getters --

    /// Current session state.
    #[must_use]
    pub fn state(&self) -> SessionState {
        self.state
    }

    /// Full conversation history.
    #[must_use]
    pub fn history(&self) -> &[Turn] {
        &self.history
    }

    /// Session configuration (read-only).
    #[must_use]
    pub fn config(&self) -> &SessionConfig {
        &self.config
    }

    /// Session configuration (mutable, e.g. to change reasoning_effort).
    pub fn config_mut(&mut self) -> &mut SessionConfig {
        &mut self.config
    }

    /// The session's event emitter.
    #[must_use]
    pub fn events(&self) -> &crate::events::EventEmitter {
        &self.events
    }

    /// The session ID from the event emitter.
    #[must_use]
    pub fn session_id(&self) -> &str {
        self.events.session_id()
    }

    /// Total number of LLM request/response cycles in this session.
    #[must_use]
    pub fn total_turns(&self) -> u32 {
        self.total_turns
    }

    /// Register an additional tool on this session's profile.
    ///
    /// Can be called at any point. Tools registered before the first
    /// `submit()` are included from the start; tools registered later
    /// will appear in subsequent LLM requests.
    ///
    /// # Errors
    ///
    /// Returns an error if the tool definition fails validation.
    pub fn register_tool(
        &mut self,
        tool: crate::registry::RegisteredTool,
    ) -> crate::error::AgentResult<()> {
        let tool_name = tool.definition().name.clone();
        self.profile.tool_registry_mut().register(tool)?;

        // Runtime-registered tools (e.g. workflow routing/context tools) must
        // also be reflected in `allowed_tools`. Otherwise request building and
        // tool-call prechecks can filter out or reject a tool that was just
        // registered on the session, making instructions like
        // `workflow_set_route` impossible to follow for allowlisted agents.
        if let Some(allowed_tools) = &mut self.config.allowed_tools
            && !allowed_tools.iter().any(|allowed| allowed == &tool_name)
        {
            allowed_tools.push(tool_name);
        }

        Ok(())
    }

    /// The tool guard for this session, if set.
    #[must_use]
    pub fn tool_guard(&self) -> Option<&Arc<ToolGuard>> {
        self.tool_guard.as_ref()
    }

    /// The guard context for this session, if set.
    #[must_use]
    pub fn guard_context(&self) -> Option<&Arc<GuardContext>> {
        self.guard_context.as_ref()
    }

    pub fn subagent_has_tool_guard(&self) -> bool {
        self.subagent_manager.has_tool_guard()
    }

    /// The MCP connection pool, if MCP/codemode is active.
    #[cfg(any(feature = "mcp", feature = "codemode"))]
    #[must_use]
    pub fn mcp_pool(&self) -> Option<&Arc<stencila_mcp::ConnectionPool>> {
        self.mcp_pool.as_ref()
    }

    // -- Core loop (spec 2.5) --

    /// Run the agentic loop for a single user input.
    ///
    /// Uses a single post-loop exit path so that follow-up processing and
    /// context-usage checks are always reached regardless of exit reason
    /// (natural completion, turn limit, or abort).
    async fn process_input(&mut self, input: &str) -> AgentResult<()> {
        // 1. Record user turn
        self.history.push(Turn::user(input));
        self.events.emit_user_input(input);

        let mut round_count: u32 = 0;
        // Set to true only when the loop exits via natural completion
        // (text-only assistant response). Prevents question detection from
        // matching stale assistant turns when exiting via limits.
        let mut natural_completion = false;
        // Track whether we have already attempted context compaction for
        // the current LLM call. Prevents infinite retry loops — we compact
        // at most once per call.
        let mut compaction_attempted = false;

        // The loop runs until natural completion, a limit, or abort.
        // SDK errors propagate immediately (session → CLOSED).
        loop {
            // 2. Check abort
            match self.abort_kind() {
                AbortKind::Hard => {
                    self.close();
                    return Ok(());
                }
                AbortKind::Soft => break,
                AbortKind::Active => {}
            }

            // 2b. Check round limit
            if self.config.max_tool_rounds_per_input > 0
                && round_count >= self.config.max_tool_rounds_per_input
            {
                self.emit_turn_limit("max_tool_rounds_per_input", round_count);
                break;
            }

            // 2c. Check session turn limit
            if self.config.max_turns > 0 && self.total_turns >= self.config.max_turns {
                self.emit_turn_limit("max_turns", self.total_turns);
                break;
            }

            // 3. Drain steering queue (after limit checks so messages are not
            //    consumed without a subsequent LLM call to deliver them)
            self.drain_steering();

            // 4. Context-usage warning (spec 5.5)
            self.check_context_usage();

            // 4b. Proactive context compaction before hitting hard overflow.
            self.maybe_proactive_compaction();

            // 5. Build and send LLM request (streaming, abort-aware)
            let request = self.build_request();
            self.events.emit_assistant_text_start();

            // Stream the LLM response, emitting text deltas incrementally.
            // When the profile does not support streaming, fall back to
            // complete() with a single synthesized delta (avoids a wasted
            // stream() round-trip).  The block isolates borrows of
            // self.events / self.abort_signal so that self.close() /
            // self.handle_sdk_error() can take &mut self after the block
            // ends.
            //
            // `partial_text` accumulates streamed deltas so that abort and
            // error paths can emit TEXT_END with the text received so far
            // (instead of an empty string that contradicts earlier deltas).
            //
            // Retryable errors (network, timeout, server, rate-limit) are
            // retried with exponential backoff. Once any content has been
            // delivered to the user (text deltas *or* reasoning events),
            // retries are skipped because the partial output cannot be
            // un-delivered.
            let retry_policy = self.config.retry_policy.clone();
            let partial_text = std::sync::Mutex::new(String::new());
            let content_emitted = std::sync::atomic::AtomicBool::new(false);
            let stream_result: Option<Result<Response, SdkError>> = {
                let events_ref = &self.events;
                let partial_ref = &partial_text;
                let emitted_ref = &content_emitted;
                let on_event = |event: StreamEvent| match event.event_type {
                    StreamEventType::TextDelta => {
                        if let Some(ref delta) = event.delta {
                            emitted_ref.store(true, std::sync::atomic::Ordering::Relaxed);
                            if let Ok(mut buf) = partial_ref.lock() {
                                buf.push_str(delta);
                            }
                            events_ref.emit_assistant_text_delta(delta);
                        }
                    }
                    StreamEventType::ReasoningStart => {
                        emitted_ref.store(true, std::sync::atomic::Ordering::Relaxed);
                        events_ref.emit_assistant_reasoning_start();
                    }
                    StreamEventType::ReasoningDelta => {
                        if let Some(ref delta) = event.reasoning_delta {
                            events_ref.emit_assistant_reasoning_delta(delta);
                        }
                    }
                    StreamEventType::ReasoningEnd => {
                        events_ref.emit_assistant_reasoning_end();
                    }
                    _ => {}
                };

                let client = Arc::clone(&self.client);
                let use_streaming = self.profile.supports_streaming();

                let call_with_retry = async {
                    let mut attempt: u32 = 0;
                    loop {
                        let result = if use_streaming {
                            client.stream_complete(request.clone(), &on_event).await
                        } else {
                            match client.complete(request.clone()).await {
                                Ok(response) => {
                                    let text = response.text();
                                    if !text.is_empty() {
                                        on_event(StreamEvent::text_delta(&text));
                                    }
                                    Ok(response)
                                }
                                Err(e) => Err(e),
                            }
                        };

                        match result {
                            Ok(response) => return Ok(response),
                            Err(error) => {
                                // Don't retry if any content (text or
                                // reasoning) has been delivered to the user.
                                let delivered =
                                    emitted_ref.load(std::sync::atomic::Ordering::Relaxed);

                                if delivered && error.is_retryable() {
                                    let label = retry_error_label(&error);
                                    tracing::debug!(
                                        error = %error,
                                        "retryable LLM error after partial content delivery, cannot retry"
                                    );
                                    events_ref.emit_info(
                                        "LLM_NO_RETRY",
                                        format!(
                                            "{label}: cannot retry because partial content was already delivered",
                                        ),
                                    );
                                    return Err(error);
                                }

                                if !delivered
                                    && let Some(delay) = retry_policy.resolve_delay(&error, attempt)
                                {
                                    let label = retry_error_label(&error);
                                    tracing::debug!(
                                        attempt,
                                        delay_secs = delay,
                                        error = %error,
                                        "retryable LLM error, backing off"
                                    );
                                    events_ref.emit_info(
                                        "LLM_RETRY",
                                        format!(
                                            "{label}, retrying ({}/{})...",
                                            attempt + 1,
                                            retry_policy.max_retries
                                        ),
                                    );
                                    tokio::time::sleep(Duration::from_secs_f64(delay)).await;
                                    attempt += 1;
                                    continue;
                                }

                                if !delivered && error.is_retryable() {
                                    let label = retry_error_label(&error);
                                    if attempt >= retry_policy.max_retries {
                                        tracing::debug!(
                                            attempt,
                                            error = %error,
                                            "retryable LLM error, retries exhausted"
                                        );
                                        events_ref.emit_info(
                                            "LLM_RETRY",
                                            format!(
                                                "{label}: all {} retries failed",
                                                retry_policy.max_retries
                                            ),
                                        );
                                    } else {
                                        // Retry-After exceeds max_delay — provider
                                        // asked us to wait longer than we're willing to.
                                        tracing::debug!(
                                            attempt,
                                            error = %error,
                                            "retryable LLM error, Retry-After too large"
                                        );
                                        events_ref.emit_info(
                                            "LLM_RETRY",
                                            format!(
                                                "{label}: server requested a retry delay that exceeds the maximum",
                                            ),
                                        );
                                    }
                                }

                                return Err(error);
                            }
                        }
                    }
                };

                if let Some(ref signal) = self.abort_signal {
                    tokio::select! {
                        result = call_with_retry => Some(result),
                        () = signal.cancelled() => None,
                    }
                } else {
                    Some(call_with_retry.await)
                }
            };

            let response = match stream_result {
                None => {
                    // Abort during streaming: emit TEXT_END with partial text.
                    // Partial text is NOT recorded in history.
                    let text = partial_text.into_inner().unwrap_or_default();
                    self.events.emit_assistant_text_end(&text, None);
                    match self.abort_kind() {
                        AbortKind::Hard => {
                            self.close();
                            return Ok(());
                        }
                        _ => break,
                    }
                }
                Some(Ok(r)) => r,
                Some(Err(e)) => {
                    // Error: emit TEXT_END with any partial text received.
                    let text = partial_text.into_inner().unwrap_or_default();
                    self.events.emit_assistant_text_end(&text, None);

                    // On context-length errors, attempt automatic compaction
                    // once before giving up. This drops thinking blocks and
                    // summarises older tool results to free context space.
                    if matches!(e, SdkError::ContextLength { .. })
                        && !compaction_attempted
                        && self.compact_history("reactive").is_some()
                    {
                        compaction_attempted = true;
                        let mut data = serde_json::Map::new();
                        data.insert("severity".into(), Value::String("warning".into()));
                        data.insert(
                            "message".into(),
                            Value::String(
                                "Context length exceeded — compacted history and retrying".into(),
                            ),
                        );
                        self.events.emit(EventKind::Error, data);
                        continue;
                    }

                    return self.handle_sdk_error(e);
                }
            };

            // Successful LLM call — reset the compaction flag so that
            // later rounds (which may accumulate new content) can compact
            // again if needed.
            compaction_attempted = false;

            // 6. Record assistant turn
            let text = response.text();
            let tool_calls = response.tool_calls();
            let reasoning = response.reasoning();
            let thinking_parts: Vec<ContentPart> = response
                .message
                .content
                .iter()
                .filter(|p| {
                    matches!(
                        p,
                        ContentPart::Thinking { .. } | ContentPart::RedactedThinking { .. }
                    )
                })
                .cloned()
                .collect();
            let usage = response.usage.clone();
            let response_id = response.id.clone();

            self.events
                .emit_assistant_text_end(&text, reasoning.clone());

            let response_content_parts = response.message.content.clone();
            self.history.push(Turn::Assistant {
                content: text,
                tool_calls: tool_calls.clone(),
                reasoning,
                thinking_parts,
                response_content_parts,
                usage,
                response_id: Some(response_id),
                timestamp: now_timestamp(),
            });
            self.total_turns += 1;

            // 7. Natural completion: no tool calls
            if tool_calls.is_empty() {
                natural_completion = true;
                break;
            }

            // 8. Execute tool calls (abort-aware per spec Graceful Shutdown)
            let results = match self.execute_tool_calls(&tool_calls).await {
                Some(r) => r,
                None => {
                    // Abort fired during tool execution (tokio::select!
                    // cancelled the work future). Fabricate [Aborted]
                    // results for all tool calls so history stays
                    // well-formed — the assistant turn with tool_calls
                    // was already recorded above.
                    match self.abort_kind() {
                        AbortKind::Hard => {
                            self.close();
                            return Ok(());
                        }
                        _ => {
                            let aborted_results = tool_calls
                                .iter()
                                .map(|tc| aborted_tool_result(&tc.id))
                                .collect();
                            self.history.push(Turn::tool_results(aborted_results));
                            break;
                        }
                    }
                }
            };
            self.history.push(Turn::tool_results(results));

            round_count += 1;

            // 9. Loop detection (spec 2.10)
            if self.config.enable_loop_detection {
                self.check_loop_detection();
            }
        }

        // -- Single post-loop exit path --
        // Reached on natural completion, turn/round limits, AND soft abort.

        let soft_aborted = self.abort_kind() == AbortKind::Soft;

        // TODO(spec-ambiguity): The spec says follow-ups trigger "after the
        // current input is fully handled (model has produced a text-only
        // response)" (spec: 2.8, line 371), implying natural completion only.
        // However, the pseudocode (line 296) places the check after the loop
        // break, which is also reached on limits. We process follow-ups on
        // both paths because it is more useful — callers that queue follow-ups
        // expect them to run. (spec: 2.8)
        //
        // Soft abort skips follow-ups: the user cancelled this exchange,
        // so queued follow-ups should wait for the next explicit submit.

        // Process follow-up queue.
        // Recursion depth is bounded by the number of queued follow-ups, which
        // is controlled by the caller (typically 0-2 items). Each level
        // consumes one entry from the queue, so the depth cannot grow.
        if !soft_aborted && let Some(followup) = self.followup_queue.pop_front() {
            return Box::pin(self.process_input(&followup)).await;
        }

        // Auto-detect AwaitingInput (spec 2.3): only on natural completion
        // (text-only response). Limit-triggered exits and soft aborts must
        // not inspect stale assistant turns from earlier in the history.
        self.state = if soft_aborted {
            // Reset the signal so the next submit() starts clean.
            if let Some(ref signal) = self.abort_signal {
                signal.reset_soft();
            }
            SessionState::Idle
        } else if natural_completion
            && self.config.auto_detect_awaiting_input
            && self.looks_like_question()
        {
            SessionState::AwaitingInput
        } else {
            SessionState::Idle
        };
        let cp = self.checkpoint();
        self.handle_checkpoint_result(cp)
    }

    /// Check whether the last assistant turn looks like a question to the user.
    ///
    /// Returns `true` when the most recent assistant turn has no tool calls and
    /// the last line of the text matches a question-like pattern: a trailing
    /// `?`, a solicitation phrase ("let me know", "would you like", etc.), or
    /// an interrogative word prefix **combined with** a trailing `?`.
    ///
    /// Interrogative words alone (e.g. "What follows is...") are not enough
    /// because they frequently begin declarative sentences.
    fn looks_like_question(&self) -> bool {
        let last = self
            .history
            .iter()
            .rev()
            .find(|t| matches!(t, Turn::Assistant { .. }));

        match last {
            Some(Turn::Assistant {
                content,
                tool_calls,
                ..
            }) => {
                // Only consider text-only responses (natural completion).
                if !tool_calls.is_empty() {
                    return false;
                }

                let trimmed = content.trim();
                if trimmed.is_empty() {
                    return false;
                }

                // Check for trailing question mark — always sufficient.
                if trimmed.ends_with('?') {
                    return true;
                }

                // Check the last line for solicitation phrases that reliably
                // indicate the model is waiting for user input, even without
                // a trailing `?`.
                let last_line = trimmed.lines().next_back().unwrap_or("").trim();
                let lower = last_line.to_lowercase();

                // Solicitation phrases: almost always request user action.
                let solicitation_prefixes = [
                    "would you",
                    "shall i",
                    "do you",
                    "should i",
                    "let me know",
                    "please confirm",
                    "please let me know",
                ];

                solicitation_prefixes
                    .iter()
                    .any(|prefix| lower.starts_with(prefix))
            }
            _ => false,
        }
    }

    // -- Request building --

    /// Build the LLM request from current state.
    fn build_request(&self) -> Request {
        let mut messages = vec![Message::system(&self.system_prompt)];
        messages.extend(self.convert_history_to_messages());

        let mut request = Request::new(self.profile.model(), messages);
        let tools = if let Some(ref allowed) = self.config.allowed_tools {
            let filtered: Vec<_> = self
                .profile
                .tools()
                .into_iter()
                .filter(|t| allowed.iter().any(|a| a == &t.name))
                .collect();
            if filtered.is_empty() {
                // No tools match — omit tools and tool_choice entirely so
                // providers don't reject the request.
                None
            } else {
                Some(filtered)
            }
        } else {
            Some(self.profile.tools())
        };
        if tools.is_some() {
            request.tool_choice = Some(ToolChoice::Auto);
        }
        request.tools = tools;
        request.reasoning_effort = self
            .config
            .reasoning_effort
            .as_ref()
            .map(|r| r.as_str().to_string());
        request.provider = Some(self.profile.id().to_string());

        if let Some(opts) = self.profile.provider_options() {
            request.provider_options = Some(opts);
        }

        request
    }

    /// Convert the conversation history into models3 Messages.
    fn convert_history_to_messages(&self) -> Vec<Message> {
        let mut messages = Vec::new();

        for turn in &self.history {
            match turn {
                Turn::User { content, .. } => {
                    messages.push(Message::user(content.as_str()));
                }
                Turn::Steering { content, .. } => {
                    // Steering messages are user-role per spec 2.6
                    messages.push(Message::user(content.as_str()));
                }
                Turn::System { content, .. } => {
                    messages.push(Message::system(content.as_str()));
                }
                Turn::Assistant {
                    content,
                    tool_calls,
                    thinking_parts,
                    response_content_parts,
                    ..
                } => {
                    if !response_content_parts.is_empty() {
                        // Use original response parts where possible, with
                        // configurable filtering of thinking/reasoning replay.
                        let parts = self.filter_history_parts(response_content_parts);

                        if parts.is_empty() && !content.is_empty() {
                            messages.push(Message::assistant(content.as_str()));
                        } else if !parts.is_empty() {
                            messages.push(Message::new(Role::Assistant, parts));
                        }
                    } else if thinking_parts.is_empty() && tool_calls.is_empty() {
                        // When both filtered parts and content are empty (e.g.
                        // an assistant turn that was purely thinking), skip the
                        // message entirely to avoid pushing an empty turn that
                        // some providers would reject.
                        if !content.is_empty() {
                            messages.push(Message::assistant(content.as_str()));
                        }
                    } else {
                        let replay = matches!(
                            self.config.history_thinking_replay,
                            HistoryThinkingReplay::Full
                        );
                        let mut parts = Vec::new();
                        // Thinking blocks must precede text/tool_call content
                        // (required by Anthropic for extended thinking).
                        if replay {
                            parts.extend(thinking_parts.iter().cloned());
                        }
                        if !content.is_empty() {
                            parts.push(ContentPart::text(content.as_str()));
                        }
                        for tc in tool_calls {
                            parts.push(ContentPart::tool_call(
                                &tc.id,
                                &tc.name,
                                tc.arguments.clone(),
                            ));
                        }
                        messages.push(Message::new(Role::Assistant, parts));
                    }
                }
                Turn::ToolResults { results, .. } => {
                    let include_images = self.profile.id() == "anthropic";
                    for result in results {
                        if include_images
                            && let Some(att) = self.image_attachments.get(&result.tool_call_id)
                        {
                            let mut msg = Message::new(
                                Role::Tool,
                                vec![
                                    ContentPart::tool_result(
                                        &result.tool_call_id,
                                        result.content.clone(),
                                        result.is_error,
                                    ),
                                    ContentPart::image_data(att.data.clone(), &att.media_type),
                                ],
                            );
                            msg.tool_call_id = Some(result.tool_call_id.clone());
                            messages.push(msg);
                            continue;
                        }
                        messages.push(Message::tool_result(
                            &result.tool_call_id,
                            result.content.clone(),
                            result.is_error,
                        ));
                    }
                }
            }
        }

        messages
    }

    /// Filter content parts for history replay based on `history_thinking_replay`.
    fn filter_history_parts(&self, parts: &[ContentPart]) -> Vec<ContentPart> {
        match self.config.history_thinking_replay {
            HistoryThinkingReplay::Full => parts.to_vec(),
            HistoryThinkingReplay::None => parts
                .iter()
                .filter(|part| {
                    !matches!(
                        part,
                        ContentPart::Thinking { .. } | ContentPart::RedactedThinking { .. }
                    )
                })
                .cloned()
                .collect(),
        }
    }

    // -- Steering --

    /// Drain the steering queue, appending turns and emitting events.
    fn drain_steering(&mut self) {
        while let Some(msg) = self.steering_queue.pop_front() {
            self.events.emit_steering_injected(&msg);
            self.history.push(Turn::steering(&msg));
        }
    }

    // -- Tool execution --

    /// Execute all tool calls from an assistant response.
    ///
    /// Returns `None` if the abort signal fires during execution (spec
    /// Graceful Shutdown Sequence). Parallel execution when the profile
    /// supports it, sequential otherwise.
    ///
    /// Subagent tool calls (`spawn_agent`, `send_input`, `wait`,
    /// `close_agent`) are intercepted and routed to the [`SubAgentManager`]
    /// rather than through the regular tool executor path (spec 7.1).
    async fn execute_tool_calls(&mut self, tool_calls: &[ToolCall]) -> Option<Vec<ToolResult>> {
        let abort = self.abort_signal.clone();

        // Pre-check: reject tool calls not in the allowed_tools list.
        // When allowed_tools is None, all tools are permitted.
        let allowed = self.config.allowed_tools.clone();

        let work = async {
            // When allowed_tools is None, all calls are permitted — take the
            // original fast path with no splitting overhead.
            if allowed.is_none() {
                let has_subagent = tool_calls
                    .iter()
                    .any(|tc| SubAgentManager::is_subagent_tool(&tc.name));

                return if has_subagent {
                    self.execute_tools_with_subagents(tool_calls).await
                } else if self.profile.supports_parallel_tool_calls() && tool_calls.len() > 1 {
                    self.execute_tools_parallel(tool_calls).await
                } else {
                    self.execute_tools_sequential(tool_calls).await
                };
            }

            let allow_list = allowed.as_ref().expect("checked above");

            // Build a results vec with slots for every tool call, pre-filled
            // with None. Rejected calls are filled immediately; permitted calls
            // are collected with their original index for batch execution.
            let mut results: Vec<Option<ToolResult>> = vec![None; tool_calls.len()];
            let mut permitted: Vec<(usize, &ToolCall)> = Vec::new();

            for (idx, tc) in tool_calls.iter().enumerate() {
                if allow_list.iter().any(|a| a == &tc.name) {
                    permitted.push((idx, tc));
                } else {
                    let error_msg = format!(
                        "Tool '{}' is not in this agent's allowedTools list. \
                         Available tools: {}",
                        tc.name,
                        allow_list.join(", ")
                    );
                    self.events
                        .emit_tool_call_start(&tc.name, &tc.id, &tc.arguments);
                    self.events.emit_tool_call_end_error(&tc.id, &error_msg);
                    results[idx] = Some(ToolResult {
                        tool_call_id: tc.id.clone(),
                        content: Value::String(error_msg),
                        is_error: true,
                    });
                }
            }

            // Execute permitted calls and place results at their original indices.
            if !permitted.is_empty() {
                let permitted_calls: Vec<ToolCall> =
                    permitted.iter().map(|(_, tc)| (*tc).clone()).collect();
                let indices: Vec<usize> = permitted.iter().map(|(i, _)| *i).collect();

                let has_subagent = permitted_calls
                    .iter()
                    .any(|tc| SubAgentManager::is_subagent_tool(&tc.name));

                let executed = if has_subagent {
                    self.execute_tools_with_subagents(&permitted_calls).await
                } else if self.profile.supports_parallel_tool_calls() && permitted_calls.len() > 1 {
                    self.execute_tools_parallel(&permitted_calls).await
                } else {
                    self.execute_tools_sequential(&permitted_calls).await
                };

                for (result, &idx) in executed.into_iter().zip(&indices) {
                    results[idx] = Some(result);
                }
            }

            // Unwrap all slots — every position was filled by either the
            // rejection path or the execution path.
            results
                .into_iter()
                .enumerate()
                .map(|(i, slot)| slot.unwrap_or_else(|| aborted_tool_result(&tool_calls[i].id)))
                .collect()
        };

        match abort {
            Some(signal) => {
                tokio::select! {
                    results = work => Some(results),
                    () = signal.cancelled() => None,
                }
            }
            None => Some(work.await),
        }
    }

    async fn execute_tools_sequential(&mut self, tool_calls: &[ToolCall]) -> Vec<ToolResult> {
        let mut results = Vec::with_capacity(tool_calls.len());
        for tc in tool_calls {
            if self.is_aborted() {
                // Backfill remaining tool calls with [Aborted] so the
                // result count matches tool_calls.len() (1:1 mapping).
                for remaining in &tool_calls[results.len()..] {
                    results.push(aborted_tool_result(&remaining.id));
                }
                break;
            }
            let (result, attachment) = self.execute_single_tool(tc).await;
            self.store_attachment_if_supported(attachment);
            results.push(result);
        }
        results
    }

    async fn execute_tools_parallel(&mut self, tool_calls: &[ToolCall]) -> Vec<ToolResult> {
        // Collect references needed by all futures
        let env = &*self.execution_env;
        let registry = self.profile.tool_registry();
        let events = &self.events;
        let trunc_config = &self.truncation_config;
        let guard = self.tool_guard.as_ref();
        let guard_ctx = self.guard_context.as_ref();

        let futs: Vec<_> = tool_calls
            .iter()
            .map(|tc| execute_tool(tc, registry, env, events, trunc_config, guard, guard_ctx))
            .collect();

        let pairs = futures::future::join_all(futs).await;
        let mut results = Vec::with_capacity(pairs.len());
        for (result, attachment) in pairs {
            self.store_attachment_if_supported(attachment);
            results.push(result);
        }
        results
    }

    /// Execute tool calls when subagent tools are present.
    ///
    /// Most calls run sequentially, but contiguous runs of `wait` calls are
    /// executed concurrently so the TUI can show all spinners at once and
    /// mark each as done as it finishes.
    ///
    /// Abort is checked at the top of each iteration. During a concurrent
    /// `wait` batch, all in-flight waits will complete before abort is
    /// noticed — this is acceptable since child sessions have their own
    /// abort signals.
    async fn execute_tools_with_subagents(&mut self, tool_calls: &[ToolCall]) -> Vec<ToolResult> {
        let mut results = Vec::with_capacity(tool_calls.len());
        let mut i = 0;
        while i < tool_calls.len() {
            if self.is_aborted() {
                for remaining in &tool_calls[i..] {
                    results.push(aborted_tool_result(&remaining.id));
                }
                break;
            }

            // Detect a contiguous run of `wait` calls for concurrent execution.
            if tool_calls[i].name == TOOL_WAIT {
                let run_start = i;
                while i < tool_calls.len() && tool_calls[i].name == TOOL_WAIT {
                    i += 1;
                }
                let wait_calls = &tool_calls[run_start..i];

                // Emit ToolCallStart for all waits before awaiting any.
                for tc in wait_calls {
                    self.events
                        .emit_tool_call_start(&tc.name, &tc.id, &tc.arguments);
                }

                // Validate and extract agent IDs. Produce an error result
                // immediately for any call missing the required argument.
                let mut agent_ids: Vec<Option<String>> = Vec::with_capacity(wait_calls.len());
                for tc in wait_calls {
                    match tc.arguments.get("agent_id").and_then(Value::as_str) {
                        Some(id) => agent_ids.push(Some(id.to_string())),
                        None => {
                            agent_ids.push(None);
                        }
                    }
                }

                // Collect the valid IDs for concurrent execution.
                let valid_ids: Vec<String> = agent_ids.iter().filter_map(|id| id.clone()).collect();

                let mut wait_results = self
                    .subagent_manager
                    .wait_agents_concurrent(&valid_ids)
                    .await
                    .into_iter();

                for (tc, agent_id) in wait_calls.iter().zip(agent_ids) {
                    let wait_result = match agent_id {
                        Some(_) => wait_results.next().unwrap_or_else(|| {
                            Err(AgentError::Io {
                                message: "missing result from concurrent wait".into(),
                            })
                        }),
                        None => Err(AgentError::ValidationError {
                            reason: "missing required string parameter: agent_id".into(),
                        }),
                    };

                    match wait_result {
                        Ok(output) => {
                            self.events.emit_tool_call_end(&tc.id, &output);
                            let truncated =
                                truncate_tool_output(&output, &tc.name, &self.truncation_config);
                            results.push(ToolResult {
                                tool_call_id: tc.id.clone(),
                                content: Value::String(truncated),
                                is_error: false,
                            });
                        }
                        Err(e) => {
                            let error_msg = e.to_string();
                            self.events.emit_tool_call_end_error(&tc.id, &error_msg);
                            results.push(ToolResult {
                                tool_call_id: tc.id.clone(),
                                content: Value::String(error_msg),
                                is_error: true,
                            });
                        }
                    }
                }
            } else if SubAgentManager::is_subagent_tool(&tool_calls[i].name) {
                results.push(self.execute_subagent_tool(&tool_calls[i]).await);
                i += 1;
            } else {
                let (result, attachment) = self.execute_single_tool(&tool_calls[i]).await;
                self.store_attachment_if_supported(attachment);
                results.push(result);
                i += 1;
            }
        }
        results
    }

    /// Execute a subagent tool call via the SubAgentManager.
    async fn execute_subagent_tool(&mut self, tool_call: &ToolCall) -> ToolResult {
        self.events
            .emit_tool_call_start(&tool_call.name, &tool_call.id, &tool_call.arguments);

        let result = self
            .subagent_manager
            .execute(&tool_call.name, tool_call.arguments.clone(), &*self.profile)
            .await;

        match result {
            Ok(output) => {
                self.events.emit_tool_call_end(&tool_call.id, &output);
                let truncated =
                    truncate_tool_output(&output, &tool_call.name, &self.truncation_config);
                ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    content: Value::String(truncated),
                    is_error: false,
                }
            }
            Err(e) => {
                let error_msg = e.to_string();
                self.events
                    .emit_tool_call_end_error(&tool_call.id, &error_msg);
                ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    content: Value::String(error_msg),
                    is_error: true,
                }
            }
        }
    }

    /// Store an image attachment only when the provider supports images in
    /// tool results. Avoids accumulating dead weight for providers that
    /// receive only the text fallback (OpenAI, Gemini).
    fn store_attachment_if_supported(&mut self, attachment: Option<(String, ImageAttachment)>) {
        if let Some((id, img)) = attachment
            && self.profile.id() == "anthropic"
        {
            self.image_attachments.insert(id, img);
        }
    }

    /// Execute a single tool call: emit events, run executor, truncate output.
    ///
    /// Returns `(ToolResult, Option<(tool_call_id, ImageAttachment)>)`.
    async fn execute_single_tool(
        &self,
        tool_call: &ToolCall,
    ) -> (ToolResult, Option<(String, ImageAttachment)>) {
        execute_tool(
            tool_call,
            self.profile.tool_registry(),
            &*self.execution_env,
            &self.events,
            &self.truncation_config,
            self.tool_guard.as_ref(),
            self.guard_context.as_ref(),
        )
        .await
    }

    // -- Context usage (spec 5.5) --

    /// Emit context usage information and warn when approaching the limit.
    ///
    /// Uses heuristic: 1 token ~ 4 characters. Always emits a
    /// `ContextUsage` event with the current percentage. Additionally emits
    /// a warning `Error` event at 80% of the profile's
    /// `context_window_size`. Informational only — no automatic compaction.
    fn check_context_usage(&self) {
        let total_chars = self.estimate_history_chars();
        let approx_tokens = total_chars / 4;
        let context_size = self.profile.context_window_size();
        let pct = if context_size > 0 {
            (approx_tokens as f64 / context_size as f64 * 100.0) as u32
        } else {
            0
        };

        // Always emit context usage info
        let mut usage_data = serde_json::Map::new();
        usage_data.insert("percent".into(), Value::Number(pct.into()));
        usage_data.insert("approx_tokens".into(), Value::Number(approx_tokens.into()));
        usage_data.insert(
            "context_window_size".into(),
            Value::Number(context_size.into()),
        );
        self.events
            .emit(crate::types::EventKind::ContextUsage, usage_data);

        // Warn at 80%
        let threshold = (context_size as f64 * 0.8) as u64;
        if approx_tokens > threshold {
            let mut data = serde_json::Map::new();
            data.insert("severity".into(), Value::String("warning".into()));
            data.insert(
                "message".into(),
                Value::String(format!("Context usage at ~{pct}% of context window")),
            );
            data.insert("approx_tokens".into(), Value::Number(approx_tokens.into()));
            data.insert(
                "context_window_size".into(),
                Value::Number(context_size.into()),
            );
            self.events.emit(crate::types::EventKind::Error, data);
        }
    }

    /// Estimate total character count across history and system prompt.
    fn estimate_history_chars(&self) -> u64 {
        let mut chars: u64 = self.system_prompt.len() as u64;
        for turn in &self.history {
            match turn {
                Turn::User { content, .. }
                | Turn::Steering { content, .. }
                | Turn::System { content, .. } => {
                    chars += content.len() as u64;
                }
                Turn::Assistant {
                    content,
                    tool_calls,
                    reasoning,
                    thinking_parts,
                    response_content_parts,
                    ..
                } => {
                    if !response_content_parts.is_empty() {
                        // Estimate from the original parts (what we actually
                        // send) to avoid undercounting provider metadata such
                        // as Gemini thought_signature strings.
                        chars += estimate_content_parts_chars(response_content_parts);
                    } else {
                        // Legacy turns without response_content_parts: fall
                        // back to the decomposed fields.
                        chars += content.len() as u64;
                        for tc in tool_calls {
                            chars += tc.name.len() as u64;
                            chars += tc.arguments.to_string().len() as u64;
                        }
                        if let Some(r) = reasoning {
                            chars += r.len() as u64;
                        }
                        for part in thinking_parts {
                            if let ContentPart::Thinking { thinking }
                            | ContentPart::RedactedThinking { thinking } = part
                            {
                                chars += thinking.text.len() as u64;
                                if let Some(ref sig) = thinking.signature {
                                    chars += sig.len() as u64;
                                }
                            }
                        }
                    }
                }
                Turn::ToolResults { results, .. } => {
                    for r in results {
                        chars += r.content.to_string().len() as u64;
                    }
                }
            }
        }
        chars
    }

    /// Trigger context compaction when projected usage exceeds the configured threshold.
    fn maybe_proactive_compaction(&mut self) {
        let context_size = self.profile.context_window_size();
        if context_size == 0 {
            return;
        }

        let trigger_pct = self.config.compaction_trigger_percent.min(100) as u64;
        if trigger_pct == 0 {
            return;
        }

        let approx_tokens = self.estimate_history_chars() / 4;
        // Reserve response headroom so proactive compaction happens before
        // we are fully pinned against the context window.
        let reserve_tokens = (context_size / 10).clamp(1_024, 8_192);
        let projected_tokens = approx_tokens.saturating_add(reserve_tokens);
        let projected_pct = (projected_tokens.saturating_mul(100)) / context_size;

        if projected_pct < trigger_pct {
            return;
        }

        self.compact_history(&format!(
            "proactive (projected {projected_pct}% >= trigger {trigger_pct}%)"
        ));
    }

    /// Attempt to compact the conversation history to reduce context size.
    ///
    /// Returns compaction stats when history was modified, or `None` when
    /// there was nothing left to compact.
    fn compact_history(&mut self, trigger: &str) -> Option<CompactionStats> {
        let before = self.estimate_history_chars();
        let mut modified = false;
        let mut stripped_reasoning_turns = 0usize;
        let mut stripped_thinking_parts = 0usize;
        let mut summarized_tool_results = 0usize;
        let mut removed_tool_result_chars = 0usize;
        let mut removed_turns = 0usize;

        // Phase 1: strip thinking/reasoning from ALL assistant turns.
        for turn in self.history.iter_mut() {
            if let Turn::Assistant {
                reasoning,
                thinking_parts,
                response_content_parts,
                ..
            } = turn
            {
                if reasoning.is_some() {
                    *reasoning = None;
                    modified = true;
                    stripped_reasoning_turns += 1;
                }
                if !thinking_parts.is_empty() {
                    stripped_thinking_parts += thinking_parts.len();
                    thinking_parts.clear();
                    modified = true;
                }
                let before_len = response_content_parts.len();
                response_content_parts.retain(|p| {
                    !matches!(
                        p,
                        ContentPart::Thinking { .. } | ContentPart::RedactedThinking { .. }
                    )
                });
                if response_content_parts.len() != before_len {
                    modified = true;
                    stripped_thinking_parts += before_len - response_content_parts.len();
                }
            }
        }

        // Phase 2: summarise tool results in older turns and remove
        // associated image attachments.
        // Keep the last 4 history entries intact (roughly the last
        // assistant + tool_results pair).
        let len = self.history.len();
        let compact_older_than = self.config.compact_tool_results_older_than_turns as usize;
        let preserve_tail = compact_older_than.min(len);
        let compactable = len.saturating_sub(preserve_tail);
        for turn in self.history[..compactable].iter_mut() {
            if let Turn::ToolResults { results, .. } = turn {
                for r in results.iter_mut() {
                    // Always strip image attachments from older turns —
                    // image bytes can be megabytes while the textual
                    // content is short.
                    if self.image_attachments.remove(&r.tool_call_id).is_some() {
                        modified = true;
                    }
                    let s = r.content.to_string();
                    let max_chars = self.config.compact_max_tool_result_chars;
                    if s.len() > max_chars {
                        let removed = s.len().saturating_sub(max_chars);
                        removed_tool_result_chars += removed;
                        summarized_tool_results += 1;
                        r.content = Value::String(format!(
                            "[Output compacted — {} chars removed to free context space]",
                            removed
                        ));
                        modified = true;
                    }
                }
            }
        }

        // Phase 3: if the history is very long, drop middle exchanges.
        // Keep the first turn (user task) and the last turns, removing
        // everything in between if there are more than 10 turns total.
        //
        // The tail boundary is adjusted backwards to avoid starting with
        // an orphaned ToolResults turn (whose matching Assistant was
        // dropped). This keeps tool-call/tool-result pairs intact so
        // providers don't reject the request.
        let preserve_recent = (self.config.compact_preserve_recent_turns as usize).max(1);
        if self.history.len() > (preserve_recent + 6) {
            let keep_head = 1;
            let total = self.history.len();
            let mut tail_start = total.saturating_sub(preserve_recent);

            // Walk forward from the candidate boundary until we find a
            // turn that is safe to start with (anything other than
            // ToolResults, which would be orphaned).
            while tail_start < total {
                if matches!(self.history[tail_start], Turn::ToolResults { .. }) {
                    tail_start += 1;
                } else {
                    break;
                }
            }

            let keep_tail = total - tail_start;
            if keep_head + keep_tail < total {
                let removed = total - keep_head - keep_tail;
                removed_turns += removed;
                // Remove image attachments for tool results in dropped turns.
                for turn in &self.history[keep_head..tail_start] {
                    if let Turn::ToolResults { results, .. } = turn {
                        for r in results {
                            self.image_attachments.remove(&r.tool_call_id);
                        }
                    }
                }
                let summary = Turn::system(format!(
                    "[Context compacted: {removed} earlier turns were removed to fit within \
                     the model's context window. The original user request and recent \
                     conversation are preserved.]"
                ));
                let mut new_history = Vec::with_capacity(keep_head + 1 + keep_tail);
                new_history.extend(self.history.drain(..keep_head));
                new_history.push(summary);
                let remaining = self.history.len();
                new_history.extend(self.history.drain(remaining - keep_tail..));
                self.history = new_history;
                modified = true;
            }
        }

        let after = self.estimate_history_chars();
        let stats = CompactionStats {
            before_tokens: before / 4,
            after_tokens: after / 4,
            stripped_reasoning_turns,
            stripped_thinking_parts,
            summarized_tool_results,
            removed_tool_result_chars,
            removed_turns,
        };
        tracing::debug!(
            before_chars = before,
            after_chars = after,
            modified,
            trigger,
            "context compaction complete"
        );
        if modified {
            self.events.emit_info(
                "CONTEXT_COMPACTION",
                format!(
                    "trigger={trigger}, ~{} -> ~{} tokens, reasoning_turns={}, thinking_parts={}, summarized_results={}, removed_result_chars={}, removed_turns={}",
                    stats.before_tokens,
                    stats.after_tokens,
                    stats.stripped_reasoning_turns,
                    stats.stripped_thinking_parts,
                    stats.summarized_tool_results,
                    stats.removed_tool_result_chars,
                    stats.removed_turns
                ),
            );
            Some(stats)
        } else {
            None
        }
    }

    // -- Loop detection --

    /// Check for repeating tool-call patterns and inject steering if found.
    ///
    /// Uses a bounded sliding window (capped at `loop_detection_window`) to
    /// avoid unbounded growth.
    fn check_loop_detection(&mut self) {
        let window = self.config.loop_detection_window as usize;

        // Collect signatures from the latest assistant turn's tool calls
        if let Some(Turn::Assistant { tool_calls, .. }) = self
            .history
            .iter()
            .rev()
            .find(|t| matches!(t, Turn::Assistant { .. }))
        {
            for tc in tool_calls {
                self.tool_call_signatures
                    .push_back(loop_detection::tool_call_signature(tc));
                // Keep window bounded
                while self.tool_call_signatures.len() > window {
                    self.tool_call_signatures.pop_front();
                }
            }
        }

        let sigs: Vec<String> = self.tool_call_signatures.iter().cloned().collect();
        if let Some(message) = loop_detection::detect_loop(&sigs, window) {
            self.events.emit_loop_detection(&message);
            self.history.push(Turn::steering(&message));
        }
    }

    // -- Error handling --

    /// Handle an SDK error from an LLM call (spec Appendix B).
    ///
    /// Non-retryable errors (authentication, invalid request, etc.) close the
    /// session — these indicate a persistent problem that the user cannot fix
    /// by simply retrying.
    ///
    /// Retryable errors (network, timeout, server, rate-limit) have already
    /// been retried with exponential backoff by the session-level retry loop.
    /// If they still fail, the session transitions back to IDLE rather than
    /// CLOSED, allowing the user to try again without losing conversation
    /// history.
    ///
    /// Context-length errors are emitted with `"severity": "warning"` per
    /// spec (the host may implement compaction), while other errors use
    /// plain ERROR events.
    fn handle_sdk_error(&mut self, error: SdkError) -> AgentResult<()> {
        let is_retryable = error.is_retryable();
        let is_context_length = matches!(error, SdkError::ContextLength { .. });
        let agent_error = AgentError::from(error);

        if is_context_length {
            // Emit with severity:warning per spec — host can implement compaction
            let mut data = serde_json::Map::new();
            data.insert("severity".into(), Value::String("warning".into()));
            data.insert("code".into(), Value::String(agent_error.code().into()));
            data.insert("message".into(), Value::String(agent_error.to_string()));
            self.events.emit(crate::types::EventKind::Error, data);
        } else if !is_retryable {
            // Only emit ERROR for non-retryable errors. For retryable errors
            // the retry loop already emitted an LLM_RETRY info event that
            // explains the outcome, so a duplicate error line is unnecessary.
            self.events
                .emit_error(agent_error.code(), agent_error.to_string());
        }

        if is_retryable {
            // Retryable errors: keep the session open so the user can retry.
            // Transition back to IDLE rather than CLOSED.
            self.state = SessionState::Idle;
            Err(agent_error)
        } else {
            self.close();
            Err(agent_error)
        }
    }

    // -- Helpers --

    /// Read the current abort kind with a single atomic load.
    ///
    /// Returns [`AbortKind::Active`] when no signal is attached.
    fn abort_kind(&self) -> AbortKind {
        self.abort_signal
            .as_ref()
            .map_or(AbortKind::Active, AbortSignal::kind)
    }

    /// Check if any abort (soft or hard) has been triggered.
    fn is_aborted(&self) -> bool {
        self.abort_kind() != AbortKind::Active
    }

    /// Emit a TURN_LIMIT event with limit details.
    fn emit_turn_limit(&self, limit_type: &str, count: u32) {
        let mut data = serde_json::Map::new();
        data.insert("limit_type".into(), Value::String(limit_type.into()));
        data.insert("count".into(), Value::Number(count.into()));
        self.events.emit_turn_limit(data);
    }
}

impl Drop for ApiSession {
    fn drop(&mut self) {
        self.close();
    }
}

/// Estimate character count from a slice of [`ContentPart`]s.
///
/// Used by [`ApiSession::estimate_history_chars`] when `response_content_parts`
/// is available, so that provider metadata (e.g. Gemini `thought_signature`)
/// is included in the estimate.
fn estimate_content_parts_chars(parts: &[ContentPart]) -> u64 {
    let mut chars: u64 = 0;
    for part in parts {
        match part {
            ContentPart::Text { text } => chars += text.len() as u64,
            ContentPart::ToolCall { tool_call } => {
                chars += tool_call.name.len() as u64;
                chars += tool_call.arguments.to_string().len() as u64;
                if let Some(ref sig) = tool_call.thought_signature {
                    chars += sig.len() as u64;
                }
            }
            ContentPart::Thinking { thinking } | ContentPart::RedactedThinking { thinking } => {
                chars += thinking.text.len() as u64;
                if let Some(ref sig) = thinking.signature {
                    chars += sig.len() as u64;
                }
            }
            _ => {}
        }
    }
    chars
}

/// Human-readable label for a retryable error, used in the `LLM_RETRY` info
/// event so the user sees an accurate description of what went wrong.
fn retry_error_label(error: &SdkError) -> &'static str {
    match error {
        SdkError::RateLimit { .. } => "Rate limited",
        SdkError::Server { .. } => "Server error",
        SdkError::RequestTimeout { .. } => "Request timeout",
        SdkError::Network { .. } => "Network error",
        SdkError::Stream { .. } => "Stream error",
        // Fallback — should not happen since only retryable errors reach here.
        _ => "Transient error",
    }
}

/// Create a synthetic `[Aborted]` tool result for a tool call that was
/// cancelled before it could complete.
fn aborted_tool_result(tool_call_id: &str) -> ToolResult {
    ToolResult {
        tool_call_id: tool_call_id.to_string(),
        content: serde_json::Value::String("[Aborted]".into()),
        is_error: true,
    }
}

// ---------------------------------------------------------------------------
// Free function for tool execution (enables parallel use without &mut self)
// ---------------------------------------------------------------------------

/// Execute a single tool call. Factored out as a free function so it can be
/// called in parallel without borrowing `&mut Session`.
///
/// Performs schema validation (spec 3.8 step 2) between tool lookup and
/// execution. Invalid arguments produce an `is_error: true` result without
/// invoking the executor.
///
/// Returns `(ToolResult, Option<(tool_call_id, ImageAttachment)>)`. The
/// second element is `Some` when the tool produced image data that should
/// be attached to the tool result message for multimodal providers.
async fn execute_tool(
    tool_call: &ToolCall,
    registry: &crate::registry::ToolRegistry,
    env: &dyn ExecutionEnvironment,
    events: &EventEmitter,
    trunc_config: &TruncationConfig,
    tool_guard: Option<&Arc<ToolGuard>>,
    guard_context: Option<&Arc<GuardContext>>,
) -> (ToolResult, Option<(String, ImageAttachment)>) {
    events.emit_tool_call_start(&tool_call.name, &tool_call.id, &tool_call.arguments);

    // Check for argument parse errors (garbled/non-JSON arguments from the model)
    if let Some(ref err) = tool_call.parse_error {
        let error_msg = format!(
            "invalid JSON arguments for tool '{}': {err}. The arguments must be a valid JSON object.",
            tool_call.name
        );
        events.emit_tool_call_end_error(&tool_call.id, &error_msg);
        return (
            ToolResult {
                tool_call_id: tool_call.id.clone(),
                content: Value::String(error_msg),
                is_error: true,
            },
            None,
        );
    }

    // VALIDATE (spec 3.8 step 2) — before execute
    if let Err(e) = registry.validate_arguments(&tool_call.name, &tool_call.arguments) {
        let error_msg = e.to_string();
        events.emit_tool_call_end_error(&tool_call.id, &error_msg);
        return (
            ToolResult {
                tool_call_id: tool_call.id.clone(),
                content: Value::String(error_msg),
                is_error: true,
            },
            None,
        );
    }

    // GUARD CHECK — evaluate before executing the tool.
    // If a guard is present but context is missing, synthesize a fallback so
    // enforcement is never silently skipped due to missing attribution.
    let guard_warn = if let Some(guard) = tool_guard {
        let fallback;
        let ctx = match guard_context {
            Some(c) => c.as_ref(),
            None => {
                fallback = GuardContext::fallback();
                &fallback
            }
        };
        let working_dir = std::path::Path::new(env.working_directory());
        match guard.evaluate(ctx, &tool_call.name, &tool_call.arguments, working_dir) {
            GuardVerdict::Deny {
                reason,
                suggestion,
                rule_id,
            } => {
                let deny_msg =
                    format!("[BLOCKED by {rule_id}] {reason}\n\nSuggestion: {suggestion}");
                events.emit_tool_call_end(&tool_call.id, &deny_msg);
                return (
                    ToolResult {
                        tool_call_id: tool_call.id.clone(),
                        content: Value::String(deny_msg),
                        is_error: false,
                    },
                    None,
                );
            }
            GuardVerdict::Warn {
                reason,
                suggestion,
                rule_id,
            } => Some(format!(
                "\n\n⚠️  [GUARD WARNING: {rule_id}] {reason}\nSuggestion: {suggestion}"
            )),
            GuardVerdict::Allow => None,
        }
    } else {
        None
    };

    let result = match registry.get(&tool_call.name) {
        Some(tool) => tool.execute(tool_call.arguments.clone(), env).await,
        None => Err(AgentError::UnknownTool {
            name: tool_call.name.clone(),
        }),
    };

    match result {
        Ok(output) => {
            let output = if let Some(warning_text) = guard_warn {
                append_guard_warning(output, &warning_text)
            } else {
                output
            };
            let text = output.as_text();
            // Full output in event (spec 2.9: TOOL_CALL_END has untruncated output)
            events.emit_tool_call_end(&tool_call.id, text);

            // Emit a typed delegation event when the delegate tool succeeds,
            // so the TUI can react to it without parsing tool arguments.
            if tool_call.name == "delegate"
                && let Some(args) = tool_call.arguments.as_object()
            {
                let kind = args
                    .get("kind")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                let name = args
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                let instruction = args
                    .get("instruction")
                    .and_then(Value::as_str)
                    .map(String::from);
                events.emit_delegation(kind, name, instruction);
            }

            // Truncated version for LLM
            let truncated = truncate_tool_output(text, &tool_call.name, trunc_config);

            let attachment = match output {
                ToolOutput::ImageWithText {
                    data, media_type, ..
                } => Some((tool_call.id.clone(), ImageAttachment { data, media_type })),
                ToolOutput::Text(_) => None,
            };

            (
                ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    content: Value::String(truncated),
                    is_error: false,
                },
                attachment,
            )
        }
        Err(e) => {
            let error_msg = e.to_string();
            events.emit_tool_call_end_error(&tool_call.id, &error_msg);
            (
                ToolResult {
                    tool_call_id: tool_call.id.clone(),
                    content: Value::String(error_msg),
                    is_error: true,
                },
                None,
            )
        }
    }
}

/// Append a guard warning to tool output, preserving the output variant.
fn append_guard_warning(output: ToolOutput, warning_text: &str) -> ToolOutput {
    match output {
        ToolOutput::Text(mut text) => {
            text.push_str(warning_text);
            ToolOutput::Text(text)
        }
        ToolOutput::ImageWithText {
            mut text,
            data,
            media_type,
        } => {
            text.push_str(warning_text);
            ToolOutput::ImageWithText {
                text,
                data,
                media_type,
            }
        }
    }
}

#[cfg(test)]
mod guard_wiring_tests {
    use super::*;
    use crate::registry::ToolOutput;
    use serde_json::json;
    use stencila_models3::types::finish_reason::{FinishReason, Reason};
    use stencila_models3::types::message::Message;
    use stencila_models3::types::request::Request;
    use stencila_models3::types::response::Response;
    use stencila_models3::types::tool::ToolDefinition;

    struct CapturingClient {
        requests: std::sync::Mutex<Vec<Request>>,
    }

    #[async_trait]
    impl LlmClient for CapturingClient {
        async fn complete(&self, request: Request) -> Result<Response, SdkError> {
            self.requests
                .lock()
                .map_err(|e| SdkError::Configuration {
                    message: format!("mock lock: {e}"),
                })?
                .push(request);

            Ok(Response {
                id: "resp-1".into(),
                model: "test-model".into(),
                provider: "test".into(),
                message: Message::assistant("done"),
                finish_reason: FinishReason::new(Reason::Stop, None),
                usage: stencila_models3::types::usage::Usage::default(),
                raw: None,
                warnings: None,
                rate_limit: None,
            })
        }
    }

    struct NoopEnv;

    #[async_trait]
    impl ExecutionEnvironment for NoopEnv {
        async fn read_file(
            &self,
            _path: &str,
            _offset: Option<usize>,
            _limit: Option<usize>,
        ) -> AgentResult<crate::execution::FileContent> {
            Err(AgentError::Io {
                message: "not implemented".into(),
            })
        }

        async fn write_file(&self, _path: &str, _content: &str) -> AgentResult<()> {
            Err(AgentError::Io {
                message: "not implemented".into(),
            })
        }

        async fn file_exists(&self, _path: &str) -> bool {
            false
        }

        async fn delete_file(&self, _path: &str) -> AgentResult<()> {
            Err(AgentError::Io {
                message: "not implemented".into(),
            })
        }

        async fn list_directory(
            &self,
            _path: &str,
            _depth: usize,
        ) -> AgentResult<Vec<crate::types::DirEntry>> {
            Ok(Vec::new())
        }

        async fn exec_command(
            &self,
            _command: &str,
            _timeout_ms: u64,
            _working_dir: Option<&str>,
            _env_vars: Option<&std::collections::HashMap<String, String>>,
        ) -> AgentResult<crate::types::ExecResult> {
            Err(AgentError::Io {
                message: "not implemented".into(),
            })
        }

        async fn grep(
            &self,
            _pattern: &str,
            _path: &str,
            _options: &crate::types::GrepOptions,
        ) -> AgentResult<String> {
            Ok(String::new())
        }

        async fn glob_files(&self, _pattern: &str, _path: &str) -> AgentResult<Vec<String>> {
            Ok(Vec::new())
        }

        fn working_directory(&self) -> &str {
            "."
        }

        fn platform(&self) -> &str {
            "linux"
        }

        fn os_version(&self) -> String {
            "test".into()
        }
    }

    #[derive(Debug)]
    struct TestProfile {
        registry: crate::registry::ToolRegistry,
    }

    impl TestProfile {
        fn new() -> Self {
            Self {
                registry: crate::registry::ToolRegistry::new(),
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

        fn tool_registry_mut(&mut self) -> &mut crate::registry::ToolRegistry {
            &mut self.registry
        }

        fn tool_registry(&self) -> &crate::registry::ToolRegistry {
            &self.registry
        }

        fn base_instructions(&self) -> &str {
            "test instructions"
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
            8_192
        }
    }

    fn dynamic_tool(name: &str) -> crate::registry::RegisteredTool {
        crate::registry::RegisteredTool::new(
            ToolDefinition {
                name: name.into(),
                description: "dynamic test tool".into(),
                parameters: json!({
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
                }),
                strict: true,
            },
            Box::new(|_args, _env| Box::pin(async { Ok(ToolOutput::Text("ok".into())) })),
        )
    }

    #[test]
    fn append_guard_warning_text() {
        let output = ToolOutput::Text("result".into());
        let warned = append_guard_warning(output, "\n\n⚠️ warning");
        match warned {
            ToolOutput::Text(s) => assert!(s.ends_with("⚠️ warning")),
            _ => panic!("expected Text variant"),
        }
    }

    #[test]
    fn append_guard_warning_image_with_text() {
        let output = ToolOutput::ImageWithText {
            text: "image info".into(),
            data: vec![1, 2, 3],
            media_type: "image/png".into(),
        };
        let warned = append_guard_warning(output, "\n\n⚠️ warning");
        match warned {
            ToolOutput::ImageWithText {
                text,
                data,
                media_type,
            } => {
                assert!(text.ends_with("⚠️ warning"));
                assert_eq!(data, vec![1, 2, 3]);
                assert_eq!(media_type, "image/png");
            }
            _ => panic!("expected ImageWithText variant"),
        }
    }

    #[tokio::test]
    async fn register_tool_extends_allowed_tools_and_request_tools() -> AgentResult<()> {
        let client = Arc::new(CapturingClient {
            requests: std::sync::Mutex::new(Vec::new()),
        });
        let (mut session, _receiver) = ApiSession::new(
            Box::new(TestProfile::new()),
            Arc::new(NoopEnv),
            client.clone(),
            SessionConfig {
                allowed_tools: Some(vec!["echo".into()]),
                ..SessionConfig::default()
            },
            "system".into(),
            0,
            ApiSessionInit::default(),
        );

        session.register_tool(dynamic_tool("workflow_set_route"))?;
        session.submit("choose a branch").await?;

        let requests = client.requests.lock().map_err(|e| AgentError::Io {
            message: format!("mock lock: {e}"),
        })?;
        let tool_names: Vec<&str> = requests[0]
            .tools
            .as_ref()
            .expect("tools should be present")
            .iter()
            .map(|tool| tool.name.as_str())
            .collect();

        assert!(tool_names.contains(&"workflow_set_route"));
        assert_eq!(
            session.config().allowed_tools.as_ref(),
            Some(&vec!["echo".to_string(), "workflow_set_route".to_string()])
        );

        Ok(())
    }
}
