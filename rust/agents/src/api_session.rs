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
use image::ImageReader;
use serde_json::Value;
use stencila_interviews::interviewer::Interviewer;
use stencila_models3::api::accumulator::StreamAccumulator;
use stencila_models3::error::SdkError;
use stencila_models3::types::content::{ContentPart, ToolResultData};
use stencila_models3::types::message::Message;
use stencila_models3::types::request::Request;
use stencila_models3::types::response::Response;
use stencila_models3::types::role::Role;
use stencila_models3::types::stream_event::{StreamEvent, StreamEventType};
use stencila_models3::types::tool::{ToolCall, ToolChoice, ToolDefinition, ToolResult};

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

const MAX_IMAGE_DIMENSION: u32 = 8_000;

impl ImageAttachment {
    fn exceeds_max_dimensions(&self) -> AgentResult<bool> {
        let format = image::guess_format(&self.data).map_err(|error| AgentError::Io {
            message: format!("failed to detect image format: {error}"),
        })?;

        let (width, height) = ImageReader::with_format(std::io::Cursor::new(&self.data), format)
            .into_dimensions()
            .map_err(|error| AgentError::Io {
                message: format!("failed to read image dimensions: {error}"),
            })?;

        Ok(width > MAX_IMAGE_DIMENSION || height > MAX_IMAGE_DIMENSION)
    }
}

/// How images from tool results are injected into the message history.
///
/// Anthropic natively supports images inside tool-result messages, so they
/// can be inlined. Other vision-capable providers (OpenAI, Gemini) do not,
/// so images are collected and sent in a follow-up user message. Non-vision
/// providers receive no images at all.
enum ImageStrategy {
    /// Embed images directly in the tool-result message (Anthropic).
    Inline,
    /// Collect images and emit them in a follow-up user message.
    Deferred,
    /// Do not include images (non-vision providers).
    None,
}

const APPROX_CHARS_PER_TOKEN: u64 = 4;
const MIN_COMPACTION_RESERVE_TOKENS: u64 = 256;
const MAX_COMPACTION_RESERVE_TOKENS: u64 = 8_192;
const INTERNAL_COMPACT_TOOL_RESULTS_OLDER_THAN_ENTRIES: usize = 2;
const INTERNAL_COMPACT_MAX_TOOL_RESULT_CHARS: usize = 600;
const INTERNAL_COMPACT_PRESERVE_RECENT_ENTRIES: usize = 4;
const DROPPED_HISTORY_SUMMARY_MAX_CHARS: usize = 1_800;
const REACTIVE_TARGET_REDUCTION_PERCENT: u64 = 10;
const REACTIVE_TARGET_REDUCTION_MIN_CHARS: u64 = 16_000;

/// Estimated size of the next provider request.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct ContextUsageEstimate {
    request_chars: u64,
    request_tokens: u64,
    tool_chars: u64,
    message_chars: u64,
}

/// Budget used to decide when to compact and what input size to target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ContextBudget {
    context_window: u64,
    trigger_percent: u8,
    reserve_tokens: u64,
    trigger_tokens: u64,
    target_input_tokens: u64,
}

#[derive(Debug, Clone, Copy)]
struct CompactionGoal<'a> {
    trigger: &'a str,
    target_chars: Option<u64>,
    force_shrink: bool,
}

/// Metrics collected during a single context compaction pass.
#[derive(Debug, Clone, Copy)]
struct CompactionStats {
    before_chars: u64,
    after_chars: u64,
    removed_chars: u64,
    before_tokens: u64,
    after_tokens: u64,
    target_chars: Option<u64>,
    target_reached: bool,
    stripped_reasoning_turns: usize,
    stripped_thinking_parts: usize,
    summarized_tool_results: usize,
    removed_tool_result_chars: usize,
    removed_turns: usize,
    phases_applied: usize,
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

    /// Agent name for checkpoint records.
    agent_name: String,

    /// Timestamp captured at session construction for checkpoint records.
    created_at: String,

    /// Workflow attribution for checkpoint records.
    workflow_attribution: Option<crate::store::WorkflowAttribution>,

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
    /// events and guard context attribution. When `None`, a time-ordered
    /// UUID v7 is generated.
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

        // Register optional tools (e.g. snap) so agents that list them in
        // allowed-tools can use them. The allowed-tools filter in
        // `build_request` ensures only agents that opt in will see them.
        if let Err(e) = crate::tools::register_optional_tools(profile.tool_registry_mut()) {
            tracing::warn!("failed to register optional tools: {e}");
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
            agent_name: String::new(),
            created_at: crate::types::now_timestamp(),
            workflow_attribution: None,
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

    /// Set the agent name recorded in checkpoint persistence records.
    pub fn set_agent_name(&mut self, name: impl Into<String>) {
        self.agent_name = name.into();
    }

    /// Set workflow attribution metadata for checkpoint persistence records.
    pub fn set_workflow_attribution(&mut self, attribution: crate::store::WorkflowAttribution) {
        self.workflow_attribution = Some(attribution);
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
        if !crate::store::should_persist(self.persistence_mode.as_ref()) {
            return Ok(());
        }

        let (workflow_run_id, workflow_thread_id, workflow_node_id) =
            crate::store::workflow_fields(self.workflow_attribution.as_ref());

        let record = crate::store::SessionRecord {
            session_id: self.session_id().to_string(),
            backend_kind: "api".to_string(),
            agent_name: self.agent_name.clone(),
            provider_name: self.profile.id().to_string(),
            model_name: self.profile.model().to_string(),
            state: self.state,
            total_turns: i64::from(self.total_turns),
            resumability: crate::store::Resumability::Full,
            created_at: self.created_at.clone(),
            updated_at: crate::types::now_timestamp(),
            workflow_run_id,
            workflow_thread_id,
            workflow_node_id,
            provider_resume_state: None,
            config_snapshot: None,
            system_prompt: None,
            lease_holder: None,
            lease_expires_at: None,
        };

        crate::store::write_checkpoint(store, &record, &self.history)
    }

    /// Handle the result of a checkpoint call according to the persistence policy.
    fn handle_checkpoint_result(&self, result: AgentResult<()>) -> AgentResult<()> {
        crate::store::handle_checkpoint_result(self.persistence_mode.as_ref(), result)
    }

    /// Restore persisted conversation history and turn counter into this
    /// session, making it ready to continue where a previous session left off.
    ///
    /// Call this after [`set_persistence`] so that the next checkpoint writes
    /// the merged history. Incomplete trailing assistant turns (from sessions
    /// that were persisted mid-processing) are automatically dropped.
    pub fn hydrate(&mut self, persisted_state: SessionState, turns: Vec<Turn>) {
        self.history = normalize_turns_for_hydration(persisted_state, turns);
        self.total_turns = self
            .history
            .iter()
            .filter(|t| matches!(t, Turn::Assistant { .. }))
            .count() as u32;
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

            // 4. Proactive context compaction before hitting hard overflow.
            // If a reactive compaction just happened for this retry path, do
            // not compact again before giving the retry a chance to succeed.
            if !compaction_attempted {
                self.maybe_proactive_compaction();
            }

            // 4b. Context-usage warning (spec 5.5). Emit after any proactive
            // compaction so usage telemetry reflects the outgoing request.
            self.check_context_usage();

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
                    if matches!(e, SdkError::ContextLength { .. }) && !compaction_attempted {
                        let current = self.estimate_request_chars();
                        let reduction = current
                            .saturating_mul(REACTIVE_TARGET_REDUCTION_PERCENT)
                            .saturating_div(100)
                            .max(REACTIVE_TARGET_REDUCTION_MIN_CHARS)
                            .min(current);
                        let target_chars = current.saturating_sub(reduction);

                        if self
                            .compact_history(CompactionGoal {
                                trigger: "reactive",
                                target_chars: Some(target_chars),
                                force_shrink: true,
                            })
                            .is_some()
                        {
                            compaction_attempted = true;
                            let mut data = serde_json::Map::new();
                            data.insert("severity".into(), Value::String("warning".into()));
                            data.insert(
                                "message".into(),
                                Value::String(
                                    "Context length exceeded — compacted history and retrying"
                                        .into(),
                                ),
                            );
                            self.events.emit(EventKind::Error, data);
                            continue;
                        }
                    }

                    return self.handle_sdk_error(e);
                }
            };

            // Successful LLM call — reset the compaction flag so that
            // later rounds (which may accumulate new content) can compact
            // again if needed.
            compaction_attempted = false;

            // 6. Record assistant turn
            let streamed_text = partial_text
                .lock()
                .map(|text| text.clone())
                .unwrap_or_default();
            let response_text = response.text();
            let text = if response_text.is_empty() && !streamed_text.is_empty() {
                streamed_text
            } else {
                response_text
            };
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

            let mut response_content_parts = response.message.content.clone();
            if response_content_parts.is_empty() && !text.is_empty() {
                response_content_parts.push(ContentPart::text(&text));
            }
            self.history.push(Turn::Assistant {
                content: text.clone(),
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
        self.events.emit_processing_end(self.state);
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
        let tools = self.request_tools();
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
                    // Determine the image injection strategy for this provider:
                    //  - Inline: provider supports images in tool results (Anthropic).
                    //  - Deferred: other vision-capable providers receive images in
                    //    a follow-up user message after all tool results.
                    //  - None: non-vision providers receive no images at all.
                    let image_strategy = if !self.profile.supports_vision() {
                        ImageStrategy::None
                    } else if self.profile.supports_image_in_tool_result() {
                        ImageStrategy::Inline
                    } else {
                        ImageStrategy::Deferred
                    };
                    let mut deferred_image_parts: Vec<ContentPart> = Vec::new();

                    for result in results {
                        if matches!(image_strategy, ImageStrategy::Inline)
                            && let Some(att) = self.image_attachments.get(&result.tool_call_id)
                        {
                            let mut msg = Message::new(
                                Role::Tool,
                                vec![ContentPart::tool_result_with_image(
                                    &result.tool_call_id,
                                    result.content.clone(),
                                    result.is_error,
                                    att.data.clone(),
                                    &att.media_type,
                                )],
                            );
                            msg.tool_call_id = Some(result.tool_call_id.clone());
                            messages.push(msg);
                            continue;
                        }
                        if matches!(image_strategy, ImageStrategy::Deferred)
                            && let Some(att) = self.image_attachments.get(&result.tool_call_id)
                        {
                            deferred_image_parts
                                .push(ContentPart::image_data(att.data.clone(), &att.media_type));
                        }
                        messages.push(Message::tool_result(
                            &result.tool_call_id,
                            result.content.clone(),
                            result.is_error,
                        ));
                    }

                    if !deferred_image_parts.is_empty() {
                        let mut parts = vec![ContentPart::text(
                            "The following images were produced by the preceding tool calls.",
                        )];
                        parts.extend(deferred_image_parts);
                        messages.push(Message::new(Role::User, parts));
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

    /// Store an image attachment only when the provider supports vision
    /// (images in tool results). Avoids accumulating dead weight for
    /// providers that receive only the text fallback.
    fn store_attachment_if_supported(&mut self, attachment: Option<(String, ImageAttachment)>) {
        if let Some((id, img)) = attachment
            && self.profile.supports_vision()
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
    /// `context_window_size`. Called after any proactive compaction so the
    /// emitted telemetry reflects the outgoing request estimate.
    fn check_context_usage(&self) {
        let usage = self.estimate_context_usage();
        let approx_tokens = usage.request_tokens;
        let context_size = self.profile.context_window_size();
        let budget = ContextBudget::new(
            context_size,
            self.config.compaction_trigger_percent,
            self.profile.max_output_tokens(),
        );
        let current_pct = if context_size > 0 {
            (approx_tokens.saturating_mul(100) / context_size) as u32
        } else {
            0
        };
        let projected_tokens = approx_tokens.saturating_add(budget.reserve_tokens);
        let projected_pct = if context_size > 0 {
            (projected_tokens.saturating_mul(100) / context_size) as u32
        } else {
            0
        };

        // Always emit context usage info
        let mut usage_data = serde_json::Map::new();
        usage_data.insert("percent".into(), Value::Number(current_pct.into()));
        usage_data.insert("current_percent".into(), Value::Number(current_pct.into()));
        usage_data.insert("approx_tokens".into(), Value::Number(approx_tokens.into()));
        usage_data.insert(
            "context_window_size".into(),
            Value::Number(context_size.into()),
        );
        usage_data.insert(
            "reserve_tokens".into(),
            Value::Number(budget.reserve_tokens.into()),
        );
        usage_data.insert(
            "projected_tokens".into(),
            Value::Number(projected_tokens.into()),
        );
        usage_data.insert(
            "projected_percent".into(),
            Value::Number(projected_pct.into()),
        );
        usage_data.insert(
            "target_input_tokens".into(),
            Value::Number(budget.target_input_tokens.into()),
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
                Value::String(format!(
                    "Context usage at ~{current_pct}% of context window (~{projected_pct}% projected with response reserve)"
                )),
            );
            data.insert("approx_tokens".into(), Value::Number(approx_tokens.into()));
            data.insert(
                "context_window_size".into(),
                Value::Number(context_size.into()),
            );
            data.insert(
                "reserve_tokens".into(),
                Value::Number(budget.reserve_tokens.into()),
            );
            data.insert(
                "projected_tokens".into(),
                Value::Number(projected_tokens.into()),
            );
            data.insert(
                "projected_percent".into(),
                Value::Number(projected_pct.into()),
            );
            data.insert(
                "target_input_tokens".into(),
                Value::Number(budget.target_input_tokens.into()),
            );
            self.events.emit(crate::types::EventKind::Error, data);
        }
    }

    /// Estimate total character count for the next provider request.
    ///
    /// The estimate is intentionally derived from the message content that
    /// `build_request` will send rather than the raw persisted history. This
    /// keeps proactive compaction aligned with history thinking replay
    /// filtering and multimodal image injection decisions.
    fn estimate_request_chars(&self) -> u64 {
        self.estimate_context_usage().request_chars
    }

    /// Estimate message and tool-schema size for the next provider request.
    fn estimate_context_usage(&self) -> ContextUsageEstimate {
        let message_chars = self.estimate_request_messages_chars();
        let tool_chars = estimate_tools_chars(&self.request_tools());
        let request_chars = message_chars.saturating_add(tool_chars);

        ContextUsageEstimate {
            request_chars,
            request_tokens: request_chars / APPROX_CHARS_PER_TOKEN,
            tool_chars,
            message_chars,
        }
    }

    /// Estimate character count for request messages only.
    fn estimate_request_messages_chars(&self) -> u64 {
        let mut messages = vec![Message::system(&self.system_prompt)];
        messages.extend(self.convert_history_to_messages());

        messages.iter().map(estimate_message_chars).sum::<u64>()
    }

    /// Resolve the tool definitions that would be sent with the next request.
    fn request_tools(&self) -> Option<Vec<ToolDefinition>> {
        if let Some(ref allowed) = self.config.allowed_tools {
            let filtered: Vec<_> = self
                .profile
                .tools()
                .into_iter()
                .filter(|tool| allowed.iter().any(|name| name == &tool.name))
                .collect();

            if filtered.is_empty() {
                None
            } else {
                Some(filtered)
            }
        } else {
            Some(self.profile.tools())
        }
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

        let usage = self.estimate_context_usage();
        let budget = ContextBudget::new(
            context_size,
            trigger_pct as u8,
            self.profile.max_output_tokens(),
        );

        if usage.request_tokens < budget.target_input_tokens {
            return;
        }

        let projected_tokens = usage.request_tokens.saturating_add(budget.reserve_tokens);
        let projected_pct = (projected_tokens.saturating_mul(100)) / context_size;

        self.compact_history(CompactionGoal {
            trigger: &format!(
                "proactive (current {} tokens + reserve {} = projected {projected_pct}% >= trigger {trigger_pct}%)",
                usage.request_tokens, budget.reserve_tokens
            ),
            target_chars: Some(compaction_target_chars(&budget)),
            force_shrink: false,
        });
    }

    /// Attempt to compact the conversation history to reduce context size.
    ///
    /// Returns compaction stats when the outgoing request shrank, or `None`
    /// when compaction could not reduce the next provider request.
    fn compact_history(&mut self, goal: CompactionGoal) -> Option<CompactionStats> {
        let before = self.estimate_request_chars();
        let mut stripped_reasoning_turns = 0usize;
        let mut stripped_thinking_parts = 0usize;
        let mut summarized_tool_results = 0usize;
        let mut removed_tool_result_chars = 0usize;
        let mut removed_turns = 0usize;
        let mut phases_applied = 0usize;

        if matches!(
            self.config.history_thinking_replay,
            HistoryThinkingReplay::Full
        ) {
            let (reasoning, thinking) = strip_replayed_thinking(&mut self.history);
            if reasoning > 0 || thinking > 0 {
                stripped_reasoning_turns += reasoning;
                stripped_thinking_parts += thinking;
                phases_applied += 1;
            }
        }

        if self.compaction_target_reached(goal.target_chars) {
            return self.finish_compaction(
                goal,
                before,
                stripped_reasoning_turns,
                stripped_thinking_parts,
                summarized_tool_results,
                removed_tool_result_chars,
                removed_turns,
                phases_applied,
            );
        }

        let (summarized, removed_chars) = compact_old_tool_results(
            &mut self.history,
            &mut self.image_attachments,
            if goal.force_shrink {
                0
            } else {
                INTERNAL_COMPACT_TOOL_RESULTS_OLDER_THAN_ENTRIES
            },
            INTERNAL_COMPACT_MAX_TOOL_RESULT_CHARS,
        );
        if summarized > 0 || removed_chars > 0 {
            summarized_tool_results += summarized;
            removed_tool_result_chars += removed_chars;
            phases_applied += 1;
        }

        if self.compaction_target_reached(goal.target_chars) {
            return self.finish_compaction(
                goal,
                before,
                stripped_reasoning_turns,
                stripped_thinking_parts,
                summarized_tool_results,
                removed_tool_result_chars,
                removed_turns,
                phases_applied,
            );
        }

        let summary_max_chars = goal
            .target_chars
            .map_or(DROPPED_HISTORY_SUMMARY_MAX_CHARS, |target| {
                let current = self.estimate_request_chars();
                if current <= target {
                    DROPPED_HISTORY_SUMMARY_MAX_CHARS
                } else {
                    DROPPED_HISTORY_SUMMARY_MAX_CHARS.min((current - target) as usize)
                }
            })
            .max(256);
        if let Some(removed) = drop_middle_history_with_summary(
            &mut self.history,
            &mut self.image_attachments,
            INTERNAL_COMPACT_PRESERVE_RECENT_ENTRIES,
            summary_max_chars,
        ) {
            removed_turns += removed;
            phases_applied += 1;
        }

        self.finish_compaction(
            goal,
            before,
            stripped_reasoning_turns,
            stripped_thinking_parts,
            summarized_tool_results,
            removed_tool_result_chars,
            removed_turns,
            phases_applied,
        )
    }

    fn compaction_target_reached(&self, target_chars: Option<u64>) -> bool {
        target_chars.is_some_and(|target| self.estimate_request_chars() <= target)
    }

    #[allow(clippy::too_many_arguments)]
    fn finish_compaction(
        &self,
        goal: CompactionGoal,
        before: u64,
        stripped_reasoning_turns: usize,
        stripped_thinking_parts: usize,
        summarized_tool_results: usize,
        removed_tool_result_chars: usize,
        removed_turns: usize,
        phases_applied: usize,
    ) -> Option<CompactionStats> {
        let after = self.estimate_request_chars();
        let removed_chars = before.saturating_sub(after);
        let target_reached = goal.target_chars.is_some_and(|target| after <= target);
        let stats = CompactionStats {
            before_chars: before,
            after_chars: after,
            removed_chars,
            before_tokens: before / APPROX_CHARS_PER_TOKEN,
            after_tokens: after / APPROX_CHARS_PER_TOKEN,
            target_chars: goal.target_chars,
            target_reached,
            stripped_reasoning_turns,
            stripped_thinking_parts,
            summarized_tool_results,
            removed_tool_result_chars,
            removed_turns,
            phases_applied,
        };
        tracing::debug!(
            before_chars = before,
            after_chars = after,
            removed_chars,
            trigger = goal.trigger,
            target_reached,
            "context compaction complete"
        );

        let shrank = after < before;
        if shrank || (goal.force_shrink && removed_chars > 0) {
            self.events.emit_info(
                "CONTEXT_COMPACTION",
                format!(
                    "trigger={}, chars={} -> {}, ~{} -> ~{} tokens, removed_chars={}, target_chars={}, target_reached={}, phases_applied={}, reasoning_turns={}, thinking_parts={}, summarized_results={}, removed_result_chars={}, removed_turns={}",
                    goal.trigger,
                    stats.before_chars,
                    stats.after_chars,
                    stats.before_tokens,
                    stats.after_tokens,
                    stats.removed_chars,
                    stats
                        .target_chars
                        .map_or_else(|| "none".to_string(), |target| target.to_string()),
                    stats.target_reached,
                    stats.phases_applied,
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

        if is_context_length || is_retryable {
            // Retryable errors: keep the session open so the user can retry.
            // Context-length errors are also warnings rather than terminal
            // errors per the spec; transition back to IDLE rather than CLOSED.
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

impl ContextBudget {
    fn new(context_window: u64, trigger_percent: u8, max_output_tokens: Option<u64>) -> Self {
        let trigger_percent = trigger_percent.min(100);
        let reserve_tokens = default_compaction_reserve_tokens(context_window, max_output_tokens);
        let trigger_tokens = context_window.saturating_mul(u64::from(trigger_percent)) / 100;
        let target_input_tokens = trigger_tokens.saturating_sub(reserve_tokens);

        Self {
            context_window,
            trigger_percent,
            reserve_tokens,
            trigger_tokens,
            target_input_tokens,
        }
    }
}

fn default_compaction_reserve_tokens(context_window: u64, max_output_tokens: Option<u64>) -> u64 {
    if context_window == 0 {
        return 0;
    }

    let half_window = (context_window / 2).max(1);
    let proportional = (context_window / 10)
        .clamp(MIN_COMPACTION_RESERVE_TOKENS, MAX_COMPACTION_RESERVE_TOKENS)
        .min(half_window);

    max_output_tokens.map_or(proportional, |max_output| proportional.min(max_output))
}

fn compaction_target_chars(budget: &ContextBudget) -> u64 {
    budget
        .target_input_tokens
        .saturating_mul(APPROX_CHARS_PER_TOKEN)
}

fn strip_replayed_thinking(history: &mut [Turn]) -> (usize, usize) {
    let mut stripped_reasoning_turns = 0usize;
    let mut stripped_thinking_parts = 0usize;

    for turn in history.iter_mut() {
        if let Turn::Assistant {
            reasoning,
            thinking_parts,
            response_content_parts,
            ..
        } = turn
        {
            if reasoning.is_some() {
                *reasoning = None;
                stripped_reasoning_turns += 1;
            }
            if !thinking_parts.is_empty() {
                stripped_thinking_parts += thinking_parts.len();
                thinking_parts.clear();
            }
            let before_len = response_content_parts.len();
            response_content_parts.retain(|part| {
                !matches!(
                    part,
                    ContentPart::Thinking { .. } | ContentPart::RedactedThinking { .. }
                )
            });
            stripped_thinking_parts += before_len - response_content_parts.len();
        }
    }

    (stripped_reasoning_turns, stripped_thinking_parts)
}

fn compact_old_tool_results(
    history: &mut [Turn],
    image_attachments: &mut HashMap<String, ImageAttachment>,
    preserve_tail_entries: usize,
    max_tool_result_chars: usize,
) -> (usize, usize) {
    let len = history.len();
    let compactable = len.saturating_sub(preserve_tail_entries.min(len));
    let mut summarized_tool_results = 0usize;
    let mut removed_tool_result_chars = 0usize;

    for turn in history[..compactable].iter_mut() {
        if let Turn::ToolResults { results, .. } = turn {
            for result in results.iter_mut() {
                image_attachments.remove(&result.tool_call_id);

                let text = tool_result_content_text(&result.content);
                let char_count = text.chars().count();
                if char_count > max_tool_result_chars {
                    let removed = char_count.saturating_sub(max_tool_result_chars);
                    removed_tool_result_chars += removed;
                    summarized_tool_results += 1;
                    result.content =
                        Value::String(compact_tool_result_text(&text, max_tool_result_chars));
                }
            }
        }
    }

    (summarized_tool_results, removed_tool_result_chars)
}

fn drop_middle_history_with_summary(
    history: &mut Vec<Turn>,
    image_attachments: &mut HashMap<String, ImageAttachment>,
    preserve_recent_entries: usize,
    summary_max_chars: usize,
) -> Option<usize> {
    let preserve_recent = preserve_recent_entries.max(1);
    if history.len() <= preserve_recent + 6 {
        return None;
    }

    let keep_head = 1;
    let total = history.len();
    let tail_start = safe_tail_start(history, total.saturating_sub(preserve_recent));
    let keep_tail = total - tail_start;
    if keep_head + keep_tail >= total {
        return None;
    }

    let removed_turns = total - keep_head - keep_tail;
    let removed = &history[keep_head..tail_start];
    for turn in removed {
        if let Turn::ToolResults { results, .. } = turn {
            for result in results {
                image_attachments.remove(&result.tool_call_id);
            }
        }
    }

    let summary = Turn::system(summarize_removed_turns(removed, summary_max_chars));
    let mut new_history = Vec::with_capacity(keep_head + 1 + keep_tail);
    new_history.extend(history.drain(..keep_head));
    new_history.push(summary);
    let remaining = history.len();
    new_history.extend(history.drain(remaining - keep_tail..));
    *history = new_history;

    Some(removed_turns)
}

fn safe_tail_start(history: &[Turn], candidate: usize) -> usize {
    let mut tail_start = candidate;
    while tail_start < history.len() && matches!(history[tail_start], Turn::ToolResults { .. }) {
        tail_start += 1;
    }
    tail_start
}

fn summarize_removed_turns(turns: &[Turn], max_chars: usize) -> String {
    let tool_call_count = turns
        .iter()
        .map(|turn| match turn {
            Turn::Assistant { tool_calls, .. } => tool_calls.len(),
            _ => 0,
        })
        .sum::<usize>();
    let tool_result_count = turns
        .iter()
        .map(|turn| match turn {
            Turn::ToolResults { results, .. } => results.len(),
            _ => 0,
        })
        .sum::<usize>();

    let mut summary = format!(
        "[Context compacted: {} earlier history entries removed. Extractive summary follows.]\n- Omitted tool calls: {tool_call_count}; omitted tool results: {tool_result_count}.",
        turns.len()
    );

    for (index, turn) in turns.iter().enumerate() {
        match turn {
            Turn::User { content, .. } => push_summary_line(
                &mut summary,
                max_chars,
                format_args!("User requested: {}", compact_summary_text(content, 220)),
            ),
            Turn::Steering { content, .. } => push_summary_line(
                &mut summary,
                max_chars,
                format_args!("Steering: {}", compact_summary_text(content, 180)),
            ),
            Turn::System { content, .. } => push_summary_line(
                &mut summary,
                max_chars,
                format_args!("System note: {}", compact_summary_text(content, 180)),
            ),
            Turn::Assistant {
                content,
                tool_calls,
                ..
            } => {
                if !content.trim().is_empty() {
                    push_summary_line(
                        &mut summary,
                        max_chars,
                        format_args!("Assistant outcome: {}", compact_summary_text(content, 220)),
                    );
                }
                for call in tool_calls {
                    push_summary_line(
                        &mut summary,
                        max_chars,
                        format_args!(
                            "Tool call: {} {}",
                            call.name,
                            compact_summary_text(&call.arguments.to_string(), 220)
                        ),
                    );
                }
            }
            Turn::ToolResults { results, .. } => {
                for result in results {
                    let status = if result.is_error { "error" } else { "ok" };
                    push_summary_line(
                        &mut summary,
                        max_chars,
                        format_args!(
                            "Tool result ({status}) for {}: {}",
                            result.tool_call_id,
                            compact_summary_text(&tool_result_content_text(&result.content), 240)
                        ),
                    );
                }
            }
        }

        if summary.chars().count() >= max_chars
            || (index + 1 < turns.len() && summary.ends_with("[summary truncated]"))
        {
            break;
        }
    }

    limit_summary(summary, max_chars)
}

fn push_summary_line(summary: &mut String, max_chars: usize, args: std::fmt::Arguments<'_>) {
    if summary.chars().count() >= max_chars {
        return;
    }

    let line = format!("\n- {args}");
    summary.push_str(&line);
    if summary.chars().count() > max_chars {
        *summary = limit_summary(std::mem::take(summary), max_chars);
    }
}

fn limit_summary(summary: String, max_chars: usize) -> String {
    let char_count = summary.chars().count();
    if char_count <= max_chars {
        return summary;
    }

    let marker = "\n- [summary truncated]";
    if max_chars <= marker.chars().count() {
        return marker.chars().take(max_chars).collect();
    }

    let keep = max_chars - marker.chars().count();
    let mut limited: String = summary.chars().take(keep).collect();
    limited.push_str(marker);
    limited
}

fn compact_summary_text(text: &str, max_chars: usize) -> String {
    let sanitized = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let char_count = sanitized.chars().count();
    if char_count <= max_chars {
        sanitized
    } else if max_chars <= 1 {
        "…".chars().take(max_chars).collect()
    } else {
        let keep = max_chars - 1;
        let mut truncated: String = sanitized.chars().take(keep).collect();
        truncated.push('…');
        truncated
    }
}

/// Estimate character count for a request message.
fn estimate_message_chars(message: &Message) -> u64 {
    let role_chars = match message.role {
        Role::System => "system".len() as u64,
        Role::User => "user".len() as u64,
        Role::Assistant => "assistant".len() as u64,
        Role::Tool => "tool".len() as u64,
        Role::Developer => "developer".len() as u64,
    };

    role_chars
        .saturating_add(message.name.as_ref().map_or(0, |name| name.len() as u64))
        .saturating_add(
            message
                .tool_call_id
                .as_ref()
                .map_or(0, |id| id.len() as u64),
        )
        .saturating_add(estimate_content_parts_chars(&message.content))
}

/// Estimate character count for a slice of [`ContentPart`]s.
fn estimate_content_parts_chars(parts: &[ContentPart]) -> u64 {
    let mut chars: u64 = 0;
    for part in parts {
        match part {
            ContentPart::Text { text } => chars += text.len() as u64,
            ContentPart::Image { image } => {
                chars += image.url.as_ref().map_or(0, |url| url.len() as u64);
                chars += image.data.as_ref().map_or(0, |data| data.len() as u64);
                chars += image
                    .media_type
                    .as_ref()
                    .map_or(0, |media_type| media_type.len() as u64);
                chars += image
                    .detail
                    .as_ref()
                    .map_or(0, |detail| detail.len() as u64);
            }
            ContentPart::Audio { audio } => {
                chars += audio.url.as_ref().map_or(0, |url| url.len() as u64);
                chars += audio.data.as_ref().map_or(0, |data| data.len() as u64);
                chars += audio
                    .media_type
                    .as_ref()
                    .map_or(0, |media_type| media_type.len() as u64);
            }
            ContentPart::Document { document } => {
                chars += document.url.as_ref().map_or(0, |url| url.len() as u64);
                chars += document.data.as_ref().map_or(0, |data| data.len() as u64);
                chars += document
                    .media_type
                    .as_ref()
                    .map_or(0, |media_type| media_type.len() as u64);
                chars += document
                    .file_name
                    .as_ref()
                    .map_or(0, |file_name| file_name.len() as u64);
            }
            ContentPart::ToolCall { tool_call } => {
                chars += tool_call.id.len() as u64;
                chars += tool_call.name.len() as u64;
                chars += tool_call.arguments.to_string().len() as u64;
                chars += tool_call.call_type.len() as u64;
                if let Some(ref sig) = tool_call.thought_signature {
                    chars += sig.len() as u64;
                }
            }
            ContentPart::ToolResult { tool_result } => {
                chars += estimate_tool_result_data_chars(tool_result);
            }
            ContentPart::Thinking { thinking } | ContentPart::RedactedThinking { thinking } => {
                chars += thinking.text.len() as u64;
                if let Some(ref sig) = thinking.signature {
                    chars += sig.len() as u64;
                }
            }
            ContentPart::Extension(value) => chars += value.to_string().len() as u64,
        }
    }
    chars
}

/// Estimate character count for tool definitions sent outside messages.
fn estimate_tools_chars(tools: &Option<Vec<ToolDefinition>>) -> u64 {
    tools.as_ref().map_or(0, |tools| {
        tools
            .iter()
            .map(|tool| {
                (tool.name.len() + tool.description.len() + tool.parameters.to_string().len())
                    as u64
            })
            .sum()
    })
}

fn estimate_tool_result_data_chars(tool_result: &ToolResultData) -> u64 {
    (tool_result.tool_call_id.len() + tool_result.content.to_string().len()) as u64
        + tool_result
            .image_data
            .as_ref()
            .map_or(0, |data| data.len() as u64)
        + tool_result
            .image_media_type
            .as_ref()
            .map_or(0, |media_type| media_type.len() as u64)
}

fn tool_result_content_text(content: &Value) -> String {
    content
        .as_str()
        .map_or_else(|| content.to_string(), ToString::to_string)
}

fn compact_tool_result_text(text: &str, max_chars: usize) -> String {
    let char_count = text.chars().count();
    if char_count <= max_chars {
        return text.to_string();
    }

    if max_chars == 0 {
        return String::new();
    }

    let no_original_marker = format!(
        "[Output compacted — {char_count} chars removed to free context space. No original output preserved.]"
    );
    if max_chars <= no_original_marker.chars().count() {
        return no_original_marker.chars().take(max_chars).collect();
    }

    let (marker, preserved_chars) = compact_tool_result_marker_and_preserved_chars(
        char_count,
        max_chars,
        " chars removed from the middle to free context space.",
    );

    if preserved_chars == 0 {
        return no_original_marker.chars().take(max_chars).collect();
    }

    let tail_chars = preserved_chars / 2;
    let head_chars = preserved_chars - tail_chars;
    let head: String = text.chars().take(head_chars).collect();
    let tail: String = text.chars().skip(char_count - tail_chars).collect();

    format!("{head}\n\n{marker}\n\n{tail}")
}

fn compact_tool_result_marker_and_preserved_chars(
    char_count: usize,
    max_chars: usize,
    marker_suffix: &str,
) -> (String, usize) {
    let mut preserved_chars = max_chars;

    loop {
        let removed = char_count.saturating_sub(preserved_chars);
        let marker = format!("[Output compacted — {removed}{marker_suffix}]");
        let separators = 4usize;
        let overhead = marker.chars().count().saturating_add(separators);
        let next_preserved_chars = max_chars.saturating_sub(overhead);

        if next_preserved_chars == preserved_chars {
            return (marker, preserved_chars);
        }

        preserved_chars = next_preserved_chars;
    }
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
                } => {
                    let attachment = ImageAttachment { data, media_type };
                    match attachment.exceeds_max_dimensions() {
                        Ok(true) => None,
                        Ok(false) => Some((tool_call.id.clone(), attachment)),
                        Err(error) => {
                            let error_msg = error.to_string();
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
                    }
                }
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

// ---------------------------------------------------------------------------
// Session hydration from persisted store
// ---------------------------------------------------------------------------

/// Reconstruct a live [`ApiSession`] from a persisted [`SessionRecord`] and
/// turn history.
///
/// The hydrated session starts in [`SessionState::Idle`] regardless of the
/// persisted state, preserves the original session ID, history, and
/// `total_turns`, and is ready for new [`submit()`](ApiSession::submit) calls.
///
/// Callers that need persistence on the hydrated session should call
/// [`set_persistence()`](ApiSession::set_persistence) after hydration.
pub fn hydrate_api_session(
    profile: Box<dyn ProviderProfile>,
    execution_env: Arc<dyn ExecutionEnvironment>,
    client: Arc<dyn LlmClient>,
    system_prompt: String,
    record: &crate::store::SessionRecord,
    turns: Vec<Turn>,
) -> (ApiSession, EventReceiver) {
    let config = SessionConfig::default();

    let init = ApiSessionInit {
        session_id: Some(record.session_id.clone()),
        ..Default::default()
    };

    let (mut session, receiver) = ApiSession::new(
        profile,
        execution_env,
        client,
        config,
        system_prompt,
        0,
        init,
    );

    // Restore persisted state: overwrite the empty history and zero total_turns
    // that `new()` initialised. The session starts in `Idle` (from `new()`);
    // `normalize_turns_for_hydration` handles incomplete turns from sessions
    // that were persisted while `Processing`.
    session.history = normalize_turns_for_hydration(record.state, turns);
    session.total_turns = u32::try_from(record.total_turns).unwrap_or(u32::MAX);

    (session, receiver)
}

/// If the session was persisted while `Processing`, the final assistant turn
/// may be incomplete (empty or whitespace-only content with no tool calls).
/// Drop it so the hydrated session can cleanly re-submit.
fn normalize_turns_for_hydration(persisted_state: SessionState, mut turns: Vec<Turn>) -> Vec<Turn> {
    if persisted_state != SessionState::Processing {
        return turns;
    }

    if let Some(Turn::Assistant {
        content,
        tool_calls,
        ..
    }) = turns.last()
        && content.trim().is_empty()
        && tool_calls.is_empty()
    {
        turns.pop();
    }

    turns
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::ToolOutput;
    use serde_json::json;
    use stencila_models3::types::finish_reason::{FinishReason, Reason};
    use stencila_models3::types::message::Message;
    use stencila_models3::types::request::Request;
    use stencila_models3::types::response::Response;
    use stencila_models3::types::tool::ToolDefinition;

    #[test]
    fn reserve_and_target_calculation_for_common_windows() {
        let cases = [
            (4_096, 409, 3_072),
            (8_192, 819, 6_144),
            (32_000, 3_200, 24_000),
            (200_000, 8_192, 161_808),
            (1_000_000, 8_192, 841_808),
        ];

        for (context_window, expected_reserve, min_target) in cases {
            let budget = ContextBudget::new(context_window, 85, None);
            assert_eq!(budget.reserve_tokens, expected_reserve);
            assert!(
                budget.target_input_tokens >= min_target,
                "target for {context_window} should not be over-eager"
            );
            assert_eq!(
                compaction_target_chars(&budget),
                budget.target_input_tokens * APPROX_CHARS_PER_TOKEN
            );
        }
    }

    #[test]
    fn reserve_uses_known_max_output_as_upper_bound() {
        let budget = ContextBudget::new(200_000, 85, Some(4_096));
        assert_eq!(budget.reserve_tokens, 4_096);
        assert_eq!(budget.target_input_tokens, 165_904);
    }

    #[test]
    fn compact_tool_result_text_respects_total_output_budget() {
        for max_chars in [0, 1, 8, 64, 600] {
            let compacted = compact_tool_result_text(&"x".repeat(2_000), max_chars);
            assert!(
                compacted.chars().count() <= max_chars,
                "compacted tool result should fit within {max_chars} chars"
            );
        }
    }

    #[test]
    fn compact_tool_result_text_preserves_head_and_tail_when_budget_allows() {
        let text = format!("{}{}{}", "a".repeat(200), "b".repeat(200), "c".repeat(200));
        let compacted = compact_tool_result_text(&text, 180);

        assert!(compacted.chars().count() <= 180);
        assert!(compacted.starts_with('a'));
        assert!(compacted.ends_with('c'));
        assert!(compacted.contains("Output compacted"));
    }

    #[test]
    fn extractive_summary_preserves_high_value_facts_and_is_bounded() {
        let turns = vec![
            Turn::user("Please inspect src/lib.rs and fix the parser error"),
            Turn::Assistant {
                content: "I found the parser failure and will patch it.".into(),
                tool_calls: vec![ToolCall {
                    id: "call-1".into(),
                    name: "shell".into(),
                    arguments: json!({"command":"cargo test -p stencila-agents", "path":"src/lib.rs"}),
                    raw_arguments: None,
                    parse_error: None,
                }],
                reasoning: None,
                thinking_parts: Vec::new(),
                response_content_parts: Vec::new(),
                usage: stencila_models3::types::usage::Usage::default(),
                response_id: None,
                timestamp: now_timestamp(),
            },
            Turn::tool_results(vec![ToolResult {
                tool_call_id: "call-1".into(),
                content: Value::String("error[E0425]: cannot find value `parser`".into()),
                is_error: true,
            }]),
        ];

        let summary = summarize_removed_turns(&turns, 700);
        assert!(summary.contains("3 earlier history entries removed"));
        assert!(summary.contains("src/lib.rs"));
        assert!(summary.contains("cargo test -p stencila-agents"));
        assert!(summary.contains("error[E0425]"));
        assert!(summary.chars().count() <= 700);
    }

    #[test]
    fn extractive_summary_truncates_long_content() {
        let turns = (0..20)
            .map(|index| Turn::user(format!("request {index} {}", "x".repeat(200))))
            .collect::<Vec<_>>();

        let summary = summarize_removed_turns(&turns, 300);
        assert!(summary.chars().count() <= 300);
        assert!(summary.contains("[summary truncated]"));
    }

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
        vision: bool,
        image_in_tool_result: bool,
        provider_id: String,
    }

    impl TestProfile {
        fn new() -> Self {
            Self {
                registry: crate::registry::ToolRegistry::new(),
                vision: false,
                image_in_tool_result: false,
                provider_id: "test".into(),
            }
        }

        fn with_vision(mut self) -> Self {
            self.vision = true;
            self
        }

        fn with_image_in_tool_result(mut self) -> Self {
            self.image_in_tool_result = true;
            self
        }

        fn with_provider_id(mut self, id: impl Into<String>) -> Self {
            self.provider_id = id.into();
            self
        }
    }

    impl ProviderProfile for TestProfile {
        fn id(&self) -> &str {
            &self.provider_id
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

        fn supports_vision(&self) -> bool {
            self.vision
        }

        fn supports_parallel_tool_calls(&self) -> bool {
            false
        }

        fn supports_image_in_tool_result(&self) -> bool {
            self.image_in_tool_result
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

    #[test]
    fn image_attachment_accepts_small_png_dimensions() -> AgentResult<()> {
        let png = {
            use image::{ColorType, ImageBuffer, ImageEncoder, Rgb, codecs::png::PngEncoder};

            let mut bytes = Vec::new();
            let image =
                ImageBuffer::<Rgb<u8>, Vec<u8>>::from_pixel(8_000, 8_000, Rgb([255, 255, 255]));
            PngEncoder::new(&mut bytes)
                .write_image(image.as_raw(), 8_000, 8_000, ColorType::Rgb8.into())
                .map_err(|error| AgentError::Io {
                    message: format!("failed to encode test png: {error}"),
                })?;
            bytes
        };

        let attachment = ImageAttachment {
            data: png,
            media_type: "image/png".into(),
        };

        assert!(!attachment.exceeds_max_dimensions()?);
        Ok(())
    }

    #[test]
    fn image_attachment_rejects_large_png_dimensions() -> AgentResult<()> {
        let png = {
            use image::{ColorType, ImageBuffer, ImageEncoder, Rgb, codecs::png::PngEncoder};

            let mut bytes = Vec::new();
            let image =
                ImageBuffer::<Rgb<u8>, Vec<u8>>::from_pixel(8_001, 8_000, Rgb([255, 255, 255]));
            PngEncoder::new(&mut bytes)
                .write_image(image.as_raw(), 8_001, 8_000, ColorType::Rgb8.into())
                .map_err(|error| AgentError::Io {
                    message: format!("failed to encode test png: {error}"),
                })?;
            bytes
        };

        let attachment = ImageAttachment {
            data: png,
            media_type: "image/png".into(),
        };

        assert!(attachment.exceeds_max_dimensions()?);
        Ok(())
    }

    #[test]
    fn test_profile_does_not_support_vision() {
        let profile = TestProfile::new();
        assert!(
            !profile.supports_vision(),
            "TestProfile mock should not support vision"
        );
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

    /// Helper to create a session using a specific profile, with no tools.
    fn session_with_profile(profile: Box<dyn ProviderProfile>) -> (ApiSession, EventReceiver) {
        let client = Arc::new(CapturingClient {
            requests: std::sync::Mutex::new(Vec::new()),
        });
        ApiSession::new(
            profile,
            Arc::new(NoopEnv),
            client,
            SessionConfig::default(),
            "system".into(),
            0,
            ApiSessionInit::default(),
        )
    }

    #[test]
    fn store_attachment_stores_when_vision_supported() {
        let (mut session, _rx) = session_with_profile(Box::new(TestProfile::new().with_vision()));

        let attachment = Some((
            "tc-123".to_string(),
            ImageAttachment {
                data: vec![0x89, 0x50, 0x4E, 0x47],
                media_type: "image/png".to_string(),
            },
        ));
        session.store_attachment_if_supported(attachment);

        assert!(
            session.image_attachments.contains_key("tc-123"),
            "store_attachment_if_supported should store the attachment \
             when the profile supports vision, regardless of provider id"
        );
    }

    #[test]
    fn store_attachment_discards_when_vision_not_supported() {
        let (mut session, _rx) = session_with_profile(Box::new(TestProfile::new()));

        let attachment = Some((
            "tc-456".to_string(),
            ImageAttachment {
                data: vec![0x89, 0x50, 0x4E, 0x47],
                media_type: "image/png".to_string(),
            },
        ));
        session.store_attachment_if_supported(attachment);

        assert!(
            !session.image_attachments.contains_key("tc-456"),
            "store_attachment_if_supported should discard the attachment \
             when the profile does not support vision"
        );
    }

    // -----------------------------------------------------------------------
    // Phase 2: provider-aware image injection strategy tests
    // -----------------------------------------------------------------------

    /// Helper: set up a session with a tool-results turn that has an image
    /// attachment, then return the messages produced by `convert_history_to_messages`.
    fn messages_with_image_attachment(profile: Box<dyn ProviderProfile>) -> Vec<Message> {
        use stencila_models3::types::tool::ToolResult;

        let (mut session, _rx) = session_with_profile(profile);

        session.image_attachments.insert(
            "tc-img-99".to_string(),
            ImageAttachment {
                data: vec![0x89, 0x50, 0x4E, 0x47],
                media_type: "image/png".to_string(),
            },
        );

        session.history.push(Turn::tool_results(vec![ToolResult {
            tool_call_id: "tc-img-99".to_string(),
            content: serde_json::Value::String("screenshot captured".into()),
            is_error: false,
        }]));

        session.convert_history_to_messages()
    }

    /// Whether any content part in a message is an image.
    fn has_image_part(msg: &Message) -> bool {
        msg.content
            .iter()
            .any(|part| matches!(part, ContentPart::Image { .. }))
    }

    /// Whether any tool result content part carries embedded image data.
    fn has_tool_result_image(msg: &Message) -> bool {
        msg.content.iter().any(|part| {
            matches!(
                part,
                ContentPart::ToolResult {
                    tool_result
                } if tool_result.image_data.is_some()
            )
        })
    }

    /// Assert that a vision-capable non-Anthropic profile defers images to a
    /// follow-up user message rather than inlining them in tool results.
    fn assert_deferred_image_injection(profile: Box<dyn ProviderProfile>) {
        use stencila_models3::types::role::Role;

        let provider = profile.id().to_string();
        let messages = messages_with_image_attachment(profile);

        // Tool-result messages must NOT contain inline image data.
        let tool_msgs: Vec<&Message> = messages.iter().filter(|m| m.role == Role::Tool).collect();
        assert!(
            !tool_msgs.is_empty(),
            "should have at least one Tool message"
        );
        for tool_msg in &tool_msgs {
            assert!(
                !has_image_part(tool_msg),
                "{provider} profile should NOT have inline image in tool-result message; \
                 images should be deferred to a follow-up user message"
            );
        }

        // A follow-up User message must contain the deferred image.
        let has_deferred = messages
            .iter()
            .skip_while(|m| m.role != Role::Tool)
            .filter(|m| m.role == Role::User)
            .any(has_image_part);
        assert!(
            has_deferred,
            "{provider} vision profile should emit a follow-up User message \
             containing deferred image data after the tool results"
        );
    }

    #[test]
    fn deferred_image_injection_for_openai_like_profile() {
        assert_deferred_image_injection(Box::new(
            TestProfile::new().with_vision().with_provider_id("openai"),
        ));
    }

    #[test]
    fn deferred_image_injection_for_gemini_like_profile() {
        assert_deferred_image_injection(Box::new(
            TestProfile::new().with_vision().with_provider_id("gemini"),
        ));
    }

    #[test]
    fn deferred_image_injection_for_default_vision_profile() {
        let profile = TestProfile::new().with_vision();
        assert_ne!(
            profile.id(),
            "anthropic",
            "default test profile should not be anthropic"
        );
        assert_deferred_image_injection(Box::new(profile));
    }

    #[test]
    fn anthropic_inline_image_injection_preserved() {
        use stencila_models3::types::role::Role;

        let profile = TestProfile::new()
            .with_vision()
            .with_image_in_tool_result()
            .with_provider_id("anthropic");
        let messages = messages_with_image_attachment(Box::new(profile));

        // Tool-result messages must carry image data embedded in the ToolResult
        // content part (not as a sibling Image part), so the Anthropic
        // translator can nest the image inside the tool_result block.
        let tool_msgs: Vec<&Message> = messages.iter().filter(|m| m.role == Role::Tool).collect();
        assert!(
            !tool_msgs.is_empty(),
            "should have at least one Tool message"
        );
        assert!(
            tool_msgs.iter().any(|m| has_tool_result_image(m)),
            "Anthropic profile should have image data embedded in ToolResult content part"
        );
        // The image must NOT be a sibling Image content part (that would
        // cause an Anthropic API error).
        assert!(
            !tool_msgs.iter().any(|m| has_image_part(m)),
            "Anthropic profile should NOT have a sibling Image content part in tool messages"
        );

        // No follow-up User message with image data should be emitted.
        let has_deferred = messages
            .iter()
            .skip_while(|m| m.role != Role::Tool)
            .filter(|m| m.role == Role::User)
            .any(has_image_part);
        assert!(
            !has_deferred,
            "Anthropic profile should NOT emit a follow-up User message with image data; \
             images should remain inline in tool results"
        );
    }

    #[test]
    fn non_vision_profile_excludes_all_images() {
        use stencila_models3::types::role::Role;

        let messages = messages_with_image_attachment(Box::new(TestProfile::new()));

        // No message should contain image data.
        for (i, msg) in messages.iter().enumerate() {
            assert!(
                !has_image_part(msg),
                "non-vision profile should have no image data in any message, \
                 but found image in message {i} (role={:?})",
                msg.role
            );
        }

        // No follow-up User message should be injected.
        let user_after_tool: Vec<&Message> = messages
            .iter()
            .skip_while(|m| m.role != Role::Tool)
            .filter(|m| m.role == Role::User)
            .collect();
        assert!(
            user_after_tool.is_empty(),
            "non-vision profile should not inject any follow-up User messages \
             after tool results"
        );
    }
}
